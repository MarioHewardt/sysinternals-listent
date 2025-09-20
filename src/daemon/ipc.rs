//! Inter-process communication for daemon control
//!
//! Provides Unix domain socket server for runtime configuration updates

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::io::split;
use uuid::Uuid;

/// IPC message types for daemon communication
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    /// Update daemon configuration with key-value pairs
    UpdateConfig { 
        updates: Vec<ConfigUpdate>,
        /// Unique request ID for tracking
        request_id: String,
    },
    /// Reload configuration from file
    ReloadConfig {
        request_id: String,
    },
    /// Get current daemon status
    GetStatus {
        request_id: String,
    },
    /// Get daemon runtime statistics
    GetStats {
        request_id: String,
    },
    /// Shutdown daemon gracefully
    Shutdown {
        request_id: String,
    },
}

/// Configuration update operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigUpdate {
    /// Configuration key in dot notation (e.g., "daemon.polling_interval")
    pub key: String,
    /// New value as string (will be parsed based on key type)
    pub value: String,
}

/// IPC response types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum IpcResponse {
    /// Successful operation
    Success { 
        request_id: String,
        data: Option<serde_json::Value>,
        message: Option<String>,
    },
    /// Operation failed
    Error { 
        request_id: String,
        code: u32, 
        message: String,
        details: Option<String>,
    },
}

/// Daemon status information
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Whether daemon is running
    pub running: bool,
    /// Process ID
    pub pid: u32,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Configuration file path
    pub config_path: PathBuf,
    /// Last configuration reload time
    pub last_config_reload: Option<chrono::DateTime<chrono::Utc>>,
}

/// Daemon runtime statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStats {
    /// Total processes monitored since startup
    pub total_processes_monitored: u64,
    /// New processes detected since startup
    pub new_processes_detected: u64,
    /// Processes with matching entitlements
    pub processes_with_entitlements: u64,
    /// Current polling interval
    pub current_polling_interval: f64,
    /// Last successful poll time
    pub last_poll_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
}

/// IPC server for handling client connections
pub struct IpcServer {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Clean up socket file when server is dropped
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }
}

/// Handler for individual IPC connections
#[derive(Clone)]
struct IpcServerHandler {
    socket_path: PathBuf,
}

impl IpcServer {
    /// Create new IPC server
    pub fn new(socket_path: PathBuf) -> Result<Self> {
        Ok(Self {
            socket_path,
            listener: None,
        })
    }

    /// Start listening for connections and handle them in a loop
    pub async fn start(&mut self) -> Result<()> {
        // Remove existing socket file if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .with_context(|| format!("Failed to remove existing socket: {}", self.socket_path.display()))?;
        }

        // Create parent directory if needed
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create socket directory: {}", parent.display()))?;
        }

        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)
            .with_context(|| format!("Failed to bind to socket: {}", self.socket_path.display()))?;

        self.listener = Some(listener);
        eprintln!("🌐 IPC server listening on {}", self.socket_path.display());

        // Accept connections in a loop
        while let Some(ref listener) = self.listener {
            match listener.accept().await {
                Ok((stream, _)) => {
                    // Handle connection in a separate task
                    let server_clone = self.clone_for_handler();
                    tokio::spawn(async move {
                        if let Err(e) = server_clone.handle_connection(stream).await {
                            eprintln!("❌ Error handling IPC connection: {}", e);
                        }
                    });
                },
                Err(e) => {
                    eprintln!("❌ Error accepting IPC connection: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Create a clone suitable for connection handling  
    fn clone_for_handler(&self) -> IpcServerHandler {
        IpcServerHandler {
            socket_path: self.socket_path.clone(),
        }
    }

    /// Generate unique request ID
    pub fn generate_request_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Stop the server and clean up socket file
    pub fn stop(&mut self) -> Result<()> {
        self.listener = None;
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .with_context(|| format!("Failed to remove socket file: {}", self.socket_path.display()))?;
        }
        Ok(())
    }
}

impl IpcServerHandler {
    /// Accept and handle a client connection
    pub async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        
        // Read JSON message from client
        reader.read_line(&mut line).await
            .context("Failed to read from client")?;

        // Parse IPC message
        let message: IpcMessage = serde_json::from_str(&line.trim())
            .context("Failed to parse IPC message")?;

        // Process message and generate response
        let response = self.process_message(message).await?;

        // Send response back to client
        let response_json = serde_json::to_string(&response)
            .context("Failed to serialize response")?;
        
        writer.write_all(response_json.as_bytes()).await
            .context("Failed to write response")?;
        writer.write_all(b"\n").await
            .context("Failed to write newline")?;

        Ok(())
    }

    /// Process an IPC message and generate appropriate response
    async fn process_message(&self, message: IpcMessage) -> Result<IpcResponse> {
        match message {
            IpcMessage::UpdateConfig { updates, request_id } => {
                // TODO: Implement configuration update logic
                Ok(IpcResponse::Error {
                    request_id,
                    code: 501,
                    message: "Configuration updates not yet implemented".to_string(),
                    details: Some(format!("Requested {} updates", updates.len())),
                })
            },
            IpcMessage::ReloadConfig { request_id } => {
                // TODO: Implement configuration reload logic
                Ok(IpcResponse::Error {
                    request_id,
                    code: 501,
                    message: "Configuration reload not yet implemented".to_string(),
                    details: None,
                })
            },
            IpcMessage::GetStatus { request_id } => {
                // TODO: Implement status reporting
                Ok(IpcResponse::Error {
                    request_id,
                    code: 501,
                    message: "Status reporting not yet implemented".to_string(),
                    details: None,
                })
            },
            IpcMessage::GetStats { request_id } => {
                // TODO: Implement stats reporting
                Ok(IpcResponse::Error {
                    request_id,
                    code: 501,
                    message: "Stats reporting not yet implemented".to_string(),
                    details: None,
                })
            },
            IpcMessage::Shutdown { request_id } => {
                // TODO: Implement graceful shutdown
                Ok(IpcResponse::Error {
                    request_id,
                    code: 501,
                    message: "Shutdown not yet implemented".to_string(),
                    details: None,
                })
            },
        }
    }
}