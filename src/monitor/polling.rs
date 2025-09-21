use crate::models::{MonitoredProcess, PollingConfiguration, ProcessSnapshot};
use crate::monitor::{ProcessTracker, init_logger};
use anyhow::Result;
use signal_hook;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use sysinfo::{System, SystemExt, ProcessExt, PidExt};

/// Start monitoring processes with the given configuration
pub fn start_monitoring(config: PollingConfiguration) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up signal handler using signal-hook
    signal_hook::flag::register(signal_hook::consts::SIGINT, r.clone())?;

    // Initialize unified logging
    let _logger = init_logger().ok(); // Graceful degradation if logging fails

    // Initialize process tracker and system info
    let mut tracker = ProcessTracker::new();
    let mut system = System::new_all();
    
    // Pre-allocate collections to reduce allocations in the loop
    let mut new_processes = Vec::new();
    let mut filtered_processes = Vec::new();

    if !config.quiet_mode {
        println!("Starting process monitoring (interval: {:.1}s)...", config.interval.as_secs_f64());
        if !config.path_filters.is_empty() {
            println!("Monitoring {} for processes", 
                config.path_filters.iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "));
        }
        if !config.entitlement_filters.is_empty() {
            println!("Monitoring for processes with entitlement: {}", 
                config.entitlement_filters.join(", "));
        }
        println!("Press Ctrl+C to stop monitoring.");
        println!();
    }

    while running.load(Ordering::SeqCst) {
        let cycle_start = Instant::now();

        // Refresh system information (only processes, not all system info)
        system.refresh_processes();

        // Create snapshot of current processes
        let snapshot = create_process_snapshot(&system)?;

        // Detect new processes (reuse vector to avoid allocations)
        new_processes.clear();
        let mut new_procs = tracker.detect_new_processes(snapshot);
        
        // Extract entitlements only for new processes (performance optimization)
        for process in &mut new_procs {
            process.entitlements = extract_process_entitlements(&process.executable_path)
                .unwrap_or_else(|_| Vec::new());
        }
        
        new_processes.extend(new_procs);

        // Apply filters (reuse vector to avoid allocations) 
        filtered_processes.clear();
        filtered_processes.extend(apply_filters(new_processes.drain(..).collect(), &config)?);

        // Output detected processes
        for process in &filtered_processes {
            output_process_detection(process, &config)?;
        }

        // Calculate sleep time to maintain interval
        let cycle_duration = cycle_start.elapsed();
        if let Some(sleep_duration) = config.interval.checked_sub(cycle_duration) {
            std::thread::sleep(sleep_duration);
        }
    }

    if !config.quiet_mode {
        println!("Monitoring stopped.");
    }

    Ok(())
}

fn create_process_snapshot(system: &System) -> Result<ProcessSnapshot> {
    let timestamp = SystemTime::now();
    let scan_start = Instant::now();
    
    let mut processes = HashMap::new();

    for (pid, process) in system.processes() {
        // Extract basic process information only (entitlements extracted later for new processes)
        let name = process.name().to_string();
        let executable_path = process.exe().to_path_buf();

        let monitored_process = MonitoredProcess {
            pid: pid.as_u32(),
            name,
            executable_path,
            entitlements: vec![], // Will be populated later for new processes only
            discovery_timestamp: timestamp,
        };

        processes.insert(pid.as_u32(), monitored_process);
    }

    Ok(ProcessSnapshot {
        processes,
        timestamp,
        scan_duration: scan_start.elapsed(),
    })
}

fn extract_process_entitlements(executable_path: &std::path::Path) -> Result<Vec<String>> {
    // Reuse existing entitlement extraction logic
    match crate::entitlements::extract_entitlements(executable_path) {
        Ok(entitlements_map) => Ok(entitlements_map.keys().cloned().collect()),
        Err(e) => Err(e),
    }
}

fn apply_filters(
    processes: Vec<MonitoredProcess>,
    config: &PollingConfiguration,
) -> Result<Vec<MonitoredProcess>> {
    let mut filtered = processes;

    // Filter out processes with no entitlements (reduce noise)
    filtered = filtered
        .into_iter()
        .filter(|process| !process.entitlements.is_empty())
        .collect();

    // Apply path filters
    filtered = ProcessTracker::apply_path_filters(filtered, &config.path_filters);

    // Apply entitlement filters
    filtered = ProcessTracker::apply_entitlement_filters(filtered, &config.entitlement_filters);

    Ok(filtered)
}

fn output_process_detection(process: &MonitoredProcess, config: &PollingConfiguration) -> Result<()> {
    if config.output_json {
        output_json_format(process)?;
    } else {
        output_human_format(process)?;
    }

    // Note: Unified logging is disabled for interactive monitoring to avoid duplicate output.
    // When daemon mode is implemented, unified logging will be used there instead.
    
    Ok(())
}

fn output_human_format(process: &MonitoredProcess) -> Result<()> {
    use time::OffsetDateTime;
    
    let timestamp = OffsetDateTime::from(process.discovery_timestamp);
    let timestamp_str = timestamp.format(&time::format_description::well_known::Iso8601::DEFAULT)?;

    println!("[{}] New process detected: {} (PID: {})", 
        timestamp_str, process.name, process.pid);
    println!("  Path: {}", process.executable_path.display());
    
    if process.entitlements.is_empty() {
        println!("  Entitlements: (none)");
    } else {
        println!("  Entitlements: {}", process.entitlements.join(", "));
    }
    println!();

    Ok(())
}

fn output_json_format(process: &MonitoredProcess) -> Result<()> {
    use time::OffsetDateTime;
    
    let timestamp = OffsetDateTime::from(process.discovery_timestamp);
    let timestamp_str = timestamp.format(&time::format_description::well_known::Iso8601::DEFAULT)?;

    let json_output = serde_json::json!({
        "timestamp": timestamp_str,
        "event_type": "process_detected",
        "process": {
            "pid": process.pid,
            "name": process.name,
            "path": process.executable_path.display().to_string(),
            "entitlements": process.entitlements
        }
    });

    println!("{}", json_output);
    Ok(())
}