//! Enhanced ULS logging for daemon process detection
//!
//! Provides structured logging to macOS Unified Logging System

use anyhow::{Context, Result};
use serde_json::json;
use oslog::OsLogger;
use log::{error, warn, info, debug};
use std::process::Command;

/// Enhanced daemon logger for macOS ULS integration
#[derive(Debug, Clone)]
pub struct DaemonLogger {
    /// macOS ULS subsystem identifier
    subsystem: String,
    /// ULS category for organizing logs
    category: String,
    /// Current logging level
    level: LogLevel,
    /// Unit value - we use log crate macros instead
    logger: (),
}

/// Log levels for daemon operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    /// Convert to ULS log level string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warning", 
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        }
    }

    /// Parse log level from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => anyhow::bail!("Invalid log level: {}", s),
        }
    }
}

impl DaemonLogger {
    /// Initialize daemon logger with ULS subsystem and category
    pub fn new(subsystem: String, category: String, level: LogLevel) -> Result<Self> {
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
            subsystem,
            category,
            level,
            logger: (), // We'll use log macros instead
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

    /// Log configuration change event
    pub fn log_config_change(&self, change_description: &str, old_value: &str, new_value: &str) -> Result<()> {
        let message = json!({
            "event": "config_change",
            "description": change_description,
            "old_value": old_value,
            "new_value": new_value,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.log_structured(LogLevel::Info, "Configuration updated", &message)
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

        self.log_structured(LogLevel::Info, &format!("New process detected: {}", process_name), &message)
    }

    /// Log IPC communication events
    pub fn log_ipc_request(&self, request_type: &str, client_info: &str) -> Result<()> {
        let message = json!({
            "event": "ipc_request",
            "request_type": request_type,
            "client": client_info,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.log_structured(LogLevel::Debug, &format!("IPC request: {}", request_type), &message)
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

    /// Log warning events
    pub fn log_warning(&self, warning_message: &str, context: Option<&str>) -> Result<()> {
        let message = json!({
            "event": "warning",
            "message": warning_message,
            "context": context,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.log_structured(LogLevel::Warn, warning_message, &message)
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
            LogLevel::Warn => {
                warn!("{}", full_message);
            },
            LogLevel::Info => {
                info!("{}", full_message);
            },
            LogLevel::Debug => {
                debug!("{}", full_message);
            },
        }
        
        Ok(())
    }

    /// Check if we should log at this level
    fn should_log(&self, level: LogLevel) -> bool {
        match (self.level, level) {
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Warn, LogLevel::Error | LogLevel::Warn) => true,
            (LogLevel::Info, LogLevel::Error | LogLevel::Warn | LogLevel::Info) => true,
            (LogLevel::Debug, _) => true,
            _ => false,
        }
    }

    /// Get current log level
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Set log level
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }
}

/// Helper function to retrieve daemon logs using `log show`
/// Returns structured JSON logs from the past specified duration
pub fn get_daemon_logs(subsystem: &str, since: &str) -> Result<Vec<String>> {
    let output = Command::new("log")
        .args([
            "show",
            "--predicate",
            &format!("process == \"logger\" AND messageText CONTAINS \"{}\"", subsystem),
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

/// Log error and return formatted error
pub fn log_and_bail(logger: &DaemonLogger, message: &str) -> anyhow::Error {
    if let Err(e) = logger.log_error(message, None) {
        eprintln!("Failed to log error: {}", e);
    }
    anyhow::anyhow!(message.to_string())
}