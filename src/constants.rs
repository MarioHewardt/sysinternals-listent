//! Global constants for listent
//!
//! Centralized location for application-wide constants

/// Application subsystem identifier for macOS Unified Logging System
/// Used for LaunchD service name, ULS logging, and daemon identification
pub const APP_SUBSYSTEM: &str = "com.microsoft.sysinternals.listent";

/// Default daemon category for ULS logging
pub const DAEMON_CATEGORY: &str = "daemon";

/// LaunchD plist file name
pub const LAUNCHD_PLIST_NAME: &str = "com.microsoft.sysinternals.listent.plist";

/// LaunchD service name (same as subsystem)
pub const LAUNCHD_SERVICE_NAME: &str = APP_SUBSYSTEM;

// Monitoring interval bounds
/// Minimum allowed polling interval in seconds
pub const MIN_POLLING_INTERVAL: f64 = 0.1;

/// Maximum allowed polling interval in seconds  
pub const MAX_POLLING_INTERVAL: f64 = 300.0;

// Default scan paths
/// Default directories to scan if no paths are provided
pub const DEFAULT_SCAN_PATHS: &[&str] = &[
    "/Applications",
];

// Error message formatting for consistency
/// Format a permission error with actionable guidance
pub fn format_permission_error(resource: &str, action: &str) -> String {
    format!("Cannot {} {}. Try running with appropriate permissions.", action, resource)
}

/// Format a validation error with specific field information
pub fn format_validation_error(field: &str, value: &str, reason: &str) -> String {
    format!("Invalid {} '{}': {}", field, value, reason)
}