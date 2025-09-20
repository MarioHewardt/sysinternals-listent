# CLI Contract: Daemon Management Commands

**Date**: September 20, 2025  
**Purpose**: Define command-line interface contracts for daemon operations

## New Command Structure

### Subcommands
The existing CLI will be extended with new subcommands for daemon management:

```bash
listent install-daemon [--config PATH] [--auto-start] [--user]
listent uninstall-daemon [--force] 
listent daemon-status [--json] [--verbose]
listent update-config KEY=VALUE [KEY=VALUE...] [--restart-if-needed]
listent logs [--follow] [--since TIME] [--json] [--filter PATTERN]
```

## Command Specifications

### install-daemon
**Purpose**: Install and configure the listent daemon via launchd

```bash
listent install-daemon [OPTIONS]
```

**Options**:
- `--config PATH`: Use custom configuration file (default: generate from current settings)
- `--auto-start`: Enable automatic startup on boot (default: true)
- `--user`: Install as user agent instead of system daemon (requires different privileges)
- `--dry-run`: Show what would be installed without making changes
- `--force`: Overwrite existing installation

**Exit Codes**:
- `0`: Success - daemon installed and started
- `1`: General error - see stderr for details
- `2`: Permission denied - requires root/sudo  
- `3`: Already installed - use --force to overwrite
- `4`: Configuration invalid - fix config and retry
- `5`: LaunchD operation failed - check system logs

**Output Examples**:
```bash
# Success
$ sudo listent install-daemon
✓ Created configuration: /etc/listent/daemon.toml
✓ Generated launchd plist: /Library/LaunchDaemons/com.github.mariohewardt.listent.plist
✓ Loaded daemon service: com.github.mariohewardt.listent
✓ Daemon started successfully (PID: 1234)

Daemon is now monitoring all processes. Use 'listent daemon-status' to check status.
Use 'listent logs' to view monitoring output.

# Error - insufficient privileges
$ listent install-daemon
Error: Installing system daemon requires root privileges.
Run: sudo listent install-daemon

# Error - already installed
$ sudo listent install-daemon  
Error: Daemon is already installed. Use --force to reinstall.
Current status: Running (PID: 1234)
```

**JSON Output** (with --json):
```json
{
  "success": true,
  "daemon_pid": 1234,
  "config_path": "/etc/listent/daemon.toml",
  "plist_path": "/Library/LaunchDaemons/com.github.mariohewardt.listent.plist",
  "service_name": "com.github.mariohewardt.listent",
  "auto_start": true
}
```

### uninstall-daemon
**Purpose**: Completely remove the listent daemon from the system

```bash
listent uninstall-daemon [OPTIONS]
```

**Options**:
- `--force`: Remove even if daemon is running
- `--keep-config`: Preserve configuration files
- `--dry-run`: Show what would be removed

**Exit Codes**:
- `0`: Success - daemon removed completely
- `1`: General error
- `2`: Permission denied - requires root/sudo
- `3`: Not installed - nothing to remove
- `4`: Daemon running - stop first or use --force
- `5`: LaunchD operation failed

**Output Examples**:
```bash
# Success
$ sudo listent uninstall-daemon
✓ Stopped daemon service: com.github.mariohewardt.listent
✓ Unloaded launchd plist
✓ Removed plist file: /Library/LaunchDaemons/com.github.mariohewardt.listent.plist
✓ Removed configuration: /etc/listent/daemon.toml
✓ Cleaned up runtime files

Daemon has been completely removed from the system.

# Warning - daemon running
$ sudo listent uninstall-daemon
Warning: Daemon is currently running (PID: 1234)
Stop the daemon first with: sudo listent daemon-status --stop
Or use --force to stop and remove: sudo listent uninstall-daemon --force
```

### daemon-status
**Purpose**: Check daemon operational status and configuration

```bash
listent daemon-status [OPTIONS]
```

**Options**:
- `--json`: Output in JSON format
- `--verbose`: Show detailed statistics and configuration
- `--stop`: Stop the daemon (requires root)
- `--start`: Start the daemon (requires root)
- `--restart`: Restart the daemon (requires root)

**Exit Codes**:
- `0`: Success - status retrieved
- `1`: General error
- `2`: Permission denied (for control operations)
- `3`: Daemon not installed
- `4`: Daemon installed but not running
- `5`: LaunchD communication failed

**Output Examples**:
```bash
# Running daemon
$ listent daemon-status
Daemon Status: Running
  PID: 1234
  Started: 2025-09-20 09:30:15
  Uptime: 2h 15m 30s
  
Configuration:
  Polling Interval: 1.0s
  Monitoring: All processes
  Entitlement Filters: None
  Log Level: Info
  
Statistics:
  Processes Detected: 1,247
  Config Updates: 3
  Memory Usage: 8.2 MB
  Last Poll: 0.45ms

Use 'listent logs --follow' to see real-time monitoring.

# Verbose output
$ listent daemon-status --verbose
[Previous output plus:]

Process Filters:
  Path Filters: /Applications, /usr/bin
  Exclude System: false
  
IPC Status:
  Socket: /var/run/listent/daemon.sock (active)
  Connections: 0 current, 15 total
  Last Request: 5m ago
  
Performance:
  Average Poll Duration: 1.2ms
  Peak Poll Duration: 45.3ms
  Polls per Minute: 60.0
  Error Rate: 0.02%
```

**JSON Output**:
```json
{
  "status": "running",
  "pid": 1234,
  "started_at": "2025-09-20T09:30:15Z",
  "uptime_seconds": 8130,
  "config": {
    "polling_interval": 1.0,
    "monitor_all_processes": true,
    "path_filters": ["/Applications", "/usr/bin"],
    "entitlement_filters": [],
    "log_level": "info"
  },
  "stats": {
    "processes_detected": 1247,
    "config_updates": 3,
    "memory_usage_mb": 8.2,
    "last_poll_duration_ms": 0.45,
    "avg_poll_duration_ms": 1.2,
    "errors": 2
  }
}
```

### update-config
**Purpose**: Update daemon configuration without restart

```bash
listent update-config KEY=VALUE [KEY=VALUE...] [OPTIONS]
```

**Supported Keys**:
- `polling_interval=FLOAT`: Set polling interval (0.1-300.0)
- `log_level=LEVEL`: Set log level (debug|info|warning|error)
- `monitor_all=BOOL`: Enable/disable monitoring all processes
- `add_path_filter=PATH`: Add path filter
- `remove_path_filter=PATH`: Remove path filter
- `add_entitlement_filter=ENT`: Add entitlement filter
- `remove_entitlement_filter=ENT`: Remove entitlement filter
- `clear_path_filters`: Remove all path filters
- `clear_entitlement_filters`: Remove all entitlement filters

**Options**:
- `--restart-if-needed`: Restart daemon if changes require it
- `--validate-only`: Check if changes are valid without applying
- `--backup`: Create backup of current config before changes

**Exit Codes**:
- `0`: Success - configuration updated
- `1`: General error
- `2`: Permission denied
- `3`: Daemon not running
- `4`: Invalid configuration value
- `5`: IPC communication failed

**Output Examples**:
```bash
# Success
$ listent update-config polling_interval=0.5 log_level=debug
✓ Updated polling_interval: 1.0 → 0.5
✓ Updated log_level: info → debug
✓ Configuration applied successfully
✓ Daemon adopted new settings (no restart required)

Current status: Running with updated configuration

# Validation error
$ listent update-config polling_interval=500
Error: Invalid value for polling_interval: 500
Valid range: 0.1 to 300.0 seconds

# Multiple updates
$ listent update-config add_path_filter=/opt/homebrew/bin monitor_all=false
✓ Added path filter: /opt/homebrew/bin
✓ Updated monitor_all: true → false
✓ Configuration applied successfully

Active path filters: /Applications, /usr/bin, /opt/homebrew/bin
Monitoring mode: Filtered processes only
```

### logs
**Purpose**: View daemon monitoring logs from ULS

```bash
listent logs [OPTIONS]
```

**Options**:
- `--follow`, `-f`: Follow log output in real-time
- `--since TIME`: Show logs since specified time (e.g., "1h", "2025-09-20", "09:30")
- `--json`: Output in JSON format
- `--filter PATTERN`: Filter logs by pattern (process name, entitlement, etc.)
- `--level LEVEL`: Show only logs at specified level or higher
- `--limit N`: Limit output to N most recent entries
- `--subsystem`: Show only listent subsystem logs (default: true)

**Exit Codes**:
- `0`: Success - logs displayed
- `1`: General error
- `2`: No logs found matching criteria
- `3`: ULS query failed

**Output Examples**:
```bash
# Recent logs
$ listent logs --limit 5
2025-09-20 10:15:23.456 [INFO] Process detected: name=ps, pid=5678, path=/bin/ps, entitlements=["com.apple.system-task-ports.read"]
2025-09-20 10:15:20.123 [INFO] Process detected: name=Safari, pid=5432, path=/Applications/Safari.app/Contents/MacOS/Safari, entitlements=["com.apple.security.network.client", "com.apple.security.files.user-selected.read-write"]
2025-09-20 10:15:15.789 [INFO] Configuration updated: polling_interval changed to 0.5
2025-09-20 10:14:45.234 [INFO] Process detected: name=TextEdit, pid=5123, path=/System/Applications/TextEdit.app/Contents/MacOS/TextEdit, entitlements=["com.apple.security.files.user-selected.read-write"]
2025-09-20 10:14:30.567 [INFO] Daemon started: PID=1234

# Filtered logs
$ listent logs --filter "Safari" --since "1h"
2025-09-20 10:15:20.123 [INFO] Process detected: name=Safari, pid=5432, path=/Applications/Safari.app/Contents/MacOS/Safari, entitlements=["com.apple.security.network.client", "com.apple.security.files.user-selected.read-write"]
2025-09-20 09:45:12.890 [INFO] Process detected: name=Safari, pid=4987, path=/Applications/Safari.app/Contents/MacOS/Safari, entitlements=["com.apple.security.network.client", "com.apple.security.files.user-selected.read-write"]

# JSON output
$ listent logs --json --limit 1
[
  {
    "timestamp": "2025-09-20T10:15:23.456Z",
    "level": "info",
    "category": "process-detection",
    "message": "Process detected",
    "process": {
      "name": "ps",
      "pid": 5678,
      "path": "/bin/ps",
      "entitlements": ["com.apple.system-task-ports.read"]
    }
  }
]
```

## Backward Compatibility

### Existing Commands
All existing commands remain unchanged:
- `listent` (default scan)
- `listent --monitor` (interactive monitoring)
- `listent --path PATH` (scan specific path)
- `listent --entitlement ENT` (filter by entitlement)
- `listent --json` (JSON output)

### Flag Conflicts
- `--daemon` flag added but only conflicts with `--monitor` (mutually exclusive)
- All existing flags work with new subcommands where applicable
- Help system updated to show both modes

### Migration Path
1. Users can continue using existing CLI without changes
2. Daemon installation is completely optional
3. Daemon mode uses same underlying logic as CLI mode
4. Configuration can be initially seeded from CLI usage patterns

## Security Considerations

### Privilege Requirements
- **System daemon**: Requires root for installation and control operations
- **User agent**: Requires user privileges, limited monitoring scope
- **Status/logs**: Read-only operations work without special privileges

### Input Validation
- All configuration values validated before application
- Path filters checked for existence and accessibility
- Entitlement filters validated for format correctness
- IPC messages size-limited and rate-limited

### Output Sanitization
- Process paths and names properly escaped in output
- JSON output uses proper escaping
- Log output prevents injection attacks
- Error messages don't leak sensitive information