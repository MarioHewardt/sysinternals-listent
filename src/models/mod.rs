//! Data models module
//!
//! Defines core data structures:
//! - BinaryRecord: Discovered executable metadata
//! - EntitlementSet: Parsed entitlement key-value pairs
//! - ScanResult: Successful entitlement enumeration 
//! - ScanSummary: Aggregated scan statistics
//!
//! Implements invariants and validation rules per data-model.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use crate::constants::{MIN_POLLING_INTERVAL, MAX_POLLING_INTERVAL};

/// Represents a single binary file with its entitlements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryResult {
    /// Absolute path to the binary file
    pub path: String,
    /// Entitlements found in the binary (key-value pairs)
    pub entitlements: HashMap<String, serde_json::Value>,
    /// Count of entitlements for quick reference
    pub entitlement_count: usize,
}

/// Summary statistics for the scan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    /// Total number of files scanned
    pub scanned: usize,
    /// Number of files that matched filters and had entitlements
    pub matched: usize,
    /// Number of files that couldn't be read due to permissions
    pub skipped_unreadable: usize,
    /// Duration of the scan in milliseconds
    pub duration_ms: u64,
    /// Whether the scan was interrupted by user signal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupted: Option<bool>,
}

/// Complete output structure for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementScanOutput {
    /// Array of binary results
    pub results: Vec<BinaryResult>,
    /// Summary statistics
    pub summary: ScanSummary,
}

/// Filter criteria for scanning operations
#[derive(Debug, Clone, Default)]
pub struct ScanFilters {
    /// Filter by specific entitlement keys
    pub entitlements: Vec<String>,
}

/// Configuration for the scan operation
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Base directories to scan (defaults to system app directories)
    pub scan_paths: Vec<String>,
    /// Filter criteria
    pub filters: ScanFilters,
    /// Whether to output JSON format
    pub json_output: bool,
    /// Whether to run in quiet mode (suppress warnings)
    pub quiet_mode: bool,
}

// TODO: Implement data structures per data-model.md specification

//
// Monitor-specific data structures (T012-T015)
//

/// Represents a monitored process and its entitlements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredProcess {
    /// Process ID (PID)
    pub pid: u32,
    /// Process name (executable name)
    pub name: String,
    /// Full path to the executable
    pub executable_path: PathBuf,
    /// Entitlements found in the process executable
    pub entitlements: Vec<String>,
    /// Timestamp when this process was first discovered
    pub discovery_timestamp: SystemTime,
}

/// Configuration for polling behavior in monitor mode
#[derive(Debug, Clone)]
pub struct PollingConfiguration {
    /// Polling interval
    pub interval: Duration,
    /// Path filters for process monitoring
    pub path_filters: Vec<PathBuf>,
    /// Entitlement filters for process monitoring
    pub entitlement_filters: Vec<String>,
    /// Whether to output JSON format
    pub output_json: bool,
    /// Whether to run in quiet mode
    pub quiet_mode: bool,
}

/// Snapshot of process state at a given moment
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    /// HashMap of PID -> MonitoredProcess for O(1) lookups
    pub processes: HashMap<u32, MonitoredProcess>,
}

/// Represents an entitlement match for filtering
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementMatch {
    /// The entitlement key that matched
    pub key: String,
    /// The full entitlement key (may include wildcards)
    pub pattern: String,
    /// Whether this was an exact match or pattern match
    pub exact_match: bool,
}

/// Log entry for Unified Logging output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log level
    pub level: String,
    /// Log message
    pub message: String,
    /// Timestamp of the log entry
    pub timestamp: SystemTime,
    /// Process context (optional)
    pub process_context: Option<MonitoredProcess>,
}

/// Custom error types for monitoring operations
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Invalid polling interval: {0}. Must be between {} and {} seconds", MIN_POLLING_INTERVAL, MAX_POLLING_INTERVAL)]
    InvalidInterval(f64),
}

impl ProcessSnapshot {
    /// Returns processes that are in this snapshot but not in the previous one
    pub fn new_processes(&self, previous: &ProcessSnapshot) -> Vec<MonitoredProcess> {
        self.processes
            .values()
            .filter(|process| !previous.processes.contains_key(&process.pid))
            .cloned()
            .collect()
    }
}

impl PollingConfiguration {
}

#[cfg(test)]
mod tests;