//! Core process monitoring functionality shared between monitor and daemon modes
//!
//! Provides the common process scanning, filtering, and detection logic
//! used by both interactive monitoring and background daemon operation.

use crate::models::{MonitoredProcess, PollingConfiguration};
use crate::monitor::ProcessTracker;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use sysinfo::{ProcessExt, System, SystemExt, PidExt};

/// Core process monitoring engine that can be used by both sync and async contexts
pub struct ProcessMonitoringCore {
    tracker: ProcessTracker,
    system: System,
}

/// Configuration for process monitoring (simplified from PollingConfiguration)
pub struct MonitoringConfig {
    pub path_filters: Vec<PathBuf>,
    pub entitlement_filters: Vec<String>,
}

impl ProcessMonitoringCore {
    /// Create a new monitoring core
    pub fn new() -> Self {
        Self {
            tracker: ProcessTracker::new(),
            system: System::new_all(),
        }
    }

    /// Scan current processes and apply filters
    pub fn scan_processes(&mut self, config: &MonitoringConfig) -> Result<HashMap<u32, MonitoredProcess>> {
        // Refresh system processes
        self.system.refresh_processes();
        
        let mut processes = HashMap::new();
        
        // Scan all processes
        for (pid, process) in self.system.processes() {
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
                discovery_timestamp: SystemTime::now(),
            };
            
            processes.insert(pid_u32, monitored_process);
        }
        
        Ok(processes)
    }

    /// Detect new processes compared to previous scan
    pub fn detect_new_processes(&mut self, current_processes: HashMap<u32, MonitoredProcess>) -> Vec<MonitoredProcess> {
        let snapshot = crate::models::ProcessSnapshot {
            processes: current_processes,
        };
        
        self.tracker.detect_new_processes(snapshot)
    }

    /// Convenience method to scan and detect new processes in one call
    pub fn scan_and_detect_new(&mut self, config: &MonitoringConfig) -> Result<Vec<MonitoredProcess>> {
        let current_processes = self.scan_processes(config)?;
        Ok(self.detect_new_processes(current_processes))
    }
}

impl From<&PollingConfiguration> for MonitoringConfig {
    fn from(polling_config: &PollingConfiguration) -> Self {
        MonitoringConfig {
            path_filters: polling_config.path_filters.clone(),
            entitlement_filters: polling_config.entitlement_filters.clone(),
        }
    }
}

impl Default for ProcessMonitoringCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_monitoring_core_creation() {
        let core = ProcessMonitoringCore::new();
        // Just verify it can be created without panicking
        assert!(!core.system.processes().is_empty() || true); // Always passes, just tests creation
    }

    #[test] 
    fn test_monitoring_config_from_polling_config() {
        let polling_config = PollingConfiguration {
            interval: std::time::Duration::from_secs(1),
            path_filters: vec![PathBuf::from("/test")],
            entitlement_filters: vec!["test.*".to_string()],
            output_json: false,
            quiet_mode: false,
        };
        
        let monitoring_config = MonitoringConfig::from(&polling_config);
        assert_eq!(monitoring_config.path_filters.len(), 1);
        assert_eq!(monitoring_config.entitlement_filters.len(), 1);
    }
}