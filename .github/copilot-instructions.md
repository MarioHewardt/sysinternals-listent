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

## Feature: LaunchD Daemon Support (Simplified Implementation)

**Implementation Note**: Phase 3 was implemented with a simplified approach that provides all core functionality without over-engineering. No TOML config files, no IPC, no automated CLI commands - just clean daemon execution via launchd.

### CLI Extensions for Daemon
```rust
// Simple daemon flags (no subcommands)
#[derive(Parser)]
pub struct Args {
    // ... existing fields
    
    /// Run as background daemon (implies --monitor)
    #[arg(long)]
    pub daemon: bool,
    
    /// Generate LaunchD plist for installation (requires --daemon)
    #[arg(long)]
    pub launchd: bool,
}
```

### LaunchD Integration
```rust
// src/daemon/launchd.rs
pub fn generate_launchd_plist(
    daemon_path: &Path,
    interval: f64,
    paths: &[PathBuf],
    entitlements: &[String],
    run_at_load: bool,
    keep_alive: bool,
) -> Result<String>;

// Generates XML plist with:
// - Label: com.github.mariohewardt.listent
// - ProgramArguments: [binary_path, --daemon, --interval, X, ...filters]
// - RunAtLoad: true (auto-start on boot)
// - KeepAlive: true (auto-restart on crash)
// - StandardOutPath/StandardErrorPath for debugging
```

### Daemon Logging
```rust
// src/daemon/logging.rs
pub struct DaemonLogger {
    subsystem: String,  // com.github.mariohewardt.listent
}

impl DaemonLogger {
    pub fn log_daemon_start(&self, interval: f64);
    pub fn log_daemon_stop(&self);
    pub fn log_process_detected(&self, process: &MonitoredProcess);
    pub fn log_error(&self, error: &str);
}
```

### What Was Simplified
- ❌ No `DaemonConfiguration` struct - CLI args in plist instead
- ❌ No `config.rs` module - removed as dead code
- ❌ No `ipc.rs` module - restart daemon to change config
- ❌ No CLI subcommands - manual `launchctl` usage
- ✅ LaunchD plist generation retained
- ✅ Enhanced ULS logging retained
- ✅ Background daemon mode works
}
```

### Integration Points

#### Daemon Mode Execution
- Extend main.rs with daemon execution path
- No terminal output in daemon mode - ULS logging only
- Reuse existing monitor::polling logic with async wrapper
- Signal handling for graceful shutdown and config reload

### Integration Points

#### Daemon Mode Execution
- Extend main.rs with daemon execution path
- No terminal output in daemon mode - ULS logging only
- Reuse existing monitor::polling logic
- Signal handling for graceful shutdown (SIGTERM/SIGINT)

### Testing Approach
```rust
// Unit tests for plist generation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_launchd_plist_creation() {
        // Test plist generation with various configurations
    }
    
    #[test]
    fn test_plist_contains_program_arguments() {
        // Test CLI args embedded in plist
    }
}

// Integration tests in tests/ directory
// Test daemon installation workflow with launchctl
```

### Performance Considerations
- Use HashMap for O(1) process lookups during comparison
- Minimize allocations in polling loop (reuse collections)
- Handle large entitlements lists efficiently
- Profile memory usage during extended monitoring

## Recent Changes

### Phase 1: Core CLI Implementation (001-macos-rust-cli)
- **Status**: ✅ COMPLETE
- **Key Features**: Basic directory scanning, entitlement extraction, JSON/human output, path filtering
- **Architecture**: Modular design with scan, entitlements, output, and CLI modules

### Phase 2: Monitor Feature Implementation (002-add-monitor-switch)
- **Status**: ✅ COMPLETE 
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

### Phase 3: LaunchD Daemon Support (Simplified Implementation)
- **Status**: ✅ COMPLETE (Simplified)
- **Key Features Implemented**:
  - ✅ `--daemon` flag runs process monitoring in background
  - ✅ `--launchd` flag generates plist file
  - ✅ Enhanced ULS logging for daemon events
  - ✅ LaunchD plist generation with 14 unit tests
  - ✅ Integration with existing monitor functionality
- **Simplified Approach**:
  - ❌ No TOML config files (CLI args in plist)
  - ❌ No IPC mechanism (restart daemon to change config)
  - ❌ No automated install/uninstall commands (manual `launchctl`)
  - ✅ Simpler, more maintainable implementation
  - ✅ Follows standard macOS launchd conventions
- **Daemon Module Structure**:
  - ✅ `src/daemon/mod.rs` - Daemon orchestration
  - ✅ `src/daemon/launchd.rs` - Plist generation
  - ✅ `src/daemon/logging.rs` - Enhanced ULS logging
  - ❌ `src/daemon/config.rs` - Removed as dead code
  - ❌ `src/daemon/ipc.rs` - Not implemented

### Phase 4: Code Quality & Cleanup
- **Status**: ✅ COMPLETE
- **Improvements**:
  - ✅ Removed `thiserror` dependency (15 → 14 dependencies)
  - ✅ Added 31 new unit tests (18 → 49 tests, +172% increase)
  - ✅ Fixed 2 flaky tests
  - ✅ Analyzed test coverage (57.3% unit, ~75-80% effective)
  - ✅ Removed 42 lines of dead code
  - ✅ Cleaned repository files (help, output.json removed)
  - ✅ Eliminated all compiler warnings (0 warnings)
  - ✅ All 158 tests passing

### Files Modified/Added (Cumulative)
- **Core Architecture**: Complete modular structure in `src/`
- **CLI Enhancement**: Comprehensive argument parsing with subcommands  
- **Performance**: Fast counting, optimized progress tracking, efficient file filtering
- **Documentation**: Updated README with all features, troubleshooting, examples
- **Testing**: Comprehensive contract, integration, and unit test coverage
- **Daemon Support**: Full LaunchD integration with configuration management

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