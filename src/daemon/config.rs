//! Configuration management for daemon mode
//!
//! Handles TOML configuration parsing, validation, and atomic updates

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main daemon configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfiguration {
    pub daemon: DaemonSettings,
    pub monitoring: MonitoringSettings,
}

/// Core daemon runtime settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    /// Polling interval in seconds (0.1-300.0)
    pub polling_interval: f64,
    /// Whether daemon should auto-start with launchd
    pub auto_start: bool,
}

/// Process monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    /// Filesystem paths to monitor for processes
    pub path_filters: Vec<PathBuf>,
    /// Entitlements to filter for (empty = all)
    pub entitlement_filters: Vec<String>,
}