#![forbid(unsafe_code)]

mod cli;
mod models;
mod scan;
mod entitlements;
mod output;
mod monitor;
mod daemon;

use anyhow::{Result, Context};
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<()> {
    // Determine execution mode from CLI arguments
    match cli::get_execution_mode()? {
        cli::ExecutionMode::Scan => run_scan_mode(),
        cli::ExecutionMode::Monitor => run_monitor_mode(),
        cli::ExecutionMode::Daemon => run_daemon_mode(),
        cli::ExecutionMode::Subcommand(command) => run_subcommand(command),
    }
}

fn run_scan_mode() -> Result<()> {
    let config = cli::parse_args()?;
    
    // Set up interrupt handling
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })?;
    
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, interrupted.clone());
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, interrupted.clone());
    
    let start_time = Instant::now();
    
    // Scan directories for binaries
    let discovered_binaries = scan::scan_directories(&config.scan_paths)?;
    
    let mut results = Vec::new();
    let mut scanned = 0;
    let mut matched = 0;
    let mut skipped_unreadable = 0;
    
    for binary in discovered_binaries {
        // Check for interruption
        if interrupted.load(Ordering::Relaxed) {
            break;
        }
        
        scanned += 1;
        
        // Progress indicator for long operations
        if !config.quiet_mode && scanned % 100 == 0 {
            eprintln!("Processed {} files...", scanned);
        }
        
        // Extract entitlements
        match entitlements::extract_entitlements(&binary.path) {
            Ok(entitlement_map) => {
                // Apply entitlement filters if specified
                let filtered_entitlements = if config.filters.entitlements.is_empty() {
                    entitlement_map
                } else {
                    entitlement_map.into_iter()
                        .filter(|(key, _)| config.filters.entitlements.contains(key))
                        .collect()
                };
                
                // Only include binaries that have entitlements (and match filters)
                if !filtered_entitlements.is_empty() {
                    matched += 1;
                    results.push(models::BinaryResult {
                        path: binary.path.to_string_lossy().to_string(),
                        entitlement_count: filtered_entitlements.len(),
                        entitlements: filtered_entitlements,
                    });
                }
            },
            Err(err) => {
                // Count as skipped if we can't read the entitlements
                skipped_unreadable += 1;
                if !config.quiet_mode {
                    eprintln!("Warning: Could not extract entitlements from {}: {}", 
                             binary.path.display(), err);
                }
            }
        }
    }
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let was_interrupted = interrupted.load(Ordering::Relaxed);
    
    let output = models::EntitlementScanOutput {
        results,
        summary: models::ScanSummary {
            scanned,
            matched,
            skipped_unreadable,
            duration_ms,
            interrupted: if was_interrupted { Some(true) } else { None },
        },
    };

    if config.json_output {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        output::format_human(&output)?;
    }

    Ok(())
}

fn run_monitor_mode() -> Result<()> {
    let config = cli::parse_monitor_config()?;
    monitor::polling::start_monitoring(config)
}

fn run_daemon_mode() -> Result<()> {
    // Get CLI args to extract config path
    let args = cli::parse_args_raw()?;
    
    // Create tokio runtime for async daemon execution
    let runtime = tokio::runtime::Runtime::new()
        .context("Failed to create tokio runtime")?;
    
    // Execute daemon mode with config path
    runtime.block_on(daemon::run_daemon_with_config(args.config))
}

fn run_subcommand(command: cli::Commands) -> Result<()> {
    match command {
        cli::Commands::InstallDaemon { config } => {
            install_daemon_service(config)
        },
        cli::Commands::UninstallDaemon => {
            uninstall_daemon_service()
        },
        cli::Commands::DaemonStatus => {
            show_daemon_status()
        },
        cli::Commands::UpdateConfig { updates } => {
            update_daemon_config(updates)
        },
        cli::Commands::Logs { follow, since, format } => {
            show_daemon_logs(follow, since, format)
        },
    }
}

/// Install daemon service with LaunchD
fn install_daemon_service(config_path: Option<std::path::PathBuf>) -> Result<()> {
    use crate::daemon::{config::DaemonConfiguration, launchd::LaunchDPlist};

    println!("🚀 Installing listent daemon service...");

    // Load or create configuration
    let daemon_config = if let Some(ref config_file) = config_path {
        println!("📄 Loading configuration from: {}", config_file.display());
        DaemonConfiguration::load_from_file(config_file)?
    } else {
        println!("📄 Using default configuration");
        DaemonConfiguration::default()
    };

    // Validate configuration
    daemon_config.validate()?;

    // Ensure required directories exist
    daemon_config.ensure_directories()?;

    // Save configuration to standard location if not provided
    let final_config_path = if let Some(config_file) = config_path {
        config_file
    } else {
        let config_path = DaemonConfiguration::user_config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        daemon_config.save_to_file(&config_path)?;
        println!("📝 Saved configuration to: {}", config_path.display());
        config_path
    };

    // Get current executable path
    let current_exe = std::env::current_exe()
        .context("Could not determine current executable path")?;

    // Create LaunchD plist and install service
    let plist = LaunchDPlist::new(&current_exe);
    plist.install_service(&current_exe, Some(&final_config_path))?;

    println!("✅ Daemon service installation complete!");
    println!("   Use 'listent daemon-status' to check service status");
    println!("   Use 'listent logs' to view daemon logs");

    Ok(())
}

/// Uninstall daemon service from LaunchD  
fn uninstall_daemon_service() -> Result<()> {
    use crate::daemon::launchd::LaunchDPlist;

    println!("🗑️  Uninstalling listent daemon service...");

    let current_exe = std::env::current_exe()
        .context("Could not determine current executable path")?;

    let plist = LaunchDPlist::new(&current_exe);
    plist.uninstall_service()?;

    println!("✅ Daemon service uninstallation complete!");
    
    Ok(())
}

/// Show daemon service status
fn show_daemon_status() -> Result<()> {
    use crate::daemon::launchd::LaunchDPlist;

    println!("📊 Checking listent daemon status...");

    let current_exe = std::env::current_exe()
        .context("Could not determine current executable path")?;

    let plist = LaunchDPlist::new(&current_exe);
    
    match plist.get_service_status()? {
        Some(status) => {
            println!("✅ Service found: {}", status.label);
            if status.is_running() {
                println!("🟢 Status: RUNNING (PID: {})", status.pid.unwrap());
            } else {
                println!("🔴 Status: STOPPED (Exit code: {})", status.status_code);
            }
        },
        None => {
            println!("❌ Service not found or not installed");
            println!("   Use 'listent install-daemon' to install the service");
        }
    }

    Ok(())
}

/// Update daemon configuration at runtime
fn update_daemon_config(updates: Vec<String>) -> Result<()> {
    use crate::daemon::config::DaemonConfiguration;

    println!("⚙️  Updating daemon configuration...");

    // Parse configuration updates
    let parsed_updates = cli::parse_config_updates(&updates)?;
    println!("📝 Applying {} configuration updates", parsed_updates.len());

    // Load current configuration
    let config_path = DaemonConfiguration::user_config_path()?;
    let mut config = if config_path.exists() {
        DaemonConfiguration::load_from_file(&config_path)?
    } else {
        anyhow::bail!("Configuration file not found: {}. Install daemon first.", config_path.display());
    };

    // Apply updates atomically
    config.apply_updates(&parsed_updates)?;

    // Save updated configuration
    config.save_to_file(&config_path)?;

    println!("✅ Configuration updated successfully!");
    for (key, value) in &parsed_updates {
        println!("   {}: {}", key, value);
    }
    println!("   Saved to: {}", config_path.display());
    println!("   Note: Restart daemon service for changes to take effect");

    Ok(())
}

/// Show daemon logs
fn show_daemon_logs(follow: bool, since: Option<String>, format: String) -> Result<()> {
    use crate::daemon::logging::get_daemon_logs;

    println!("📄 Retrieving daemon logs...");

    // Validate time format if provided
    if let Some(ref time_str) = since {
        cli::validate_time_format(time_str)?;
    }

    // Retrieve logs from ULS
    let logs = get_daemon_logs(
        "com.github.mariohewardt.listent",
        since.as_deref().unwrap_or("1m"),
    )?;

    if logs.is_empty() {
        println!("📭 No daemon logs found");
        if since.is_some() {
            println!("   Try expanding the time range or check if daemon is running");
        }
        return Ok(());
    }

    println!("📄 Found {} log entries", logs.len());
    
    match format.as_str() {
        "json" => {
            for log_line in &logs {
                println!("{}", log_line);
            }
        },
        "human" => {
            for log_line in &logs {
                // Parse JSON and format for human readability
                if let Ok(log_entry) = serde_json::from_str::<serde_json::Value>(log_line) {
                    if let Some(timestamp) = log_entry.get("timestamp") {
                        print!("[{}] ", timestamp.as_str().unwrap_or("unknown"));
                    }
                    if let Some(message) = log_entry.get("message") {
                        println!("{}", message.as_str().unwrap_or(log_line));
                    } else {
                        println!("{}", log_line);
                    }
                } else {
                    println!("{}", log_line);
                }
            }
        },
        _ => {
            anyhow::bail!("Invalid format: '{}'. Use 'human' or 'json'", format);
        }
    }

    Ok(())
}
