# Quickstart: Process Monitor Feature

**Feature**: 002-add-monitor-switch  
**Date**: September 19, 2025  
**Purpose**: Validate monitor functionality with real-world scenarios

## Prerequisites

### System Requirements
- macOS 10.15+ (Catalina or newer)
- Administrator or developer privileges for process access
- Console.app access for Unified Logging verification

### Tool Installation
```bash
# Build from source
cargo build --release

# Install binary
cp target/release/listent /usr/local/bin/

# Verify installation
listent --version
listent --help
```

## Basic Monitoring Scenarios

### Scenario 1: Basic Process Detection
**Goal**: Verify monitor mode detects new processes

```bash
# Terminal 1: Start monitoring
listent --monitor

# Terminal 2: Launch applications
open -a Calculator
open -a TextEdit  
open -a Safari

# Expected Output (Terminal 1):
# Starting process monitoring (interval: 1.0s)...
# Press Ctrl+C to stop monitoring.
# 
# [2025-09-19T10:30:45Z] New process detected: Calculator (PID: 12345)
#   Path: /System/Applications/Calculator.app/Contents/MacOS/Calculator
#   Entitlements: com.apple.security.app-sandbox, com.apple.security.files.user-selected.read-only
#
# [2025-09-19T10:30:47Z] New process detected: TextEdit (PID: 12346)
#   Path: /System/Applications/TextEdit.app/Contents/MacOS/TextEdit  
#   Entitlements: com.apple.security.app-sandbox, com.apple.security.files.user-selected.read-write
#
# [2025-09-19T10:30:49Z] New process detected: Safari (PID: 12347)
#   Path: /Applications/Safari.app/Contents/MacOS/Safari
#   Entitlements: com.apple.security.network.client, com.apple.security.device.camera
```

**Validation Steps**:
1. ✅ Monitor starts with status message
2. ✅ Each launched app detected within 1-2 seconds  
3. ✅ Process name, PID, path, and entitlements displayed
4. ✅ Ctrl+C stops monitoring cleanly

### Scenario 2: Path Filtering
**Goal**: Verify monitoring respects path filters

```bash
# Terminal 1: Monitor only /Applications
listent --monitor -p /Applications

# Terminal 2: Launch apps from different locations
open -a Calculator                    # /System/Applications - should be ignored
open -a Safari                       # /Applications - should be detected
open /usr/bin/vim                     # /usr/bin - should be ignored

# Expected Output (Terminal 1):
# Starting process monitoring (interval: 1.0s)...
# Monitoring /Applications for processes
# Press Ctrl+C to stop monitoring.
#
# [2025-09-19T10:35:15Z] New process detected: Safari (PID: 23456)
#   Path: /Applications/Safari.app/Contents/MacOS/Safari
#   Entitlements: com.apple.security.network.client, com.apple.security.device.camera
```

**Validation Steps**:
1. ✅ Only processes from /Applications detected
2. ✅ System applications and command-line tools ignored
3. ✅ Filter message displayed at startup

### Scenario 3: Entitlement Filtering  
**Goal**: Verify entitlement-based filtering works

```bash
# Terminal 1: Monitor for camera entitlements
listent --monitor -e com.apple.security.device.camera

# Terminal 2: Launch various applications
open -a Calculator                    # No camera - should be ignored
open -a Safari                       # Has camera - should be detected  
open -a Zoom                         # Has camera - should be detected
open -a TextEdit                     # No camera - should be ignored

# Expected Output (Terminal 1):
# Starting process monitoring (interval: 1.0s)...
# Monitoring for processes with entitlement: com.apple.security.device.camera
# Press Ctrl+C to stop monitoring.
#
# [2025-09-19T10:40:30Z] New process detected: Safari (PID: 34567)
#   Path: /Applications/Safari.app/Contents/MacOS/Safari
#   Entitlements: com.apple.security.device.camera
#
# [2025-09-19T10:40:35Z] New process detected: zoom.us (PID: 34568)
#   Path: /Applications/zoom.us.app/Contents/MacOS/zoom.us
#   Entitlements: com.apple.security.device.camera
```

**Validation Steps**:
1. ✅ Only camera-enabled applications detected
2. ✅ Non-camera apps ignored even if launched
3. ✅ Entitlement filter message displayed at startup

### Scenario 4: Combined Filtering
**Goal**: Verify path and entitlement filters work together

```bash  
# Terminal 1: Monitor Applications directory for camera apps
listent --monitor -p /Applications -e com.apple.security.device.camera --interval 2.0

# Terminal 2: Launch mixed applications
open -a Calculator                    # /System/Applications + no camera - ignored
open -a Safari                       # /Applications + camera - detected
/usr/bin/vim                         # /usr/bin + no camera - ignored
open /Applications/zoom.us.app       # /Applications + camera - detected

# Expected Output (Terminal 1):
# Starting process monitoring (interval: 2.0s)...
# Monitoring /Applications for processes with entitlement: com.apple.security.device.camera
# Press Ctrl+C to stop monitoring.
#
# [2025-09-19T10:45:12Z] New process detected: Safari (PID: 45678)
#   Path: /Applications/Safari.app/Contents/MacOS/Safari
#   Entitlements: com.apple.security.device.camera
#
# [2025-09-19T10:45:16Z] New process detected: zoom.us (PID: 45679)
#   Path: /Applications/zoom.us.app/Contents/MacOS/zoom.us  
#   Entitlements: com.apple.security.device.camera
```

**Validation Steps**:
1. ✅ Custom polling interval (2.0s) respected
2. ✅ Both path and entitlement filters applied
3. ✅ Only applications matching both criteria detected

## JSON Output Scenarios

### Scenario 5: JSON Format Output
**Goal**: Verify JSON output format for programmatic consumption

```bash
# Terminal 1: Monitor with JSON output
listent --monitor --json

# Terminal 2: Launch applications  
open -a Calculator
open -a Safari

# Expected Output (Terminal 1):
# {"timestamp":"2025-09-19T10:50:30Z","event_type":"process_detected","process":{"pid":56789,"name":"Calculator","path":"/System/Applications/Calculator.app/Contents/MacOS/Calculator","entitlements":["com.apple.security.app-sandbox","com.apple.security.files.user-selected.read-only"]}}
# {"timestamp":"2025-09-19T10:50:32Z","event_type":"process_detected","process":{"pid":56790,"name":"Safari","path":"/Applications/Safari.app/Contents/MacOS/Safari","entitlements":["com.apple.security.network.client","com.apple.security.device.camera"]}}
```

**Validation Steps**:
1. ✅ Each detection is a single JSON line
2. ✅ JSON structure matches schema (timestamp, event_type, process)
3. ✅ Process object contains pid, name, path, entitlements
4. ✅ Valid JSON parsing with standard tools (`jq`, `python -m json.tool`)

### Scenario 6: Quiet Mode
**Goal**: Verify quiet mode suppresses non-essential output

```bash
# Terminal 1: Quiet monitoring  
listent --monitor --quiet

# Terminal 2: Launch application
open -a Calculator

# Expected Output (Terminal 1):
# [2025-09-19T10:55:15Z] New process detected: Calculator (PID: 67890)
#   Path: /System/Applications/Calculator.app/Contents/MacOS/Calculator
#   Entitlements: com.apple.security.app-sandbox
```

**Validation Steps**:
1. ✅ No startup message displayed
2. ✅ Process detections still shown
3. ✅ Ctrl+C exit without shutdown message
4. ✅ Error messages still displayed if needed

## System Integration Scenarios

### Scenario 7: Unified Logging Verification
**Goal**: Verify integration with macOS Unified Logging System

```bash
# Terminal 1: Start monitoring
listent --monitor

# Terminal 2: Launch application
open -a Safari

# Terminal 3: Verify system logging
log show --predicate 'subsystem == "com.sysinternals.entlist"' --last 5m

# Expected Output (Terminal 3):
# 2025-09-19 10:58:30.123456-0700  listent[12345]  com.sysinternals.entlist[monitor]: Process detected: Safari (PID: 78901) Path: /Applications/Safari.app/Contents/MacOS/Safari Entitlements: [com.apple.security.network.client, com.apple.security.device.camera]
```

**Validation Steps**:
1. ✅ Events appear in system log within seconds
2. ✅ Correct subsystem (com.sysinternals.entlist) and category (monitor)  
3. ✅ Process information matches console output
4. ✅ Searchable in Console.app

### Scenario 8: Error Handling
**Goal**: Verify graceful error handling

```bash
# Test 1: Invalid interval
listent --monitor --interval 0.05
# Expected: Error message about invalid interval (min 0.1s)

# Test 2: Invalid path  
listent --monitor -p /nonexistent
# Expected: Error message about non-existent path

# Test 3: Permission issues (run as limited user)
listent --monitor  
# Expected: Warnings about inaccessible processes, but monitoring continues
```

**Validation Steps**:
1. ✅ Clear error messages for invalid configurations
2. ✅ Graceful handling of permission issues
3. ✅ Monitoring continues despite individual process access failures

## Performance Scenarios  

### Scenario 9: Extended Monitoring
**Goal**: Verify stability during extended operation

```bash
# Terminal 1: Start long-running monitor
listent --monitor --interval 5.0

# Let run for 10+ minutes while using system normally
# Launch various applications, open/close programs

# Validation Steps:
# 1. ✅ No memory leaks (check with Activity Monitor)
# 2. ✅ CPU usage remains low (<1% average)
# 3. ✅ All process launches detected reliably
# 4. ✅ Clean shutdown with Ctrl+C after extended run
```

### Scenario 10: High Activity Stress Test
**Goal**: Verify performance under high process creation load

```bash
# Terminal 1: Fast polling monitor
listent --monitor --interval 0.5

# Terminal 2: Create process activity
for i in {1..20}; do
  open -a Calculator
  sleep 0.1
  killall Calculator
  sleep 0.1
done

# Validation Steps:
# 1. ✅ All process launches detected
# 2. ✅ No dropped events or missed detections  
# 3. ✅ Output remains clean and readable
# 4. ✅ System responsiveness maintained
```

## Acceptance Criteria Summary

### Functional Requirements
- [x] **Monitor Mode Activation**: `--monitor` flag enables real-time monitoring
- [x] **Configurable Interval**: `--interval` parameter sets polling frequency (0.1s-300s)
- [x] **Path Filtering**: `-p` option limits monitoring to specific directories
- [x] **Entitlement Filtering**: `-e` option filters by code signing entitlements
- [x] **Process Detection**: New processes detected within polling interval
- [x] **Console Output**: Human-readable format with process details
- [x] **JSON Output**: Machine-readable format for automation
- [x] **Unified Logging**: Events logged to macOS system logging
- [x] **Graceful Shutdown**: Ctrl+C stops monitoring cleanly
- [x] **Error Handling**: Invalid configurations rejected with clear messages

### Non-Functional Requirements  
- [x] **Performance**: <1% CPU usage during monitoring
- [x] **Memory**: No memory leaks during extended operation
- [x] **Responsiveness**: Process detection within polling interval
- [x] **Stability**: Runs for hours without degradation
- [x] **Compatibility**: Works with existing CLI options and workflows

### Integration Requirements
- [x] **Backward Compatibility**: Existing functionality unchanged
- [x] **Help System**: Updated help text includes monitor options
- [x] **Exit Codes**: Consistent error code behavior
- [x] **Output Formats**: Consistent with existing scan mode formatting