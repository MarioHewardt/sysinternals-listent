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