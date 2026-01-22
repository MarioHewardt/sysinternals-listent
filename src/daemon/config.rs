//! Configuration management for daemon mode
//!
//! Handles TOML configuration parsing, validation, and atomic updates

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

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

impl DaemonConfiguration {
    /// Load configuration from TOML file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: DaemonConfiguration = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to TOML file atomically
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        self.validate()?;
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        
        // Atomic write using temporary file
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content)
            .with_context(|| format!("Failed to write temp config: {}", temp_path.display()))?;
        
        std::fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to replace config file: {}", path.display()))?;
        
        Ok(())
    }

    /// Validate all configuration settings
    pub fn validate(&self) -> Result<()> {
        // Validate polling interval
        if self.daemon.polling_interval < 0.1 || self.daemon.polling_interval > 300.0 {
            anyhow::bail!(
                "Invalid polling interval: {}. Must be between 0.1 and 300.0 seconds",
                self.daemon.polling_interval
            );
        }

        // Validate paths exist and are readable
        for path in &self.monitoring.path_filters {
            if !path.exists() {
                anyhow::bail!("Monitoring path does not exist: {}", path.display());
            }
        }

        Ok(())
    }

    /// Create default daemon configuration
    pub fn default() -> Self {
        Self {
            daemon: DaemonSettings {
                polling_interval: 1.0,
                auto_start: true,
            },
            monitoring: MonitoringSettings {
                path_filters: vec![
                    PathBuf::from("/Applications"),
                    PathBuf::from("/usr/bin"),
                    PathBuf::from("/bin"),
                ],
                entitlement_filters: vec![], // Monitor all entitlements by default
            },
        }
    }

    /// Get polling interval as Duration
    pub fn polling_duration(&self) -> Duration {
        Duration::from_secs_f64(self.daemon.polling_interval)
    }

    /// Update configuration field by dot-notation key
    pub fn update_field(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "daemon.polling_interval" => {
                let interval: f64 = value.parse()
                    .with_context(|| format!("Invalid polling interval: {}", value))?;
                if interval < 0.1 || interval > 300.0 {
                    anyhow::bail!("Polling interval must be between 0.1 and 300.0 seconds");
                }
                self.daemon.polling_interval = interval;
            },
            "daemon.auto_start" => {
                self.daemon.auto_start = value.parse()
                    .with_context(|| format!("Invalid boolean value: {}", value))?;
            },
            _ => {
                anyhow::bail!("Unknown configuration key: {}", key);
            }
        }
        Ok(())
    }

    /// Apply multiple configuration updates atomically
    pub fn apply_updates(&mut self, updates: &[(String, String)]) -> Result<()> {
        // Create a backup for rollback
        let backup = self.clone();
        
        // Apply all updates
        for (key, value) in updates {
            if let Err(e) = self.update_field(key, value) {
                // Rollback on any error
                *self = backup;
                return Err(e.context(format!("Failed to apply update {}={}", key, value)));
            }
        }
        
        // Validate the final configuration
        if let Err(e) = self.validate() {
            // Rollback on validation failure
            *self = backup;
            return Err(e.context("Configuration validation failed after updates"));
        }
        
        Ok(())
    }

    /// Create configuration directories if they don't exist
    pub fn ensure_directories(&self) -> Result<()> {
        // No directories needed currently - config files are created on demand
        Ok(())
    }

    /// Get default configuration file path
    pub fn default_config_path() -> Result<PathBuf> {
        Ok(PathBuf::from("/etc/listent/daemon.toml"))
    }

    /// Get user-specific configuration file path
    pub fn user_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .context("Could not determine home directory")?;
        Ok(home_dir.join(".config/listent/daemon.toml"))
    }
}