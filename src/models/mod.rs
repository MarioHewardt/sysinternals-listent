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