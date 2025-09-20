
# Implementation Plan: Real-time Process Monitoring with Entitlement Filtering

**Branch**: `002-add-monitor-switch` | **Date**: September 19, 2025 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/Users/marioh/listent/specs/002-add-monitor-switch/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → ✅ Feature spec loaded successfully
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → ✅ Rust CLI project with existing structure
   → ✅ Project Type: Single CLI tool
3. Fill the Constitution Check section based on the content of the constitution document.
   → ✅ Constitution loaded from CONSTITUTION.md
4. Evaluate Constitution Check section below
   → ✅ No violations - feature aligns with Rust CLI tool principles
   → ✅ Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → ✅ Technical context clear, no NEEDS CLARIFICATION remain
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file
7. Re-evaluate Constitution Check section
   → ✅ Design maintains constitutional compliance
   → ✅ Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. ✅ STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Add real-time process monitoring capabilities to the existing listent CLI tool. The feature adds a `--monitor` switch that continuously polls running processes (configurable interval, default 1.0s), applies existing path and entitlement filters, and outputs newly detected processes to both console and macOS Unified Logging System. This extends the tool's one-time scanning capability to provide ongoing security monitoring for system administrators and security analysts.

## Technical Context
**Language/Version**: Rust 1.75+ (existing project)  
**Primary Dependencies**: clap (existing), serde_json (existing), plus new: sysinfo for process enumeration, oslog for Unified Logging  
**Storage**: In-memory process state tracking between polling cycles  
**Testing**: cargo test (existing framework), integration tests for monitoring behavior  
**Target Platform**: macOS (Apple Silicon + Intel, existing target)  
**Project Type**: Single CLI tool (existing structure)  
**Performance Goals**: Process detection within polling interval, minimal CPU overhead during polling  
**Constraints**: <50ms startup time maintained, graceful interrupt handling (Ctrl+C), memory efficient process tracking  
**Scale/Scope**: Handle typical macOS process counts (100-500 processes), monitor for hours/days without memory leaks

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Initial Assessment
- ✅ **Single binary**: Extends existing CLI tool, no additional binaries
- ✅ **Simplicity first**: Leverages existing architecture, adds minimal complexity  
- ✅ **Minimal dependencies**: Only adding essential crates for process enumeration and logging
- ✅ **Security posture**: No unsafe code required, maintains existing safety standards
- ✅ **CLI requirements**: Extends existing help/version, maintains error handling patterns
- ✅ **Performance targets**: Monitoring mode won't affect startup time for non-monitor usage
- ✅ **Testing standards**: Can unit test monitoring logic, integration test full workflows

### Post-Design Assessment
- ✅ **No new constitutional violations introduced**
- ✅ **Complexity remains justified and minimal**
- ✅ **Testing approach maintains quality standards**

## Project Structure

### Documentation (this feature)
```
specs/[###-feature]/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
src/
├── models/              # Existing - extend with monitoring types
├── cli/                 # Existing - add monitor flags
├── entitlements/        # Existing - reuse for filtering
├── scan/                # Existing - extend for process enumeration
├── output/              # Existing - extend for real-time output
└── monitor/             # NEW - monitoring-specific logic
    ├── mod.rs
    ├── process_tracker.rs
    ├── polling.rs
    └── unified_logging.rs

tests/
├── contract/            # Existing - add monitor contract tests
├── integration/         # Existing - add monitor integration tests
└── unit/                # Existing - add monitor unit tests
```

**Structure Decision**: Option 1 (Single project) - extends existing CLI tool structure

## Phase 0: Outline & Research

### Research Areas Completed
✅ **macOS Process Enumeration**: `sysinfo` crate selected for safe, efficient process listing  
✅ **Unified Logging Integration**: `oslog` crate provides native macOS logging integration  
✅ **Process State Tracking**: HashMap-based approach for O(1) process lookups and comparison  
✅ **Signal Handling**: `ctrlc` crate for graceful Ctrl+C shutdown handling  
✅ **Memory Management**: Bounded memory usage with automatic cleanup of terminated processes

**Output**: ✅ research.md created with all technical decisions documented

## Phase 1: Design & Contracts

### Data Model Design ✅
Core entities mapped to Rust types:
- **MonitoredProcess**: PID, path, entitlements, discovery timestamp
- **PollingConfiguration**: Interval, filters, output settings  
- **ProcessSnapshot**: HashMap of processes for efficient comparison
- **EntitlementMatch**: Process with specific matching entitlements
- **LogEntry**: Unified Logging event structure

### API Contracts ✅
Monitor mode extends existing CLI interface:
- **CLI Extension**: `--monitor` and `--interval` flags
- **Output Formats**: Human-readable and JSON for process detection events
- **Error Handling**: Validation and graceful degradation patterns
- **Unified Logging**: Structured events with metadata

### Contract Tests Generated ✅
Comprehensive test coverage designed:
- CLI argument parsing and validation tests
- Output format verification (human-readable and JSON)
- Unified Logging integration tests
- Error handling and edge case tests

### Integration Test Scenarios ✅
User story validation scenarios:
- Basic monitoring with process detection
- Path filtering for specific directories
- Entitlement filtering for security analysis
- Combined filtering with custom intervals
- JSON output for programmatic consumption

### Agent Context Updated ✅
GitHub Copilot instructions extended with:
- Monitor module architecture and dependencies
- CLI integration patterns and data structures
- Testing approach for long-running processes
- Performance considerations and memory management

**Output**: ✅ data-model.md, contracts/, quickstart.md, .github/copilot-instructions.md

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `.specify/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs (contracts, data model, quickstart)
- Each contract → contract test task [P]
- Each entity → model creation task [P] 
- Each user story → integration test task
- Implementation tasks to make tests pass

**Ordering Strategy**:
- TDD order: Tests before implementation 
- Dependency order: Models before services before UI
- Mark [P] for parallel execution (independent files)

**Estimated Output**: 25-30 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |


## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [ ] Phase 0: Research complete (/plan command)
- [ ] Phase 1: Design complete (/plan command)
- [ ] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [ ] Initial Constitution Check: PASS
- [ ] Post-Design Constitution Check: PASS
- [ ] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*
