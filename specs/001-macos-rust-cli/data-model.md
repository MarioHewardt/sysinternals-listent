# Data Model: macOS Entitlement Listing CLI

## Purpose
Define conceptual data structures used internally. This is an abstract model (no code / crate specifics).

## Entities

### BinaryRecord
Represents a discovered executable candidate.
- Fields:
  - path: Absolute filesystem path (string)
  - readable: Boolean indicating successful access for entitlement extraction
  - file_type: Enum { MachO, Bundle, Other }
  - skipped_reason: Optional string (present if unreadable or unsupported)

### EntitlementSet
Represents entitlement key-value pairs extracted from a binary.
- Fields:
  - entries: Map<string, string|boolean|number> (values treated as string form for output; JSON retains native types if determinable)
  - source_signed: Boolean (true if code signature present)
  - raw_size_bytes: Integer (optional diagnostic)

### ScanResult
Represents a successful entitlement enumeration.
- Fields:
  - binary_path: string (duplicate of BinaryRecord.path for flattening)
  - entitlements: EntitlementSet.entries (map)
  - entitlement_count: Integer

### ScanSummary
Aggregated metrics for a completed (or interrupted) scan.
- Fields:
  - scanned: Integer (all candidate binaries examined, including unreadable)
  - matched: Integer (results emitted respecting filters)
  - skipped_unreadable: Integer
  - duration_ms: Integer
  - interrupted: Optional boolean (true only if user interrupted)
  - start_timestamp: ISO-8601 string
  - end_timestamp: ISO-8601 string

## Relationships
- BinaryRecord → may produce either ScanResult (if readable + entitlements processed) or increment skipped counts.
- ScanResult uses data from BinaryRecord and EntitlementSet.
- ScanSummary aggregates across all processed BinaryRecords.

## Invariants
- If readable = false then no ScanResult with that path.
- entitlement_count = size(entitlements keys).
- interrupted present only if true.
- duration_ms = end - start ≥ 0.

## Validation Rules
- path must be absolute.
- entitlements keys non-empty strings.
- skipped_reason only present if readable = false.

## Open Decisions Impacting Model
- Whether to include non-executable ignored count (deferred; would add field `ignored_non_executable`).
- Whether to include raw entitlement size (optional; may be omitted initially).

## JSON Representation (Proposed)
Top-level object:
```
{
  "results": [
    { "path": "...", "entitlements": { "key": value, ... }, "entitlement_count": N },
    ...
  ],
  "summary": {
      "scanned": N,
      "matched": M,
      "skipped_unreadable": S,
      "duration_ms": T,
      "interrupted": true
  }
}
```
`interrupted` omitted if false.

## Notes
- Deterministic ordering achieved by sorting results by path before final output (if not streamed) or inserting into ordered structure.
- Streaming approach requires buffering paths to ensure ordering; alternative is to emit unsorted streaming JSON but that would conflict with deterministic requirement—final choice pending performance measurement.
