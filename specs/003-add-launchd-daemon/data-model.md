# Data Model: LaunchD Daemon Support

**Date**: September 20, 2025  
**Phase**: Design phase data structures and relationships

## Core Data Structures

### DaemonConfiguration
**Purpose**: Central configuration for daemon operation
**Location**: `/etc/listent/daemon.toml`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfiguration {
    pub daemon: DaemonSettings,
    pub logging: LoggingSettings, 
    pub monitoring: MonitoringSettings,
    pub ipc: IpcSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    pub polling_interval: f64,      // 0.1-300.0 seconds
    pub auto_start: bool,           // launchd RunAtLoad setting
    pub pid_file: PathBuf,          // /var/run/listent/daemon.pid
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    pub subsystem: String,          // "com.microsoft.sysinternals.listent"
    pub level: LogLevel,            // Debug, Info, Warning, Error
    pub structured: bool,           // Enable structured logging
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    pub monitor_all_processes: bool,
    pub path_filters: Vec<PathBuf>,
    pub entitlement_filters: Vec<String>,
    pub exclude_system_processes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcSettings {
    pub socket_path: PathBuf,       // /var/run/listent/daemon.sock
    pub socket_permissions: u32,    // 0o600
    pub max_connections: usize,     // Concurrent IPC connections
}
```

### LaunchDaemonState
**Purpose**: Runtime state management for daemon lifecycle

```rust
#[derive(Debug, Clone)]
pub struct LaunchDaemonState {
    pub installation_status: InstallationStatus,
    pub runtime_status: RuntimeStatus,
    pub configuration: DaemonConfiguration,
    pub last_config_update: SystemTime,
    pub process_tracker: ProcessTracker,  // Reuse existing
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallationStatus {
    NotInstalled,
    Installed { plist_path: PathBuf },
    InstallError { error: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeStatus {
    Stopped,
    Starting,
    Running { pid: u32, started_at: SystemTime },
    Stopping,
    Error { error: String },
}
```

### IPC Message Protocol
**Purpose**: Communication between CLI and daemon for configuration updates

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    // Configuration updates
    UpdateConfig { updates: ConfigUpdates },
    ReloadConfig,
    GetConfig,
    
    // Status queries
    GetStatus,
    GetStats,
    
    // Control operations
    Shutdown,
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcResponse {
    Success { data: Option<serde_json::Value> },
    Error { code: u32, message: String },
    ConfigUpdated { new_config: DaemonConfiguration },
    Status { state: LaunchDaemonState },
    Stats { stats: DaemonStats },
    Pong,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigUpdates {
    pub polling_interval: Option<f64>,
    pub path_filters: Option<Vec<PathBuf>>,
    pub entitlement_filters: Option<Vec<String>>,
    pub log_level: Option<LogLevel>,
    pub monitor_all_processes: Option<bool>,
}
```

### LaunchD Integration Types
**Purpose**: Management of macOS launchd service lifecycle

```rust
#[derive(Debug, Clone)]
pub struct LaunchDPlist {
    pub label: String,              // com.microsoft.sysinternals.listent
    pub program_arguments: Vec<String>,
    pub run_at_load: bool,
    pub keep_alive: bool,
    pub user_name: Option<String>,  // root for system-wide
    pub working_directory: Option<PathBuf>,
    pub standard_out_path: Option<PathBuf>,
    pub standard_error_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct LaunchDCommand {
    pub operation: LaunchDOperation,
    pub service_name: String,
    pub plist_path: Option<PathBuf>,
}

#[derive(Debug, PartialEq)]
pub enum LaunchDOperation {
    Load,      // launchctl load
    Unload,    // launchctl unload  
    Start,     // launchctl start
    Stop,      // launchctl stop
    List,      // launchctl list (status check)
}
```

### Enhanced Logging Types
**Purpose**: Structured logging for ULS integration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info, 
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessDetectionEvent {
    pub timestamp: SystemTime,
    pub process: MonitoredProcess,      // Reuse existing type
    pub detection_method: DetectionMethod,
    pub matched_filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum DetectionMethod {
    PollingCycle,
    SystemNotification,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigurationEvent {
    pub timestamp: SystemTime,
    pub event_type: ConfigEventType,
    pub old_config: Option<DaemonConfiguration>,
    pub new_config: Option<DaemonConfiguration>,
    pub source: ConfigUpdateSource,
}

#[derive(Debug, Clone, Serialize)]
pub enum ConfigEventType {
    Created,
    Updated,
    Reloaded,
    ValidationFailed,
}

#[derive(Debug, Clone, Serialize)]
pub enum ConfigUpdateSource {
    CliCommand,
    FileChange,
    IpcRequest,
    SystemSignal,
}
```

### Performance Monitoring
**Purpose**: Track daemon performance for optimization and debugging

```rust
#[derive(Debug, Clone, Serialize)]
pub struct DaemonStats {
    pub uptime: Duration,
    pub processes_detected: u64,
    pub config_updates: u32,
    pub ipc_requests: u64,
    pub errors: u32,
    pub memory_usage: MemoryStats,
    pub performance: PerformanceStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub resident_bytes: u64,
    pub virtual_bytes: u64,
    pub peak_resident: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceStats {
    pub avg_poll_duration: Duration,
    pub max_poll_duration: Duration,
    pub last_poll_duration: Duration,
    pub polls_per_minute: f64,
}
```

## Data Relationships

```
DaemonConfiguration
├── Contains → DaemonSettings (polling, startup)
├── Contains → LoggingSettings (ULS config) 
├── Contains → MonitoringSettings (filters, scope)
└── Contains → IpcSettings (socket config)

LaunchDaemonState
├── References → DaemonConfiguration
├── Contains → InstallationStatus
├── Contains → RuntimeStatus
└── Contains → ProcessTracker (existing)

IpcMessage/IpcResponse
├── Carries → ConfigUpdates
├── References → DaemonConfiguration
└── Contains → DaemonStats

ProcessDetectionEvent
├── Contains → MonitoredProcess (existing)
├── References → DetectionMethod
└── References → matched filters from MonitoringSettings

ConfigurationEvent  
├── References → DaemonConfiguration (old/new)
├── Contains → ConfigEventType
└── Contains → ConfigUpdateSource
```

## State Transitions

### Installation State Machine
```
NotInstalled → [install-daemon] → Installed
Installed → [uninstall-daemon] → NotInstalled  
Any State → [error] → InstallError
InstallError → [install-daemon] → Installed
```

### Runtime State Machine
```
Stopped → [start] → Starting → [success] → Running
Running → [stop] → Stopping → [success] → Stopped
Running → [error] → Error → [restart] → Starting
Any State → [shutdown] → Stopped
```

### Configuration State Flow
```
File Update → Validation → [valid] → Apply → Notify IPC Clients
                        → [invalid] → Log Error + Keep Current Config

IPC Update → Validation → [valid] → Apply → Persist → Notify
                       → [invalid] → Return Error Response
```

## Validation Rules

### Configuration Validation
- `polling_interval`: 0.1 ≤ value ≤ 300.0
- `path_filters`: Must be absolute paths, must exist
- `entitlement_filters`: Non-empty strings, valid entitlement format
- `socket_path`: Must be in writable directory with appropriate permissions
- `log_level`: Must be valid LogLevel enum value

### IPC Message Validation  
- Message size: ≤ 64KB per message
- Connection limits: ≤ 10 concurrent connections
- Rate limiting: ≤ 100 requests per minute per connection
- Authentication: Socket permissions-based (file system security)

### LaunchD Plist Validation
- `label`: Must match reverse DNS format
- `program_arguments`: First argument must be valid executable path
- Paths: All paths must exist and be accessible
- Permissions: Validate plist file has correct ownership and permissions

## Error Handling Strategy

### Configuration Errors
- **Validation Failures**: Log error, keep current config, return specific error
- **File I/O Errors**: Retry with exponential backoff, fallback to defaults
- **Permission Errors**: Clear error message with remediation steps

### IPC Errors
- **Connection Failures**: Auto-retry for clients, graceful degradation
- **Protocol Errors**: Version mismatch detection, backward compatibility
- **Socket Errors**: Automatic socket cleanup and recreation

### System Integration Errors
- **LaunchD Errors**: Parse launchctl output, provide actionable feedback
- **ULS Errors**: Fallback to stderr logging, continue operation
- **Process Access Errors**: Handle SIP restrictions gracefully

This data model supports the complete daemon lifecycle while maintaining compatibility with existing monitor functionality and ensuring robust error handling throughout the system.