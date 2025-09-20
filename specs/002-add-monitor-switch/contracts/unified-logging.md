# Unified Logging Contract

**Version**: 1.0  
**Date**: September 19, 2025  
**Feature**: 002-add-monitor-switch

## Logging Configuration

### Subsystem
```
com.sysinternals.entlist
```

### Category  
```
monitor
```

### Log Level
```
default (info level)
```

## Message Format

### Process Detection Event
```
Process detected: {process_name} (PID: {pid}) Path: {executable_path} Entitlements: [{entitlement_list}]
```

### Message Examples

#### With Entitlements
```
Process detected: Safari (PID: 54321) Path: /Applications/Safari.app/Contents/MacOS/Safari Entitlements: [com.apple.security.network.client, com.apple.security.device.camera, com.apple.security.device.microphone]
```

#### No Entitlements
```
Process detected: my-script (PID: 98765) Path: /usr/local/bin/my-script Entitlements: []
```

#### Multiple Entitlements Formatting
```
Process detected: Zoom (PID: 11111) Path: /Applications/zoom.us.app/Contents/MacOS/zoom.us Entitlements: [com.apple.security.device.camera, com.apple.security.device.microphone, com.apple.security.network.client, com.apple.security.network.server]
```

## Structured Metadata

### Required Fields
- **process_name**: String - Executable name
- **pid**: Integer - Process identifier  
- **executable_path**: String - Full path to executable
- **entitlement_count**: Integer - Number of entitlements
- **detection_timestamp**: String - ISO 8601 timestamp

### Optional Fields
- **path_filtered**: Boolean - Whether path filtering was applied
- **entitlement_filtered**: Boolean - Whether entitlement filtering was applied
- **filter_match_count**: Integer - Number of matching filter criteria

## Log Message Structure

### os_log Call Pattern
```rust
os_log!(
    logger,
    "Process detected: {} (PID: {}) Path: {} Entitlements: [{}]",
    process_name,
    pid, 
    executable_path,
    entitlements.join(", ")
);
```

### Metadata Attachment
```rust
os_log_with_args!(
    logger,
    log_level,
    "Process detected: %{public}@ (PID: %{public}d) Path: %{public}@ Entitlements: [%{public}@]",
    process_name,
    pid,
    executable_path, 
    entitlements_string,
    metadata: {
        "process_name": process_name,
        "pid": pid,
        "executable_path": executable_path,
        "entitlement_count": entitlements.len(),
        "detection_timestamp": timestamp.to_rfc3339()
    }
);
```

## Privacy and Security

### Public vs Private Data
- **Public Data**: Process names, PIDs, executable paths, entitlement identifiers
- **Private Data**: None (all monitoring data is considered public for security analysis)
- **Rationale**: Security monitoring requires visibility into process details

### Data Sensitivity
- Process information is already visible via system tools (ps, Activity Monitor)
- Entitlements are part of code signing and publicly readable
- No user data or private information is logged

## Console.app Integration

### Filtering Instructions
To view listent monitoring events in Console.app:

1. Open Console.app
2. Select the appropriate device/simulator
3. In the search bar, enter: `subsystem:com.sysinternals.entlist`
4. Filter by category: `category:monitor`
5. Optionally filter by process: `process:listent`

### Search Queries
```
# All listent monitoring events
subsystem:com.sysinternals.entlist AND category:monitor

# Specific process detection
subsystem:com.sysinternals.entlist AND "Safari"

# Camera-related detections  
subsystem:com.sysinternals.entlist AND "com.apple.security.device.camera"

# High-privilege processes
subsystem:com.sysinternals.entlist AND "com.apple.security.network.server"
```

## Error Logging

### Warning Events
```
Warning: Failed to extract entitlements for PID {pid} ({path}). Reason: {error_reason}
```

### Error Events  
```
Error: Unable to access process information. Reason: {error_reason}
Error: Unified logging unavailable. Reason: {error_reason}
```

### Performance Events
```
Performance: Process scan completed in {duration}ms. Processes: {count}
```

## Log Rotation and Retention

### System Behavior
- Unified Logging handles rotation automatically
- Retention follows system policies (typically 7-30 days)
- No manual log management required

### Storage Impact
- Estimated 100-200 bytes per process detection event
- Typical monitoring session: 10-50 events per hour
- Daily storage: ~5-50KB depending on system activity

## Integration Testing

### Verification Methods
1. **Console.app**: Manual verification of log appearance
2. **log show**: Command-line verification
3. **Automated tests**: Programmatic log capture and verification

### Test Commands
```bash
# View recent listent logs
log show --predicate 'subsystem == "com.sysinternals.entlist"' --last 1h

# Filter for monitoring events
log show --predicate 'subsystem == "com.sysinternals.entlist" AND category == "monitor"' --last 1h

# Export for analysis
log show --predicate 'subsystem == "com.sysinternals.entlist"' --style json --last 1d > listent_logs.json
```

## Error Handling

### Logging Failures
- **Symptom**: Unable to create os_log logger
- **Handling**: Continue monitoring, output warning to stderr
- **Fallback**: Continue console output, skip system logging

### Permission Issues
- **Symptom**: Logging calls fail due to permissions
- **Handling**: Log permission warning once, continue monitoring
- **Impact**: Reduced visibility in Console.app, console output continues

### System Resource Issues
- **Symptom**: Logging system overloaded
- **Handling**: Throttle logging, prioritize critical events
- **Recovery**: Resume normal logging when resources available