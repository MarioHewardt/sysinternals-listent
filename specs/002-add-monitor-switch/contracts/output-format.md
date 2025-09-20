# Output Contract: Monitor Mode Formats

**Version**: 1.0  
**Date**: September 19, 2025  
**Feature**: 002-add-monitor-switch

## Human-Readable Output Format

### Process Detection Event
```
[2025-09-19T10:30:45Z] New process detected: Calculator (PID: 12345)
  Path: /System/Applications/Calculator.app/Contents/MacOS/Calculator
  Entitlements: com.apple.security.app-sandbox, com.apple.security.files.user-selected.read-only
```

### Format Specification
- **Timestamp**: ISO 8601 format in UTC timezone
- **Process Name**: Executable name extracted from path
- **PID**: Process identifier in parentheses
- **Path**: Full absolute path to executable, indented 2 spaces
- **Entitlements**: Comma-separated list, indented 2 spaces
- **Line Breaks**: Each detection is one multi-line block, separated by blank line

### Multiple Entitlements Example
```
[2025-09-19T10:30:45Z] New process detected: Safari (PID: 54321)
  Path: /Applications/Safari.app/Contents/MacOS/Safari
  Entitlements: com.apple.security.network.client, com.apple.security.network.server, 
                com.apple.security.device.camera, com.apple.security.device.microphone,
                com.apple.security.personal-information.location
```

### No Entitlements Example
```
[2025-09-19T10:30:45Z] New process detected: my-script (PID: 98765)
  Path: /usr/local/bin/my-script
  Entitlements: (none)
```

### Filtered Output Example
When using entitlement filters, only matching entitlements are shown:
```bash
# Command: listent --monitor -e com.apple.security.camera

[2025-09-19T10:30:45Z] New process detected: Zoom (PID: 11111)
  Path: /Applications/zoom.us.app/Contents/MacOS/zoom.us
  Entitlements: com.apple.security.device.camera
```

## JSON Output Format

### Process Detection Event
```json
{
  "timestamp": "2025-09-19T10:30:45Z",
  "event_type": "process_detected",
  "process": {
    "pid": 12345,
    "name": "Calculator",
    "path": "/System/Applications/Calculator.app/Contents/MacOS/Calculator",
    "entitlements": [
      "com.apple.security.app-sandbox",
      "com.apple.security.files.user-selected.read-only"
    ]
  }
}
```

### Schema Definition
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["timestamp", "event_type", "process"],
  "properties": {
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp in UTC"
    },
    "event_type": {
      "type": "string",
      "enum": ["process_detected"],
      "description": "Type of monitoring event"
    },
    "process": {
      "type": "object",
      "required": ["pid", "name", "path", "entitlements"],
      "properties": {
        "pid": {
          "type": "integer",
          "minimum": 1,
          "description": "Process identifier"
        },
        "name": {
          "type": "string",
          "minLength": 1,
          "description": "Process executable name"
        },
        "path": {
          "type": "string",
          "minLength": 1,
          "description": "Absolute path to process executable"
        },
        "entitlements": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "List of code signing entitlements"
        }
      }
    }
  }
}
```

### Multiple Processes Example
Each process detection is output as a separate JSON line:
```json
{"timestamp":"2025-09-19T10:30:45Z","event_type":"process_detected","process":{"pid":12345,"name":"Calculator","path":"/System/Applications/Calculator.app/Contents/MacOS/Calculator","entitlements":["com.apple.security.app-sandbox"]}}
{"timestamp":"2025-09-19T10:30:47Z","event_type":"process_detected","process":{"pid":54321,"name":"Safari","path":"/Applications/Safari.app/Contents/MacOS/Safari","entitlements":["com.apple.security.network.client","com.apple.security.device.camera"]}}
```

### No Entitlements Example
```json
{
  "timestamp": "2025-09-19T10:30:45Z",
  "event_type": "process_detected", 
  "process": {
    "pid": 98765,
    "name": "my-script",
    "path": "/usr/local/bin/my-script",
    "entitlements": []
  }
}
```

## Status Messages

### Startup Messages (unless --quiet)
```
Starting process monitoring (interval: 1.0s)...
Monitoring /Applications for processes with entitlement: com.apple.security.camera
Press Ctrl+C to stop monitoring.
```

### Shutdown Messages (unless --quiet)  
```
Monitoring stopped.
```

### Error Messages (always to stderr)
```
Warning: Failed to extract entitlements for process 12345 (/usr/bin/example). Continuing monitoring.
Warning: Unable to access process 99999. Permission denied.
Error: System overloaded. Consider increasing polling interval.
```

## Quiet Mode Behavior

### Suppressed Output
- Startup messages
- Shutdown messages  
- Progress indicators
- Non-critical warnings

### Retained Output
- Process detection events (human-readable or JSON)
- Critical error messages (to stderr)
- Help and version information

### Example Quiet Output
```bash
# Command: listent --monitor --quiet

[2025-09-19T10:30:45Z] New process detected: Calculator (PID: 12345)
  Path: /System/Applications/Calculator.app/Contents/MacOS/Calculator
  Entitlements: com.apple.security.app-sandbox
```

## Output Streaming

### Real-time Behavior  
- Process detections output immediately when discovered
- No buffering or batching of detection events
- Output flushed after each detection for real-time visibility

### Signal Handling
- Ctrl+C produces clean shutdown message (unless --quiet)
- No partial output or corrupted JSON during shutdown
- All pending detections output before termination

## Integration with System Logging

### Unified Logging Output (not visible in CLI)
Messages logged to macOS Unified Logging System:
```
subsystem: com.sysinternals.entlist
category: monitor
message: Process detected: Calculator (PID: 12345) Path: /System/Applications/Calculator.app/Contents/MacOS/Calculator Entitlements: [com.apple.security.app-sandbox]
```

### Console.app Visibility
Administrators can view monitoring events in Console.app by filtering:
- Subsystem: com.sysinternals.entlist  
- Category: monitor
- Process: listent

## Error Output Format

### stderr Message Format
```
listent: error: {error_description}
listent: warning: {warning_description}
```

### Error Categories
- **Configuration errors**: Invalid arguments, validation failures
- **Permission errors**: Insufficient privileges for monitoring
- **System errors**: Resource exhaustion, system call failures  
- **Runtime warnings**: Individual process access failures