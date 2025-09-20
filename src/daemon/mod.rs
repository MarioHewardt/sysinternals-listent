//! Daemon module for launchd integration and background process monitoring
//!
//! This module provides functionality to run listent as a macOS daemon:
//! - Configuration management with atomic updates
//! - Inter-process communication for runtime configuration changes
//! - LaunchD integration for system service management
//! - Enhanced Unified Logging System integration

pub mod config;
pub mod ipc;
pub mod launchd;
pub mod logging;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::signal;
use crate::models::{PollingConfiguration, ProcessSnapshot, MonitoredProcess};
use crate::daemon::config::DaemonConfiguration;
use crate::daemon::ipc::IpcServer;
use crate::daemon::logging::{DaemonLogger, LogLevel};
use crate::monitor::process_tracker::ProcessTracker;

/// Daemon runtime state
pub struct DaemonState {
    /// Current configuration
    config: Arc<Mutex<DaemonConfiguration>>,
    /// Process tracker for monitoring
    process_tracker: Arc<Mutex<ProcessTracker>>,
    /// Daemon logger
    logger: DaemonLogger,
    /// IPC server for runtime communication
    ipc_server: Option<IpcServer>,
}

impl DaemonState {
    /// Create new daemon state with configuration
    pub fn new(config: DaemonConfiguration) -> Result<Self> {
        let logger = DaemonLogger::new(
            "com.github.mariohewardt.listent".to_string(),
            "daemon".to_string(),
            LogLevel::Info,
        )?;

        let process_tracker = ProcessTracker::new();

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            process_tracker: Arc::new(Mutex::new(process_tracker)),
            logger,
            ipc_server: None,
        })
    }

    /// Initialize IPC server
    pub fn with_ipc_server(mut self, socket_path: PathBuf) -> Result<Self> {
        self.ipc_server = Some(IpcServer::new(socket_path)?);
        Ok(self)
    }

    /// Get current configuration
    pub async fn get_config(&self) -> DaemonConfiguration {
        self.config.lock().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: DaemonConfiguration) -> Result<()> {
        let mut config = self.config.lock().await;
        self.logger.log_config_change(
            "Configuration updated",
            &format!("{:?}", *config),
            &format!("{:?}", new_config),
        )?;
        *config = new_config;
        Ok(())
    }
}

/// Run the daemon in background mode
/// This function replaces terminal output with ULS logging
pub async fn run_daemon_mode() -> Result<()> {
    run_daemon_with_config(None).await
}

/// Run daemon with specific configuration path
pub async fn run_daemon_with_config(config_path: Option<PathBuf>) -> Result<()> {
    // Load configuration
    let config = if let Some(ref path) = config_path {
        DaemonConfiguration::load_from_file(path)?
    } else {
        DaemonConfiguration::default()
    };

    // Create daemon state
    let socket_path = PathBuf::from(&config.ipc.socket_path);
    let mut daemon_state = DaemonState::new(config.clone())?
        .with_ipc_server(socket_path.clone())?;

    // Log startup
    daemon_state.logger.log_startup(
        config_path.as_deref().unwrap_or(&DaemonConfiguration::default_config_path()?),
        std::process::id(),
    )?;

    // Setup signal handling for graceful shutdown
    let shutdown_signal = setup_signal_handlers();

    // Start IPC server in background
    let ipc_task = if let Some(ref mut ipc_server) = daemon_state.ipc_server {
        let mut server_clone = IpcServer::new(socket_path)?;
        Some(tokio::spawn(async move {
            if let Err(e) = server_clone.start().await {
                eprintln!("❌ IPC server error: {}", e);
            }
        }))
    } else {
        None
    };

    // Main monitoring loop
    let monitoring_task = {
        let process_tracker = daemon_state.process_tracker.clone();
        let config = daemon_state.config.clone();
        // Clone logger fields instead of the whole struct
        let logger_subsystem = daemon_state.logger.subsystem.clone();
        let logger_category = daemon_state.logger.category.clone();
        let logger_level = daemon_state.logger.level();
        
        tokio::spawn(async move {
            let logger = DaemonLogger::new(logger_subsystem, logger_category, logger_level).unwrap();
            if let Err(e) = run_monitoring_loop(process_tracker, config, logger).await {
                eprintln!("❌ Monitoring loop error: {}", e);
            }
        })
    };

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_signal => {
            daemon_state.logger.log_shutdown("Received shutdown signal")?;
        }
        _ = monitoring_task => {
            daemon_state.logger.log_shutdown("Monitoring loop ended")?;
        }
        _ = async {
            if let Some(task) = ipc_task {
                task.await.ok();
            }
        } => {
            daemon_state.logger.log_shutdown("IPC server ended")?;
        }
    }

    // Cleanup
    if let Some(ref mut ipc_server) = daemon_state.ipc_server {
        ipc_server.stop()?;
    }

    Ok(())
}

/// Main monitoring loop that runs continuously
async fn run_monitoring_loop(
    process_tracker: Arc<Mutex<ProcessTracker>>,
    config: Arc<Mutex<DaemonConfiguration>>,
    logger: DaemonLogger,
) -> Result<()> {
    let mut interval = {
        let config = config.lock().await;
        tokio::time::interval(config.polling_duration())
    };

    loop {
        interval.tick().await;

        eprintln!("🔍 Monitoring loop tick at {:?}", std::time::SystemTime::now());

        // Get current processes using polling logic
        let current_config = config.lock().await;
        let polling_config = PollingConfiguration {
            interval: current_config.polling_duration(),
            path_filters: current_config.monitoring.path_filters.clone(),
            entitlement_filters: current_config.monitoring.entitlement_filters.clone(),
            output_json: false, // ULS logging instead
            quiet_mode: false,  // Log all detections
        };
        drop(current_config);

        // Create current snapshot using polling logic
        let current_processes = match scan_current_processes(&polling_config).await {
            Ok(processes) => processes,
            Err(e) => {
                logger.log_error(&format!("Failed to scan processes: {}", e), None)?;
                continue;
            }
        };
        
        let current_snapshot = ProcessSnapshot {
            processes: current_processes,
            timestamp: std::time::SystemTime::now(),
            scan_duration: std::time::Duration::from_millis(0),
        };

        // Detect new processes
        let mut tracker = process_tracker.lock().await;
        let new_processes = tracker.detect_new_processes(current_snapshot);

        // Log any new processes with entitlements
        for process in new_processes {
            if !process.entitlements.is_empty() {
                logger.log_process_detection(
                    process.pid,
                    &process.name,
                    &process.executable_path.to_string_lossy(),
                    &process.entitlements,
                )?;
            }
        }
    }
}

/// Setup signal handlers for graceful shutdown
async fn setup_signal_handlers() {
    let _ = signal::ctrl_c().await;
}

/// Initialize daemon with configuration
pub fn initialize_daemon(config_path: Option<PathBuf>) -> Result<()> {
    // Validate configuration exists and is readable
    if let Some(ref path) = config_path {
        if !path.exists() {
            return Err(anyhow::anyhow!("Configuration file not found: {}", path.display()));
        }
        
        // Try to load and validate configuration
        let _config = DaemonConfiguration::load_from_file(path)
            .with_context(|| format!("Failed to load configuration from {}", path.display()))?;
    }

    // Ensure required directories exist
    let config = if let Some(ref path) = config_path {
        DaemonConfiguration::load_from_file(path)?
    } else {
        DaemonConfiguration::default()
    };

    config.ensure_directories()
        .context("Failed to create required directories")?;

    Ok(())
}

/// Stop daemon gracefully
pub fn stop_daemon() -> Result<()> {
    // Load config to get socket path
    let config = DaemonConfiguration::default();
    let socket_path = PathBuf::from(&config.ipc.socket_path);
    
    // Remove socket file if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .with_context(|| format!("Failed to remove daemon socket: {}", socket_path.display()))?;
    }

    // TODO: Send shutdown signal to running daemon via IPC
    // For now, this is a basic cleanup function
    
    Ok(())
}

/// Scan current processes and their entitlements
async fn scan_current_processes(config: &PollingConfiguration) -> Result<std::collections::HashMap<u32, MonitoredProcess>> {
    use sysinfo::{ProcessExt, System, SystemExt, PidExt};
    
    let mut system = System::new_all();
    system.refresh_processes();
    
    let mut processes = std::collections::HashMap::new();
    
    for (pid, process) in system.processes() {
        let pid_u32 = pid.as_u32();
        let process_name = process.name().to_string();
        
        // Get executable path
        let executable_path = process.exe().to_path_buf();
        
        // Apply path filters if specified
        if !config.path_filters.is_empty() {
            let matches_filter = config.path_filters.iter().any(|filter| {
                executable_path.starts_with(filter)
            });
            if !matches_filter {
                continue;
            }
        }
        
        // Extract entitlements - convert HashMap to Vec of keys
        let entitlements = match crate::entitlements::extract_entitlements(&executable_path) {
            Ok(entitlements_map) => entitlements_map.keys().cloned().collect::<Vec<String>>(),
            Err(_) => Vec::new(), // Continue with empty entitlements if extraction fails
        };
        
        // Apply entitlement filters if specified
        if !config.entitlement_filters.is_empty() {
            let matches_filter = config.entitlement_filters.iter().any(|filter| {
                entitlements.iter().any(|ent| ent.contains(filter))
            });
            if !matches_filter {
                continue;
            }
        }
        
        // Create monitored process
        let monitored_process = MonitoredProcess {
            pid: pid_u32,
            name: process_name,
            executable_path,
            entitlements,
            discovery_timestamp: std::time::SystemTime::now(),
        };
        
        processes.insert(pid_u32, monitored_process);
    }
    
    Ok(processes)
}