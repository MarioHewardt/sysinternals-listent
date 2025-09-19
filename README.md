# listent

A fast command-line tool to discover and list code signing entitlements for macOS executable binaries.

## Overview

`listent` recursively scans directories to find executable binaries and extracts their code signing entitlements using the `codesign` utility. It's designed for security researchers, developers, and system administrators who need to audit or understand the permissions requested by macOS applications.

## Features

- **Fast scanning**: Efficiently traverses directory trees with smart filtering
- **Entitlement extraction**: Uses macOS `codesign` to extract entitlements from binaries  
- **Flexible filtering**: Filter by paths and specific entitlement keys
- **Multiple output formats**: Human-readable and JSON output
- **Interrupt handling**: Graceful cancellation with Ctrl+C
- **Progress indication**: Shows progress for long-running scans

## Quick Start
```bash
# Build
cargo build --release

# Run help
./target/release/listent --help

# Show version
./target/release/listent --version
```

## Development
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all
```

## Roadmap Triggers
See "Expansion Triggers" in `CONSTITUTION.md` before adding complexity.

## License
MIT OR Apache-2.0
