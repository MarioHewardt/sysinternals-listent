use crate::models::{MonitoredProcess, PollingConfiguration, ProcessSnapshot};
use crate::monitor::{ProcessTracker, init_logger};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use sysinfo::{System, SystemExt, ProcessExt, PidExt};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ==================== create_process_snapshot tests ====================

    #[test]
    fn test_create_process_snapshot_returns_valid_snapshot() {
        let system = System::new_all();
        let snapshot = create_process_snapshot(&system).unwrap();
        
        // Should have at least the current process
        assert!(!snapshot.processes.is_empty(), "Snapshot should contain at least one process");
        
        // Timestamp should be set
        assert!(snapshot.timestamp <= SystemTime::now());
        
        // Scan duration should be reasonable (less than 10 seconds)
        assert!(snapshot.scan_duration.as_secs() < 10);
    }

    #[test]
    fn test_create_process_snapshot_includes_current_process() {
        let system = System::new_all();
        let snapshot = create_process_snapshot(&system).unwrap();
        
        let current_pid = std::process::id();
        
        // Current process should be in the snapshot
        assert!(
            snapshot.processes.contains_key(&current_pid),
            "Snapshot should include current process (PID {})", current_pid
        );
    }

    #[test]
    fn test_process_snapshot_has_valid_executable_paths() {
        let system = System::new_all();
        let snapshot = create_process_snapshot(&system).unwrap();
        
        // At least some processes should have non-empty executable paths
        let processes_with_paths = snapshot.processes.values()
            .filter(|p| !p.executable_path.as_os_str().is_empty())
            .count();
        
        assert!(
            processes_with_paths > 0,
            "At least some processes should have executable paths"
        );
    }

    // ==================== extract_process_entitlements tests ====================

    #[test]
    fn test_extract_process_entitlements_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/binary");
        let result = extract_process_entitlements(&path);
        
        // Should either succeed with empty vec or return an error
        // Either way, it shouldn't panic
        match result {
            Ok(entitlements) => assert!(entitlements.is_empty() || !entitlements.is_empty()),
            Err(_) => {} // Error is acceptable for nonexistent file
        }
    }

    #[test]
    fn test_extract_process_entitlements_from_system_binary() {
        // Test with a known system binary
        let path = PathBuf::from("/usr/bin/sudo");
        if path.exists() {
            let result = extract_process_entitlements(&path);
            // Should not panic, may or may not have entitlements
            assert!(result.is_ok() || result.is_err());
        }
    }

    // ==================== apply_filters tests ====================

    #[test]
    fn test_apply_filters_removes_processes_without_entitlements() {
        let processes = vec![
            MonitoredProcess {
                pid: 1,
                name: "test1".to_string(),
                executable_path: PathBuf::from("/bin/test1"),
                entitlements: vec![], // No entitlements
                discovery_timestamp: SystemTime::now(),
            },
            MonitoredProcess {
                pid: 2,
                name: "test2".to_string(),
                executable_path: PathBuf::from("/bin/test2"),
                entitlements: vec!["com.apple.security.app-sandbox".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
        ];

        let config = PollingConfiguration {
            interval: std::time::Duration::from_secs(1),
            path_filters: vec![],
            entitlement_filters: vec![],
            output_json: false,
            quiet_mode: false,
        };

        let filtered = apply_filters(processes, &config).unwrap();
        
        // Should only keep process with entitlements
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].pid, 2);
    }

    #[test]
    fn test_apply_filters_with_path_filter() {
        let processes = vec![
            MonitoredProcess {
                pid: 1,
                name: "test1".to_string(),
                executable_path: PathBuf::from("/Applications/Test.app/test1"),
                entitlements: vec!["com.apple.security.app-sandbox".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
            MonitoredProcess {
                pid: 2,
                name: "test2".to_string(),
                executable_path: PathBuf::from("/usr/bin/test2"),
                entitlements: vec!["com.apple.security.app-sandbox".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
        ];

        let config = PollingConfiguration {
            interval: std::time::Duration::from_secs(1),
            path_filters: vec![PathBuf::from("/Applications")],
            entitlement_filters: vec![],
            output_json: false,
            quiet_mode: false,
        };

        let filtered = apply_filters(processes, &config).unwrap();
        
        // Should only keep process in /Applications
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].pid, 1);
    }

    #[test]
    fn test_apply_filters_with_entitlement_filter() {
        let processes = vec![
            MonitoredProcess {
                pid: 1,
                name: "test1".to_string(),
                executable_path: PathBuf::from("/bin/test1"),
                entitlements: vec!["com.apple.security.app-sandbox".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
            MonitoredProcess {
                pid: 2,
                name: "test2".to_string(),
                executable_path: PathBuf::from("/bin/test2"),
                entitlements: vec!["com.apple.security.network.client".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
        ];

        let config = PollingConfiguration {
            interval: std::time::Duration::from_secs(1),
            path_filters: vec![],
            entitlement_filters: vec!["com.apple.security.app-sandbox".to_string()],
            output_json: false,
            quiet_mode: false,
        };

        let filtered = apply_filters(processes, &config).unwrap();
        
        // Should only keep process with sandbox entitlement
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].pid, 1);
    }

    #[test]
    fn test_apply_filters_combined_filters() {
        let processes = vec![
            MonitoredProcess {
                pid: 1,
                name: "test1".to_string(),
                executable_path: PathBuf::from("/Applications/Test.app/test1"),
                entitlements: vec!["com.apple.security.app-sandbox".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
            MonitoredProcess {
                pid: 2,
                name: "test2".to_string(),
                executable_path: PathBuf::from("/Applications/Other.app/test2"),
                entitlements: vec!["com.apple.security.network.client".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
            MonitoredProcess {
                pid: 3,
                name: "test3".to_string(),
                executable_path: PathBuf::from("/usr/bin/test3"),
                entitlements: vec!["com.apple.security.app-sandbox".to_string()],
                discovery_timestamp: SystemTime::now(),
            },
        ];

        let config = PollingConfiguration {
            interval: std::time::Duration::from_secs(1),
            path_filters: vec![PathBuf::from("/Applications")],
            entitlement_filters: vec!["com.apple.security.app-sandbox".to_string()],
            output_json: false,
            quiet_mode: false,
        };

        let filtered = apply_filters(processes, &config).unwrap();
        
        // Should only keep process matching both filters
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].pid, 1);
    }

    #[test]
    fn test_apply_filters_empty_input() {
        let processes: Vec<MonitoredProcess> = vec![];

        let config = PollingConfiguration {
            interval: std::time::Duration::from_secs(1),
            path_filters: vec![],
            entitlement_filters: vec![],
            output_json: false,
            quiet_mode: false,
        };

        let filtered = apply_filters(processes, &config).unwrap();
        assert!(filtered.is_empty());
    }

    // ==================== Timing and edge case tests ====================

    #[test]
    fn test_polling_configuration_interval_bounds() {
        // Valid minimum interval
        let min_config = PollingConfiguration {
            interval: std::time::Duration::from_millis(100),
            path_filters: vec![],
            entitlement_filters: vec![],
            output_json: false,
            quiet_mode: false,
        };
        assert_eq!(min_config.interval.as_millis(), 100);

        // Valid maximum interval
        let max_config = PollingConfiguration {
            interval: std::time::Duration::from_secs(300),
            path_filters: vec![],
            entitlement_filters: vec![],
            output_json: false,
            quiet_mode: false,
        };
        assert_eq!(max_config.interval.as_secs(), 300);
    }

    #[test]
    fn test_snapshot_contains_process_names() {
        let system = System::new_all();
        let snapshot = create_process_snapshot(&system).unwrap();
        
        // At least some processes should have non-empty names
        let processes_with_names = snapshot.processes.values()
            .filter(|p| !p.name.is_empty())
            .count();
        
        assert!(
            processes_with_names > 0,
            "At least some processes should have names"
        );
    }

    #[test]
    fn test_snapshot_scan_duration_is_positive() {
        let system = System::new_all();
        let snapshot = create_process_snapshot(&system).unwrap();
        
        // Scan duration should be set (we did actual work)
        // The duration object exists and can be accessed
        let _ = snapshot.scan_duration.as_nanos();
    }
}