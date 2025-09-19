# Implementation Plan: macOS Entitlement Listing CLI

**Branch**: `001-macos-rust-cli` | **Date**: 2025-09-18 | **Spec**: `specs/001-macos-rust-cli/spec.md`
**Input**: Feature specification from `specs/001-macos-rust-cli/spec.md`

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
Command line tool for macOS that enumerates code signing entitlements for executable binaries. Default scope: application directories. Users can narrow scanning via directory path filters and can filter results by entitlement keys. Provides human-readable and JSON output, summary stats (optional), graceful handling of unreadable files, and Homebrew distribution. Emphasis on performance (fast traversal and extraction) and predictable exit codes.

## Technical Context
**Language/Version**: Rust (stable toolchain; target latest stable at implementation time)  
**Primary Dependencies**: TBD (CLI parsing, likely `clap`; JSON serialization, likely `serde_json`; performance scanning: native std + possible parallel iterator crate)  
**Storage**: None (read-only filesystem scanning)  
**Testing**: cargo test (unit + integration), potential golden snapshot tests for output
**Target Platform**: macOS (x86_64 + arm64)  
**Project Type**: Single CLI binary  
**Performance Goals**: Default app directories scan <8s (Apple Silicon M1), <12s (Intel i5 quad-core)  
**Constraints**: Low memory overhead; avoid redundant parsing; deterministic output ordering  
**Scale/Scope**: Thousands of binaries per run; entitlement extraction per binary small

## Constitution Check
*GATE: Initial pass (constitution file is currently placeholder/incomplete)*

Observations:
- Constitution placeholders (principle names/descriptions not filled) → Cannot derive concrete mandatory gates.
- No explicit test-first enforcement text beyond placeholder → Adopt internal rule: write failing tests before implementation.

Interim Gates Applied:
1. Library/CLI separation: CLI logic thin, entitlement scanning core extracted to internal module for testability.
2. Text I/O: Human-readable + JSON supported (stdout); errors to stderr.
3. Test-First: Contract tests for CLI flags & integration scenario created before implementation.
4. Simplicity: Single binary, no multi-project split.

Status: PASS (provisional — revisit after constitution is fully authored). No complexity deviations recorded.

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

**Structure Decision**: Option 1 (single project). Justification: Single CLI; no frontend/mobile requirements; minimal internal modules: `scan`, `entitlements`, `output`, `cli`.

## Phase 0: Outline & Research
All former unknowns resolved (see `research.md`, Status: COMPLETE). Key decisions locked: performance targets, default directory set, JSON schema structure, interrupt behavior, summary metrics, verbosity model, minimum macOS version, entitlement extraction method, result aggregation strategy.

Status: COMPLETE.

## Phase 1: Design & Contracts
*Prerequisites: research.md complete (will proceed once Phase 0 done)*

Adaptation (CLI context – no network API):
1. Data model (`data-model.md`): Define core internal structs: `BinaryRecord`, `EntitlementSet`, `ScanResult`, `ScanSummary` with logical fields (no crate specifics).
2. Contracts: Represent CLI contract (flags & output schemas) instead of HTTP endpoints. Provide:
   - CLI Options Contract (`contracts/cli-options.md`): list flags, types, required/optional, mutual exclusivity rules.
   - Output Schemas (`contracts/output-json-schema.json`, `contracts/output-human-format.md`).
3. Contract Tests (as design artifacts only – failing until implemented): test specs in markdown describing assertions for integration tests (defer actual test files until implementation start to keep plan phase boundary or optionally stub under `tests/contract/`).
4. Quickstart: Minimal usage examples demonstrating default scan, path-filter scan, entitlement-filter scan, combined filters, JSON mode, no matches scenario, interrupt handling.
5. Update agent context file (create `.github/copilot-instructions.md` if required) summarizing decisions, limited to new concepts.

Exit Criteria:
- All contracts documented.
- Data model stable and supports requirements.
- Quickstart flows cover each acceptance scenario.
- Constitution re-check passes (still simple single binary design).

Status: COMPLETE.

## Phase 2: Task Planning Approach
*Description (actual tasks file generated later)*

Task Generation Outline:
1. Tasks for finalizing Phase 0 research decisions (if any remain unresolved when entering /tasks).
2. Tasks to create data model Rust structs & associated unit tests.
3. Tasks to implement entitlement scanning module (filesystem traversal + entitlement extraction) with performance instrumentation placeholders.
4. Tasks to implement CLI option parsing; validation logic for mutually compatible flags.
5. Tasks to implement output formatting (human + JSON) and summary statistics generation.
6. Tasks for integration tests matching acceptance scenarios.
7. Tasks for performance validation harness (benchmark over sample directory set).
8. Tasks for Homebrew packaging (formula, build, release notes, version flag verification).
9. Tasks for documentation updates (README usage examples, quickstart sync).

Ordering Principles:
- Test-first per feature slice.
- Core data & scanning before formatting.
- Output & CLI integration before packaging.
- Packaging & perf validation after functional stability.

Parallelizable ([P]): model struct creation, JSON format spec, human output layout, Homebrew formula draft (after initial binary builds).

Estimated Task Count: 22–28.

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
No deviations from simplicity principle. Table omitted.


## Progress Tracking
**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [ ] Phase 2: Task planning complete (/plan command - description only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS (provisional)
- [ ] Post-Design Constitution Check: PASS (pending review of Phase 1 artifacts)
- [x] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented (N/A currently)

---
*Based on Constitution (placeholder draft) - See `/memory/constitution.md`*
