# Research: LaunchD Daemon Support Implementation

**Date**: September 20, 2025  
**Research Phase**: Resolving technical unknowns and design decisions

## Clarifications Resolved

### FR-013: Auto-start behavior on system boot
**Decision**: Daemon MUST auto-start on system boot by default
**Rationale**: Primary use case is continuous security monitoring - manual start defeats the purpose
**Implementation**: launchd plist will include `<key>RunAtLoad</key><true/>` and `<key>KeepAlive</key><true/>`

### FR-014: Required installation permissions  
**Decision**: Installation REQUIRES root/sudo privileges
**Rationale**: 
- System-wide monitoring requires elevated privileges to access all process information
- LaunchDaemon (not LaunchAgent) needed for system-wide scope and boot-time startup
- Configuration in `/Library/LaunchDaemons/` requires root write access
**Implementation**: CLI will check for root privileges and provide clear error message if insufficient

### FR-015: Method for users to query/view logged monitoring data
**Decision**: Provide built-in ULS query commands and document external tools
**Rationale**: ULS integration is core requirement, need both programmatic and human access
**Implementation**: 
- Add `listent logs` command using `log show` with appropriate predicates
- Document Console.app usage patterns
- Support JSON output for automated analysis

## Technical Research

### LaunchD Integration Patterns
**Plist Location**: `/Library/LaunchDaemons/com.github.mariohewardt.listent.plist`
**Service Name**: `com.github.mariohewardt.listent`
**User Context**: Run as root for system-wide process access

**Required Plist Keys**:
```xml
<key>Label</key>
<string>com.github.mariohewardt.listent</string>
<key>ProgramArguments</key>
<array>
    <string>/usr/local/bin/listent</string>
    <string>--daemon</string>
</array>
<key>RunAtLoad</key>
<true/>
<key>KeepAlive</key>
<true/>
<key>StandardOutPath</key>
<string>/dev/null</string>
<key>StandardErrorPath</key>
<string>/var/log/listent-daemon.log</string>
```

### IPC Mechanism Selection
**Chosen**: Unix Domain Sockets + SIGHUP signal
**Alternatives Considered**:
- File watching: Too slow, race conditions
- Shared memory: Overkill for config updates
- Named pipes: Less reliable than UDS

**Implementation**:
- Socket at `/var/run/listent/daemon.sock` 
- JSON message protocol for config updates
- SIGHUP signal for graceful config reload trigger
- Atomic config file replacement for persistence

### Configuration Management
**Config File Location**: `/etc/listent/daemon.toml`
**Format**: TOML for human readability and Rust ecosystem support
**Atomic Updates**: Write to temp file, validate, then atomic rename

**Configuration Schema**:
```toml
[daemon]
polling_interval = 1.0
auto_start = true

[logging]
subsystem = "com.github.mariohewardt.listent"
level = "info"

[monitoring]
monitor_all_processes = true
path_filters = ["/Applications", "/usr/bin"]
entitlement_filters = ["com.apple.security.network.client"]

[ipc]
socket_path = "/var/run/listent/daemon.sock"
socket_permissions = 0o600
```

### ULS Integration Enhancement
**Current State**: Basic oslog placeholder in unified_logging.rs
**Enhancement Needed**: Full structured logging with log levels

**Log Categories**:
- Process Detection: New processes found
- Configuration: Config changes, validation errors  
- System: Daemon lifecycle, errors, performance metrics
- Security: Permission issues, invalid access attempts

**Log Format**:
```
[subsystem: com.github.mariohewardt.listent]
[category: process-detection]
Process detected: name=ps, pid=12345, path=/bin/ps, entitlements=["com.apple.system-task-ports.read"]
```

### Dependency Analysis
**Required New Dependencies**:
- `tokio`: Async runtime for non-blocking IPC and signal handling
- `toml`: Configuration file parsing (widely used, small)
- `serde`: Serialization for config and IPC messages (already present)
- `nix`: Unix domain sockets and signal handling (Unix-specific, well-maintained)

**Async Runtime Justification**:
- Monitor loop must continue during config updates (non-blocking)
- Multiple concurrent IPC connections possible
- Signal handling integration with async context
- Alternative: Custom event loop - significantly more complex

### Performance Considerations
**Startup Time**: Constitution requires <50ms for no-op
- Daemon mode startup will be longer (process scanning, socket setup)
- Regular CLI commands must maintain <50ms (conditional loading)

**Memory Usage**: Target <10MB resident for daemon
- Reuse existing process tracking structures
- Lazy initialization of IPC components
- Bounded message queues

**CPU Usage**: <1% average during monitoring
- Efficient polling using existing optimizations
- Minimal allocations in hot paths

## Risk Assessment

### High Risk
1. **Root Privilege Requirement**: Security scrutiny, user resistance
   - Mitigation: Clear documentation, principle of least privilege where possible
2. **Process Access Permissions**: SIP restrictions, changing macOS security model
   - Mitigation: Comprehensive testing across macOS versions, fallback behaviors

### Medium Risk  
1. **LaunchD Integration Complexity**: Plist generation, service lifecycle
   - Mitigation: Extensive testing, follow Apple best practices
2. **IPC Reliability**: Socket permissions, cleanup on crash
   - Mitigation: Robust error handling, automatic socket cleanup

### Low Risk
1. **Configuration Validation**: Schema evolution, backward compatibility
   - Mitigation: Versioned config format, migration strategies
2. **ULS Integration**: API stability, log volume management
   - Mitigation: Fallback logging, configurable verbosity

## Implementation Strategy

### Phase 1: Core Daemon Infrastructure
1. Extend CLI parsing for daemon commands
2. Implement daemon mode execution path
3. Basic launchd plist generation and installation
4. ULS logging enhancement

### Phase 2: Configuration System  
1. TOML configuration parsing and validation
2. Configuration persistence and atomic updates
3. Default configuration generation

### Phase 3: IPC Implementation
1. Unix domain socket server in daemon
2. Client commands for config updates
3. Signal handling for graceful operations

### Phase 4: Integration and Testing
1. Full launchd integration testing
2. Configuration update scenarios
3. Error handling and recovery
4. Performance validation

## Success Criteria
- ✅ Daemon installs and starts automatically on boot
- ✅ Process monitoring works identical to CLI mode
- ✅ Configuration updates apply without daemon restart
- ✅ All output goes to ULS with structured format
- ✅ Installation/uninstallation is clean and complete
- ✅ Performance maintains constitutional requirements for CLI operations