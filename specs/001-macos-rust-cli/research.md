# Phase 0 Research: macOS Entitlement Listing CLI

## Objective
Resolve outstanding unknowns and establish concrete, testable targets for performance, output schema, and operational behavior.

## Open Questions & Decisions

| ID | Topic | Decision | Rationale | Alternatives Considered | Status |
|----|-------|----------|-----------|--------------------------|--------|
| Q1 | Performance Target | Default app dirs < 8s (Apple Silicon M1 baseline), < 12s (Intel i5 quad-core) | Time-based target maps to user expectation of "fast" UX | Binary count based target (binaries/sec) | DECIDED |
| Q2 | Default Directories | `/Applications`, `/System/Applications`, `/System/Library/CoreServices`, `~/Applications` | Covers primary GUI + system apps | Add `/usr/bin` (too many & mostly system utilities) | DECIDED |
| Q3 | JSON Schema Fields | Top-level: `{ results: [ { path, entitlements, entitlement_count } ], summary: { scanned, matched, skipped_unreadable, duration_ms, interrupted? } }` | Clear separation; minimal duplication | Per-result embedding of stats (redundant) | DECIDED |
| Q4 | Ctrl+C Behavior | Emit partial results + summary with `interrupted: true` | Maximizes utility of already-processed data | Discard all (loses data) | DECIDED |
| Q5 | Summary Statistics | scanned, matched, skipped_unreadable, duration_ms (omit ignored_non_executable initially) | Minimizes noise; can extend later semver-minor | Include non_executable_ignored (adds cognitive load) | DECIDED |
| Q6 | Quiet/Verbose Flags | `--quiet` (suppresses WARN), `-v` repeatable (each -v increases level: v1=info, v2=debug) | Flexible granularity; standard pattern | Single verbose flag only (less control) | DECIDED |
| Q7 | Ordering | Lexicographic by normalized absolute path | Deterministic, testable | Sort by discovery order (non-deterministic) | DECIDED |
| Q8 | Minimum macOS Version | macOS 12+ (Monterey) | Reduces legacy edge cases; aligns with current security baselines | Include macOS 11 (adds testing burden) | DECIDED |
| Q9 | Entitlement Extraction Method | `codesign -d --entitlements -` subprocess parsing (phase 1) | Correctness via OS tooling; faster iteration; revisit if perf fails targets | Direct Mach-O parsing (higher complexity) | DECIDED |
| Q10| JSON Output Aggregation | Collect all, sort, then output final object | Ensures deterministic ordering without complex streaming buffering | Streaming first then sorting (would require buffering paths anyway) | DECIDED |

## Research Topics & Notes

### Entitlement Extraction Approaches
- System Tool (`codesign`): Pros: authoritative, stable. Cons: process spawn overhead. Mitigation: parallelization (bounded). Need benchmark.
- Direct Parsing: Requires parsing CMS signature and plist; higher complexity and maintenance. Defer unless performance insufficient.

### Filesystem Traversal
- Use depth-first or breadth-first scanning; exclude hidden directories? (Not mandated.)
- Avoid following symlinks into cycles → track visited inodes if following symlinks is allowed. Consider not following symlinks first iteration.

### Performance Measurement Plan
1. Count binaries scanned vs elapsed wall time.
2. Provide optional `--perf` flag to emit timing summary (maybe combine with verbose?). (To decide if added to scope.)
3. Benchmark sets: default dirs, single large third-party app bundle, mixed custom path set.

### Output Design (JSON)
Option A: Array of result objects + trailing summary object.
Option B: Object with `results: [...], summary: {...}` (simpler for consumers). → Prefer Option B.
Remaining: finalize summary metrics list.

### Interrupt Handling
- On SIGINT: flush buffered current result (if any) and write summary with `interrupted: true`.
- Ensure partial results structurally valid JSON (close arrays properly).

### Homebrew Distribution
- Use GitHub release artifacts with universal or dual binaries.
- Provide tap formula with `sha256` for each release.
- Version from Cargo.toml; `--version` prints semantic version + commit hash (short) when built with env var.

### Logging / Error Reporting
- Human mode: concise one-line warnings to stderr for unreadable files when not quiet.
- JSON mode: maintain `skipped` count only; do not interleave error lines into JSON stream.

## Preliminary Decisions (Pending Validation)
- JSON top-level: `{ "results": [...], "summary": { ... } }`.
- Summary fields: `scanned`, `matched`, `skipped_unreadable`, `duration_ms`, `interrupted` (boolean only if true).
- Quiet mode: suppress unreadable file warnings entirely; summary still reflects counts.
- Verbose mode: may add per-directory start notices (optional; may defer initial version).

## Risks
| Risk | Impact | Mitigation |
|------|--------|------------|
| codesign subprocess overhead | Slower scans | Batch parallelization, early measurement; fallback to optimization cycle |
| Large directory tree memory | High memory if results buffered | Stream output or incremental flush (JSON structure supports) |
| Permission errors in system paths | Incomplete visibility | Document expected skips; counts surfaced |
| Interrupt mid-write of JSON | Corrupted output | Use buffered writer; ensure signal handler sets flag and main loop handles graceful closure |

## Next Steps
- Benchmark prototype of scanning default directories (after minimal POC) to set concrete performance target.
- Finalize JSON schema and add to contracts in Phase 1.
- Resolve open decisions (Q1, Q3, Q4, Q5, Q6, Q8, Q9, Q10) before Phase 1 completion.

## Exit Criteria
All OPEN statuses transitioned to DECIDED with documented rationale.

Status: COMPLETE
