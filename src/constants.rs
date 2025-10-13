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
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_permission_error() {
        let result = format_permission_error("/var/log/test.log", "write to");
        assert_eq!(result, "Cannot write to /var/log/test.log. Try running with appropriate permissions.");
        
        let result = format_permission_error("system directory", "access");
        assert_eq!(result, "Cannot access system directory. Try running with appropriate permissions.");
    }
    
    #[test]
    fn test_format_validation_error() {
        let result = format_validation_error("interval", "500", "must be between 0.1 and 300");
        assert_eq!(result, "Invalid interval '500': must be between 0.1 and 300");
        
        let result = format_validation_error("path", "/invalid/path", "does not exist");
        assert_eq!(result, "Invalid path '/invalid/path': does not exist");
    }
    
    #[test]
    fn test_app_subsystem_constant() {
        assert_eq!(APP_SUBSYSTEM, "com.microsoft.sysinternals.listent");
        assert_eq!(LAUNCHD_SERVICE_NAME, APP_SUBSYSTEM);
    }
    
    #[test]
    fn test_interval_bounds() {
        assert_eq!(MIN_POLLING_INTERVAL, 0.1);
        assert_eq!(MAX_POLLING_INTERVAL, 300.0);
        assert!(MIN_POLLING_INTERVAL < MAX_POLLING_INTERVAL);
    }
    
    #[test]
    fn test_default_scan_paths() {
        assert_eq!(DEFAULT_SCAN_PATHS.len(), 1);
        assert_eq!(DEFAULT_SCAN_PATHS[0], "/Applications");
    }
}
