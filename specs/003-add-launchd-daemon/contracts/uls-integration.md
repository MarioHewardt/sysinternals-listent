# ULS Integration Contract: Unified Logging System Implementation

**Date**: September 20, 2025  
**Purpose**: Define enhanced Unified Logging System integration for daemon mode

## Logging Architecture

### Subsystem and Categories
**Subsystem**: `com.github.mariohewardt.listent`
**Categories**:
- `process-detection`: New process discoveries and monitoring events
- `configuration`: Configuration changes, validation, and reloads
- `daemon-lifecycle`: Daemon start, stop, restart, and error states  
- `performance`: Performance metrics, timing, and resource usage
- `ipc`: Inter-process communication events and errors
- `security`: Permission issues, authentication, and access control

### Log Levels Mapping
```rust
pub enum LogLevel {
    Debug,   // os_log_debug  - Detailed debugging information
    Info,    // os_log_info   - General operational information  
    Warning, // os_log_error  - Recoverable errors and warnings
    Error,   // os_log_fault  - Serious errors requiring attention
}
```

## Structured Logging Format

### Process Detection Events
**Category**: `process-detection`  
**Level**: `Info` (normal), `Debug` (verbose)

```rust
// Structured log entry
os_log!(
    subsystem: "com.github.mariohewardt.listent",
    category: "process-detection", 
    "Process detected: name=%{public}s pid=%d path=%{private}s entitlements=%{public}s",
    process.name,
    process.pid,
    process.executable_path.display(),
    entitlements_json
);
```

**Console Output Example**:
```
2025-09-20 10:15:23.456789-0700 listent[1234:567890] [com.github.mariohewardt.listent:process-detection] Process detected: name=Safari pid=5432 path=<private> entitlements=["com.apple.security.network.client","com.apple.security.files.user-selected.read-write"]
```

**Structured Data**:
```json
{
  "timestamp": "2025-09-20T17:15:23.456789Z",
  "subsystem": "com.github.mariohewardt.listent", 
  "category": "process-detection",
  "level": "info",
  "process": {
    "name": "Safari",
    "pid": 5432,
    "path": "/Applications/Safari.app/Contents/MacOS/Safari",
    "entitlements": [
      "com.apple.security.network.client",
      "com.apple.security.files.user-selected.read-write"
    ],
    "discovery_method": "polling_cycle",
    "matched_filters": ["path:/Applications"]
  },
  "daemon": {
    "pid": 1234,
    "uptime_seconds": 7890,
    "poll_cycle": 1247
  }
}
```

### Configuration Events
**Category**: `configuration`  
**Level**: `Info` (changes), `Warning` (validation failures), `Error` (critical failures)

```rust
// Configuration update
os_log!(
    subsystem: "com.github.mariohewardt.listent",
    category: "configuration",
    "Configuration updated: field=%{public}s old_value=%{public}s new_value=%{public}s source=%{public}s",
    field_name,
    old_value,
    new_value,
    update_source
);

// Validation failure
os_log_error!(
    subsystem: "com.github.mariohewardt.listent", 
    category: "configuration",
    "Configuration validation failed: field=%{public}s value=%{public}s error=%{public}s",
    field_name,
    invalid_value,
    error_message
);
```

### Daemon Lifecycle Events
**Category**: `daemon-lifecycle`  
**Level**: `Info` (normal operations), `Error` (failures)

```rust
// Daemon startup
os_log!(
    subsystem: "com.github.mariohewardt.listent",
    category: "daemon-lifecycle",
    "Daemon started: pid=%d config_path=%{public}s monitoring_scope=%{public}s",
    daemon_pid,
    config_path,
    monitoring_scope
);

// Daemon shutdown
os_log!(
    subsystem: "com.github.mariohewardt.listent", 
    category: "daemon-lifecycle",
    "Daemon stopping: reason=%{public}s uptime=%{public}s processes_detected=%ld",
    shutdown_reason,
    uptime_string,
    total_processes
);
```

### Performance Metrics
**Category**: `performance`  
**Level**: `Debug` (regular metrics), `Warning` (performance issues)

```rust
// Regular performance logging (debug level, rate-limited)
os_log_debug!(
    subsystem: "com.github.mariohewardt.listent",
    category: "performance", 
    "Poll cycle completed: duration_ms=%f processes_scanned=%d new_processes=%d memory_mb=%f",
    poll_duration.as_secs_f64() * 1000.0,
    processes_scanned,
    new_processes_found,
    memory_usage_mb
);

// Performance warning
os_log_error!(
    subsystem: "com.github.mariohewardt.listent",
    category: "performance",
    "Performance warning: poll_duration_ms=%f exceeds_threshold=%f memory_mb=%f",
    slow_poll_duration,
    threshold_ms,
    current_memory
);
```

## Query Interface

### Built-in Log Query Commands

#### Basic Log Viewing
```bash
# View recent daemon logs
listent logs --limit 100

# Follow logs in real-time  
listent logs --follow

# Filter by time range
listent logs --since "2025-09-20 09:00" --until "2025-09-20 12:00"
```

#### Category-specific Queries
```bash
# Process detection events only
listent logs --category process-detection

# Configuration changes
listent logs --category configuration --level info

# Performance metrics
listent logs --category performance --level debug
```

#### Advanced Filtering
```bash
# Filter by process name
listent logs --filter-process "Safari"

# Filter by entitlement
listent logs --filter-entitlement "com.apple.security.network.client"

# Filter by specific fields
listent logs --filter-json '{"process.pid": 5432}'
```

### External ULS Query Integration

#### Using macOS log command
```bash
# All listent daemon logs
log show --predicate 'subsystem == "com.github.mariohewardt.listent"' --last 1h

# Process detection events only  
log show --predicate 'subsystem == "com.github.mariohewardt.listent" AND category == "process-detection"' --style json

# Performance issues
log show --predicate 'subsystem == "com.github.mariohewardt.listent" AND category == "performance" AND messageType == "error"'

# Specific time range with structured output
log show --predicate 'subsystem == "com.github.mariohewardt.listent"' --start "2025-09-20 09:00" --end "2025-09-20 12:00" --style ndjson
```

#### Console.app Integration
1. **Filter Setup**: Create saved search for subsystem `com.github.mariohewardt.listent`
2. **Category Columns**: Add category column for easy filtering
3. **Custom Predicates**: Save common queries for reuse
4. **Export Options**: JSON/CSV export for analysis

## Log Rate Limiting and Performance

### Rate Limiting Strategy
```rust
pub struct LogRateLimiter {
    max_logs_per_minute: usize,
    current_minute: u64,
    logs_this_minute: usize,
    suppressed_count: usize,
}

impl LogRateLimiter {
    pub fn should_log(&mut self, level: LogLevel) -> bool {
        let current_minute = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() / 60;
            
        if current_minute != self.current_minute {
            // Log suppression summary if needed
            if self.suppressed_count > 0 {
                os_log!(
                    subsystem: "com.github.mariohewardt.listent",
                    category: "performance",
                    "Log rate limiting: suppressed %d messages in previous minute",
                    self.suppressed_count
                );
            }
            
            self.current_minute = current_minute;
            self.logs_this_minute = 0;
            self.suppressed_count = 0;
        }
        
        // Always allow errors and warnings
        if matches!(level, LogLevel::Error | LogLevel::Warning) {
            return true;
        }
        
        if self.logs_this_minute < self.max_logs_per_minute {
            self.logs_this_minute += 1;
            true
        } else {
            self.suppressed_count += 1;
            false
        }
    }
}
```

### Performance Optimization
- **Lazy String Formatting**: Only format expensive strings when logging is enabled
- **Structured Data Caching**: Cache JSON serialization for repeated data
- **Batch Logging**: Group related events when possible
- **Memory Management**: Pre-allocate log buffers to avoid allocations

## Privacy and Security

### Data Sensitivity Levels
```rust
// Public data (safe for logs)
os_log!("Process name: %{public}s", process.name);

// Private data (redacted in logs by default)  
os_log!("Process path: %{private}s", process.executable_path);

// Sensitive data (never logged)
// - User data from process memory
// - Detailed system information
// - Cryptographic material
```

### Log Sanitization
```rust
pub fn sanitize_for_logging(input: &str) -> String {
    input
        .replace('\n', "\\n")
        .replace('\r', "\\r") 
        .replace('\0', "\\0")
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect()
}

pub fn sanitize_entitlement_list(entitlements: &[String]) -> String {
    let sanitized: Vec<String> = entitlements
        .iter()
        .map(|e| sanitize_for_logging(e))
        .collect();
    
    serde_json::to_string(&sanitized).unwrap_or_else(|_| "[]".to_string())
}
```

## Error Handling and Fallbacks

### ULS Unavailable Scenarios
```rust
pub fn log_with_fallback(level: LogLevel, category: &str, message: &str) {
    // Try ULS first
    if let Err(e) = log_to_uls(level, category, message) {
        // Fallback to stderr for critical messages
        if matches!(level, LogLevel::Error | LogLevel::Warning) {
            eprintln!("[{}] [{}] {}", level, category, message);
        }
        
        // Log the logging failure (once per session)
        if !LOGGING_FAILURE_LOGGED.load(Ordering::Relaxed) {
            eprintln!("Warning: ULS logging failed: {}", e);
            LOGGING_FAILURE_LOGGED.store(true, Ordering::Relaxed);
        }
    }
}
```

### Log Rotation and Management
- **System Managed**: ULS handles rotation automatically
- **Disk Space**: Monitor via system tools, no manual cleanup needed
- **Retention**: Follows system-wide ULS retention policies
- **Archival**: Export important events before retention expiry

## Testing and Validation

### Log Output Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_detection_logging() {
        let process = MonitoredProcess {
            name: "test-process".to_string(),
            pid: 12345,
            executable_path: PathBuf::from("/usr/bin/test"),
            entitlements: vec!["com.example.test".to_string()],
            discovery_timestamp: SystemTime::now(),
        };
        
        // Test structured logging format
        let log_entry = create_process_detection_log(&process);
        assert!(log_entry.contains("name=test-process"));
        assert!(log_entry.contains("pid=12345"));
    }
    
    #[test]
    fn test_log_rate_limiting() {
        let mut limiter = LogRateLimiter::new(10);
        
        // Should allow first 10 logs
        for _ in 0..10 {
            assert!(limiter.should_log(LogLevel::Info));
        }
        
        // Should block 11th log
        assert!(!limiter.should_log(LogLevel::Info));
        
        // Should still allow errors
        assert!(limiter.should_log(LogLevel::Error));
    }
}
```

### Integration Testing
```bash
# Test log output during daemon operation
sudo listent install-daemon --config test-config.toml
sleep 5
listent logs --limit 10 --json > test-logs.json

# Validate log structure
python3 -c "
import json
logs = json.load(open('test-logs.json'))
assert all('subsystem' in log for log in logs)
assert all('category' in log for log in logs)
print('Log structure validation passed')
"
```

This ULS integration contract ensures comprehensive, structured, and performant logging that integrates seamlessly with macOS system logging infrastructure while maintaining privacy and security standards.