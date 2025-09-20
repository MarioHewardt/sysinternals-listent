
# Implementation Plan: LaunchD Daemon Support for Process Monitoring

**Branch**: `003-add-launchd-daemon` | **Date**: September 20, 2025 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/Users/marioh/listent/specs/003-add-launchd-daemon/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, `GEMINI.md` for Gemini CLI, `QWEN.md` for Qwen Code or `AGENTS.md` for opencode).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Enable running the existing process monitoring capability as a macOS launchd daemon with dynamic configuration updates. The daemon monitors newly started processes, extracts their entitlements using existing logic, logs findings to ULS, and supports runtime configuration changes without restart. Installation/uninstallation commands manage launchd integration automatically.

## Technical Context
**Language/Version**: Rust 1.75+ (existing project constraint)  
**Primary Dependencies**: sysinfo (process monitoring), tokio (async runtime for daemon), toml (config parsing), nix (Unix domain sockets)  
**Storage**: Configuration files in `/Library/LaunchDaemons/` for plist, `/etc/listent/` or `~/.config/listent/` for daemon config  
**Testing**: cargo test with existing contract/integration/unit test structure  
**Target Platform**: macOS (Apple Silicon + Intel) - existing project scope  
**Project Type**: single - extends existing Rust CLI tool  
**Performance Goals**: <50ms startup (constitutional requirement), polling configurable 0.1-300s, minimal memory footprint  
**Constraints**: No unsafe code (constitutional), ULS-only logging in daemon mode, no terminal dependencies, existing monitor logic reuse  
**Scale/Scope**: System-wide process monitoring, single daemon instance per system, config updates via IPC

User provided technical details: "lets plan"

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Single Binary Requirement**: ✅ PASS - Feature extends existing single binary CLI tool, no new binaries
**Minimal Dependencies**: ⚠️ ACCEPTABLE - Adding tokio (async), toml (config), nix (sockets) justified by core requirements
**No Network Services/Daemons**: ❌ CONSTITUTIONAL VIOLATION - JUSTIFIED by core feature requirement
**Simplicity First**: ⚠️ ACCEPTABLE - Complexity justified by essential background monitoring requirement
**Security Posture**: ✅ PASS - No unsafe code planned, reuses existing security patterns
**Performance Requirements**: ✅ PASS - Maintains <50ms startup, configurable polling aligns with constitution
**CLI Standards**: ✅ PASS - Extends existing CLI with standard help/version support
**Exit Codes**: ✅ PASS - Will follow existing exit code patterns

**Post-Design Re-evaluation**:
- **Daemon Architecture**: Design maintains existing CLI as primary interface, daemon as optional extension
- **Dependency Justification**: 
  - tokio: Essential for non-blocking IPC and signal handling in daemon mode
  - toml: Standard configuration format, minimal overhead, widely used in Rust ecosystem
  - nix: Unix-specific functionality for daemon operations, well-maintained, no alternatives
- **Complexity Mitigation**: Reuses 90% of existing code, adds daemon mode as separate execution path
- **Constitutional Compliance**: Violation justified and mitigated through careful design

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
# Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure]
```

**Structure Decision**: [DEFAULT to Option 1 unless Technical Context indicates web/mobile app]

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context** above:
   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:
   ```
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:
   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:
   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `.specify/scripts/bash/update-agent-context.sh copilot` for your AI assistant
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

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
| Daemon functionality | Background system monitoring requires persistent service | CLI-only approach cannot provide continuous monitoring for security compliance |
| Async runtime (tokio) | Non-blocking IPC and signal handling essential for config updates | Blocking I/O would freeze monitoring during config updates, defeating purpose |
| IPC mechanism | Dynamic configuration updates without restart required by spec | File-watching approach has race conditions and slower response times |

## Progress Tracking
*This checklist is updated during execution flow*

- [x] **Phase 0**: Load feature spec and resolve technical unknowns
  - [x] Clarified auto-start behavior (yes, on boot)
  - [x] Clarified privilege requirements (root for system-wide)
  - [x] Clarified ULS query interface (built-in commands + external tools)
  - [x] Research completed: research.md generated

- [x] **Phase 1**: Design and contracts completed  
  - [x] Initial Constitution Check passed (violations justified)
  - [x] Data model designed: data-model.md generated
  - [x] CLI contract specified: contracts/cli-contract.md generated
  - [x] Configuration contract specified: contracts/configuration-contract.md generated
  - [x] ULS integration contract specified: contracts/uls-integration.md generated
  - [x] Implementation quickstart guide: quickstart.md generated
  - [x] Post-Design Constitution Check passed

- [x] **Phase 2**: Task generation completed
  - [x] Generated 54 sequential tasks (T001-T054) 
  - [x] TDD approach: Tests (T004-T016) before implementation (T017-T054)
  - [x] Parallel execution identified for independent files
  - [x] Dependencies mapped: Setup → Tests → Models → Services → Commands → Polish
  - [x] All contracts and entities covered with appropriate tasks
  - [x] Constitutional compliance maintained throughout task design

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
