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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use rayon::prelude::*;
use crate::constants::APP_SUBSYSTEM;

fn main() {
    // Determine execution mode from CLI arguments
    let result = (|| -> Result<()> {
        match cli::get_execution_mode()? {
            cli::ExecutionMode::Scan => run_scan_mode(),
            cli::ExecutionMode::Monitor { path, entitlement, interval, json, quiet } => {
                run_monitor_mode(path, entitlement, interval, json, quiet)
            }
            cli::ExecutionMode::Daemon(action) => run_daemon_command(action),
        }
    })();

    if let Err(e) = result {
        // Print the error first
        eprintln!("Error: {:?}", e);

        // Then check for permission denied and show hint
        let err_string = format!("{:?}", e);
        if err_string.contains("os error 13") || err_string.contains("Permission denied") {
            eprintln!("\n💡 Hint: Some paths require elevated privileges. Try:");
            eprintln!("   sudo listent [PATH...]");
        }

        std::process::exit(1);
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
    
    // ========== PHASE 1: Collect all binaries (sequential, fast) ==========
    let mut discovered_binaries = Vec::new();
    let mut skipped_count = 0usize;
    
    for path_str in &config.scan_paths {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            // Update progress to show current top-level directory
            if let Some(ref mut progress) = progress {
                progress.set_current_directory(path);
            }
            
            if path.is_file() {
                if let Some(binary) = scan::check_single_file(path) {
                    discovered_binaries.push(binary);
                } else {
                    skipped_count += 1;
                    if let Some(ref mut progress) = progress {
                        progress.increment_skipped();
                    }
                }
            } else {
                collect_binaries_from_directory(
                    path, 
                    &mut discovered_binaries, 
                    &mut skipped_count,
                    &mut progress, 
                    &interrupted
                )?;
            }
        }
        
        if interrupted.load(Ordering::Relaxed) {
            break;
        }
    }
    
    // Complete progress indicator after discovery phase
    if let Some(mut progress) = progress {
        progress.complete_scanning();
    }
    
    // Check if interrupted during discovery
    let was_interrupted_early = interrupted.load(Ordering::Relaxed);
    if was_interrupted_early && discovered_binaries.is_empty() {
        return Ok(());
    }
    
    // ========== PHASE 2: Extract entitlements in parallel (slow part) ==========
    let scanned = AtomicUsize::new(0);
    let matched = AtomicUsize::new(0);
    let skipped_unreadable = AtomicUsize::new(0);
    let config_ref = &config;
    let interrupted_ref = &interrupted;
    
    // Process binaries in parallel using rayon
    let results: Vec<models::BinaryResult> = discovered_binaries
        .par_iter()
        .filter_map(|binary| {
            // Check for interruption
            if interrupted_ref.load(Ordering::Relaxed) {
                return None;
            }
            
            scanned.fetch_add(1, Ordering::Relaxed);
            
            match entitlements::extract_entitlements(&binary.path) {
                Ok(entitlement_map) => {
                    let entitlement_keys: Vec<String> = entitlement_map.keys().cloned().collect();
                    
                    if entitlements::pattern_matcher::entitlements_match_filters(
                        &entitlement_keys, 
                        &config_ref.filters.entitlements
                    ) {
                        let filtered_entitlements = if config_ref.filters.entitlements.is_empty() {
                            entitlement_map
                        } else {
                            entitlement_map.into_iter()
                                .filter(|(key, _)| {
                                    config_ref.filters.entitlements.iter().any(|filter| {
                                        entitlements::pattern_matcher::matches_entitlement_filter(key, filter)
                                    })
                                })
                                .collect()
                        };
                        
                        matched.fetch_add(1, Ordering::Relaxed);
                        Some(models::BinaryResult {
                            path: binary.path.to_string_lossy().to_string(),
                            entitlement_count: filtered_entitlements.len(),
                            entitlements: filtered_entitlements,
                        })
                    } else {
                        None
                    }
                },
                Err(_) => {
                    skipped_unreadable.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        })
        .collect();
    
    // Sort results by path for deterministic output
    let mut results = results;
    results.sort_by(|a, b| a.path.cmp(&b.path));
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let was_interrupted = interrupted.load(Ordering::Relaxed);
    
    let output = models::EntitlementScanOutput {
        results,
        summary: models::ScanSummary {
            scanned: scanned.load(Ordering::Relaxed),
            matched: matched.load(Ordering::Relaxed),
            skipped_unreadable: skipped_unreadable.load(Ordering::Relaxed),
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

/// Collect all binaries from a directory recursively (Phase 1 - fast)
fn collect_binaries_from_directory(
    dir_path: &std::path::Path,
    binaries: &mut Vec<scan::DiscoveredBinary>,
    skipped: &mut usize,
    progress: &mut Option<output::progress::ScanProgress>,
    interrupted: &Arc<AtomicBool>,
) -> Result<()> {
    use std::fs;
    
    for entry in fs::read_dir(dir_path)? {
        if interrupted.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(binary) = scan::check_single_file(&path) {
                binaries.push(binary);
                if let Some(ref mut progress) = progress {
                    progress.increment_scanned();
                }
            } else {
                *skipped += 1;
                if let Some(ref mut progress) = progress {
                    progress.increment_skipped();
                }
            }
        } else if path.is_dir() {
            collect_binaries_from_directory(&path, binaries, skipped, progress, interrupted)?;
        }
    }
    
    Ok(())
}

fn run_monitor_mode(
    path: Vec<std::path::PathBuf>,
    entitlement: Vec<String>,
    interval: f64,
    json: bool,
    quiet: bool,
) -> Result<()> {
    let config = cli::parse_monitor_config(path, entitlement, interval, json, quiet)?;
    
    // Set up interrupt handling using signal-hook (same as scan mode)
    let interrupted = Arc::new(AtomicBool::new(false));
    
    // Register signal handlers for SIGINT and SIGTERM
    signal_hook::flag::register(signal_hook::consts::SIGINT, interrupted.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, interrupted.clone())?;
    
    monitor::polling::start_monitoring_with_interrupt(config, interrupted)
}

fn run_daemon_command(action: cli::DaemonCommands) -> Result<()> {
    // Get command name before moving action
    let cmd_name = match &action {
        cli::DaemonCommands::Install { .. } => "install",
        cli::DaemonCommands::Uninstall => "uninstall",
        cli::DaemonCommands::Run { .. } => "run",
        cli::DaemonCommands::Stop => "stop",
        cli::DaemonCommands::Status => "status",
        cli::DaemonCommands::Logs { .. } => "logs",
    };

    let result = match action {
        cli::DaemonCommands::Run { config } => {
            run_daemon_mode(config)
        }
        cli::DaemonCommands::Install { config } => {
            install_daemon_service(config)
        }
        cli::DaemonCommands::Uninstall => {
            uninstall_daemon_service()
        }
        cli::DaemonCommands::Status => {
            show_daemon_status()
        }
        cli::DaemonCommands::Stop => {
            stop_daemon_process()
        }
        cli::DaemonCommands::Logs { follow, since, format } => {
            show_daemon_logs(follow, since, format)
        }
    };

    // Check for permission denied errors and suggest sudo
    if let Err(ref e) = result {
        let err_string = format!("{:?}", e);
        if err_string.contains("os error 13") || err_string.contains("Permission denied") {
            eprintln!("\n💡 Hint: This operation requires root privileges. Try:");
            eprintln!("   sudo listent daemon {}", cmd_name);
        }
    }

    result
}

fn run_daemon_mode(config: Option<std::path::PathBuf>) -> Result<()> {
    // Create tokio runtime for async daemon execution
    let runtime = tokio::runtime::Runtime::new()
        .context("Failed to create tokio runtime")?;
    
    // Execute daemon mode with config path
    runtime.block_on(daemon::run_daemon_with_config(config))
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
    println!("   Use 'listent daemon status' to check service status");
    println!("   Use 'listent daemon logs' to view daemon logs");

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

    // Check for running listent daemon processes (reuse shared helper)
    let daemon_running = daemon::is_daemon_running();

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
            println!("  • View logs: listent daemon logs");
            println!("  • Stop daemon: listent daemon uninstall");
        }
        (true, None) => {
            println!("✓ Daemon running directly (not as LaunchD service)");
            println!("  • View logs: listent daemon logs");
            println!("  • Stop daemon: listent daemon stop");
            println!("  • Install as service: listent daemon install");
        }
        (false, Some(_)) => {
            println!("⚠ LaunchD service exists but no daemon process found");
            println!("  • Service may be starting up or crashed");
            println!("  • Restart: listent daemon uninstall && listent daemon install");
        }
        (false, None) => {
            println!("ℹ No daemon running");
            println!("  • Start daemon: listent daemon install");
        }
        _ => {
            println!("⚠ Inconsistent state detected");
            println!("  • Clean restart recommended: listent daemon uninstall && listent daemon install");
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
        println!("   To permanently remove: sudo listent daemon uninstall");
        
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
            // Only match actual listent daemon processes running 'daemon run', not sudo commands
            if cmd_line.contains("listent") && 
               cmd_line.contains("daemon") && 
               cmd_line.contains("run") &&
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
fn show_daemon_logs(_follow: bool, since: Option<String>, format: String) -> Result<()> {
    use crate::daemon::logging::get_daemon_logs;

    // Note: `follow` parameter reserved for future log streaming implementation
    
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
