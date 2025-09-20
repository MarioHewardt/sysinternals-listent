# Tasks: Real-time Process Monitoring with Entitlement Filtering

**Input**: Design documents from `/Users/marioh/listent/specs/002-add-monitor-switch/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → ✅ Implementation plan loaded successfully
   → ✅ Extract: Rust CLI, sysinfo/oslog/ctrlc dependencies, single project structure
2. Load optional design documents:
   → ✅ data-model.md: 5 entities identified → model tasks
   → ✅ contracts/: 3 contract files → contract test tasks
   → ✅ research.md: Technical decisions → setup tasks
3. Generate tasks by category:
   → ✅ Setup: dependencies, module structure, linting
   → ✅ Tests: contract tests, integration tests
   → ✅ Core: models, monitor engine, CLI extension
   → ✅ Integration: unified logging, signal handling
   → ✅ Polish: unit tests, performance validation
4. Apply task rules:
   → ✅ Different files = mark [P] for parallel
   → ✅ Same file = sequential (no [P])
   → ✅ Tests before implementation (TDD)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency graph
7. Create parallel execution examples
8. Validate task completeness:
   → ✅ All contracts have tests
   → ✅ All entities have models
   → ✅ All CLI features implemented
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- Paths assume existing listent CLI tool structure

## Phase 3.1: Setup

### T001: Add Monitor Dependencies
Add new dependencies to `Cargo.toml`:
- sysinfo = "0.29" (process enumeration)
- oslog = "0.1" (Unified Logging)
- ctrlc = "3.4" (signal handling)

### T002: Create Monitor Module Structure
Create new monitor module directory structure in `src/monitor/`:
- `src/monitor/mod.rs` (module exports)
- `src/monitor/process_tracker.rs` (process state tracking)
- `src/monitor/polling.rs` (polling loop implementation)
- `src/monitor/unified_logging.rs` (macOS logging integration)

### T003: [P] Configure New Dependencies
Update main.rs to include monitor module and configure new dependencies for conditional compilation on macOS.

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### T004: [P] CLI Contract Test - Monitor Flag
Create `tests/contract/test_monitor_cli.rs`:
- Test --monitor flag parsing
- Test --interval parameter validation (0.1s-300s range)
- Test error handling for invalid intervals
- Test help text includes monitor options

### T005: [P] CLI Contract Test - Integration with Existing Filters
Create `tests/contract/test_monitor_filters.rs`:
- Test --monitor with -p (path) filter combinations
- Test --monitor with -e (entitlement) filter combinations  
- Test --monitor with --json output format
- Test --monitor with --quiet mode

### T006: [P] Output Format Contract Test
Create `tests/contract/test_monitor_output.rs`:
- Test human-readable process detection output format
- Test JSON process detection output format
- Test timestamp formatting (ISO 8601 UTC)
- Test entitlements list formatting

### T007: [P] Unified Logging Contract Test
Create `tests/contract/test_unified_logging.rs`:
- Test log events created with correct subsystem/category
- Test log message format matches specification
- Test graceful degradation when logging unavailable
- Test metadata attachment for structured logging

### T008: [P] Integration Test - Basic Monitoring
Create `tests/integration/test_basic_monitoring.rs`:
- Test monitor mode starts and detects processes
- Test Ctrl+C shutdown handling
- Test polling interval timing accuracy
- Test process detection within interval bounds

### T009: [P] Integration Test - Path Filtering
Create `tests/integration/test_monitor_path_filtering.rs`:
- Test monitoring specific directories only
- Test multiple path filters
- Test non-existent path error handling
- Test process filtering by executable path

### T010: [P] Integration Test - Entitlement Filtering
Create `tests/integration/test_monitor_entitlement_filtering.rs`:
- Test filtering by specific entitlements
- Test multiple entitlement filters
- Test processes with no entitlements
- Test entitlement extraction error handling

### T011: [P] Integration Test - Output Formats
Create `tests/integration/test_monitor_output_formats.rs`:
- Test JSON output format in monitor mode
- Test quiet mode output suppression  
- Test error message formatting
- Test real-time output streaming

## Phase 3.3: Core Implementation (ONLY after tests are failing)

### T012: [P] MonitoredProcess Model
Create `src/models/mod.rs` extensions:
- Add MonitoredProcess struct with fields (pid, name, executable_path, entitlements, discovery_timestamp)
- Add validation methods for PID, path existence
- Add Display implementation for debugging

### T013: [P] PollingConfiguration Model
Add to `src/models/mod.rs`:
- Add PollingConfiguration struct with interval, filters, output settings
- Add validation for interval bounds (0.1s-300s)
- Add path filter validation (existing directories)
- Add entitlement filter validation

### T014: [P] ProcessSnapshot Model
Add to `src/models/mod.rs`:
- Add ProcessSnapshot struct with HashMap<u32, MonitoredProcess>
- Implement new_processes() method for comparison
- Implement terminated_processes() method
- Add timestamp and scan_duration tracking

### T015: [P] EntitlementMatch and LogEntry Models
Add to `src/models/mod.rs`:
- Add EntitlementMatch struct linking process to matched entitlements
- Add LogEntry struct for Unified Logging events
- Add validation for matched_entitlements non-empty constraint

### T016: CLI Extension - Monitor Arguments
Extend `src/cli/mod.rs`:
- Add --monitor boolean flag to Args struct
- Add --interval parameter with f64 type and validation
- Update help text to include monitor mode documentation
- Add argument validation in build() method

### T017: Process Tracker Implementation
Implement `src/monitor/process_tracker.rs`:
- ProcessTracker struct managing current/previous snapshots
- detect_new_processes() method for snapshot comparison
- apply_path_filters() method reusing existing logic
- apply_entitlement_filters() method reusing existing logic

### T018: Polling Engine Implementation  
Implement `src/monitor/polling.rs`:
- start_monitoring() function with polling loop
- Integration with existing scan and entitlement modules
- Interval timing and sleep implementation
- Error handling for individual process access failures

### T019: Unified Logging Implementation
Implement `src/monitor/unified_logging.rs`:
- Logger initialization with subsystem "com.sysinternals.entlist"
- log_process_detection() function for detected processes
- Error handling for logging failures (graceful degradation)
- Message formatting per contract specification

## Phase 3.4: Integration

### T020: Signal Handler Integration
Extend `src/monitor/polling.rs`:
- Integrate ctrlc for graceful shutdown
- Set up atomic boolean for clean exit signaling
- Ensure final status message before termination
- Handle signal during sleep/polling operations

### T021: CLI Integration - Monitor Mode
Extend `src/main.rs`:
- Add monitor mode detection in main() function
- Create PollingConfiguration from CLI arguments
- Route to monitoring workflow when --monitor specified
- Maintain existing non-monitor functionality unchanged

### T022: Output Integration - Monitor Events
Extend `src/output/mod.rs`:
- Add process detection event formatting (human-readable)
- Add JSON format for process detection events
- Integrate with existing --json and --quiet flag handling
- Add real-time output flushing for immediate visibility

### T023: Entitlements Integration - Reuse Existing Logic
Update `src/monitor/process_tracker.rs`:
- Integrate existing entitlement extraction from src/entitlements/mod.rs
- Handle entitlement extraction failures gracefully
- Apply existing entitlement filtering logic
- Maintain consistent error handling patterns

## Phase 3.5: Polish

### T024: [P] Unit Tests - Process Tracker
Create `tests/unit/test_process_tracker.rs`:
- Test ProcessSnapshot comparison logic
- Test new_processes() and terminated_processes() methods
- Test filter application with various scenarios
- Test edge cases (empty snapshots, duplicate PIDs)

### T025: [P] Unit Tests - Polling Configuration
Create `tests/unit/test_polling_configuration.rs`:
- Test interval validation (bounds checking)
- Test path filter validation 
- Test configuration creation from CLI arguments
- Test error cases and validation failures

### T026: [P] Unit Tests - Monitor Models
Create `tests/unit/test_monitor_models.rs`:
- Test MonitoredProcess creation and validation
- Test EntitlementMatch validation rules
- Test LogEntry message formatting
- Test model serialization for JSON output

### T027: [P] Performance Tests
Create `tests/integration/test_monitor_performance.rs`:
- Test CPU usage during monitoring (<1% target)
- Test memory usage stability over time
- Test polling accuracy with different intervals
- Test system impact under high process activity

### T028: Error Handling Polish
Update monitor modules:
- Add comprehensive error messages for all failure modes
- Ensure graceful degradation for permission issues
- Add warning messages for non-critical failures
- Test error scenarios and recovery behavior

### T029: [P] Documentation Updates
Update project documentation:
- Update `README.md` with monitor mode examples
- Update help text with comprehensive monitor options
- Add monitor mode to quickstart examples
- Update `CONSTITUTION.md` if constitutional impact

### T030: Manual Testing Validation
Execute comprehensive manual testing:
- Run through all quickstart scenarios
- Verify Unified Logging integration with Console.app
- Test extended monitoring (30+ minutes)
- Validate all CLI combinations and edge cases

## Dependencies

### Critical Path
1. **Setup (T001-T003)** → **Tests (T004-T011)** → **Core Implementation (T012-T019)** → **Integration (T020-T023)** → **Polish (T024-T030)**

### Specific Dependencies
- T016 (CLI extension) blocks T021 (CLI integration)
- T012-T015 (models) block T017 (process tracker)
- T017 (process tracker) blocks T018 (polling engine)
- T018 (polling engine) blocks T020 (signal handling)
- T019 (unified logging) and T022 (output) can be parallel
- T004-T011 (all contract tests) must complete before any T012+ implementation

### Same-File Conflicts (No Parallel Execution)
- T012, T013, T014, T015 all modify `src/models/mod.rs` (sequential)
- T016 and T021 both modify CLI-related files (sequential)
- T022 and existing output functionality (sequential)

## Parallel Execution Examples

### Tests Phase (T004-T011)
```bash
# All contract tests can run in parallel (different files):
Task: "CLI Contract Test - Monitor Flag in tests/contract/test_monitor_cli.rs"
Task: "CLI Contract Test - Filter Integration in tests/contract/test_monitor_filters.rs"  
Task: "Output Format Contract Test in tests/contract/test_monitor_output.rs"
Task: "Unified Logging Contract Test in tests/contract/test_unified_logging.rs"

# All integration tests can run in parallel (different files):
Task: "Integration Test - Basic Monitoring in tests/integration/test_basic_monitoring.rs"
Task: "Integration Test - Path Filtering in tests/integration/test_monitor_path_filtering.rs"
Task: "Integration Test - Entitlement Filtering in tests/integration/test_monitor_entitlement_filtering.rs"
Task: "Integration Test - Output Formats in tests/integration/test_monitor_output_formats.rs"
```

### Implementation Phase (Selected Tasks)
```bash
# These can run in parallel (different modules):
Task: "Unified Logging Implementation in src/monitor/unified_logging.rs"
Task: "Process Tracker Implementation in src/monitor/process_tracker.rs"

# These are sequential (same file):
Task: "MonitoredProcess Model in src/models/mod.rs"
# Then:
Task: "PollingConfiguration Model in src/models/mod.rs"
# Then:
Task: "ProcessSnapshot Model in src/models/mod.rs"
```

### Polish Phase (T024-T030)
```bash
# Unit tests can run in parallel (different files):
Task: "Unit Tests - Process Tracker in tests/unit/test_process_tracker.rs"
Task: "Unit Tests - Polling Configuration in tests/unit/test_polling_configuration.rs"
Task: "Unit Tests - Monitor Models in tests/unit/test_monitor_models.rs"
Task: "Performance Tests in tests/integration/test_monitor_performance.rs"
Task: "Documentation Updates (README.md, help text)"
```

## Validation Checklist
*GATE: Checked before task execution*

- [x] All contracts have corresponding tests (T004-T007 cover 3 contract files)
- [x] All entities have model tasks (T012-T015 cover 5 data model entities)
- [x] All tests come before implementation (T004-T011 before T012+)
- [x] Parallel tasks truly independent (different files, no shared dependencies)
- [x] Each task specifies exact file path (all tasks include specific file paths)
- [x] No task modifies same file as another [P] task (verified no conflicts)
- [x] CLI extension tasks cover argument parsing and integration
- [x] Integration tasks cover signal handling, logging, and output
- [x] Polish tasks include performance validation and documentation

## Notes
- Monitor mode extends existing CLI tool architecture
- Reuses existing path/entitlement filtering and output formatting
- Maintains constitutional compliance (single binary, minimal dependencies)
- TDD approach ensures tests validate contracts before implementation
- Performance targets: <1% CPU, no memory leaks, <50ms startup maintained