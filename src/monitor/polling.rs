use crate::models::{MonitoredProcess, PollingConfiguration};
use crate::monitor::{ProcessTracker, init_logger, ProcessMonitoringCore, MonitoringConfig};
use anyhow::Result;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Instant, Duration};

/// Start monitoring processes with external interrupt flag (called from main.rs)
pub fn start_monitoring_with_interrupt(config: PollingConfiguration, interrupted: Arc<AtomicBool>) -> Result<()> {
    // Convert interrupted (false = continue) to running (true = continue)
    let running = Arc::new(AtomicBool::new(true));
    
    // Create a thread to monitor the interrupted flag and update running
    let running_monitor = running.clone();
    std::thread::spawn(move || {
        while !interrupted.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        running_monitor.store(false, Ordering::SeqCst);
    });

    start_monitoring_internal(config, running)
}

/// Internal monitoring implementation
fn start_monitoring_internal(config: PollingConfiguration, running: Arc<AtomicBool>) -> Result<()> {

    // Initialize unified logging
    let _logger = init_logger().ok(); // Graceful degradation if logging fails

    // Initialize shared process monitoring core
    let mut monitoring_core = ProcessMonitoringCore::new();
    let monitoring_config = MonitoringConfig::from(&config);
    
    // Pre-allocate collections to reduce allocations in the loop
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

        // Use shared monitoring core to scan and detect new processes
        let new_processes = match monitoring_core.scan_and_detect_new(&monitoring_config) {
            Ok(processes) => processes,
            Err(e) => {
                if !config.quiet_mode {
                    eprintln!("Warning: Failed to scan processes: {}", e);
                }
                Vec::new()
            }
        };

        // Apply additional filters (reuse vector to avoid allocations) 
        filtered_processes.clear();
        filtered_processes.extend(apply_filters(new_processes, &config)?);

        // Output detected processes
        for process in &filtered_processes {
            output_process_detection(process, &config)?;
        }

        // Calculate sleep time to maintain interval
        let cycle_duration = cycle_start.elapsed();
        if let Some(sleep_duration) = config.interval.checked_sub(cycle_duration) {
            // Break sleep into small chunks to ensure responsive signal handling
            let sleep_chunk = Duration::from_millis(100);
            let mut remaining = sleep_duration;
            
            while remaining > Duration::ZERO && running.load(Ordering::SeqCst) {
                let sleep_time = std::cmp::min(remaining, sleep_chunk);
                std::thread::sleep(sleep_time);
                remaining = remaining.saturating_sub(sleep_time);
            }
        }
    }

    if !config.quiet_mode {
        println!("Monitoring stopped.");
    }

    Ok(())
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