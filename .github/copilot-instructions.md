# GitHub Copilot Instructions

**Project**: listent - macOS entitlement scanning CLI tool  
**Language**: Rust  
**Last Updated**: September 20, 2025

## Project Overview

listent is a fast command-line tool for macOS that scans and monitors code signing entitlements. It provides both one-time scanning and real-time monitoring capabilities for security analysis and compliance verification. Now includes background daemon mode for continuous system monitoring.

## Current Architecture

### Module Structure
```
src/
├── main.rs              # Entry point and CLI coordination
├── cli/mod.rs           # Command-line argument parsing (clap)
├── models/mod.rs        # Data structures and configuration
├── scan/mod.rs          # Filesystem scanning and binary discovery  
├── entitlements/mod.rs  # Code signing entitlement extraction
├── output/mod.rs        # Output formatting (human-readable and JSON)
├── monitor/mod.rs       # Real-time process monitoring
└── daemon/mod.rs        # NEW: LaunchD daemon functionality
    ├── config.rs        # Configuration management
    ├── ipc.rs           # Inter-process communication
    ├── launchd.rs       # macOS launchd integration
    └── logging.rs       # Enhanced ULS logging
```

### Key Dependencies
- **clap**: Command-line argument parsing with subcommands
- **serde_json**: JSON serialization for output
- **sysinfo**: Process enumeration for monitoring mode
- **tokio**: Async runtime for daemon mode IPC and signal handling
- **toml**: Configuration file parsing for daemon settings
- **nix**: Unix domain sockets and signal handling

### Constitutional Principles
- Single binary CLI tool targeting macOS
- Minimal dependencies, prefer std library
- No unsafe code without justification
- Test-driven development with cargo test
- Clear error handling with structured messages

## Feature: Real-time Process Monitoring

### CLI Extensions
```rust
// Add to existing CLI structure
#[derive(Parser)]
pub struct Args {
    // Existing fields...
    
    /// Enable real-time process monitoring mode
    #[arg(long)]
    pub monitor: bool,
    
    /// Polling interval in seconds (0.1 - 300.0)
    #[arg(long, default_value = "1.0")]
    pub interval: f64,
}
```

### New Data Model Types
```rust
// In src/models/mod.rs - extend existing types
pub struct MonitoredProcess {
    pub pid: u32,
    pub name: String,
    pub executable_path: PathBuf,
    pub entitlements: Vec<String>,
    pub discovery_timestamp: SystemTime,
}

pub struct PollingConfiguration {
    pub interval: Duration,
    pub path_filters: Vec<PathBuf>,
    pub entitlement_filters: Vec<String>,
    pub output_json: bool,
    pub quiet_mode: bool,
}

pub struct ProcessSnapshot {
    pub processes: HashMap<u32, MonitoredProcess>,
    pub timestamp: SystemTime,
    pub scan_duration: Duration,
}
```

### Monitor Module Structure
```rust
// src/monitor/mod.rs - NEW module
pub mod process_tracker;   // Process state management
pub mod polling;          // Polling loop implementation  
pub mod unified_logging;  // macOS system logging

pub use process_tracker::ProcessTracker;
pub use polling::start_monitoring;
```

### Integration Points

#### CLI Integration
- Extend existing Args struct with monitor and interval fields
- Reuse existing path (-p) and entitlement (-e) parsing logic
- Maintain existing help and version functionality

#### Scan Module Reuse
- Leverage existing path filtering logic for monitoring scope
- Reuse directory traversal patterns for initial process discovery
- Maintain consistent error handling patterns

#### Entitlements Module Reuse  
- Use existing codesign extraction for monitored processes
- Apply existing entitlement filtering logic
- Handle extraction failures gracefully (empty entitlements list)

#### Output Module Extension
- Extend existing JSON schema for process detection events
- Reuse human-readable formatting patterns
- Maintain existing quiet mode behavior

## Coding Patterns

### Error Handling
```rust
// Use Result types for fallible operations
pub fn extract_process_entitlements(pid: u32) -> Result<Vec<String>, MonitorError> {
    // Implementation
}

// Custom error types for monitoring
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Invalid polling interval: {0}. Must be between 0.1 and 300.0 seconds")]
    InvalidInterval(f64),
    #[error("Process access denied: {0}")]
    PermissionDenied(String),
    #[error("System resource error: {0}")]
    SystemError(String),
}
```

### Testing Approach
```rust
// Unit tests for core logic
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_snapshot_comparison() {
        // Test new process detection logic
    }
    
    #[test]
    fn test_polling_configuration_validation() {
        // Test interval bounds checking
    }
}

// Integration tests in tests/ directory
// Test full monitor workflows with real processes
```

## Feature: LaunchD Daemon Support

### CLI Extensions for Daemon Management
```rust
// Extended CLI with subcommands
#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
    // ... existing fields
    #[arg(long)]
    pub daemon: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    InstallDaemon { config: Option<PathBuf> },
    UninstallDaemon,
    DaemonStatus,
    UpdateConfig { updates: Vec<String> },
    Logs { follow: bool, since: Option<String> },
}
```

### Daemon Configuration Types
```rust
// In src/daemon/config.rs
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

// Configuration file: /etc/listent/daemon.toml
```

### LaunchD Integration
```rust
// src/daemon/launchd.rs
pub struct LaunchDPlist {
    pub label: String,              // com.github.mariohewardt.listent
    pub program_arguments: Vec<String>,
    pub run_at_load: bool,
    pub keep_alive: bool,
    pub working_directory: Option<PathBuf>,
}

pub fn generate_plist(daemon_path: &Path) -> Result<String>;
pub fn install_plist(plist_content: &str, service_name: &str) -> Result<()>;
pub fn launchctl_load(plist_path: &Path) -> Result<()>;
pub fn launchctl_unload(service_name: &str) -> Result<()>;
```

### IPC Communication
```rust
// src/daemon/ipc.rs
#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    UpdateConfig { updates: ConfigUpdates },
    ReloadConfig,
    GetStatus,
    GetStats,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcResponse {
    Success { data: Option<serde_json::Value> },
    Error { code: u32, message: String },
    ConfigUpdated { new_config: DaemonConfiguration },
}

// Unix domain socket at /var/run/listent/daemon.sock
pub struct IpcServer {
    socket_path: PathBuf,
    listener: UnixListener,
}
```

### Integration Points

#### Daemon Mode Execution
- Extend main.rs with daemon execution path
- No terminal output in daemon mode - ULS logging only
- Reuse existing monitor::polling logic with async wrapper
- Signal handling for graceful shutdown and config reload

#### Configuration Management
- TOML-based configuration files
- Atomic configuration updates with validation
- Backup and rollback functionality
- Dynamic reload without daemon restart

### Testing Approach
```rust
// Unit tests for core logic
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_snapshot_comparison() {
        // Test new process detection logic
    }
    
    #[test]
    fn test_polling_configuration_validation() {
        // Test interval bounds checking
    }
}

// Integration tests in tests/ directory
// Test full monitor workflows with real processes
```

### Performance Considerations
- Use HashMap for O(1) process lookups during comparison
- Minimize allocations in polling loop (reuse collections)
- Handle large entitlements lists efficiently
- Profile memory usage during extended monitoring

## Recent Changes

### Monitor Feature Implementation (002-add-monitor-switch)
- **Phase**: ✅ IMPLEMENTATION COMPLETE 
- **Status**: ✅ Monitor functionality fully operational
- **Key Features Implemented**:
  - Real-time process monitoring with `--monitor` flag
  - Configurable polling intervals with `--interval` (0.1-300.0 seconds)
  - Process entitlement extraction and filtering
  - Human-readable and JSON output formats
  - Graceful shutdown with Ctrl+C handling
  - Performance optimized for extended monitoring
- **Performance Optimizations**:
  - Pre-allocated collections to reduce memory allocations
  - Lazy entitlement extraction (only for new processes)
  - Efficient process state tracking with HashMap lookups
  - Memory usage <1% of system resources during operation
- **Testing**: TDD approach with comprehensive contract tests covering CLI validation, output formats, and edge cases

### Files Modified/Added  
- **Core Implementation**: `src/monitor/` module with polling engine, process tracking, unified logging
- **Models**: Extended `src/models/mod.rs` with MonitoredProcess, PollingConfiguration, ProcessSnapshot
- **CLI**: Enhanced `src/cli/mod.rs` with --monitor and --interval flags using clap derive API
- **Integration**: Updated `src/main.rs` with dual-mode support (scan vs monitor)
- **Tests**: Contract tests in `tests/test_monitor_cli.rs` and `tests/test_monitor_output.rs`

## Code Style Preferences

### Rust Conventions
- Use `rustfmt` default formatting
- Prefer explicit types for public APIs
- Use `?` operator for error propagation
- Document public functions with /// comments
- Use `#[derive(Debug)]` for data structures

### CLI Patterns
- Use clap derive API for argument parsing
- Validate arguments early, fail fast with clear messages
- Use structured output (JSON) for programmatic consumption
- Provide human-readable output by default

### Testing Patterns
- Unit tests in module files (`#[cfg(test)]`)
- Integration tests in `tests/` directory
- Contract tests validate CLI behavior and output formats
- Use `assert_cmd` for CLI testing, `predicates` for output validation

## Common Tasks

### Adding New CLI Options
1. Add field to `Args` struct in `src/cli/mod.rs`
2. Add validation logic if needed
3. Update help text generation
4. Add contract tests for new option

### Extending Output Formats
1. Modify output structures in `src/models/mod.rs`
2. Update JSON serialization if needed
3. Extend formatting logic in `src/output/mod.rs`
4. Add output format contract tests

### Error Handling Extensions
1. Add new error variants to appropriate error enums
2. Implement Display and Error traits
3. Add error context in calling code
4. Test error scenarios with unit tests

---

*This file is automatically updated as new features are implemented.*