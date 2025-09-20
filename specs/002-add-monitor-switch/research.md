# Research: Real-time Process Monitoring with Entitlement Filtering

**Date**: September 19, 2025  
**Feature**: 002-add-monitor-switch  
**Status**: Complete

## Research Areas

### 1. macOS Process Enumeration

**Decision**: Use `sysinfo` crate for cross-platform process enumeration  
**Rationale**: 
- Mature, safe, well-maintained crate with macOS-specific optimizations
- Provides clean API for process information retrieval
- No unsafe code required
- Actively maintained with good documentation

**Alternatives considered**:
- Direct libc calls: Requires unsafe code, platform-specific implementation
- nix crate: More complex, lower-level interface
- procfs approach: Not available on macOS

**Implementation notes**:
- Use `System::new_all()` for initial process snapshot
- Use `System::refresh_processes()` for incremental updates
- Access process info via `Process` struct methods

### 2. Unified Logging Integration

**Decision**: Use `oslog` crate for native macOS logging integration  
**Rationale**:
- Direct binding to os_log APIs
- Maintains subsystem organization ("com.sysinternals.entlist")
- Native macOS integration for Console.app viewing
- Lightweight with minimal dependencies

**Alternatives considered**:
- Custom logging: Would require reimplementing os_log functionality
- tracing with oslog backend: Overkill for simple event logging
- Standard println!: Doesn't integrate with macOS logging system

**Implementation notes**:
- Create logger with subsystem "com.sysinternals.entlist"
- Use category "monitor" for process detection events
- Log level: default (info level for process detection)

### 3. Process State Tracking

**Decision**: HashMap<PID, ProcessInfo> with timestamp-based new process detection  
**Rationale**:
- O(1) lookups for efficient process comparison
- Memory efficient for typical process counts (100-500)
- Simple implementation with clear semantics
- Natural cleanup when processes terminate

**Alternatives considered**:
- Process trees: More complex, unnecessary for detection use case
- Persistent storage: Overkill, monitoring is session-based
- Vector scanning: O(n) lookups, less efficient

**Implementation notes**:
- Store minimal process info: PID, path, entitlements
- Track discovery timestamp for reporting
- Clear terminated processes each cycle to prevent memory growth

### 4. Signal Handling

**Decision**: Use `ctrlc` crate for graceful shutdown handling  
**Rationale**:
- Lightweight, proven solution for Ctrl+C handling in CLI tools
- Simple API for registering shutdown handlers
- Cross-platform compatibility
- Integrates well with polling loops

**Alternatives considered**:
- Custom signal handling: More complex, error-prone
- tokio signals: Overkill for simple CLI tool
- No signal handling: Poor user experience

**Implementation notes**:
- Register handler before starting monitoring loop
- Use atomic boolean or channel for clean shutdown signaling
- Ensure final status message before exit

### 5. Memory Management

**Decision**: Bounded memory usage with periodic cleanup  
**Rationale**:
- Monitor can run for hours/days without accumulating memory
- Process lists naturally churn, don't need infinite history
- Constitutional requirement: no memory leaks

**Implementation strategy**:
- Clear terminated processes from tracking map each cycle
- Limit historical data to current monitoring session only
- Use Rust's ownership system to prevent leaks
- Profile memory usage during extended monitoring

**Alternatives considered**:
- Persistent process history: Unnecessary for real-time monitoring
- LRU cache: Adds complexity without clear benefit
- Manual memory management: Error-prone, unnecessary with Rust

## Technology Stack Summary

### New Dependencies
- `sysinfo = "0.29"`: Process enumeration and system information
- `oslog = "0.1"`: macOS Unified Logging integration  
- `ctrlc = "3.4"`: Graceful signal handling

### Integration Points
- **CLI Module**: Extend existing clap-based argument parsing
- **Models Module**: Add monitoring-specific data structures
- **Scan Module**: Reuse existing path filtering logic
- **Entitlements Module**: Reuse existing entitlement extraction
- **Output Module**: Extend for real-time event output

## Performance Considerations

### Memory Usage
- Estimated 50-100 bytes per tracked process
- Typical macOS system: 200-500 processes = 10-50KB memory overhead
- Cleanup terminated processes each cycle

### CPU Usage  
- Process enumeration: ~1-5ms per polling cycle
- Entitlement extraction: Only for new processes matching filters
- Target: <1% CPU usage with 1-second polling interval

### I/O Impact
- codesign calls only for new processes passing filters
- Unified Logging: Asynchronous, minimal performance impact
- Console output: Minimal for typical process launch rates

## Risk Mitigation

### Permission Issues
- Gracefully handle processes that can't be accessed
- Continue monitoring other processes
- Log warning for permission failures

### System Resource Limits
- Validate polling interval bounds (0.1s - 300s)
- Handle system under heavy load gracefully
- Monitor memory usage during extended operation

### Error Recovery
- Continue monitoring if individual process checks fail
- Graceful degradation if Unified Logging unavailable
- Clear error messages for configuration issues

## Conclusion

The research confirms that the monitoring feature can be implemented efficiently using well-established Rust crates while maintaining the tool's performance characteristics and constitutional requirements. The approach leverages existing architecture patterns and adds minimal complexity to the codebase.