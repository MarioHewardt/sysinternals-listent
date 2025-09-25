//! Daemon module for launchd integration and background process monitoring
//!
//! This module provides functionality to run listent as a macOS daemon:
//! - Configuration management with atomic updates
//! - Inter-process communication for runtime configuration changes
//! - LaunchD integration for system service management
//! - Enhanced Unified Logging System integration

pub mod config;
pub mod launchd;
pub mod logging;

use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;
use crate::models::{PollingConfiguration, ProcessSnapshot, MonitoredProcess};
use crate::constants::{APP_SUBSYSTEM, DAEMON_CATEGORY, format_permission_error};
use crate::daemon::logging::DaemonLogger;
use crate::monitor::process_tracker::ProcessTracker;

/// Check if a listent daemon process is already running
fn is_daemon_running() -> bool {
    use std::process::Command;
    
    // Look for listent processes with daemon flags
    let output = Command::new("pgrep")
        .args(["-f", "listent"])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() && !result.stdout.is_empty() {
                // Get all listent PIDs and check their command lines
                let pids: Vec<u32> = String::from_utf8_lossy(&result.stdout)
                    .lines()
                    .filter_map(|line| line.trim().parse::<u32>().ok())
                    .collect();
                
                let current_pid = std::process::id();
                
                // Check each PID to see if it's a daemon process
                for pid in pids {
                    if pid == current_pid {
                        continue; // Skip current process
                    }
                    
                    // Check command line arguments
                    if let Ok(cmd_output) = Command::new("ps")
                        .args(["-p", &pid.to_string(), "-o", "args="])
                        .output()
                    {
                        let cmd_line = String::from_utf8_lossy(&cmd_output.stdout);
                        // Only match actual listent processes, not sudo commands
                        if cmd_line.contains("listent") && 
                           cmd_line.contains("--daemon") &&
                           !cmd_line.contains("sudo") {
                            return true;
                        }
                    }
                }
                false
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Run daemon with CLI arguments (simplified approach)
/// This function directly accepts daemon configuration via CLI arguments
pub async fn run_daemon_with_args(
    interval: f64,
    paths: Vec<PathBuf>,
    entitlements: Vec<String>,
) -> Result<()> {
    // Check if we're already running as the daemon child process
    if std::env::var("LISTENT_DAEMON_CHILD").is_ok() {
        // We're the child process - run the daemon directly
        run_daemon_process_with_args(interval, paths, entitlements).await
    } else {
        // We're the parent - spawn child and exit
        spawn_daemon_child_with_args(interval, paths, entitlements).await
    }
}

/// Spawn daemon child process with CLI arguments
async fn spawn_daemon_child_with_args(
    interval: f64,
    paths: Vec<PathBuf>,
    entitlements: Vec<String>,
) -> Result<()> {
    // Check if daemon is already running BEFORE spawning
    if is_daemon_running() {
        anyhow::bail!(
            "Daemon already running. Use 'pkill -f listent' to stop it first."
        );
    }
    
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    let mut cmd = std::process::Command::new(current_exe);
    cmd.env("LISTENT_DAEMON_CHILD", "1");
    cmd.arg("--daemon");
    cmd.arg("--interval").arg(interval.to_string());
    
    // Add paths as individual arguments (same as scan/monitor modes)
    for path in &paths {
        cmd.arg(path);
    }
    
    // Add entitlements as individual -e arguments (same as scan/monitor modes)
    for entitlement in &entitlements {
        cmd.arg("-e").arg(entitlement);
    }
    
    // Spawn the child process detached
    cmd.spawn()
        .context("Failed to spawn daemon child process")?;
    
    println!("🚀 listent daemon starting...");
    
    // Wait a moment for the child to start, then verify it's running
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    if is_daemon_running() {
        println!("✅ listent daemon started successfully");
        println!("  View logs: log show --predicate 'subsystem == \"{}\"' --info", APP_SUBSYSTEM);
        println!("  Stop daemon: pkill -f 'listent.*--daemon'");
        Ok(())
    } else {
        eprintln!("❌ Failed to start listent daemon");
        eprintln!("   The daemon process exited unexpectedly");
        eprintln!("   Check logs: log show --predicate 'subsystem == \"{}\"' --info", APP_SUBSYSTEM);
        bail!("Daemon startup failed")
    }
}

/// Run the actual daemon process with CLI arguments
async fn run_daemon_process_with_args(
    interval: f64,
    paths: Vec<PathBuf>,
    entitlements: Vec<String>,
) -> Result<()> {
    // Create simplified logger (no complex config needed)
    let logger = DaemonLogger::new(
        APP_SUBSYSTEM.to_string(),
        DAEMON_CATEGORY.to_string(),
    )?;

    // Log startup with CLI arguments
    logger.log_startup_with_args(interval, &paths, &entitlements, std::process::id())?;

    // Setup signal handling for graceful shutdown
    let shutdown_signal = setup_signal_handlers();

    // Main monitoring loop with CLI arguments
    let monitoring_task = {
        let logger_clone = logger.clone();
        
        tokio::spawn(async move {
            if let Err(e) = run_simplified_monitoring_loop(interval, paths, entitlements, logger_clone).await {
                eprintln!("❌ Monitoring loop error: {}", e);
            }
        })
    };

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_signal => {
            logger.log_shutdown("Received shutdown signal")?;
        }
        _ = monitoring_task => {
            logger.log_shutdown("Monitoring loop ended")?;
        }
    }

    Ok(())
}

/// Simplified monitoring loop that uses direct CLI arguments
async fn run_simplified_monitoring_loop(
    interval: f64,
    paths: Vec<PathBuf>,
    entitlements: Vec<String>,
    logger: DaemonLogger,
) -> Result<()> {
    let mut interval_timer = tokio::time::interval(Duration::from_secs_f64(interval));
    let mut process_tracker = ProcessTracker::new();

    loop {
        interval_timer.tick().await;

        // Create polling configuration from CLI arguments
        let polling_config = PollingConfiguration {
            interval: Duration::from_secs_f64(interval),
            path_filters: paths.clone(),
            entitlement_filters: entitlements.clone(),
            output_json: false, // ULS logging instead
            quiet_mode: false,  // Log all detections
        };

        // Create current snapshot using polling logic
        let current_processes = match scan_current_processes(&polling_config).await {
            Ok(processes) => processes,
            Err(e) => {
                logger.log_error(&format!("Failed to scan processes: {}", e), None)?;
                continue;
            }
        };
        
        let current_snapshot = ProcessSnapshot {
            processes: current_processes,
        };

        // Detect new processes
        let new_processes = process_tracker.detect_new_processes(current_snapshot);
        
        // Log any new processes with entitlements (silent operation)
        for process in new_processes {
            if !process.entitlements.is_empty() {
                if let Err(e) = logger.log_process_detection(
                    process.pid,
                    &process.name,
                    &process.executable_path,
                    &process.entitlements,
                ) {
                    eprintln!("❌ Failed to log process {}: {}", process.name, e);
                }
            }
        }
    }
}

/// Setup signal handlers for graceful shutdown
async fn setup_signal_handlers() {
    let _ = signal::ctrl_c().await;
}

/// Scan current processes and their entitlements
async fn scan_current_processes(config: &PollingConfiguration) -> Result<std::collections::HashMap<u32, MonitoredProcess>> {
    use sysinfo::{ProcessExt, System, SystemExt, PidExt};
    
    let mut system = System::new_all();
    system.refresh_processes();
    
    let mut processes = std::collections::HashMap::new();
    
    // Scan all processes
    for (pid, process) in system.processes() {
        let pid_u32 = pid.as_u32();
        let process_name = process.name().to_string();
        
        // Get executable path
        let executable_path = process.exe().to_path_buf();
        
        // Apply path filters if specified
        if !config.path_filters.is_empty() {
            let matches_filter = config.path_filters.iter().any(|filter| {
                executable_path.starts_with(filter)
            });
            if !matches_filter {
                continue;
            }
        }
        
        // Extract entitlements - convert HashMap to Vec of keys
        let entitlements = match crate::entitlements::extract_entitlements(&executable_path) {
            Ok(entitlements_map) => entitlements_map.keys().cloned().collect::<Vec<String>>(),
            Err(_) => Vec::new(), // Continue with empty entitlements if extraction fails
        };
        
        // Apply entitlement filters if specified using consistent pattern matching
        if !crate::entitlements::pattern_matcher::entitlements_match_filters(&entitlements, &config.entitlement_filters) {
            continue;
        }
        
        // Create monitored process
        let monitored_process = MonitoredProcess {
            pid: pid_u32,
            name: process_name,
            executable_path,
            entitlements,
            discovery_timestamp: std::time::SystemTime::now(),
        };
        
        processes.insert(pid_u32, monitored_process);
    }
    
    Ok(processes)
}

/// Install listent as a LaunchD service with CLI arguments
pub async fn install_launchd_service(
    interval: f64,
    paths: Vec<PathBuf>,
    entitlements: Vec<String>,
) -> Result<()> {
    use crate::daemon::launchd::LaunchDPlist;
    
    // Check if we can write to the LaunchDaemons directory (safer than checking uid)
    let launch_daemons_dir = std::path::Path::new("/Library/LaunchDaemons");
    if !launch_daemons_dir.exists() || std::fs::metadata(launch_daemons_dir).is_err() {
        bail!(format_permission_error("/Library/LaunchDaemons directory", "access"));
    }
    
    // Try to create a test file to check write permissions
    let test_file = launch_daemons_dir.join(".listent-test");
    match std::fs::File::create(&test_file) {
        Ok(_) => {
            // Clean up test file
            let _ = std::fs::remove_file(&test_file);
        }
        Err(_) => {
            bail!(format_permission_error("/Library/LaunchDaemons directory", "write to"));
        }
    }
    
    // Get current executable path
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    // Create LaunchD plist with daemon arguments
    let mut plist = LaunchDPlist::new(&current_exe);
    
    // Set program arguments to include our CLI parameters
    let mut program_args = vec![current_exe.to_string_lossy().to_string()];
    program_args.push("--daemon".to_string());
    program_args.push("--interval".to_string());
    program_args.push(interval.to_string());
    
    // Add paths
    for path in &paths {
        program_args.push(path.to_string_lossy().to_string());
    }
    
    // Add entitlements
    for entitlement in &entitlements {
        program_args.push("-e".to_string());
        program_args.push(entitlement.clone());
    }
    
    // Set the arguments in the plist
    plist.program_arguments = program_args;
    
    // Generate plist content
    let _plist_content = plist.generate_plist()
        .context("Failed to generate plist content")?;
    
    // Install the plist and load the service
    match plist.install_service(&current_exe, None) {
        Ok(_) => {
            println!("✅ LaunchD service installed successfully");
            println!("  Service name: {}", crate::constants::LAUNCHD_SERVICE_NAME);
            println!("  Polling interval: {}s", interval);
            println!("  Monitoring paths: {}", paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "));
            println!("  Entitlement filters: {}", entitlements.join(", "));
            println!("  Plist location: /Library/LaunchDaemons/{}", crate::constants::LAUNCHD_PLIST_NAME);
            println!("  View logs: log show --predicate 'subsystem == \"{}\"' --info", APP_SUBSYSTEM);
            println!("  Check status: sudo launchctl list | grep listent");
            println!("  Uninstall: sudo launchctl unload /Library/LaunchDaemons/{} && sudo rm /Library/LaunchDaemons/{}", 
                crate::constants::LAUNCHD_PLIST_NAME, crate::constants::LAUNCHD_PLIST_NAME);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to install LaunchD service: {}", e);
            bail!("LaunchD service installation failed")
        }
    }
}