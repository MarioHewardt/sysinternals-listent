//! Enhanced Unified Logging System (ULS) integrat    /// Initialize daemon logger with ULS subsystem and category
    pub fn new(subsystem: String, category: String, level: LogLevel) -> Result<Self> {
        // Validate subsystem format (should be reverse DNS)
        if !subsystem.contains('.') {
            anyhow::bail!("Subsystem must be in reverse DNS format (e.g., 'com.example.app')");
        }

        // Initialize macOS oslog logger
        let os_logger = OsLogger::new(&subsystem, &category)
            .context("Failed to initialize macOS Unified Logging System logger")?;

        Ok(Self {
            subsystem,
            category,
            level,
            os_logger,
        })
    }n mode
//!
//! Provides structured logging specifically for daemon operations

use anyhow::{Context, Result};
use std::process::Command;
use serde_json::json;
use oslog::{OsLogger, Level as OsLogLevel};

/// Enhanced ULS logger for daemon mode
pub struct DaemonLogger {
    pub subsystem: String,
    pub category: String,
    level: LogLevel,
    os_logger: OsLogger,
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
            LogLevel::Warn => "default",  // ULS uses "default" for warning level
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        }
    }

    /// Parse from string
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

        Ok(Self {
            subsystem,
            category,
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

        self.log_structured(LogLevel::Info, "Daemon shutdown", &message)
    }

    /// Log configuration changes
    pub fn log_config_change(&self, change_description: &str, old_value: Option<&str>, new_value: &str) -> Result<()> {
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

        // Convert our LogLevel to OsLogLevel
        let os_level = match level {
            LogLevel::Error => OsLogLevel::Error,
            LogLevel::Warn => OsLogLevel::Default, // oslog doesn't have warn, use default
            LogLevel::Info => OsLogLevel::Info,
            LogLevel::Debug => OsLogLevel::Debug,
        };

        // Format the complete log message with structured data
        let full_message = format!("{} | {}", message, data.to_string());
        
        // Send to macOS Unified Logging System
        self.os_logger.log(os_level, &full_message);
        
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
pub fn retrieve_daemon_logs(
    subsystem: &str,
    category: &str,
    since: Option<&str>,
    follow: bool,
) -> Result<Vec<String>> {
    let mut args = vec![
        "show".to_string(),
        "--predicate".to_string(),
        format!("subsystem == '{}' AND category == '{}'", subsystem, category),
        "--style".to_string(),
        "compact".to_string(),
    ];

    // Add time filter if specified
    if let Some(since_time) = since {
        args.push("--last".to_string());
        args.push(since_time.to_string());
    }

    // Add follow mode if requested
    if follow {
        args.push("--follow".to_string());
    }

    let output = Command::new("log")
        .args(&args)
        .output()
        .context("Failed to execute log show command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to retrieve logs: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}