# listent - User Scenarios

**Project**: listent - macOS Code Signing Entitlement Scanner  
**Version**: 1.0.0  
**Last Updated**: October 21, 2025

---

## Overview

listent is a command-line tool for macOS that discovers and analyzes code signing entitlements in executable binaries. It provides three operating modes: static scanning, real-time monitoring, and background daemon operation.

---

## Primary User Stories

### 1. Security Analyst: Static System Audit

**Scenario**: A security analyst needs to audit all applications on a macOS system to identify which ones have network access entitlements.

**Workflow**:
```bash
# Scan /Applications for binaries with network entitlements
listent /Applications -e "com.apple.security.network.*"

# Get JSON output for automated processing
listent /Applications -e "com.apple.security.network.*" --json > audit_report.json
```

**Expected Outcome**: 
- List of all applications with network entitlements
- Full entitlement details for each binary
- Structured output suitable for compliance reporting

### 2. Developer: Pre-deployment Entitlement Check

**Scenario**: A developer needs to verify their application has the correct entitlements before deployment.

**Workflow**:
```bash
# Check specific application bundle
listent /path/to/MyApp.app -e "*"

# Verify specific entitlement exists
listent /path/to/MyApp.app -e "com.apple.security.app-sandbox"
```

**Expected Outcome**:
- Confirmation of required entitlements
- Detection of unexpected entitlements
- Clear human-readable output for quick verification

### 3. System Administrator: Real-time Process Monitoring

**Scenario**: An administrator wants to monitor all newly launched processes to detect privilege escalation attempts.

**Workflow**:
```bash
# Monitor all new processes for security-related entitlements
listent --monitor -e "com.apple.security.*" --interval 1.0

# Monitor with path filtering
listent --monitor /Applications /usr/bin -e "*private*"
```

**Expected Outcome**:
- Real-time alerts when matching processes start
- Continuous monitoring with configurable interval
- Graceful shutdown with Ctrl+C

### 4. Security Team: Continuous Background Monitoring

**Scenario**: A security team needs persistent monitoring of process launches integrated with their logging infrastructure.

**Workflow**:
```bash
# Generate launchd plist for daemon installation
listent --daemon --launchd -e "com.apple.private.*" --interval 2.0 > ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist

# Install and start daemon
launchctl load ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist

# Query daemon logs
log show --predicate 'subsystem == "com.github.mariohewardt.listent"' --last 1h
```

**Expected Outcome**:
- Persistent monitoring across system reboots
- All events logged to macOS Unified Logging System
- Integration with existing log aggregation tools

### 5. Researcher: Pattern Discovery

**Scenario**: A researcher wants to discover which applications use specific entitlement patterns to understand macOS security behaviors.

**Workflow**:
```bash
# Find all apps with any debugging-related entitlements
listent /Applications -e "*debug*" --json | jq '.summary.matched_files'

# Discover private entitlements
listent /System/Library /usr/bin -e "com.apple.private.*" --quiet
```

**Expected Outcome**:
- Comprehensive pattern matching with wildcards
- JSON output for data analysis
- Quiet mode to suppress non-critical warnings

---

## Feature Usage Scenarios

### Static Scanning

#### Default Scan
```bash
# Scan default path (/Applications)
listent
```
**Use Case**: Quick overview of installed applications

#### Multi-path Scan
```bash
# Scan multiple directories
listent /Applications /usr/bin /usr/sbin
```
**Use Case**: Comprehensive system audit

#### Entitlement Filtering
```bash
# Exact match
listent -e "com.apple.security.network.client"

# Wildcard pattern
listent -e "com.apple.security.*"

# Multiple patterns (OR logic)
listent -e "com.apple.security.*" -e "*network*"

# Comma-separated patterns
listent -e "com.apple.security.*,*network*,*debug*"
```
**Use Case**: Focus on specific security concerns

#### Output Formats
```bash
# Human-readable (default)
listent /Applications

# JSON for automation
listent /Applications --json

# JSON with JQ processing
listent /Applications --json | jq '.summary'
```
**Use Case**: Integration with other tools

### Real-time Monitoring

#### Basic Monitoring
```bash
# Monitor with default 1-second interval
listent --monitor

# Custom polling interval
listent --monitor --interval 0.5
```
**Use Case**: Detect process launches immediately

#### Filtered Monitoring
```bash
# Monitor specific paths
listent --monitor /Applications

# Monitor for specific entitlements
listent --monitor -e "com.apple.security.network.*"

# Combined filters
listent --monitor /Applications -e "*network*" --interval 2.0
```
**Use Case**: Focused security monitoring

#### Graceful Shutdown
```bash
# Start monitoring (Ctrl+C to stop)
listent --monitor -e "com.apple.*"

# Terminal reset if needed
reset
```
**Use Case**: Interactive monitoring sessions

### Background Daemon

#### Daemon Installation
```bash
# Generate plist with configuration
listent --daemon --launchd -e "com.apple.security.*" --interval 1.0 > ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist

# Load daemon (user-level)
launchctl load ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist

# Load daemon (system-wide, requires sudo)
sudo listent --daemon --launchd -e "com.apple.*" > /Library/LaunchDaemons/com.github.mariohewardt.listent.plist
sudo launchctl load /Library/LaunchDaemons/com.github.mariohewardt.listent.plist
```
**Use Case**: Set up persistent monitoring

#### Daemon Management
```bash
# Check daemon status
launchctl list | grep listent

# View daemon logs
log show --predicate 'subsystem == "com.github.mariohewardt.listent"' --last 1h

# Stream live events
log stream --predicate 'subsystem == "com.github.mariohewardt.listent"'

# Stop daemon
launchctl unload ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist

# Update configuration (edit plist and reload)
launchctl unload ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist
listent --daemon --launchd -e "new.pattern.*" > ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist
launchctl load ~/Library/LaunchAgents/com.github.mariohewardt.listent.plist
```
**Use Case**: Ongoing daemon lifecycle management

---

## Edge Cases & Error Scenarios

### Permission Denied
```bash
# Scanning system directories
sudo listent /System/Library
```
**Scenario**: User lacks permissions for certain paths  
**Behavior**: Logs warning, continues scanning accessible files

### No Matches Found
```bash
# Filter that matches nothing
listent -e "nonexistent.entitlement"
```
**Scenario**: No files match the filter criteria  
**Behavior**: Reports zero matches, exits with success

### Invalid Interval
```bash
# Out-of-range interval
listent --monitor --interval 500
```
**Scenario**: Interval outside 0.1-300.0 second range  
**Behavior**: Error message, exits with error code

### Non-Mach-O Files
```bash
# Scanning directory with scripts/text files
listent /usr/share
```
**Scenario**: Directory contains non-executable files  
**Behavior**: Skips non-Mach-O files, reports in summary

### Binary Without Entitlements
```bash
# Scan unsigned or basic binaries
listent /usr/bin/true
```
**Scenario**: Valid binary with no code signing entitlements  
**Behavior**: Reports binary with empty entitlements list

---

## Success Criteria

### Static Scanning
- ✅ Successfully scans directories recursively
- ✅ Identifies all Mach-O binaries
- ✅ Extracts entitlements via codesign
- ✅ Filters by path and entitlement patterns
- ✅ Outputs human-readable and JSON formats
- ✅ Handles permission errors gracefully

### Real-time Monitoring  
- ✅ Detects new processes within polling interval
- ✅ Extracts entitlements from running processes
- ✅ Applies same filters as static mode
- ✅ Handles Ctrl+C gracefully
- ✅ Reports process metadata (PID, path, entitlements)

### Background Daemon
- ✅ Generates valid launchd plist
- ✅ Runs monitoring in background
- ✅ Logs to macOS Unified Logging System
- ✅ Survives system reboots (if RunAtLoad=true)
- ✅ Auto-restarts on crash (if KeepAlive=true)

---

## Performance Expectations

### Static Scanning
- **Default scan** (/Applications): < 5 seconds
- **Full system scan**: < 30 seconds
- **Memory usage**: < 100 MB

### Real-time Monitoring
- **Process detection latency**: Within configured interval (0.1-300s)
- **Memory usage**: < 50 MB baseline, < 1% system resources
- **CPU usage**: Negligible during idle, < 5% during active scanning

### Background Daemon
- **Startup time**: < 2 seconds
- **Memory footprint**: < 50 MB persistent
- **Log overhead**: Minimal (structured ULS entries only)

---

## Integration Patterns

### CI/CD Pipeline Integration
```bash
#!/bin/bash
# Verify entitlements before deployment
listent build/Release/MyApp.app -e "com.apple.security.app-sandbox" --json > entitlements.json

# Parse results
if ! jq -e '.summary.matched_files > 0' entitlements.json; then
  echo "ERROR: Required sandbox entitlement not found"
  exit 1
fi
```

### Security Monitoring Dashboard
```bash
# Continuous log ingestion
log stream --predicate 'subsystem == "com.github.mariohewardt.listent"' --style json | \
  jq -r 'select(.eventMessage | contains("Detected process")) | .eventMessage'
```

### Compliance Reporting
```bash
# Generate audit report
listent /Applications /System/Library --json | \
  jq '{
    audit_date: now | strftime("%Y-%m-%d"),
    total_binaries: .summary.total_files,
    signed_binaries: .summary.matched_files,
    results: [.results[] | {path: .path, entitlements: .entitlements}]
  }' > compliance_report.json
```

---

## Future Enhancement Scenarios

While not currently implemented, these scenarios inform potential v1.1+ features:

- **Configuration Files**: TOML-based saved filter configurations
- **Advanced Patterns**: Regex support for complex entitlement matching
- **Process Tree Monitoring**: Track child process launches
- **Alert Thresholds**: Trigger actions on specific entitlement combinations
- **Export Formats**: CSV, SQLite database output
- **Remote Monitoring**: Centralized logging from multiple machines

---

*These scenarios describe the current v1.0.0 implementation as of October 21, 2025.*
