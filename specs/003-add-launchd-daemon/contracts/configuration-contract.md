# Configuration Contract: Daemon Settings and Management

**Date**: September 20, 2025  
**Purpose**: Define configuration file format, validation rules, and update mechanisms

## Configuration File Format

### Location and Naming
- **System Daemon**: `/etc/listent/daemon.toml`
- **User Agent**: `~/.config/listent/daemon.toml`
- **Backup**: `{config_path}.backup.{timestamp}`
- **Lock File**: `{config_path}.lock` (during atomic updates)

### TOML Schema

```toml
# Listent Daemon Configuration
# Version: 1.0.0
# Generated: 2025-09-20T10:30:15Z

[daemon]
# Polling interval in seconds (0.1 to 300.0)
polling_interval = 1.0

# Automatically start daemon on system boot
auto_start = true

# Process ID file location
pid_file = "/var/run/listent/daemon.pid"

# Maximum memory usage in MB (0 = unlimited)
max_memory_mb = 50

[logging]
# Unified Logging System subsystem identifier
subsystem = "com.github.mariohewardt.listent"

# Log level: debug, info, warning, error
level = "info"

# Enable structured logging for machine parsing
structured = true

# Include performance metrics in logs
include_performance = false

# Maximum log entries per minute (rate limiting)
max_log_rate = 1000

[monitoring]
# Monitor all processes (true) or only filtered ones (false)
monitor_all_processes = true

# Only monitor processes in these paths (empty = all paths)
path_filters = [
    "/Applications",
    "/usr/bin",
    "/usr/local/bin"
]

# Only report processes with these entitlements (empty = all entitlements)
entitlement_filters = [
    "com.apple.security.network.client",
    "com.apple.security.files.user-selected.read-write"
]

# Exclude system processes (kernel, launchd, etc.)
exclude_system_processes = true

# Exclude processes without entitlements
exclude_no_entitlements = true

# Minimum process lifetime to report (seconds)
min_process_lifetime = 0.1

[ipc]
# Unix domain socket path for configuration updates
socket_path = "/var/run/listent/daemon.sock"

# Socket file permissions (octal format)
socket_permissions = 0o600

# Maximum concurrent IPC connections
max_connections = 10

# Request timeout in seconds
request_timeout = 30.0

# Enable IPC authentication
require_auth = false

[performance]
# Process discovery batch size
batch_size = 100

# Maximum processes to track simultaneously
max_tracked_processes = 10000

# Memory cleanup interval in seconds
cleanup_interval = 300

# Enable process caching for performance
enable_caching = true

# Cache TTL in seconds
cache_ttl = 60

[security]
# Run with minimal privileges (drop unnecessary capabilities)
drop_privileges = true

# Allowed users for IPC connections (empty = any user)
allowed_users = ["root"]

# Enable audit logging for configuration changes
audit_config_changes = true

# Validate entitlement signatures
validate_signatures = false
```

## Configuration Validation Rules

### Value Constraints

#### [daemon] Section
```rust
pub fn validate_daemon_settings(settings: &DaemonSettings) -> Result<(), ConfigError> {
    // polling_interval: 0.1 ≤ value ≤ 300.0
    if settings.polling_interval < 0.1 || settings.polling_interval > 300.0 {
        return Err(ConfigError::InvalidRange {
            field: "daemon.polling_interval",
            value: settings.polling_interval.to_string(),
            min: "0.1",
            max: "300.0",
        });
    }

    // max_memory_mb: 0 ≤ value ≤ 1024 (0 = unlimited)
    if settings.max_memory_mb > 1024 {
        return Err(ConfigError::InvalidRange {
            field: "daemon.max_memory_mb", 
            value: settings.max_memory_mb.to_string(),
            min: "0",
            max: "1024",
        });
    }

    // pid_file: Must be in writable directory
    validate_writable_path(&settings.pid_file, "daemon.pid_file")?;
    
    Ok(())
}
```

#### [monitoring] Section
```rust
pub fn validate_monitoring_settings(settings: &MonitoringSettings) -> Result<(), ConfigError> {
    // path_filters: Must be absolute paths that exist
    for path in &settings.path_filters {
        if !path.is_absolute() {
            return Err(ConfigError::InvalidPath {
                field: "monitoring.path_filters",
                path: path.to_string_lossy().to_string(),
                reason: "Must be absolute path",
            });
        }
        
        if !path.exists() {
            return Err(ConfigError::InvalidPath {
                field: "monitoring.path_filters",
                path: path.to_string_lossy().to_string(), 
                reason: "Path does not exist",
            });
        }
    }

    // entitlement_filters: Must be valid entitlement format
    for entitlement in &settings.entitlement_filters {
        validate_entitlement_format(entitlement)?;
    }

    // min_process_lifetime: 0.0 ≤ value ≤ 60.0  
    if settings.min_process_lifetime < 0.0 || settings.min_process_lifetime > 60.0 {
        return Err(ConfigError::InvalidRange {
            field: "monitoring.min_process_lifetime",
            value: settings.min_process_lifetime.to_string(),
            min: "0.0", 
            max: "60.0",
        });
    }

    Ok(())
}
```

### Cross-Section Validation
```rust
pub fn validate_config_consistency(config: &DaemonConfiguration) -> Result<(), ConfigError> {
    // IPC socket must be accessible if daemon allows external connections
    if config.ipc.max_connections > 0 {
        validate_socket_path(&config.ipc.socket_path)?;
    }

    // Performance settings must be reasonable for polling interval
    if config.daemon.polling_interval < 1.0 && config.performance.batch_size > 50 {
        return Err(ConfigError::InconsistentSettings {
            message: "High batch_size with fast polling may cause performance issues".to_string(),
            suggestion: "Reduce batch_size or increase polling_interval".to_string(),
        });
    }

    // Security: If authentication disabled, warn about access
    if !config.ipc.require_auth && config.ipc.max_connections > 1 {
        warn!("IPC authentication disabled with multiple connections allowed");
    }

    Ok(())
}
```

## Configuration Update Protocol

### Atomic Update Process
1. **Validate**: Parse and validate new configuration without applying
2. **Backup**: Create timestamped backup of current configuration  
3. **Lock**: Acquire exclusive lock on configuration file
4. **Write**: Write new configuration to temporary file
5. **Verify**: Re-parse temporary file to ensure correctness
6. **Replace**: Atomically move temporary file to final location
7. **Signal**: Send SIGHUP to daemon for graceful reload
8. **Unlock**: Release configuration file lock

### Update Sources

#### CLI Command Updates
```bash
listent update-config polling_interval=0.5 log_level=debug
```

**Process**:
1. Parse KEY=VALUE pairs into ConfigUpdates struct
2. Load current configuration from file
3. Apply updates to loaded configuration
4. Validate modified configuration
5. Execute atomic update process
6. Send IPC message to daemon for immediate reload

#### File-Based Updates
```bash
# Direct file editing
sudo vim /etc/listent/daemon.toml
sudo kill -HUP $(cat /var/run/listent/daemon.pid)
```

**Process**:
1. Daemon receives SIGHUP signal
2. Reload configuration from file with validation
3. Log configuration changes
4. Apply new settings to running daemon
5. Notify active IPC clients of changes

#### IPC Updates
```rust
// Programmatic updates via Unix socket
let message = IpcMessage::UpdateConfig {
    updates: ConfigUpdates {
        polling_interval: Some(0.5),
        log_level: Some(LogLevel::Debug),
        ..Default::default()
    }
};
```

### Configuration Diff and Rollback

#### Change Detection
```rust
#[derive(Debug, Serialize)]
pub struct ConfigurationDiff {
    pub timestamp: SystemTime,
    pub previous_config: DaemonConfiguration,
    pub new_config: DaemonConfiguration,
    pub changes: Vec<ConfigChange>,
    pub rollback_available: bool,
}

#[derive(Debug, Serialize)]
pub struct ConfigChange {
    pub field: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub requires_restart: bool,
}
```

#### Rollback Process
```bash
# Automatic rollback on validation failure
listent update-config polling_interval=invalid_value
Error: Invalid configuration - rolling back to previous version
✓ Restored configuration from backup

# Manual rollback
listent rollback-config [--to-timestamp TIMESTAMP]
```

## Default Configuration Generation

### Initial Installation
```rust
impl Default for DaemonConfiguration {
    fn default() -> Self {
        Self {
            daemon: DaemonSettings {
                polling_interval: 1.0,
                auto_start: true,
                pid_file: PathBuf::from("/var/run/listent/daemon.pid"),
                max_memory_mb: 50,
            },
            logging: LoggingSettings {
                subsystem: "com.github.mariohewardt.listent".to_string(),
                level: LogLevel::Info,
                structured: true,
                include_performance: false,
                max_log_rate: 1000,
            },
            monitoring: MonitoringSettings {
                monitor_all_processes: true,
                path_filters: vec![
                    PathBuf::from("/Applications"),
                    PathBuf::from("/usr/bin"),
                    PathBuf::from("/usr/local/bin"),
                ],
                entitlement_filters: vec![],
                exclude_system_processes: true,
                exclude_no_entitlements: true,
                min_process_lifetime: 0.1,
            },
            ipc: IpcSettings {
                socket_path: PathBuf::from("/var/run/listent/daemon.sock"),
                socket_permissions: 0o600,
                max_connections: 10,
                request_timeout: 30.0,
                require_auth: false,
            },
            performance: PerformanceSettings {
                batch_size: 100,
                max_tracked_processes: 10000,
                cleanup_interval: 300,
                enable_caching: true,
                cache_ttl: 60,
            },
            security: SecuritySettings {
                drop_privileges: true,
                allowed_users: vec!["root".to_string()],
                audit_config_changes: true,
                validate_signatures: false,
            },
        }
    }
}
```

### Configuration Templates
```toml
# Minimal monitoring (performance optimized)
[daemon]
polling_interval = 5.0
[monitoring]
monitor_all_processes = false
path_filters = ["/Applications"]
exclude_system_processes = true

# Security focused (comprehensive monitoring)  
[daemon]
polling_interval = 0.5
[monitoring]
monitor_all_processes = true
exclude_system_processes = false
[security]
audit_config_changes = true
validate_signatures = true

# Development mode (verbose logging)
[logging]
level = "debug"
include_performance = true
[performance]
enable_caching = false
```

## Migration and Versioning

### Configuration Version Schema
```toml
[meta]
version = "1.0.0"
created_at = "2025-09-20T10:30:15Z"
last_modified = "2025-09-20T15:45:30Z"
modified_by = "listent update-config"
```

### Migration Rules
- **1.0.0 → 1.1.0**: Add new optional fields, keep defaults
- **1.x.x → 2.0.0**: Breaking changes require manual migration
- **Unsupported versions**: Error with upgrade instructions

### Backward Compatibility
- Unknown fields: Warn but ignore (forward compatibility)
- Missing fields: Use defaults from current version
- Type mismatches: Error with clear correction guidance

## Error Handling

### Configuration Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid range for {field}: {value} (must be between {min} and {max})")]
    InvalidRange {
        field: String,
        value: String, 
        min: String,
        max: String,
    },

    #[error("Invalid path for {field}: {path} ({reason})")]
    InvalidPath {
        field: String,
        path: String,
        reason: String,
    },

    #[error("File operation failed: {operation} on {path}: {error}")]
    FileError {
        operation: String,
        path: String,
        error: String,
    },

    #[error("Configuration inconsistency: {message}. Suggestion: {suggestion}")]
    InconsistentSettings {
        message: String,
        suggestion: String,
    },
}
```

### Recovery Strategies
- **Parse errors**: Load backup configuration
- **Validation errors**: Keep current config, log error
- **File corruption**: Regenerate from defaults + user notification
- **Permission errors**: Clear instructions for resolution

This configuration contract ensures robust, validated configuration management with atomic updates, comprehensive error handling, and smooth migration paths.