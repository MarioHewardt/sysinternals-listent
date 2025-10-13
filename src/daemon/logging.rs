//! Enhanced ULS logging for daemon process detection

use anyhow::Result;
use std::path::PathBuf;
use crate::constants::APP_SUBSYSTEM;
use oslog::OsLog;

/// Enhanced daemon logger for macOS ULS integration
pub struct DaemonLogger {
    /// ULS logger instance
    logger: OsLog,
}

impl DaemonLogger {
    /// Create a new DaemonLogger instance with APP_SUBSYSTEM
    pub fn new(category: String) -> Result<Self> {
        let logger = OsLog::new(APP_SUBSYSTEM, &category);
        Ok(Self {
            logger,
        })
    }

    /// Log daemon startup with configuration
    pub fn log_startup_with_args(
        &self,
        interval: f64,
        paths: &[PathBuf],
        entitlements: &[String],
        pid: u32,
    ) -> Result<()> {
        let paths_str = paths.iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let entitlements_str = entitlements.join(", ");
        
        let message = format!(
            "Daemon started: pid={} interval={}s paths=[{}] entitlements=[{}]",
            pid, interval, paths_str, entitlements_str
        );
        self.logger.info(&message);
        Ok(())
    }

    /// Log daemon shutdown
    pub fn log_shutdown(&self, message: &str) -> Result<()> {
        let log_message = format!("Daemon shutdown: {}", message);
        self.logger.info(&log_message);
        Ok(())
    }

    /// Log error message
    pub fn log_error(&self, message: &str, details: Option<&str>) -> Result<()> {
        let log_message = match details {
            Some(details) => format!("Error: {} - {}", message, details),
            None => format!("Error: {}", message),
        };
        self.logger.error(&log_message);
        Ok(())
    }
    
    /// Log informational message
    pub fn log_info(&self, message: &str) -> Result<()> {
        self.logger.info(message);
        Ok(())
    }

    /// Log process detection event
    pub fn log_process_detection(
        &self,
        pid: u32,
        name: &str,
        path: &std::path::Path,
        entitlements: &[String],
    ) -> Result<()> {
        let entitlements_str = entitlements.join(", ");
        
        let message = format!(
            "Process detected: pid={} name={} path={} entitlements=[{}]",
            pid, name, path.display(), entitlements_str
        );
        self.logger.info(&message);
        Ok(())
    }
}
