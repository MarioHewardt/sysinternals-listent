#![forbid(unsafe_code)]

mod cli;
mod models;
mod scan;
mod entitlements;
mod output;
mod monitor;
mod daemon;
mod constants;

use anyhow::{Result, Context};
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::constants::APP_SUBSYSTEM;

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
    
    // Set up interrupt handling using signal-hook
    let interrupted = Arc::new(AtomicBool::new(false));
    
    // Register signal handlers for SIGINT and SIGTERM
    signal_hook::flag::register(signal_hook::consts::SIGINT, interrupted.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, interrupted.clone())?;
    
    let start_time = Instant::now();
    
    // Progress indicator for animated scanning
    let mut progress = if !config.quiet_mode {
        Some(output::progress::ScanProgress::new())
    } else {
        None
    };
    
    let mut results = Vec::new();
    let mut scanned = 0;
    let mut matched = 0;
    let mut skipped_unreadable = 0;
    
    // Fast count total files (like find command) with interrupt support
    let total_files = scan::count_total_files_with_interrupt(&config.scan_paths, &interrupted)
        .context("Failed to count total files")?;
    
    // Check if interrupted during counting
    if interrupted.load(Ordering::Relaxed) {
        return Ok(());
    }
    
    // Start progress with total file count
    if let Some(ref mut progress) = progress {
        progress.start_scanning(total_files);
    }
    
    // Process files one by one
    for path_str in &config.scan_paths {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            // Update progress to show current top-level directory (only once per directory)
            if let Some(ref mut progress) = progress {
                progress.set_current_directory(path);
            }
            
            if path.is_file() {
                // Single file case
                process_single_file(path, &config, &mut results, &mut scanned, &mut matched, 
                                  &mut skipped_unreadable, &mut progress, &interrupted)?;
            } else {
                // Directory case
                process_directory_files(path, &config, &mut results, &mut scanned, &mut matched,
                                      &mut skipped_unreadable, &mut progress, &interrupted)?;
            }
        }
        
        // Check for interruption between directories
        if interrupted.load(Ordering::Relaxed) {
            break;
        }
    }

    // Complete progress indicator
    if let Some(mut progress) = progress {
        progress.complete_scanning();
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

/// Process a single file, checking if it's a binary and extracting entitlements
fn process_single_file(
    path: &std::path::Path,
    config: &models::ScanConfig,
    results: &mut Vec<models::BinaryResult>,
    scanned: &mut usize,
    matched: &mut usize,
    skipped_unreadable: &mut usize,
    progress: &mut Option<output::progress::ScanProgress>,
    interrupted: &Arc<AtomicBool>
) -> Result<()> {
    // Check for interruption
    if interrupted.load(Ordering::Relaxed) {
        return Ok(());
    }
    
    // Check if this file is a binary
    if let Some(binary) = scan::check_single_file(path) {
        process_binary(binary, config, results, scanned, matched, skipped_unreadable, progress)?;
    } else {
        // Non-binary file, just increment skipped count
        if let Some(ref mut progress) = progress {
            progress.increment_skipped();
        }
    }
    
    Ok(())
}

/// Process all files in a directory recursively
fn process_directory_files(
    dir_path: &std::path::Path,
    config: &models::ScanConfig,
    results: &mut Vec<models::BinaryResult>,
    scanned: &mut usize,
    matched: &mut usize,
    skipped_unreadable: &mut usize,
    progress: &mut Option<output::progress::ScanProgress>,
    interrupted: &Arc<AtomicBool>
) -> Result<()> {
    use std::fs;
    
    for entry in fs::read_dir(dir_path)? {
        // Check for interruption at the start of each directory entry
        if interrupted.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            process_single_file(&path, config, results, scanned, matched, 
                              skipped_unreadable, progress, interrupted)?;
            
            // Check for interruption after processing each file
            if interrupted.load(Ordering::Relaxed) {
                return Ok(());
            }
        } else if path.is_dir() {
            // Recursively process subdirectories without updating progress directory name
            process_directory_files(&path, config, results, scanned, matched,
                                  skipped_unreadable, progress, interrupted)?;
            
            // Check for interruption after processing each subdirectory
            if interrupted.load(Ordering::Relaxed) {
                return Ok(());
            }
        }
    }
    
    Ok(())
}

/// Process a binary file and extract entitlements
fn process_binary(
    binary: scan::DiscoveredBinary,
    config: &models::ScanConfig,
    results: &mut Vec<models::BinaryResult>,
    scanned: &mut usize,
    matched: &mut usize,
    skipped_unreadable: &mut usize,
    progress: &mut Option<output::progress::ScanProgress>
) -> Result<()> {
    *scanned += 1;
    
    // Update progress
    if let Some(ref mut progress) = progress {
        progress.increment_scanned();
    }
    
    // Extract entitlements
    match entitlements::extract_entitlements(&binary.path) {
        Ok(entitlement_map) => {
            // Get list of entitlement keys for pattern matching
            let entitlement_keys: Vec<String> = entitlement_map.keys().cloned().collect();
            
            // Check if any entitlements match the filters using consistent pattern matching
            if entitlements::pattern_matcher::entitlements_match_filters(&entitlement_keys, &config.filters.entitlements) {
                // Apply entitlement filters to output (only show matching entitlements)
                let filtered_entitlements = if config.filters.entitlements.is_empty() {
                    entitlement_map
                } else {
                    entitlement_map.into_iter()
                        .filter(|(key, _)| {
                            config.filters.entitlements.iter().any(|filter| {
                                entitlements::pattern_matcher::matches_entitlement_filter(key, filter)
                            })
                        })
                        .collect()
                };
                
                *matched += 1;
                results.push(models::BinaryResult {
                    path: binary.path.to_string_lossy().to_string(),
                    entitlement_count: filtered_entitlements.len(),
                    entitlements: filtered_entitlements,
                });
            }
        },
        Err(err) => {
            // Count as skipped if we can't read the entitlements
            *skipped_unreadable += 1;
            if !config.quiet_mode {
                eprintln!("Warning: Could not extract entitlements from {}: {}", 
                         binary.path.display(), err);
            }
        }
    }
    
    Ok(())
}

fn run_monitor_mode() -> Result<()> {
    let config = cli::parse_monitor_config()?;
    
    // Set up interrupt handling using signal-hook (same as scan mode)
    let interrupted = Arc::new(AtomicBool::new(false));
    
    // Register signal handlers for SIGINT and SIGTERM
    signal_hook::flag::register(signal_hook::consts::SIGINT, interrupted.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, interrupted.clone())?;
    
    monitor::polling::start_monitoring_with_interrupt(config, interrupted)
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
        cli::Commands::DaemonStop => {
            stop_daemon_process()
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

    // Check for running listent daemon processes
    let daemon_running = {
        let output = std::process::Command::new("pgrep")
            .args(["-f", "listent"])
            .output();
        
        let mut found_daemon = false;
        if let Ok(result) = output {
            if result.status.success() && !result.stdout.is_empty() {
                // Check each listent process to see if it's a daemon
                let pids: Vec<u32> = String::from_utf8_lossy(&result.stdout)
                    .lines()
                    .filter_map(|line| line.trim().parse::<u32>().ok())
                    .collect();
                
                for pid in pids {
                    // Check command line arguments
                    if let Ok(cmd_output) = std::process::Command::new("ps")
                        .args(["-p", &pid.to_string(), "-o", "args="])
                        .output()
                    {
                        let cmd_line = String::from_utf8_lossy(&cmd_output.stdout);
                        if cmd_line.contains("--daemon") && cmd_line.contains("--monitor") {
                            found_daemon = true;
                            break;
                        }
                    }
                }
            }
        }
        found_daemon
    };

    // Check LaunchD service status
    let current_exe = std::env::current_exe()
        .context("Could not determine current executable path")?;

    let plist = LaunchDPlist::new(&current_exe);
    let service_status = plist.get_service_status()?;

    // Display comprehensive status
    println!("\n🔍 Daemon Status Report:");
    println!("========================");

    if daemon_running {
        println!("✅ Process Status: listent daemon RUNNING");
    } else {
        println!("❌ Process Status: No listent daemon found");
    }

    match &service_status {
        Some(status) => {
            println!("✅ LaunchD Service: {} (found)", status.label);
            if status.is_running() {
                println!("🟢 Service Status: RUNNING (PID: {})", status.pid.unwrap());
            } else {
                println!("🔴 Service Status: STOPPED (Exit code: {})", status.status_code);
            }
        },
        None => {
            println!("❌ LaunchD Service: not found or not installed");
        }
    }

    // Provide helpful next steps
    println!("\n💡 Next Steps:");
    match (daemon_running, &service_status) {
        (true, Some(status)) if status.is_running() => {
            println!("✓ Daemon is running normally via LaunchD");
            println!("  • View logs: log show --predicate 'subsystem == \"com.microsoft.sysinternals.listent\"' --info");
            println!("  • Stop daemon: listent uninstall-daemon");
        }
        (true, None) => {
            println!("✓ Daemon running directly (not as LaunchD service)");
            println!("  • View logs: log show --predicate 'subsystem == \"com.microsoft.sysinternals.listent\"' --info");
            println!("  • Stop daemon: listent daemon-stop");
            println!("  • Install as service: listent install-daemon");
        }
        (false, Some(_)) => {
            println!("⚠ LaunchD service exists but no daemon process found");
            println!("  • Service may be starting up or crashed");
            println!("  • Restart: listent uninstall-daemon && listent install-daemon");
        }
        (false, None) => {
            println!("ℹ No daemon running");
            println!("  • Start daemon: listent install-daemon");
        }
        _ => {
            println!("⚠ Inconsistent state detected");
            println!("  • Clean restart recommended: listent uninstall-daemon && listent install-daemon");
        }
    }

    Ok(())
}

/// Stop running daemon process
fn stop_daemon_process() -> Result<()> {
    use crate::daemon::launchd::LaunchDPlist;
    
    println!("🛑 Stopping listent daemon...");

    // First, check if daemon is running as LaunchD service
    let current_exe = std::env::current_exe()
        .context("Could not determine current executable path")?;
    let plist = LaunchDPlist::new(&current_exe);
    
    // Check if LaunchD service exists
    let service_loaded = plist.is_service_loaded().unwrap_or(false);
    
    if service_loaded {
        // If running under LaunchD, we need to unload it (KeepAlive will restart if we just kill)
        println!("📋 Detected LaunchD service, stopping...");
        println!("⚠️  Note: Service will remain installed. To restart: sudo launchctl bootstrap system /Library/LaunchDaemons/com.microsoft.sysinternals.listent.plist");
        println!("   To permanently remove: sudo listent uninstall-daemon");
        
        if let Err(e) = plist.launchctl_unload() {
            println!("⚠️  Failed to stop LaunchD service: {}", e);
            println!("   Attempting to kill process directly...");
        } else {
            println!("✅ Daemon stopped successfully");
            return Ok(());
        }
    }

    // If not a LaunchD service (or unload failed), kill the process directly
    let output = std::process::Command::new("pgrep")
        .args(["-f", "listent"])
        .output()
        .context("Failed to search for listent processes")?;

    if !output.status.success() || output.stdout.is_empty() {
        println!("❌ No listent daemon processes found");
        return Ok(());
    }

    // Get all listent PIDs and check their command lines
    let pids: Vec<u32> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect();

    let mut daemon_pids = Vec::new();
    let current_pid = std::process::id();

    for pid in pids {
        if pid == current_pid {
            continue; // Skip current process
        }

        // Check if this is a daemon process
        if let Ok(cmd_output) = std::process::Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "args="])
            .output()
        {
            let cmd_line = String::from_utf8_lossy(&cmd_output.stdout);
            // Only match actual listent daemon processes, not sudo commands
            if cmd_line.contains("listent") && 
               cmd_line.contains("--daemon") && 
               cmd_line.contains("--monitor") &&
               !cmd_line.contains("sudo") {
                daemon_pids.push(pid);
            }
        }
    }

    if daemon_pids.is_empty() {
        println!("❌ No listent daemon processes found");
        return Ok(());
    }

    // Stop each daemon process gracefully with SIGTERM
    let mut any_failed = false;
    for pid in &daemon_pids {
        let result = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output();

        if let Err(_) = result {
            any_failed = true;
        } else if let Ok(output) = result {
            if !output.status.success() {
                any_failed = true;
            }
        }
    }

    if any_failed {
        println!("❌ Failed to stop some daemon processes");
        return Ok(());
    }

    // Wait a moment for graceful shutdown
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Check if processes are still running
    let mut still_running = Vec::new();
    for pid in &daemon_pids {
        if let Ok(output) = std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])  // Signal 0 just checks if process exists
            .output()
        {
            if output.status.success() {
                still_running.push(*pid);
            }
        }
    }

    if still_running.is_empty() {
        println!("✅ Daemon stopped successfully");
    } else {
        // Force kill remaining processes
        for pid in still_running {
            let _ = std::process::Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .output();
        }
        println!("✅ Daemon stopped (forced)");
    }

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
        APP_SUBSYSTEM,
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
