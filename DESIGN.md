# listent - Design Specification

**Project**: listent - macOS Code Signing Entitlement Scanner  
**Version**: 1.0.0  
**Last Updated**: October 21, 2025

---

## Architecture Overview

listent is a single-binary CLI tool built in Rust for macOS. It follows a modular architecture with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│          CLI Interface (clap)           │
│  Argument parsing & validation          │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴────────┐
       │                │
       ▼                ▼
┌─────────────┐  ┌─────────────┐
│  Static     │  │  Monitor/   │
│  Scan Mode  │  │  Daemon     │
└──────┬──────┘  └──────┬──────┘
       │                │
       └────────┬───────┘
                │
     ┌──────────┴──────────┐
     │                     │
     ▼                     ▼
┌──────────┐       ┌──────────────┐
│  Scan    │       │  Entitlements│
│  Engine  │◄─────►│  Extractor   │
└──────────┘       └──────────────┘
     │                     │
     └──────────┬──────────┘
                │
                ▼
        ┌──────────────┐
        │   Output     │
        │  Formatter   │
        └──────────────┘
```

---

## Module Structure

### Core Modules

```
src/
├── main.rs              # Entry point, mode dispatch
├── lib.rs               # Library exports
├── constants.rs         # Global constants
├── cli/
│   └── mod.rs           # CLI argument parsing (clap)
├── models/
│   ├── mod.rs           # Data structures
│   └── tests.rs         # Model validation tests
├── scan/
│   └── mod.rs           # File system scanning
├── entitlements/
│   ├── mod.rs           # Entitlement extraction
│   ├── native.rs        # codesign integration
│   └── pattern_matcher.rs # Glob pattern matching
├── output/
│   ├── mod.rs           # Output formatting
│   └── progress.rs      # Progress indicators
├── monitor/
│   ├── mod.rs           # Module exports
│   ├── core.rs          # Monitoring engine
│   ├── polling.rs       # Polling loop
│   ├── process_tracker.rs # Process state
│   └── unified_logging.rs # macOS ULS integration
└── daemon/
    ├── mod.rs           # Daemon orchestration
    ├── launchd.rs       # LaunchD plist generation
    └── logging.rs       # Enhanced daemon logging
```

---

## Data Models

### Core Types

#### MonitoredProcess
```rust
#[derive(Debug, Clone, Serialize)]
pub struct MonitoredProcess {
    pub pid: u32,
    pub name: String,
    pub executable_path: PathBuf,
    pub entitlements: Vec<String>,
    pub discovery_timestamp: SystemTime,
}
```
**Purpose**: Represents a process detected during monitoring  
**Used by**: Monitor and daemon modes

#### ScanFilters
```rust
pub struct ScanFilters {
    pub path_filters: Vec<PathBuf>,
    pub entitlement_filters: Vec<String>,
}
```
**Purpose**: Filter criteria for scanning and monitoring  
**Used by**: All operating modes

#### PollingConfiguration
```rust
pub struct PollingConfiguration {
    pub interval: Duration,
    pub path_filters: Vec<PathBuf>,
    pub entitlement_filters: Vec<String>,
    pub output_json: bool,
    pub quiet_mode: bool,
}
```
**Purpose**: Configuration for monitor/daemon polling loop  
**Used by**: Monitor and daemon modes

#### ScanResult
```rust
#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub path: PathBuf,
    pub entitlements: Vec<String>,
}
```
**Purpose**: Result from scanning a single binary  
**Used by**: Static scan mode, JSON output

---

## Component Design

### 1. CLI Module (`src/cli/mod.rs`)

**Responsibility**: Parse and validate command-line arguments

**Key Structure**:
```rust
#[derive(Parser)]
pub struct Args {
    // Paths to scan (default: /Applications)
    pub path: Vec<PathBuf>,
    
    // Entitlement filters (glob patterns)
    pub entitlement: Vec<String>,
    
    // Output format
    pub json: bool,
    pub quiet: bool,
    
    // Operating modes
    pub monitor: bool,
    pub daemon: bool,
    pub launchd: bool,
    
    // Monitoring configuration
    pub interval: f64,  // 0.1-300.0 seconds
}
```

**Validation**:
- Interval range: 0.1-300.0 seconds
- Path existence checks (warnings only)
- Flag compatibility (--launchd requires --daemon)

### 2. Scan Module (`src/scan/mod.rs`)

**Responsibility**: File system traversal and Mach-O binary detection

**Key Functions**:
```rust
// Discover binary files in paths
pub fn discover_binaries(
    paths: &[PathBuf],
    filters: &ScanFilters,
    quiet: bool,
    progress_tracker: &mut ProgressTracker,
) -> Result<Vec<PathBuf>>

// Check if file is Mach-O binary
fn is_macho_binary(path: &Path) -> bool
```

**Algorithm**:
1. Recursive directory traversal
2. File type detection (Mach-O magic bytes: `0xFEEDFACE`, `0xFEEDFACF`, `0xCAFEBABE`)
3. Permission handling (skip inaccessible files with warning)
4. Path filtering (if specified)
5. Progress tracking

**Performance**:
- Pre-counts files for progress tracking (parallel with `find`)
- Skips non-executable files early
- Minimal memory footprint (streaming approach)

### 3. Entitlements Module (`src/entitlements/`)

**Responsibility**: Extract and filter code signing entitlements

#### Native Extraction (`native.rs`)
```rust
pub fn extract_entitlements(binary_path: &Path) -> Result<Vec<String>>
```
**Implementation**:
- Executes `codesign -d --entitlements - <path>`
- Parses XML plist output
- Extracts `<key>` elements
- Returns sorted unique list

**Error Handling**:
- Missing code signature → Empty entitlements list
- Invalid binary → Logs warning, returns empty list
- Permission denied → Propagates error

#### Pattern Matching (`pattern_matcher.rs`)
```rust
pub fn matches_pattern(entitlement: &str, pattern: &str) -> bool
```
**Glob Support**:
- `*` - matches any sequence of characters
- `?` - matches single character  
- `[abc]` - matches character set
- Exact match if no wildcards

**Examples**:
- `com.apple.security.*` matches `com.apple.security.network.client`
- `*network*` matches `com.apple.security.network.client`
- `com.apple.*.client` matches `com.apple.security.network.client`

### 4. Output Module (`src/output/`)

**Responsibility**: Format scan results for output

#### Human-Readable Format
```
Found 3 binaries with entitlements:

/Applications/Safari.app/Contents/MacOS/Safari:
  - com.apple.security.network.client
  - com.apple.security.network.server
  - com.apple.security.cs.allow-jit

Scan Summary:
  - Total files scanned: 150
  - Matched binaries: 3
  - Duration: 2.3s
```

#### JSON Format
```json
{
  "results": [
    {
      "path": "/Applications/Safari.app/Contents/MacOS/Safari",
      "entitlements": [
        "com.apple.security.network.client",
        "com.apple.security.network.server"
      ]
    }
  ],
  "summary": {
    "total_files": 150,
    "matched_files": 3,
    "duration_ms": 2300
  }
}
```

#### Progress Tracking (`progress.rs`)
```
Scanning: Processed 45/150 files (scanned: 12, skipped: 33) - /Applications/Utilities
```
**Features**:
- Real-time file counting
- Directory name display
- Scan/skip statistics
- No flickering (overwrites same line)

### 5. Monitor Module (`src/monitor/`)

**Responsibility**: Real-time process monitoring

#### Core Engine (`core.rs`)
```rust
pub struct ProcessMonitoringCore {
    known_processes: HashMap<u32, MonitoredProcess>,
    sysinfo: System,
}

impl ProcessMonitoringCore {
    pub fn detect_new_processes(&mut self) -> Result<Vec<MonitoredProcess>>
}
```

**Algorithm**:
1. Query all running processes (via `sysinfo` crate)
2. Compare with known process PIDs
3. Extract entitlements for new processes
4. Apply path and entitlement filters
5. Update known processes set
6. Return new matching processes

**Performance Optimizations**:
- HashMap for O(1) PID lookups
- Lazy entitlement extraction (only for new processes)
- Pre-allocated collections
- Process path caching

#### Polling Loop (`polling.rs`)
```rust
pub fn start_polling(config: PollingConfiguration) -> Result<()>
```

**Implementation**:
1. Initialize ProcessMonitoringCore
2. Set up signal handlers (SIGINT, SIGTERM)
3. Loop:
   - Sleep for interval
   - Detect new processes
   - Output results
   - Check for shutdown signal
4. Clean shutdown

**Signal Handling**:
- Ctrl+C (SIGINT) → Clean shutdown, exit code 0
- SIGTERM → Clean shutdown, exit code 0
- Daemon mode: No terminal output, ULS logging only

#### Unified Logging (`unified_logging.rs`)
```rust
pub fn log_process_detection(
    process: &MonitoredProcess,
    subsystem: &str,
)
```

**macOS ULS Integration**:
- Subsystem: `com.github.mariohewardt.listent`
- Category: `monitor` or `daemon`
- Structured log entries with process metadata
- Query: `log show --predicate 'subsystem == "com.github.mariohewardt.listent"'`

### 6. Daemon Module (`src/daemon/`)

**Responsibility**: Background daemon operation and LaunchD integration

#### LaunchD Integration (`launchd.rs`)
```rust
pub fn generate_launchd_plist(
    daemon_path: &Path,
    interval: f64,
    paths: &[PathBuf],
    entitlements: &[String],
    run_at_load: bool,
    keep_alive: bool,
) -> Result<String>
```

**Generated Plist Structure**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.github.mariohewardt.listent</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/listent</string>
        <string>--daemon</string>
        <string>--interval</string>
        <string>1.0</string>
        <!-- CLI args for filters -->
    </array>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>/var/log/listent.log</string>
    
    <key>StandardErrorPath</key>
    <string>/var/log/listent.error.log</string>
</dict>
</plist>
```

**Key Design Decision**: Configuration stored as CLI arguments in plist rather than separate TOML files. This simplifies implementation while maintaining full functionality.

#### Daemon Orchestration (`mod.rs`)
```rust
pub fn run_daemon_mode(config: PollingConfiguration) -> Result<()>
```

**Implementation**:
1. Initialize DaemonLogger
2. Log daemon start to ULS
3. Set up signal handlers (SIGTERM, SIGINT)
4. Create ProcessMonitoringCore
5. Run polling loop (no terminal output)
6. Log all events to ULS
7. Clean shutdown on signal

**Logging** (`logging.rs`):
```rust
pub struct DaemonLogger {
    subsystem: String,
}

impl DaemonLogger {
    pub fn log_daemon_start(&self, interval: f64);
    pub fn log_daemon_stop(&self);
    pub fn log_process_detected(&self, process: &MonitoredProcess);
    pub fn log_error(&self, error: &str);
}
```

---

## Operating Modes

### Static Scan Mode (Default)

**Entry Point**: `main.rs` when `!args.monitor && !args.daemon`

**Execution Flow**:
```
Parse CLI args
    ↓
Validate interval (if present)
    ↓
Initialize scan filters
    ↓
Discover binaries (with progress)
    ↓
Extract entitlements (parallel where possible)
    ↓
Filter by entitlement patterns
    ↓
Format output (human or JSON)
    ↓
Exit
```

### Monitor Mode

**Entry Point**: `main.rs` when `args.monitor`

**Execution Flow**:
```
Parse CLI args
    ↓
Validate interval
    ↓
Create PollingConfiguration
    ↓
Start polling loop
    ↓
    ┌──────────────────┐
    │ Poll processes   │
    │ Extract ents     │
    │ Filter & output  │
    │ Sleep interval   │
    └────────┬─────────┘
             │
        Signal? ──No──┐
             │        │
            Yes       │
             │        │
             ▼        │
        Clean exit◄───┘
```

### Daemon Mode

**Entry Point**: `main.rs` when `args.daemon`

**Special Behavior**:
- No terminal output (stdout/stderr to log files)
- All events logged to ULS
- Signal handling for graceful shutdown
- Reuses monitor polling logic

**LaunchD Plist Generation**:
```
args.daemon && args.launchd
    ↓
Generate plist XML with CLI args
    ↓
Output to stdout
    ↓
Exit (user installs plist manually)
```

---

## Dependencies

### Production Dependencies
```toml
[dependencies]
clap = "4.5"          # CLI argument parsing
serde = "1.0"         # Serialization
serde_json = "1.0"    # JSON output
glob = "0.3"          # Pattern matching
sysinfo = "0.30"      # Process enumeration
signal-hook = "0.3"   # Signal handling
anyhow = "1.0"        # Error handling
```

### Development Dependencies
```toml
[dev-dependencies]
assert_cmd = "2.0"    # CLI testing
predicates = "3.1"    # Output assertions
tempfile = "3.12"     # Temporary directories
```

**Design Principle**: Minimal dependencies, prefer std library where possible

---

## Error Handling

### Error Types
All functions return `Result<T, anyhow::Error>` for consistent error handling.

### Error Scenarios

| Scenario | Behavior | Exit Code |
|----------|----------|-----------|
| Invalid interval | Print error message | 1 |
| Permission denied (scan) | Log warning, continue | 0 |
| No binaries found | Print summary with 0 matches | 0 |
| Binary extraction fails | Log warning, skip binary | 0 |
| Monitor mode interrupted | Clean shutdown | 0 |
| Daemon start failure | Log error to ULS | 1 |

### Logging Strategy
- **Static/Monitor**: stderr for warnings, stdout for results
- **Quiet mode**: Suppress warnings
- **Daemon mode**: All output to ULS

---

## Performance Characteristics

### Static Scan
- **Time Complexity**: O(n) where n = number of files
- **Space Complexity**: O(m) where m = number of matching binaries
- **Optimization**: Streaming results, minimal memory buffering

### Monitor Mode
- **Time Complexity**: O(p) per poll where p = number of processes
- **Space Complexity**: O(p) for known processes set
- **Optimization**: HashMap for O(1) PID lookups, lazy extraction

### Pattern Matching
- **Time Complexity**: O(p × e) where p = patterns, e = entitlements
- **Optimization**: Early termination on first match (OR logic)

---

## Testing Strategy

### Test Types

#### Unit Tests (49 tests)
- Constants validation
- Model error handling
- Mach-O binary detection
- LaunchD plist generation
- Pattern matching logic

#### Integration Tests (109 tests)
- Full scan workflows
- Monitor mode operation
- Filter combinations
- JSON schema validation
- Signal handling
- LaunchD integration

### Test Coverage
- **Unit tests**: 57.3% of production code
- **Effective coverage**: ~75-80% with integration tests
- **Key areas**: All public APIs, critical paths, error scenarios

### Test Helpers
```
tests/helpers/
├── mod.rs                 # Test environment setup
└── reliable_runner.rs     # Process lifecycle management
```

---

## Build & Deployment

### Build Configuration
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization
strip = true            # Strip symbols
```

### Targets
- **macOS Intel**: `x86_64-apple-darwin`
- **macOS Apple Silicon**: `aarch64-apple-darwin`
- **Universal Binary**: `--target x86_64-apple-darwin --target aarch64-apple-darwin`

### Installation Methods
1. **Homebrew**: `brew install mariohewardt/tools/listent`
2. **Pre-built binary**: Download from GitHub releases
3. **From source**: `cargo install --path .`

---

## Security Considerations

### Code Signing
- Tool itself should be signed for distribution
- Uses system `codesign` utility (trusted)
- No entitlements required for basic operation
- System directories may require `sudo`

### Permissions
- **Read-only**: Tool only reads files, never modifies
- **Process access**: Uses standard macOS APIs (sysinfo)
- **Daemon**: Can run as user or system-level service

### Privacy
- No network access
- No data collection or telemetry
- All logging stays local (ULS)
- No third-party services

---

## Design Decisions & Rationale

### 1. Single Binary
**Decision**: Compile as single executable, no shared libraries  
**Rationale**: Simplifies deployment, reduces dependencies

### 2. Rust Language
**Decision**: Implement in Rust  
**Rationale**: Memory safety, performance, excellent macOS support

### 3. CLI-only Interface
**Decision**: No GUI, terminal-based only  
**Rationale**: Target audience (developers, sysadmins) prefer CLI

### 4. No Configuration Files (Daemon)
**Decision**: CLI args in launchd plist instead of TOML config  
**Rationale**: Simpler implementation, standard macOS pattern, easier troubleshooting

### 5. Manual LaunchD Management
**Decision**: User runs `launchctl` commands manually  
**Rationale**: Follows macOS conventions, avoids privilege escalation complexity

### 6. Glob Patterns (Not Regex)
**Decision**: Use glob patterns for entitlement filtering  
**Rationale**: Simpler for users, sufficient for common use cases

### 7. Lazy Entitlement Extraction
**Decision**: Extract entitlements only when needed  
**Rationale**: Significant performance improvement (avoid unnecessary codesign calls)

### 8. JSON Schema
**Decision**: Flat structure with results array and summary object  
**Rationale**: Easy to parse, compatible with jq, suitable for automation

---

## Future Considerations

While not implemented in v1.0, these design considerations inform potential future enhancements:

- **Configuration Files**: TOML support for saved filter presets
- **Advanced Patterns**: Regex support for complex matching
- **Process Trees**: Parent-child process relationship tracking
- **Export Formats**: CSV, SQLite, other structured formats
- **Remote Logging**: Centralized log aggregation
- **Performance Metrics**: Detailed timing and resource usage stats

---

*This design reflects the v1.0.0 implementation as of October 21, 2025.*
