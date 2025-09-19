# Quickstart: macOS Entitlement Listing CLI

## Installation (Planned)
```
brew install <tap>/listent   # Placeholder tap path
```
(Will be updated once Homebrew formula exists.)

## Basic Usage
List entitlements for default application directories:
```
listent
```

Filter to specific directories:
```
listent -p /Applications -p ~/Applications
```

Filter to entitlement keys (logical OR across provided keys):
```
listent -e com.apple.security.network.client -e com.apple.security.files.user-selected.read-write
```

Combine path + entitlement filters:
```
listent -p /Applications -e com.apple.security.app-sandbox
```

JSON output:
```
listent --json > entitlements.json
```

Suppress unreadable warnings:
```
listent --quiet
```

Show version/help:
```
listent --version
listent --help
```

Graceful interrupt (Ctrl+C) still prints summary (example):
```
^C
---
Scanned: 452
Matched: 37
Skipped (unreadable): 5
Duration: 1875ms
Interrupted: yes
```

No matches:
```
listent -e some.nonexistent.entitlement
(no matches)
---
Scanned: 210
Matched: 0
Skipped (unreadable): 0
Duration: 800ms
```

## Exit Codes
- 0 Success (even with zero matches)
- 1 Invalid arguments
- 2 Internal error (unexpected failure)

## Next Steps
Refer to `contracts/` for output schemas and CLI option specification.
