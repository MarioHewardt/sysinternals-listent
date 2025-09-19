# Tasks: macOS Entitlement Listing CLI

**Input**: Design documents from `specs/001-macos-rust-cli/`
**Prerequisites**: `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md` (all present)

## Execution Flow (generated)
Derived from templates and project-specific artifacts. Follow strictly; do not skip test-first gates.

## Format: `[ID] [P?] Description`
- [P] indicates task can be executed in parallel (different files, no dependency ordering conflict).
- Absence of [P] means sequential or dependent.

## Phase 3.1: Setup
- [ ] T001 Initialize Rust toolchain & update `Cargo.toml` metadata (name, version, authors, license) in repository root.
- [ ] T002 Add baseline dependencies in `Cargo.toml` (clap, serde, serde_json, anyhow, time) without implementing logic.
- [ ] T003 Configure formatting & linting: add `rustfmt.toml` (if needed) & enable `cargo clippy` workflow (document command in README) [P].
- [ ] T004 Create source module skeleton: `src/cli/mod.rs`, `src/scan/mod.rs`, `src/entitlements/mod.rs`, `src/output/mod.rs`, `src/models/mod.rs` with placeholder module comments.
- [ ] T005 Add feature gate constants & version build metadata hook in `build.rs` (commit hash injection) (sequential after dependencies).

## Phase 3.2: Tests First (TDD) – Contract & Integration Tests
Write tests so they FAIL initially. Do not implement production code beyond stubs compiling.

### Contract Tests (CLI Options & Output Schemas)
- [ ] T006 [P] Create `tests/contract/test_cli_help.rs` validating `--help` includes required options (paths, entitlement, json, quiet, verbose, version, summary flags).
- [ ] T007 [P] Create `tests/contract/test_cli_version.rs` ensuring `--version` prints semantic version + short commit hash pattern.
- [ ] T008 [P] Create `tests/contract/test_cli_args_validation.rs` covering invalid path, path not directory, quiet+verbose conflict, duplicate entitlements deduped.
- [ ] T009 [P] Create `tests/contract/test_json_schema.rs` validating JSON output structure matches `contracts/output-json-schema.json` (use fixture sample; placeholder now).
- [ ] T010 [P] Create `tests/contract/test_human_output_format.rs` asserting formatting rules (path line + entitlement lines + blank separator + summary block format).

### Data Model & Entity Tests
- [ ] T011 [P] Create `tests/unit/test_models.rs` verifying struct field presence & invariants (entitlement_count equals map key count, interrupted only if true, duration non-negative).

### Integration / User Story Tests
(Mirror quickstart scenarios; rely on temporary fixture directories & mock binaries placeholder until implementation.)
- [ ] T012 [P] `tests/integration/test_default_scan.rs` expects default directories enumeration call scaffold (use temp override env to avoid real filesystem for now).
- [ ] T013 [P] `tests/integration/test_path_filters.rs` path filtering behavior.
- [ ] T014 [P] `tests/integration/test_entitlement_filters.rs` entitlement filtering behavior.
- [ ] T015 [P] `tests/integration/test_combined_filters.rs` combined path + entitlement filter.
- [ ] T016 [P] `tests/integration/test_json_output.rs` JSON mode shape & ordering placeholder (assert deterministic sorting once implemented).
- [ ] T017 [P] `tests/integration/test_no_matches.rs` zero-results scenario output & exit code.
- [ ] T018 [P] `tests/integration/test_interrupt_handling.rs` simulate interrupt (design: helper to inject signal flag) ensures partial results + summary with interrupted flag.
- [ ] T019 [P] `tests/integration/test_unreadable_files.rs` ensures unreadable files counted & warnings suppressed with quiet.

## Phase 3.3: Core Implementation (Only after tests exist & fail)
- [ ] T020 Implement models in `src/models/mod.rs` (BinaryRecord, EntitlementSet, ScanResult, ScanSummary) with basic constructors.
- [ ] T021 Implement CLI argument parsing in `src/cli/mod.rs` using clap (define flags, validation: quiet vs verbose conflict, path existence checks).
- [ ] T022 Implement entitlement extraction placeholder in `src/entitlements/mod.rs` that will shell out to `codesign` (stub returns TODO for tests initially).
- [ ] T023 Implement filesystem scanning in `src/scan/mod.rs` collecting candidate binaries & classifying file types (no entitlements yet) deterministic path ordering.
- [ ] T024 Integrate scanning + entitlement extraction + filtering pipeline in `src/scan/mod.rs` returning intermediate data structures.
- [ ] T025 Implement filtering by entitlement keys (case-sensitive exact match) integrated into pipeline.
- [ ] T026 Implement human-readable output formatting in `src/output/mod.rs` following contract (including summary generation).
- [ ] T027 Implement JSON output serialization in `src/output/mod.rs` conforming to schema (results + summary).
- [ ] T028 Implement interrupt handling logic (register ctrl-c handler sets atomic flag; pipeline checks & breaks; output partial results) in scanning pipeline.
- [ ] T029 Implement summary statistics computation and integration (scanned, matched, skipped_unreadable, duration_ms, interrupted).
- [ ] T030 Implement version metadata embedding (commit hash + semantic version) accessed by CLI version flag.
- [ ] T031 Wire main entry (`src/main.rs`) orchestrating CLI parse → scan pipeline → output mode selection → exit code semantics.

## Phase 3.4: Integration & Hardening
- [ ] T032 Add unreadable file handling logic with concise stderr warnings (respect quiet flag).
- [ ] T033 Add verbose level handling (v1: summary timings; v2: per-directory start logs) in scanning & output modules.
- [ ] T034 Optimize codesign invocation (batch concurrency with bounded worker pool) measuring performance vs target.
- [ ] T035 Add deterministic ordering enforcement (final sort before output) if streaming introduces disorder (validate against tests).
- [ ] T036 Add duration measurement instrumentation (high-resolution timing) centralized utility.
- [ ] T037 Add environmental override for default directories (for integration tests isolation) via e.g. LISTENT_DEFAULT_DIRS env var.
- [ ] T038 Implement fallback behavior when codesign returns no entitlements (treat as empty set; still counted as scanned).

## Phase 3.5: Performance & Validation
- [ ] T039 [P] Create performance benchmark harness `tests/perf/bench_default_dirs.rs` (skip by default unless env PERF=1).
- [ ] T040 Record baseline performance numbers & document in `research.md` appendix (update with actual timings) (sequential after harness).
- [ ] T041 [P] Add snapshot tests for representative JSON output `tests/snapshots/json/*.snap` (ensure stable ordering & schema).

## Phase 3.6: Packaging & Distribution
- [ ] T042 Add Homebrew formula template in `packaging/homebrew/listent.rb` (placeholder SHA + install block).
- [ ] T043 Add release build script `scripts/release.sh` (build universal or dual binaries, produce checksums, update formula).
- [ ] T044 Update `README.md` with install instructions, examples, performance claim, JSON schema link.
- [ ] T045 Add `LICENSE` file if missing & verify Cargo.toml license field.

## Phase 3.7: Polish & QA
- [ ] T046 [P] Add unit tests for output formatting edge cases `tests/unit/test_output_format.rs` (long values, boolean, numbers).
- [ ] T047 [P] Add unit tests for CLI validation edge cases `tests/unit/test_cli_validation.rs` (quiet+verbose, duplicate entitlements, invalid paths).
- [ ] T048 Review and refactor for simplicity (remove unused abstractions) across src/ modules.
- [ ] T049 Run full test suite with `--nocapture` to ensure clean stderr (no extraneous logs) & update docs if mismatch.
- [ ] T050 Final pass updating `contracts/output-human-format.md` & `output-json-schema.json` if implementation drift occurred.
- [ ] T051 Prepare changelog entry `CHANGELOG.md` initial release (v0.1.0) with features & known limitations.

## Dependencies Overview
- Setup (T001–T005) precedes all tests.
- Contract & integration tests (T006–T019) precede corresponding implementation tasks (T020+).
- Models (T020) needed before scan/filter (T023–T025) & output (T026–T027).
- CLI parsing (T021) precedes main wiring (T031) & validation behavior tests.
- Entitlement extraction stub (T022) required before pipeline assembly (T024+).
- Interrupt handling (T028) before validation test T018 can pass.
- Performance tasks (T039–T041) after core pipeline (T020–T035).
- Packaging (T042–T045) after stable binary behavior.

## Parallel Execution Examples
```
# Example: Run independent contract tests authoring in parallel
T006 T007 T008 T009 T010 T011 T012 T013 T014 T015 T016 T017 T018 T019

# Example: Parallel model & output tests (later phase)
T020 (sequential) then T026 [P] T027 [P] after T024/T025 complete

# Example: Performance & snapshot tasks
T039 [P] can run with T041 [P] after core implementation while T040 waits for benchmark completion
```

## Validation Checklist
- [ ] All contracts have corresponding test tasks (CLI help/version/options, human + JSON)
- [ ] Each entity mapped to model implementation task (BinaryRecord, EntitlementSet, ScanResult, ScanSummary consolidated in T020)
- [ ] All integration scenarios reflected (default scan, path filter, entitlement filter, combined, json, no matches, interrupt, unreadable)
- [ ] Tests precede implementation tasks
- [ ] [P] only used where file independence exists
- [ ] Performance targets covered by benchmark + documentation update
- [ ] Packaging & distribution addressed before release

## Notes
- Avoid implementing logic while creating stubs for tests—ensure red state first.
- Keep each task atomic; commit after completion.
- If schema changes occur, update contracts then regenerate affected tests before implementation adjustments.
