//! Enhanced ULS logging for daemon process detection
//!
//! Provides structured logging to macOS Unified Logging System

use anyhow::{Context, Result};
use serde_json::json;
use oslog::OsLogger;
use log::{error, info};
use std::process::Command;

/// Enhanced daemon logger for macOS ULS integration
#[derive(Debug, Clone)]
pub struct DaemonLogger {
    /// Current logging level
    level: LogLevel,
}

/// Log levels for daemon operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Info,
}

impl DaemonLogger {
    /// Initialize daemon logger with ULS subsystem and category
    pub fn new(subsystem: String, _category: String, level: LogLevel) -> Result<Self> {
        // Validate subsystem format (should be reverse DNS)
        if !subsystem.contains('.') {
            anyhow::bail!("Subsystem must be in reverse DNS format (e.g., 'com.example.app')");
        }

        // Initialize the oslog backend for the log crate
        let logger = OsLogger::new(&subsystem);
        log::set_boxed_logger(Box::new(logger))
            .map_err(|e| anyhow::anyhow!("Failed to set logger: {}", e))?;
        log::set_max_level(log::LevelFilter::Debug);

        Ok(Self {
            level,
        })
    }

    /// Log daemon startup event
    pub fn log_startup(&self, config_path: &std::path::Path, pid: u32) -> Result<()> {
        let message = json!({
            "event": "daemon_startup",
            "pid": pid,
            "config_path": config_path.display().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.log_structured(LogLevel::Info, "Daemon started", &message)
    }

    /// Log daemon shutdown event
    pub fn log_shutdown(&self, reason: &str) -> Result<()> {
        let message = json!({
            "event": "daemon_shutdown", 
            "reason": reason,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.log_structured(LogLevel::Info, "Daemon shutting down", &message)
    }

    /// Log process detection events
    pub fn log_process_detection(&self, pid: u32, process_name: &str, executable_path: &str, entitlements: &[String]) -> Result<()> {
        let message = json!({
            "event": "process_detected",
            "pid": pid,
            "process_name": process_name,
            "executable_path": executable_path,
            "entitlement_count": entitlements.len(),
            "entitlements": entitlements,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.log_structured(LogLevel::Info, &format!("New process detected: {}", executable_path), &message)
    }

    /// Log error events
    pub fn log_error(&self, error_message: &str, context: Option<&str>) -> Result<()> {
        let message = json!({
            "event": "error",
            "message": error_message,
            "context": context,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.log_structured(LogLevel::Error, error_message, &message)
    }

    /// Send structured log message to ULS
    fn log_structured(&self, level: LogLevel, message: &str, data: &serde_json::Value) -> Result<()> {
        // Skip logging if below configured level
        if !self.should_log(level) {
            return Ok(());
        }

        // Format the complete log message with structured data
        let full_message = format!("{} | {}", message, data.to_string());
        
        // Use log crate macros which will go to ULS
        match level {
            LogLevel::Error => {
                error!("{}", full_message);
            },
            LogLevel::Info => {
                info!("{}", full_message);
            },
        }
        
        Ok(())
    }

    /// Check if we should log at this level
    fn should_log(&self, level: LogLevel) -> bool {
        match (self.level, level) {
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Info, LogLevel::Error | LogLevel::Info) => true,
            _ => false,
        }
    }
}

/// Helper function to retrieve daemon logs using `log show`
/// Returns structured JSON logs from the past specified duration
pub fn get_daemon_logs(subsystem: &str, since: &str) -> Result<Vec<String>> {
    let output = Command::new("log")
        .args([
            "show",
            "--predicate",
            &format!("subsystem == \"{}\"", subsystem),
            "--last",
            since,
            "--style",
            "compact",
        ])
        .output()
        .context("Failed to execute log show command")?;

    if !output.status.success() {
        anyhow::bail!(
            "log show command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in log output")?;

    Ok(output_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect())
}