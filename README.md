# listent

A fast command-line tool to discover and list code signing entitlements for macOS executable binaries. Supports static scanning, real-time process monitoring, and background daemon operation.

## Overview

`listent` recursively scans directories to find executable binaries and extracts their code signing entitlements using the `codesign` utility. It's designed for security researchers, developers, and system administrators who need to audit or understand the permissions requested by macOS applications.

## Features

### Core Capabilities
- **Fast scanning**: Efficiently traverses directory trees with smart filtering and progress indicators
- **Entitlement extraction**: Uses macOS `codesign` to extract entitlements from binaries  
- **Flexible filtering**: Filter by paths and specific entitlement keys with glob pattern support
- **Multiple output formats**: Human-readable and structured JSON output
- **Multiple paths**: Scan multiple directories in a single command
- **Graceful interrupts**: Clean cancellation with Ctrl+C

### Operating Modes

#### 1. Static Scan Mode (Default)
Scan files and directories for entitlements:
```bash
# Scan default location (/Applications)
listent

# Scan specific paths
listent /usr/bin /Applications

# Filter by entitlement patterns
listent -e "com.apple.security.*"
listent -e "*network*" -e "*debug*"

# JSON output for automation
listent /usr/bin -e "*security*" --json
```

#### 2. Real-time Monitor Mode
Monitor new processes for entitlements:
```bash
# Monitor all new processes
listent --monitor

# Monitor with custom polling interval
listent --monitor --interval 0.5

# Monitor specific entitlements only
listent --monitor -e "com.apple.security.network.*"
```

#### 3. Background Daemon Mode
Run monitoring as a persistent system service:
```bash
# Install and start daemon
listent install-daemon

# Check daemon status
listent daemon-status

# Update configuration without restart
listent update-config --interval 2.0

# View daemon logs
listent logs --follow

# Stop and uninstall
listent daemon-stop
listent uninstall-daemon
```

## Examples

### Static Scanning
```bash
# Basic scan with progress
listent /Applications

# Multi-directory scan with filtering
listent /usr/bin /Applications -e "*security*"

# Find all network-related entitlements
listent -e "*network*" --json | jq '.results[].entitlements'

# Scan quietly (suppress warnings)
listent /System/Applications --quiet
```

### Process Monitoring
```bash
# Monitor all processes with 2-second intervals
listent --monitor --interval 2.0

# Monitor only security-related entitlements
listent --monitor -e "com.apple.security.*"

# Run as daemon with custom config
listent --daemon --monitor --config /etc/listent/daemon.toml
```

### Daemon Management
```bash
# Install daemon with default monitoring
listent install-daemon

# Update daemon to monitor specific entitlements
listent update-config -e "com.apple.private.*"

# View recent daemon activity
listent logs --since "1 hour ago"

# Check if daemon is running
listent daemon-status
```

## Installation

### From Source
```bash
# Clone and build
git clone https://github.com/mariohewardt/listent
cd listent
cargo build --release

# The binary will be at ./target/release/listent
```

### Quick Start
```bash
# Build
cargo build --release

# Show help
./target/release/listent --help

# Basic scan
./target/release/listent /Applications

# Show version
./target/release/listent --version
```

## Configuration

### Command Line Options
- **Paths**: Multiple paths can be specified: `listent /path1 /path2`
- **Entitlement filtering**: `-e "pattern"` supports exact matches and globs (`*`, `?`, `[]`)
- **Output format**: `--json` for structured output, default is human-readable
- **Monitoring**: `--monitor` enables real-time process monitoring
- **Daemon mode**: `--daemon` runs as background service (requires `--monitor`)

### Entitlement Patterns
```bash
# Exact match
-e "com.apple.security.network.client"

# Wildcard patterns
-e "com.apple.security.*"        # All Apple security entitlements
-e "*network*"                   # Any entitlement containing "network"
-e "*.debug.*"                   # Debug-related entitlements

# Multiple patterns (OR logic)
-e "com.apple.private.*" -e "*.debug.*"
```

### Daemon Configuration
Daemon settings can be configured via:
1. **Configuration file**: `/etc/listent/daemon.toml` or custom path with `--config`
2. **Runtime updates**: `listent update-config` command
3. **Command line**: Initial settings when starting daemon

Example daemon configuration:
```toml
[daemon]
polling_interval = 1.0
auto_start = true

[monitoring]
entitlement_filters = ["com.apple.security.*", "*network*"]
output_json = false

[logging]
level = "info"
subsystem = "com.github.mariohewardt.listent"
```

## Troubleshooting

### Ctrl+C Not Working in External Terminals

If Ctrl+C doesn't interrupt the scan in Terminal.app or iTerm2, this is due to a macOS terminal signal handling issue. 

**Workaround**: Before running `listent`, execute:
```bash
trap - INT
```

This removes any existing interrupt trap and restores the default SIGINT behavior. After this, Ctrl+C should work normally.

Note: This issue doesn't affect VS Code's integrated terminal.

## Output Formats

### Human-Readable (Default)
```
Found 2 binaries with 5 total entitlements:

/usr/bin/security:
  com.apple.private.platformsso.security: true

/usr/bin/nc:
  com.apple.security.network.client: true
  com.apple.security.network.server: true

Scan Summary:
  Scanned: 156 files
  Matched: 2 files
  Duration: 2.34s
```

### JSON Format
```json
{
  "results": [
    {
      "path": "/usr/bin/security",
      "entitlements": {
        "com.apple.private.platformsso.security": true
      },
      "entitlement_count": 1
    }
  ],
  "summary": {
    "scanned": 156,
    "matched": 2,
    "duration_ms": 2340,
    "skipped_unreadable": 0
  }
}
```

## Architecture

### Components
- **CLI Module**: Command-line argument parsing and validation
- **Scan Engine**: Fast directory traversal and binary discovery
- **Entitlement Extractor**: Uses macOS `codesign` to extract entitlements
- **Monitor Engine**: Real-time process monitoring with configurable polling
- **Daemon Controller**: LaunchD integration for background operation
- **Output Formatter**: Human-readable and JSON output generation

### Performance
- **Smart filtering**: Checks file permissions before expensive operations
- **Progress tracking**: Real-time progress with file counts and directory context
- **Optimized I/O**: Minimal file operations, efficient memory usage
- **Interrupt handling**: Immediate response to Ctrl+C with graceful cleanup

## Requirements

### System Requirements
- **macOS**: 10.15+ (uses `codesign` utility)
- **Architecture**: x86_64 or ARM64 (Apple Silicon)
- **Permissions**: Read access to target directories; admin privileges for daemon installation

### Runtime Dependencies
- **codesign**: System utility for entitlement extraction (included with macOS)
- **launchd**: System service manager for daemon mode (included with macOS)

## Security Considerations

### Permissions
- **Static scanning**: Requires read access to target directories
- **Process monitoring**: Requires ability to enumerate running processes
- **Daemon installation**: Requires administrator privileges for LaunchD registration

### Privacy
- **No data collection**: All processing is local, no network communication
- **System logging**: Daemon mode logs to macOS Unified Logging System only
- **Entitlement access**: Only reads publicly accessible entitlement information

## Development

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt --all

# Lint code
cargo clippy --all-targets -- -D warnings
```

### Project Structure
```
src/
├── main.rs              # Entry point and CLI coordination
├── cli/                 # Command-line argument parsing
├── models/              # Data structures and configuration
├── scan/                # Static file/directory scanning
├── entitlements/        # Entitlement extraction and filtering
├── output/              # Output formatting and progress tracking
├── monitor/             # Real-time process monitoring
└── daemon/              # LaunchD daemon integration
```

### Testing
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Contract tests (CLI behavior)
cargo test --test contract

# Build and test all features
cargo test --all-features
```

### Contributing
1. Follow the project's constitutional principles in `CONSTITUTION.md`
2. Check "Expansion Triggers" before adding complexity
3. Ensure all tests pass
4. Update documentation for new features

## Related Projects

- **macOS Security Research**: Tools for analyzing application entitlements and permissions
- **System Monitoring**: Real-time process and security event monitoring tools
- **DevOps Security**: Automated security compliance and audit tools

## Roadmap

See "Expansion Triggers" in `CONSTITUTION.md` before adding complexity.

### Potential Enhancements
- **Entitlement analysis**: Pattern detection and security recommendations  
- **Historical tracking**: Process monitoring with persistent storage
- **Integration APIs**: Webhooks and external system integration
- **Performance optimization**: Parallel processing and caching

## Security

For vulnerability reporting, supported versions, and a condensed risk overview see [`SECURITY.md`](SECURITY.md).

A detailed, expanded threat model (architecture, STRIDE analysis, mitigations, and roadmap) is available at [`docs/threat-model.md`](docs/threat-model.md).

If you believe you have found a security issue, please follow the disclosure process in `SECURITY.md` rather than opening a public issue.

## License

MIT OR Apache-2.0

## Support

For issues, feature requests, or contributions, please use the project's GitHub repository.
