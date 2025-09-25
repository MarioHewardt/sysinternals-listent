# Sysinternals listent - macOS Entitlement Monitor

Sysinternals `listent` is a fast command-line tool for discovering and analyzing code signing entitlements in macOS executable binaries. Designed for security researchers, developers, and system administrators who need to audit application permissions and understand the security capabilities requested by macOS applications.

![listent Demo](assets/listent-demo.gif)

## Features

- **Static Directory Scanning**: Efficiently traverse directory trees with intelligent filtering and real-time progress indicators
- **Real-time Process Monitoring**: Monitor new processes as they launch with configurable polling intervals  
- **Background Daemon Mode**: Persistent process monitoring with consistent CLI argument structure and macOS LaunchD integration
- **Flexible Entitlement Filtering**: Pattern matching with glob support (`*`, `?`, `[]`) for precise entitlement discovery
- **Multiple Output Formats**: Human-readable tables and structured JSON for automation and scripting
- **Multi-Path Support**: Scan multiple directories simultaneously in a single operation
- **Graceful Interruption**: Clean cancellation with Ctrl+C and proper resource cleanup
- **Performance Optimized**: Smart file filtering, minimal I/O operations, and efficient memory usage
- **Security Focused**: Local processing only, no network communication, respects system permissions

## Install

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
# Show help
./target/release/listent --help

# Basic scan (default location: /Applications)
./target/release/listent

# Scan specific directories
./target/release/listent /usr/bin /Applications

# Show version
./target/release/listent --version
```

## Development

Please see development instructions in [`DEVELOPMENT.md`](DEVELOPMENT.md).

## Usage

```
Usage: listent [OPTIONS] [PATH]...

Arguments:
  [PATH]...
          Directory or file paths to scan (default: /Applications)
          
          Supports multiple paths: listent /path1 /path2 /path3

Options:
  -e, --entitlement <KEY>
          Filter by entitlement key (exact match or glob pattern)
          
          Supports exact matching (e.g., "com.apple.security.network.client") and glob patterns (e.g.,
          "com.apple.security.*", "*network*", "*.client").
          
          Multiple filters: -e key1 -e key2 OR -e key1,key2 (logical OR)

  -j, --json
          Output in JSON format

  -q, --quiet
          Suppress warnings about unreadable files

  -m, --monitor
          Enable real-time process monitoring mode

      --interval <SECONDS>
          Polling interval in seconds (0.1 - 300.0) [monitoring mode only]
          
          [default: 1.0]

      --daemon
          Run as background daemon (implies --monitor)

      --launchd
          Install as LaunchD service (requires --daemon and sudo)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Examples

#### Static Scanning

```bash
# Basic scan with progress
listent /Applications

# Multi-directory scan with entitlement filtering
listent /usr/bin /Applications -e "*security*"

# Find all network-related entitlements
listent -e "*network*" --json | jq '.results[].entitlements'

# Scan quietly (suppress progress indicators)
listent /System/Applications --quiet

# Multiple entitlement patterns
listent -e "com.apple.security.*" -e "*debug*" /Applications
```

#### Case Sensitivity Examples

```bash
# Default behavior is case-sensitive
listent -e "com.apple.security.Network"  # Matches exact case only
listent -e "*Network*"                   # Matches any case in pattern

# Use glob patterns for flexible matching
listent -e "*network*"     # Matches: network, Network, NETWORK, etc.
listent -e "*[Nn]etwork*"  # Matches: Network, network (bracket notation)
```

#### Real-time Process Monitoring

```bash
# Monitor all new processes with default 1-second interval
listent --monitor

# Monitor with custom polling interval
listent --monitor --interval 0.5

# Monitor specific entitlements only
listent --monitor -e "com.apple.security.network.*"

# Monitor with JSON output for automation
listent --monitor --json -e "*security*"
```

#### Background Daemon Mode

```bash
# Start daemon with specific paths and entitlement filters
listent --daemon /Applications /usr/bin -e "com.apple.security.*"

# Daemon with custom polling interval
listent --daemon /Applications -e "*network*" --interval 2.0

# Quiet daemon mode (no console output)
listent --daemon /Applications -e "com.apple.*" --quiet

# JSON output for log aggregation systems
listent --daemon /Applications -e "*security*" --json
```

### Advanced Usage

#### Entitlement Pattern Matching

`listent` supports flexible entitlement filtering using glob patterns:

```bash
# Exact match
listent -e "com.apple.security.network.client"

# Wildcard patterns  
listent -e "com.apple.security.*"        # All Apple security entitlements
listent -e "*network*"                   # Any entitlement containing "network"
listent -e "*.debug.*"                   # Debug-related entitlements

# Character class patterns
listent -e "com.apple.[sp]*"             # Matches 'security' or 'private' branches
listent -e "*[Nn]etwork*"                # Case variations

# Multiple patterns (OR logic)
listent -e "com.apple.private.*" -e "*.debug.*"
```

#### Output Format Options

**Human-readable output (default):**
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

**JSON output for automation:**
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

### Daemon Integration

The daemon mode provides continuous process monitoring with the same CLI interface as scan and monitor modes:

**Key Features:**
- Consistent argument structure across all modes (`listent [--mode] [paths...] [-e entitlements...]`)
- Background process monitoring with configurable intervals
- macOS Unified Logging System integration
- Automatic LaunchD integration for system services
- No configuration files or IPC complexity

**Usage Pattern:**
```bash
# Same arguments work across modes:

# Static scan
listent /Applications -e "com.apple.*"

# Monitor mode  
listent --monitor /Applications -e "com.apple.*" --interval 1.5

# Daemon mode (background)
listent --daemon /Applications -e "com.apple.*" --interval 1.5
```

### Troubleshooting

#### Ctrl+C Not Working in External Terminals

If Ctrl+C doesn't interrupt scans in Terminal.app or iTerm2, this is due to macOS terminal signal handling:

```bash
# Workaround - run before using listent:
trap - INT

# Then Ctrl+C should work normally
listent /Applications
```

*Note: This issue doesn't affect VS Code's integrated terminal.*

#### Permission Issues

```bash
# For system directories requiring elevated access:
sudo listent /System/Library

# For daemon mode with system paths:
sudo listent --daemon /System/Library /usr/bin -e "com.apple.*"

# Check file permissions:
ls -la /path/to/directory
```

#### Performance Optimization

```bash
# Scan with quiet mode for faster execution:
listent /large/directory --quiet

# Use specific entitlement filters to reduce processing:
listent -e "com.apple.security.*" /Applications

# Monitor mode with longer intervals for less CPU usage:
listent --monitor --interval 5.0
```

## License

This project is licensed under the MIT OR Apache-2.0 License. See the [LICENSE](LICENSE) file for details.

## Additional Links
- [Code](https://github.com/mariohewardt/listent)
- [Issues](https://github.com/mariohewardt/listent/issues)
- [Pull requests](https://github.com/mariohewardt/listent/pulls)
- [Actions](https://github.com/mariohewardt/listent/actions)
- [Security](SECURITY.md)
- [Threat Model](docs/threat-model.md)
- [Development Guide](DEVELOPMENT.md)
- [Release Checklist](RELEASE_CHECKLIST.md)
