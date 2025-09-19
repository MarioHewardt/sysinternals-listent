# Feature Specification: macOS Entitlement Listing CLI

**Feature Branch**: `001-macos-rust-cli`  
**Created**: 2025-09-18  
**Status**: Draft  
**Input**: User description: "macOS Rust CLI tool to list entitlements of binaries; filtering by entitlement keys and optional directory paths; fast; brew installable"

## Execution Flow (main)
```
1. Parse user description from Input
	→ If empty: ERROR "No feature description provided"
2. Extract key concepts from description
	→ Identify: target platform (macOS), purpose (list entitlements), scope (all binaries vs filtered paths), filters (entitlement keys, directory roots), non-functional (performance, Homebrew distribution), standard CLI needs (help, version)
3. For each unclear aspect:
	→ Mark with unknown
4. Fill User Scenarios & Testing section
	→ If no clear user flow: ERROR "Cannot determine user scenarios"
5. Generate Functional Requirements
	→ Each requirement must be testable
	→ Mark ambiguous requirements
6. Identify Key Entities (data involved in representing entitlement results)
7. Run Review Checklist
	→ If implementation details found: ERROR "Remove tech details"
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no specific Rust crates, filesystem traversal algorithms, caching strategies, parallelism techniques, code structure)
- 👥 Written for business / product stakeholders and high-level technical reviewers

### Section Requirements
- **Mandatory sections**: Completed
- **Optional sections**: Included only if relevant
- Non-applicable sections removed

### For AI Generation
1. **Ambiguities marked** where information is missing
2. Assumptions minimized; questions surfaced
3. Requirements framed as testable behavior
4. Common underspecified areas reviewed (performance targets, error handling, security)

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a macOS power user / security engineer / developer, I want a fast command line tool that enumerates code signing entitlements of executable binaries on my machine so that I can audit capabilities, detect anomalous privilege usage, and filter results to the entitlements and locations I care about.

### Acceptance Scenarios
1. **Given** the user runs the tool with no arguments, **When** it executes, **Then** it enumerates entitlements for binaries within the default directory set (`/Applications`, `/System/Applications`, `/System/Library/CoreServices`, `~/Applications`) and outputs them in a readable list format including binary path and entitlement key-value pairs. 
2. **Given** the user provides one or more directory paths via a path-filter option, **When** the tool runs, **Then** only binaries under those directories are processed and listed.
3. **Given** the user provides one or more entitlement keys via an entitlement-filter option, **When** the tool runs, **Then** only binaries containing at least one of the specified entitlements appear in the output.
4. **Given** the user specifies both directory filters and entitlement filters, **When** the tool runs, **Then** results are limited to binaries under the provided directories AND matching at least one entitlement filter.
5. **Given** the user requests version info via a version switch, **When** the tool runs, **Then** it prints version metadata and exits without scanning.
6. **Given** the user invokes help, **When** the tool runs, **Then** it displays usage, option descriptions, examples, and exits without scanning.
7. **Given** the user includes an option for json output (if provided), **When** the tool runs, **Then** output is produced in that structured format. 
8. **Given** a directory contains binaries the user cannot read, **When** the tool encounters them, **Then** it skips them while reporting (or optionally suppressing) access errors without aborting the whole run. The error reporting should be short. 
9. **Given** the tool encounters a file that is not a Mach-O executable or bundle, **When** scanning, **Then** it ignores it without error
10. **Given** the user specifies zero matching entitlement results (filters too restrictive), **When** the scan completes, **Then** the tool exits successfully with an empty result set and a clear indication no matches were found (not an error).

### Edge Cases
- Binaries with malformed or missing entitlements plist segments.
- Universal (fat) binaries containing multiple architectures—entitlements should not duplicate in output.
- Code signed ad-hoc or unsigned binaries lacking entitlements.
- Skip restricted system paths requiring elevated permissions. 
- Output truncation risk in very wide entitlement sets (multiple keys) — need defined formatting or wrapping rules. 
- Simultaneous path and entitlement filters resulting in zero matches.
- User cancels execution (Ctrl+C) mid-scan: partial results MUST be shown followed by summary with Interrupted flag.

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: Tool MUST enumerate entitlements of binaries within its scan scope.
- **FR-002**: Tool MUST allow specifying one or more directory roots to restrict scan scope via a path filter option.
- **FR-003**: Tool MUST, when no custom paths are provided, use a documented default set of directories. Default is application directories
- **FR-004**: Tool MUST allow specifying one or more entitlement keys (exact match) to filter output to binaries containing at least one of those keys.
- **FR-005**: Tool MUST provide a help command/switch displaying usage, options, and examples.
- **FR-006**: Tool MUST provide a version command/switch showing version and build metadata.
- **FR-007**: Tool MUST exit with non-zero status only on unrecoverable errors (e.g., invalid arguments, internal failure), not on zero matches.
- **FR-008**: Tool MUST handle unreadable files gracefully without terminating the entire scan.
- **FR-009**: Tool MUST ignore non-executable files silently
- **FR-011**: Tool MUST provide an option to limit output format to human-readable table/list.
- **FR-012**: Tool SHOULD provide an optional structured output mode (JSON) for downstream tooling (top-level object with `results` array and `summary` object containing `scanned`, `matched`, `skipped_unreadable`, `duration_ms`, optional `interrupted`).
- **FR-013**: Tool MUST document each CLI flag in help output.
- **FR-014**: Tool MUST allow combining entitlement filters and path filters.
- **FR-015**: Tool MUST clearly indicate when no results match.
- **FR-018**: Tool MUST be distributable via Homebrew (tap or core) with install instructions.
- **FR-019**: Tool MUST, on baseline hardware, complete default directory scan within performance target (<8s Apple Silicon M1, <12s Intel i5 quad-core) assuming typical application set.
- **FR-021**: Tool SHOULD provide optional summary statistics (scanned, matched, skipped_unreadable, duration_ms; interrupted shown only if true). 
- **FR-022**: Tool MUST handle user interrupt (Ctrl+C) gracefully, emitting partial results accumulated so far and a summary with `interrupted` flag.
### Key Entities *(include if feature involves data)*
- **Binary**: Represents an executable candidate discovered under scan scope; attributes: path, accessibility status, entitlement presence.
- **Entitlement Set**: Collection of entitlement key-value pairs associated with a binary's code signature.
- **Scan Result**: Aggregation unit containing Binary reference plus its Entitlement Set filtered according to user-specified entitlement keys (if any).
- **Scan Summary**: Optional aggregate counts (binaries scanned, binaries skipped unreadable, binaries matched filters, total entitlement keys output). 
---

## Review & Acceptance Checklist
*GATE: To be used before moving to implementation planning*

### Content Quality
- [ ] No implementation details (keeps out specific libraries, traversal algorithms)
- [ ] Focus remains on user value (auditing entitlements, filtering, speed)
- [ ] Written so a security/product stakeholder can understand without code knowledge
- [ ] Mandatory sections present (User Scenarios, Requirements, Entities)

### Functional Acceptance (Must Pass)
- [ ] Running with no args scans only default application directories (documented in help)
- [ ] Path filter option restricts scope to only specified directories
- [ ] Entitlement filter option returns only binaries containing at least one specified key
- [ ] Combining path + entitlement filters applies logical AND correctly
- [ ] Non-executable files are ignored without noisy output
- [ ] Unreadable binaries are skipped with concise single-line notices (or suppressed in quiet mode if implemented)
- [ ] Help flag prints usage, options, examples and exits 0 without scanning
- [ ] Version flag prints version/build info and exits 0 without scanning
- [ ] Zero matches scenario exits 0 and states clearly that no matches were found
- [ ] JSON output flag (when used) produces valid JSON with deterministic key ordering (if ordering requirement is later defined)
- [ ] Exit codes: 0 = success (even zero matches), >0 = invalid arguments/internal failure
- [ ] Ctrl+C interrupt yields graceful termination behavior (partial results policy decided below)

### Output & Reporting
- [ ] Human-readable mode displays each binary path plus entitlement key-value pairs
- [ ] Summary statistics include: scanned, matched, skipped_unreadable, duration_ms (and interrupted flag when applicable)
- [ ] JSON mode schema documented (object with `results` array + `summary` object) and matches contracts

### Performance & Non-Functional
- [ ] Meets baseline performance target (<8s Apple Silicon M1, <12s Intel i5) for default directories
- [ ] Tool behavior remains responsive (no long pauses without output when verbose/progress mode on)
- [ ] Handles large directories without crashing or memory exhaustion assumptions

### Distribution / Packaging
- [ ] Homebrew formula instructions prepared (tap or core submission path)
- [ ] Version flag aligns with semantic versioning strategy
- [ ] README documents install, usage examples, JSON mode, exit codes

### Completeness & Clarity
- [ ] All functional requirements are testable as written
- [ ] Ambiguous language (e.g., "optionally", "fast") replaced with measurable criteria OR tracked in Open Decisions
- [ ] Scope boundaries clear (only entitlement enumeration; no privilege escalation, no signing modifications)
- [ ] Assumptions listed (e.g., user has read permissions for most application binaries)

### Removed: Open Decisions
All prior open decisions resolved in `research.md`; checklist items migrated to concrete acceptance criteria above.

### Final Gate
- [ ] All Open Decisions resolved
- [ ] Checklist re-reviewed; no unchecked critical items
- [ ] Ready for task breakdown / implementation planning

---

## Execution Status
*Updated by main() during processing*

- [ ] User description parsed
- [ ] Key concepts extracted
- [ ] Ambiguities marked
- [ ] User scenarios defined
- [ ] Requirements generated
- [ ] Entities identified
- [ ] Review checklist passed

---

