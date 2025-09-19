# Project Constitution: Rust macOS Command Line Tool

Version: 0.1.0
Status: Draft (Minimum Viable Constitution)
Applies To: Entire repository until superseded

## 1. Purpose
Provide a small, fast, secure, and maintainable Rust-based command line tool targeting macOS (Apple Silicon + Intel) that can be built, tested, and released reproducibly with minimal friction.

## 2. Scope
In scope:
- Single binary Rust CLI (no workspace unless growth requires it)
- Core logic in `src/` with clear separation between CLI parsing and domain logic
- Basic semantic versioning (MAJOR.MINOR.PATCH)
- Unit + doc tests, optional integration tests directory
- Minimal dependencies (prefer std + widely used crates)

Out of scope (until justified):
- Multiple binaries / crates
- Persistent databases
- Network services / daemons
- Plugin architectures
- Complex async unless required by a feature

## 3. Non-Negotiable Principles
1. Simplicity first: Reject abstractions until duplication is proven harmful
2. Reproducibility: Every developer must be able to build with one command
3. Determinism: CI and local builds must produce identical artifacts (same features)
4. Security posture: Deny unsafe code unless explicitly justified and reviewed
5. Observability: All user-facing failures return clear exit codes & stderr messages

## 4. Minimal Functional Requirements
- CLI MUST provide `--help` and `--version`
- Exit codes MUST follow: 0 success, 1 generic error, >1 domain-specific (document)
- Logging (if needed) MUST default to quiet; verbose enabled via `-v/--verbose`
- Subcommands MAY be added; each MUST have its own help section

## 5. Minimal Non-Functional Requirements
| Area | Requirement |
|------|-------------|
| Performance | Startup < 50ms for no-op command on typical macOS hardware |
| Binary Size | < 5MB uncompressed (initial target) |
| Build Time | Clean build < 20s on Apple Silicon M-series |
| Safety | No `unsafe` blocks without comment justification |
| Linting | `cargo clippy -- -D warnings` passes |
| Formatting | `cargo fmt -- --check` passes |
| Testing | `cargo test` green and < 5s runtime (initial scope) |
| Toolchain | Pinned Rust toolchain in `rust-toolchain.toml` |

## 6. Directory Layout (Mandated)
```
/CONSTITUTION.md       # This file
/Cargo.toml            # Manifest
/rust-toolchain.toml   # Toolchain pin
/src/main.rs           # Entry point (arg parsing only)
/src/lib.rs            # (Optional) Core logic if code grows
/tests/                # Integration tests (optional, add when needed)
/README.md             # Quick start
```
No other top-level directories unless ratified.

## 7. Dependencies Policy
Allowed categories: argument parsing, logging, serialization (if needed), testing utilities.
MUST justify adding any dependency beyond: `clap` (or `argh`), `anyhow` (or custom error), `thiserror` (optional), `tracing` (optional), `serde` (only if serialization required).
Prohibited until justified: build scripts (`build.rs`), proc-macro authoring, heavy frameworks.

## 8. Build & Tooling
Required commands (MUST succeed):
- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Test: `cargo test --all`
- Build: `cargo build --release`
Optional (add later): security audit via `cargo audit`.

## 9. Testing Standards
- Each functional requirement → at least one test (unit or integration)
- Panic-free core logic (avoid panics in library code; use `Result`)
- Use doc tests for trivial examples
- Integration tests go under `tests/` and use only public API

## 10. Error Handling
- Use structured errors (`thiserror` or custom enums) OR `anyhow` for app layer
- User-facing errors printed to stderr
- Provide actionable messages (avoid opaque: "Error occurred")

## 11. Versioning & Release
- Tag releases: `vX.Y.Z`
- Changelog required starting with first public release (keep `CHANGELOG.md` minimal)
- Binary reproducibility: build with `--locked`

## 12. Security & Compliance
- No dynamic code execution
- No network access unless feature expressly added
- Audit new dependencies for maintenance & license (Apache/MIT preferred)
- Forbid `unsafe` globally via `#![forbid(unsafe_code)]` unless removed with justification

## 13. Contribution Rules
A change may be merged only if:
1. Constitution not violated (or violation justified inline in PR)
2. All checks pass (fmt, clippy, test)
3. New code has tests or rationale why not testable
4. Public-facing behavior documented in README or CHANGELOG

## 14. Change Control
To modify this constitution: open PR titled "Amend Constitution: <summary>" including rationale + impact + migration steps. Approval requires at least one reviewer sign-off (if team grows) or explicit self-review note (solo).

## 15. Expansion Triggers (When to Revisit)
Revisit structure if ANY of:
- > 5 subcommands
- > 1000 lines in `main.rs` + `lib.rs`
- Need for async or background tasks
- Need for plugin/extensibility

## 16. Minimal Initial Acceptance Checklist
- [ ] Constitution file exists
- [ ] Toolchain pinned
- [ ] `cargo new` initialized
- [ ] `--help` and `--version` work
- [ ] Formatting/linting/test commands succeed

---
End of Constitution v0.1.0
