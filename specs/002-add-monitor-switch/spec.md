# Feature Specification: Real-time Process Monitoring with Entitlement Filtering

**Feature Branch**: `002-add-monitor-switch`  
**Created**: September 19, 2025  
**Status**: Draft  
**Input**: User description: "Add --monitor switch for real-time process monitoring with entitlement filtering. Monitor new process creation by polling running processes every X seconds (configurable). Honor existing -p (path) and -e (entitlement) switches. Output app name, PID, and entitlements to console and Unified Logging System under subsystem com.sysinternals.entlist."

## Execution Flow (main)
```
1. Parse user description from Input
   → Feature description provided and clear
2. Extract key concepts from description
   → Actors: system administrators, security analysts
   → Actions: monitor, filter, output, log
   → Data: processes, PIDs, entitlements, paths
   → Constraints: configurable polling interval, path filtering, entitlement filtering
3. For each unclear aspect:
   → Default polling interval: 1 second (responsive but not excessive)
   → Polling interval limits: minimum 0.1s, maximum 300s (5 minutes)
   → Process disappearance: ignore processes that terminate between polls
4. Fill User Scenarios & Testing section
   → User flow: start monitoring → filter processes → detect new processes → output results
5. Generate Functional Requirements
   → Each requirement testable and measurable
6. Identify Key Entities
   → MonitoredProcess, EntitlementMatch, PollingConfiguration
7. Run Review Checklist
   → Spec complete with reasonable defaults and limits specified
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

---

## User Scenarios & Testing

### Primary User Story
As a security analyst or system administrator, I want to monitor new process creation in real-time with entitlement filtering so that I can immediately detect when applications with specific entitlements or from specific paths are launched, enabling proactive security monitoring and compliance verification.

### Acceptance Scenarios
1. **Given** listent is running with `--monitor --interval 5.0` flags, **When** a new process is launched, **Then** the system detects it within 5 seconds and outputs process information to console
2. **Given** listent is running with `--monitor -p /Applications -e com.apple.security.camera`, **When** an application from /Applications with camera entitlement launches, **Then** the system immediately displays app name, PID, and camera entitlement details
3. **Given** listent is running in monitor mode, **When** a monitored process matches filters, **Then** the event is logged to Unified Logging System under com.sysinternals.entlist subsystem
4. **Given** listent is monitoring with `--interval 2.0`, **When** multiple processes launch simultaneously, **Then** all matching processes are detected and reported within one polling cycle
5. **Given** listent is monitoring with path filter, **When** a process launches outside the filtered path, **Then** the process is ignored and not reported

### Edge Cases
- What happens when polling interval is set below 0.1s or above 300s? (System should reject with error)
- How does system handle when a process launches and terminates between polling cycles? (Ignore - process monitoring focuses on persistent processes)
- What occurs when entitlement extraction fails for a detected process? (Log error but continue monitoring other processes)
- How does the system behave when Unified Logging System is unavailable? (Continue console output, log warning about logging failure)
- What happens when user lacks permissions to read process information? (Skip inaccessible processes, continue monitoring others)

## Requirements

### Functional Requirements
- **FR-001**: System MUST provide a `--monitor` command-line switch to enable real-time process monitoring mode
- **FR-002**: System MUST provide a configurable polling interval via `--interval` parameter with default of 1.0 seconds
- **FR-003**: System MUST detect newly launched processes by comparing current process list with previous poll
- **FR-004**: System MUST honor existing `-p` (path) filter to monitor only processes from specified directories
- **FR-005**: System MUST honor existing `-e` (entitlement) filter to report only processes with matching entitlements
- **FR-006**: System MUST output detected processes to console showing app name, PID, and relevant entitlements
- **FR-007**: System MUST log detected process events to Unified Logging System under subsystem "com.sysinternals.entlist"
- **FR-008**: System MUST continue monitoring until user terminates the process (Ctrl+C)
- **FR-009**: System MUST validate polling interval parameters (minimum 0.1s, maximum 300s)
- **FR-010**: System MUST handle process permission errors gracefully without terminating monitoring
- **FR-011**: System MUST provide clear feedback when monitoring starts and stops
- **FR-012**: System MUST maintain monitoring state between polling cycles without memory leaks

### Key Entities
- **MonitoredProcess**: Represents a detected process with PID, executable path, entitlements, and discovery timestamp
- **PollingConfiguration**: Represents monitoring settings including interval, path filters, and entitlement filters
- **ProcessSnapshot**: Represents the current state of running processes at a specific polling cycle
- **EntitlementMatch**: Represents a process that satisfies the entitlement filtering criteria
- **LogEntry**: Represents an event logged to Unified Logging System with process details and timestamp

---

## Review & Acceptance Checklist

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous  
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---
