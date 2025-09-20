# Data Model: Real-time Process Monitoring

**Date**: September 19, 2025  
**Feature**: 002-add-monitor-switch  
**Status**: Complete

## Core Entities

### MonitoredProcess
Represents a detected process with relevant information for security monitoring.

**Purpose**: Immutable snapshot of a process at discovery time  
**Lifecycle**: Created when new process detected, stored until monitoring session ends  

**Fields**:
- `pid: u32` - Process identifier (required, > 0)
- `name: String` - Process name/executable name (required)
- `executable_path: PathBuf` - Full path to process executable (required)
- `entitlements: Vec<String>` - List of code signing entitlements (optional, empty if extraction fails)
- `discovery_timestamp: SystemTime` - When process was first detected (required)

**Validation Rules**:
- PID must be positive non-zero value
- Executable path must be absolute and point to existing file
- Name extracted from executable path if not provided
- Entitlements list may be empty if extraction fails or no entitlements present

**Relationships**:
- Contained within ProcessSnapshot
- Referenced in LogEntry for Unified Logging

### PollingConfiguration  
Represents monitoring session settings and filters.

**Purpose**: Configuration for monitoring behavior and filtering criteria  
**Lifecycle**: Created at monitoring start, immutable during session

**Fields**:
- `interval: Duration` - Time between polling cycles (required, 0.1s - 300s)
- `path_filters: Vec<PathBuf>` - Directories to monitor (optional, empty = all paths)
- `entitlement_filters: Vec<String>` - Entitlements to match (optional, empty = all entitlements)
- `output_json: bool` - Whether to output JSON format (required, default false)
- `quiet_mode: bool` - Whether to suppress non-essential output (required, default false)

**Validation Rules**:
- Interval must be between 0.1 seconds and 300 seconds (inclusive)
- Path filters must be absolute paths to existing directories
- Entitlement filters must be valid entitlement identifier strings
- Boolean flags default to false if not specified

**State Transitions**:
- Immutable once monitoring starts
- Validation occurs at creation time

### ProcessSnapshot
Represents the current state of running processes at a specific time.

**Purpose**: Point-in-time view of system processes for comparison between polling cycles  
**Lifecycle**: Created each polling cycle, replaced by next cycle

**Fields**:
- `processes: HashMap<u32, MonitoredProcess>` - Map of PID to process info (required)
- `timestamp: SystemTime` - When snapshot was taken (required)
- `scan_duration: Duration` - Time taken to create snapshot (required)

**Operations**:
- `new_processes(&self, previous: &ProcessSnapshot) -> Vec<MonitoredProcess>` - Find processes in current but not previous snapshot
- `terminated_processes(&self, previous: &ProcessSnapshot) -> Vec<u32>` - Find PIDs in previous but not current snapshot

**Relationships**:
- Contains multiple MonitoredProcess instances
- Compared with previous snapshots to detect changes

### EntitlementMatch
Represents a process that satisfies entitlement filtering criteria.

**Purpose**: Link between a process and the specific entitlements that caused it to match filters  
**Lifecycle**: Created when process matches entitlement filters, used for output formatting

**Fields**:
- `process: MonitoredProcess` - The matching process (required)
- `matched_entitlements: Vec<String>` - Specific entitlements that matched filters (required, non-empty)

**Validation Rules**:
- Must have at least one matched entitlement
- Matched entitlements must be subset of process.entitlements
- Process must be valid MonitoredProcess

### LogEntry
Represents an event to be logged to macOS Unified Logging System.

**Purpose**: Structured representation of monitoring events for system logging  
**Lifecycle**: Created for each detected process, immediately logged and discarded

**Fields**:
- `subsystem: &'static str` - Always "com.sysinternals.entlist" (required)
- `category: &'static str` - Always "monitor" (required)  
- `message: String` - Formatted log message with process details (required)
- `process_info: MonitoredProcess` - Associated process information (required)

**Message Format**:
```
Process detected: {name} (PID: {pid}) Path: {path} Entitlements: [{entitlements}]
```

## Data Flow

### Monitoring Cycle Data Flow
```
1. Create ProcessSnapshot from system state
2. Compare with previous ProcessSnapshot to find new processes  
3. Filter new processes by path (PollingConfiguration.path_filters)
4. Extract entitlements for filtered processes
5. Filter by entitlements (PollingConfiguration.entitlement_filters)
6. Create EntitlementMatch for each matching process
7. Create LogEntry for each match and log to Unified Logging
8. Output matches to console (format determined by PollingConfiguration)
9. Store current ProcessSnapshot for next cycle comparison
```

### State Management
- **Current Snapshot**: Active ProcessSnapshot for comparison
- **Configuration**: Immutable PollingConfiguration for session
- **Shutdown Signal**: Atomic boolean for graceful termination

## Memory Management

### Memory Footprint Estimates
- MonitoredProcess: ~200 bytes (path strings, entitlements vector)
- ProcessSnapshot: ~50KB for 250 processes (typical macOS system)
- PollingConfiguration: ~1KB (filter vectors)
- Total steady-state: ~51KB plus temporary allocations

### Cleanup Strategy
- ProcessSnapshot replaced each cycle (automatic cleanup of previous)
- No persistent storage of historical process information
- Entitlement extraction results not cached (re-extracted if needed)

## Error Handling

### Process Access Errors
- **Cause**: Insufficient permissions to read process information
- **Handling**: Skip process, log warning, continue monitoring
- **Impact**: Reduced visibility into system processes

### Entitlement Extraction Errors
- **Cause**: Process not code-signed, codesign utility failure
- **Handling**: Process included with empty entitlements list
- **Impact**: Process detected but entitlement filtering may not work

### System Resource Errors
- **Cause**: High system load, resource exhaustion
- **Handling**: Extend polling interval temporarily, log warning
- **Impact**: Reduced monitoring frequency

## Integration Points

### CLI Integration
- PollingConfiguration created from command-line arguments
- Existing path and entitlement parsing logic reused
- Monitor mode flag triggers monitoring workflow

### Output Integration
- EntitlementMatch feeds into existing output formatting logic
- JSON output format extends existing schema
- Human-readable output follows existing patterns

### Logging Integration
- LogEntry integrates with macOS Unified Logging
- Subsystem organization maintains tool identity
- Log levels follow macOS conventions