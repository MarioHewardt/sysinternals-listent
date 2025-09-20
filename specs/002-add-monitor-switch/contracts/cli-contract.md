# CLI Contract: Monitor Mode

**Version**: 1.0  
**Date**: September 19, 2025  
**Feature**: 002-add-monitor-switch

## Command Line Interface

### Monitor Mode Activation
```bash
listent --monitor [OPTIONS]
```

### New Command Line Options

#### --monitor
- **Type**: Boolean flag
- **Required**: Yes (for monitor mode)
- **Default**: false  
- **Description**: Enable real-time process monitoring mode
- **Conflicts**: None (compatible with existing options)

#### --interval SECONDS
- **Type**: Floating-point number
- **Required**: No
- **Default**: 1.0
- **Range**: 0.1 to 300.0 (inclusive)
- **Description**: Polling interval between process scans in seconds
- **Examples**: `--interval 5.0`, `--interval 0.5`

### Existing Options Compatibility

#### -p, --path PATH (Existing)
- **Behavior in Monitor Mode**: Monitor only processes from specified directory paths
- **Multiple Values**: Supported via multiple `-p` flags
- **Examples**: `-p /Applications`, `-p /usr/local/bin`

#### -e, --entitlement ENTITLEMENT (Existing)  
- **Behavior in Monitor Mode**: Report only processes with matching entitlements
- **Multiple Values**: Supported via multiple `-e` flags  
- **Examples**: `-e com.apple.security.camera`, `-e com.apple.security.microphone`

#### --json (Existing)
- **Behavior in Monitor Mode**: Output detected processes in JSON format instead of human-readable
- **Format**: One JSON object per line for each detected process

#### --quiet (Existing)
- **Behavior in Monitor Mode**: Suppress startup/shutdown messages, output only process detections

#### --help (Existing)
- **Behavior**: Show updated help text including monitor mode options
- **Monitor Section**: Added to help output

#### --version (Existing)
- **Behavior**: Unchanged, shows tool version

## Command Examples

### Basic Monitoring
```bash
# Monitor all processes with 1-second interval (default)
listent --monitor

# Monitor with custom interval
listent --monitor --interval 5.0
```

### Filtered Monitoring  
```bash
# Monitor applications directory only
listent --monitor -p /Applications

# Monitor for camera-enabled processes
listent --monitor -e com.apple.security.camera

# Combined filtering
listent --monitor -p /Applications -e com.apple.security.camera --interval 2.0
```

### Output Formatting
```bash
# JSON output for programmatic consumption
listent --monitor --json

# Quiet mode (minimal output)
listent --monitor --quiet

# Combined formatting options
listent --monitor -p /Applications --json --quiet
```

## Exit Behavior

### Normal Termination
- **Trigger**: User presses Ctrl+C (SIGINT)
- **Exit Code**: 0
- **Output**: "Monitoring stopped." (unless --quiet)

### Error Termination  
- **Invalid interval**: Exit code 1, error message to stderr
- **Invalid path**: Exit code 1, error message to stderr  
- **Permission denied**: Exit code 1, error message to stderr
- **System error**: Exit code 1, error message to stderr

### Help/Version
- **--help**: Exit code 0, help text to stdout
- **--version**: Exit code 0, version to stdout

## Error Messages

### Validation Errors
```
Error: Invalid interval '0.05'. Must be between 0.1 and 300.0 seconds.
Error: Path '/nonexistent' does not exist or is not a directory.
Error: Cannot access system process information. Run with elevated permissions.
```

### Runtime Errors  
```
Warning: Failed to extract entitlements for process 12345. Continuing monitoring.
Warning: Unable to log to Unified Logging System. Console output will continue.
Error: System overloaded. Consider increasing polling interval.
```

## Backward Compatibility

### Existing Functionality
- **Non-monitor mode**: All existing functionality unchanged
- **Help text**: Extended but existing options documented identically
- **Exit codes**: Unchanged for existing error conditions
- **Output formats**: Unchanged for scan mode

### Breaking Changes
- **None**: Monitor mode is additive functionality
- **Configuration**: No changes to existing argument parsing
- **Behavior**: Default behavior (no --monitor flag) unchanged

## Integration Requirements

### Process Detection
- Must honor path filters (-p) for monitoring scope
- Must honor entitlement filters (-e) for output filtering  
- Must handle multiple filter values (multiple -p or -e flags)

### Output Formatting
- Must support both human-readable and JSON formats
- Must respect quiet mode for minimal output
- Must maintain consistent formatting with existing scan mode

### Error Handling
- Must validate all arguments before starting monitoring
- Must provide clear error messages for invalid configurations
- Must handle system errors gracefully without terminating monitoring