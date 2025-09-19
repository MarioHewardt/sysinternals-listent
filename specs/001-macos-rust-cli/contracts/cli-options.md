# CLI Options Contract

| Flag | Long | Argument | Required | Multiple | Description | Notes |
|------|------|----------|----------|----------|-------------|-------|
| -h | --help | none | No | No | Show help/usage and exit 0 | Terminates before scanning |
| -V | --version | none | No | No | Show version/build info and exit 0 | Terminates before scanning |
| -p | --path | <path> | No | Yes | Restrict scan to one or more directory roots | If omitted, defaults applied |
| -e | --entitlement | <key> | No | Yes | Filter: include only binaries containing at least one specified entitlement key | Exact match only |
| -j | --json | none | No | No | Output JSON format instead of human-readable | Mutually compatible with other flags |
| -q | --quiet | none | No | No | Suppress unreadable file warnings | Warnings still counted in summary |
| -v | --verbose | none | No | Repeatable? TBD | Increase diagnostic verbosity (timing, directory notices) | Mutually exclusive with --quiet (DECIDE) |
|    | --summary | none | No | No | Force printing summary stats (if human output) | May be default in JSON mode |
|    | --no-summary | none | No | No | Suppress summary stats (human mode) | Ignored in JSON mode |
|    | --max-depth | <n> | No | No | Limit recursive descent depth | Optional initial scope (DECIDE include?) |
|    | --timeout | <ms> | No | No | Abort scan after duration emitting partial results | Optional future feature |

## Rules
- help/version exclusive: if present ignore other operational flags.
- --quiet and --verbose cannot both be active (enforce at parse stage) (DECIDE final behavior for multiple -v).
- At least one of default directories or provided paths used (never empty set).
- Entitlement keys treated as raw string tokens (no wildcard matching initial version).

## Validation Errors
- Non-existent path argument → error with exit code >0.
- Path not a directory → error.
- Duplicate entitlement keys allowed (deduplicate internally).

## Open Decisions
- Final inclusion of --max-depth in MVP.
- Verbosity model: single -v vs multi-level.
