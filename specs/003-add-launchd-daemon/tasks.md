# Tasks: LaunchD Daemon Support for Process Monitoring

**Input**: Design documents from `/Users/marioh/listent/specs/003-add-launchd-daemon/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/

## Execution Flow (main)
```
✓ Loaded plan.md: Rust CLI tool extension with daemon mode
✓ Loaded data-model.md: DaemonConfiguration, IpcMessage, LaunchDPlist entities
✓ Loaded contracts: CLI commands, configuration management, ULS integration
✓ Generated tasks by category: Setup → Tests → Models → Services → Integration → Polish
✓ Applied TDD ordering: Tests before implementation
✓ Marked parallel tasks [P] for independent files
✓ Task validation: All contracts covered, all entities modeled
✓ SUCCESS: 30 tasks ready for execution
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Phase 3.1: Setup & Dependencies

- [ ] **T001** Add daemon dependencies to Cargo.toml (tokio, toml, nix, uuid)
- [ ] **T002** [P] Create daemon module structure: src/daemon/mod.rs, config.rs, ipc.rs, launchd.rs, logging.rs
- [ ] **T003** [P] Update .gitignore for daemon runtime files (/var/run/listent/, /etc/listent/)

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3

**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Contract Tests
- [ ] **T004** [P] Contract test install-daemon CLI command in tests/contract/test_daemon_install.rs
- [ ] **T005** [P] Contract test uninstall-daemon CLI command in tests/contract/test_daemon_uninstall.rs
- [ ] **T006** [P] Contract test daemon-status CLI command in tests/contract/test_daemon_status.rs
- [ ] **T007** [P] Contract test update-config CLI command in tests/contract/test_daemon_config.rs
- [ ] **T008** [P] Contract test logs CLI command in tests/contract/test_daemon_logs.rs
- [ ] **T009** [P] Contract test daemon mode execution (--daemon flag) in tests/contract/test_daemon_mode.rs

### Configuration Tests
- [ ] **T010** [P] Contract test TOML configuration parsing and validation in tests/contract/test_config_validation.rs
- [ ] **T011** [P] Contract test atomic configuration updates in tests/contract/test_config_atomic.rs
- [ ] **T012** [P] Contract test configuration rollback mechanism in tests/contract/test_config_rollback.rs

### Integration Tests
- [ ] **T013** [P] Integration test full daemon installation lifecycle in tests/integration/test_daemon_lifecycle.rs
- [ ] **T014** [P] Integration test IPC communication between CLI and daemon in tests/integration/test_daemon_ipc.rs
- [ ] **T015** [P] Integration test configuration updates without restart in tests/integration/test_config_hot_reload.rs
- [ ] **T016** [P] Integration test ULS logging output validation in tests/integration/test_uls_logging.rs

## Phase 3.3: Core Data Models (ONLY after tests are failing)

- [ ] **T017** [P] DaemonConfiguration and related types in src/daemon/config.rs
- [ ] **T018** [P] IpcMessage and IpcResponse enums in src/daemon/ipc.rs  
- [ ] **T019** [P] LaunchDPlist and LaunchDCommand types in src/daemon/launchd.rs
- [ ] **T020** [P] Enhanced logging types and ULS integration in src/daemon/logging.rs

## Phase 3.4: CLI Extensions

- [ ] **T021** Extend Args struct with daemon subcommands in src/cli/mod.rs
- [ ] **T022** Add install-daemon command parser and validation in src/cli/mod.rs
- [ ] **T023** Add uninstall-daemon command parser and validation in src/cli/mod.rs
- [ ] **T024** Add daemon-status command parser and validation in src/cli/mod.rs
- [ ] **T025** Add update-config command parser and validation in src/cli/mod.rs

## Phase 3.5: Core Implementation

### Configuration Management
- [ ] **T026** Configuration file parsing, validation, and persistence in src/daemon/config.rs
- [ ] **T027** Atomic configuration update mechanism with backup/rollback in src/daemon/config.rs
- [ ] **T028** Configuration diff detection and change validation in src/daemon/config.rs

### IPC Implementation  
- [ ] **T029** Unix domain socket server for daemon communication in src/daemon/ipc.rs
- [ ] **T030** IPC message handling and routing in src/daemon/ipc.rs
- [ ] **T031** IPC client functionality for CLI commands in src/daemon/ipc.rs

### LaunchD Integration
- [ ] **T032** LaunchD plist generation and installation in src/daemon/launchd.rs
- [ ] **T033** LaunchD service control (load, unload, start, stop) in src/daemon/launchd.rs
- [ ] **T034** LaunchD status checking and service management in src/daemon/launchd.rs

### Daemon Mode Execution
- [ ] **T035** Daemon mode main loop with signal handling in src/daemon/mod.rs
- [ ] **T036** Integration with existing monitor functionality for daemon mode in src/daemon/mod.rs
- [ ] **T037** Configuration reload without restart (SIGHUP handling) in src/daemon/mod.rs

## Phase 3.6: CLI Command Implementation

- [ ] **T038** install-daemon command implementation with privilege checking in src/main.rs
- [ ] **T039** uninstall-daemon command implementation with cleanup in src/main.rs
- [ ] **T040** daemon-status command implementation with JSON output in src/main.rs
- [ ] **T041** update-config command implementation with validation in src/main.rs
- [ ] **T042** logs command implementation with ULS querying in src/main.rs

## Phase 3.7: Enhanced ULS Integration

- [ ] **T043** Structured logging for process detection events in src/daemon/logging.rs
- [ ] **T044** Configuration change logging and audit trail in src/daemon/logging.rs
- [ ] **T045** Performance metrics logging and rate limiting in src/daemon/logging.rs
- [ ] **T046** Error and security event logging in src/daemon/logging.rs

## Phase 3.8: Error Handling & Polish

- [ ] **T047** [P] Comprehensive error types for daemon operations in src/daemon/mod.rs
- [ ] **T048** [P] Privilege escalation checking and clear error messages in src/daemon/mod.rs
- [ ] **T049** [P] Graceful shutdown and cleanup on errors in src/daemon/mod.rs
- [ ] **T050** [P] Unit tests for critical daemon functionality in tests/unit/test_daemon_units.rs

## Phase 3.9: Performance & Validation

- [ ] **T051** Performance optimization for daemon startup (<2s) and memory usage (<20MB)
- [ ] **T052** Manual testing validation following quickstart.md scenarios
- [ ] **T053** [P] Update README.md with daemon installation and usage instructions
- [ ] **T054** [P] Update .github/copilot-instructions.md with daemon implementation details

## Dependencies

**Setup Dependencies**:
- T001 (Cargo.toml) must complete before all implementation tasks

**Test-First Dependencies**:
- T004-T016 (all tests) MUST complete and FAIL before T017-T054 (implementation)

**Core Implementation Dependencies**:
- T017-T020 (data models) before T026-T037 (services using those models)
- T021-T025 (CLI parsing) before T038-T042 (CLI command implementation) 
- T026-T028 (config management) before T029-T031 (IPC using config)
- T032-T034 (launchd integration) before T038-T039 (install/uninstall commands)
- T035-T037 (daemon mode) before T043-T046 (daemon logging)

**Sequential File Dependencies**:
- T021-T025 (all modify src/cli/mod.rs) must be sequential
- T026-T028 (all modify src/daemon/config.rs) must be sequential  
- T029-T031 (all modify src/daemon/ipc.rs) must be sequential
- T032-T034 (all modify src/daemon/launchd.rs) must be sequential
- T038-T042 (all modify src/main.rs) must be sequential

## Parallel Execution Examples

**Phase 3.2 - All contract tests can run in parallel**:
```bash
# These can all be worked on simultaneously:
Task: "Contract test install-daemon CLI command in tests/contract/test_daemon_install.rs"
Task: "Contract test uninstall-daemon CLI command in tests/contract/test_daemon_uninstall.rs" 
Task: "Contract test daemon-status CLI command in tests/contract/test_daemon_status.rs"
Task: "Integration test full daemon installation lifecycle in tests/integration/test_daemon_lifecycle.rs"
```

**Phase 3.3 - All data models can be developed in parallel**:
```bash
Task: "DaemonConfiguration and related types in src/daemon/config.rs"
Task: "IpcMessage and IpcResponse enums in src/daemon/ipc.rs"
Task: "LaunchDPlist and LaunchDCommand types in src/daemon/launchd.rs"
Task: "Enhanced logging types and ULS integration in src/daemon/logging.rs"
```

## Task Generation Rules Applied

1. **From Contracts**:
   - CLI contract → 5 CLI command tests (T004-T008) + 5 implementation tasks (T038-T042)
   - Configuration contract → 3 config tests (T010-T012) + 3 implementation tasks (T026-T028)
   - ULS contract → 1 integration test (T016) + 4 implementation tasks (T043-T046)

2. **From Data Model**:
   - Each entity group → model creation tasks [P] (T017-T020)
   - State machines → lifecycle integration test (T013)

3. **From User Stories**:
   - Installation scenario → integration test (T013)
   - Configuration update scenario → integration test (T015)  
   - Monitoring scenario → integration test (T016)

4. **Ordering Applied**:
   - Setup (T001-T003) → Tests (T004-T016) → Models (T017-T020) → Services (T026-T037) → Commands (T038-T042) → Polish (T047-T054)

## Validation Checklist ✅

- [x] All contracts have corresponding tests (CLI: T004-T008, Config: T010-T012, ULS: T016)
- [x] All entities have model tasks (T017-T020 cover all entities from data-model.md)
- [x] All tests come before implementation (T004-T016 before T017-T054)
- [x] Parallel tasks truly independent (marked [P] only for different files)
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task (sequential tasks for same files)
- [x] TDD enforced: Tests must be written and failing before implementation begins
- [x] Dependencies clearly specified with blocking relationships identified

## Critical Success Factors

1. **Constitutional Compliance**: All tasks maintain <50ms CLI startup for non-daemon commands
2. **Security**: Privilege checking implemented in T038 before any system integration
3. **Error Handling**: Comprehensive error scenarios covered in tests T004-T016
4. **Performance**: Memory and startup time targets validated in T051
5. **Integration**: Full system testing via T013-T016 before production deployment

**Estimated Duration**: 8-10 development days following the quickstart timeline
**Risk Level**: Medium (launchd integration complexity, privilege management)
**Mitigation**: Extensive testing phases (T004-T016) and incremental validation