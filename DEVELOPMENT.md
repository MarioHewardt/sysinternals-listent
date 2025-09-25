# Development Guide

This document provides detailed instructions for developing, building, testing, and contributing to `listent`.

## Prerequisites

### System Requirements
- **macOS**: 10.15+ (Catalina or later)
- **Rust**: 1.70+ (install via [rustup](https://rustup.rs/))
- **Xcode Command Line Tools**: For native compilation and `codesign` utility
- **Git**: For version control and contribution workflow

### Optional Tools
- **cargo-watch**: For continuous compilation during development
- **cargo-audit**: For security vulnerability scanning
- **cargo-deny**: For dependency license and security checking

```bash
# Install optional development tools
cargo install cargo-watch cargo-audit cargo-deny
```

## Getting Started

### Clone and Setup
```bash
# Clone the repository
git clone https://github.com/mariohewardt/listent
cd listent

# Create development branch
git checkout -b listent

# Build debug version
cargo build

# Run tests to verify setup
cargo test
```

### Development Workflow
```bash
# Start development server with auto-rebuild
cargo watch -x build

# Run tests continuously
cargo watch -x test

# Run specific test suites
cargo test --test simple_functional
cargo test --test integration  
cargo test unit::
```

## Building

### Debug Build (Development)
```bash
# Standard debug build
cargo build

# Debug build with all features
cargo build --all-features

# Check code without building
cargo check
```

### Release Build (Production)
```bash
# Optimized release build
cargo build --release

# Release build with all features
cargo build --release --all-features

# The binary will be at ./target/release/listent
```

### Cross-Compilation
```bash
# Build for Intel Macs (x86_64)
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon (ARM64)
cargo build --release --target aarch64-apple-darwin

# Build universal binary (requires additional setup)
lipo -create -output listent-universal \
  target/x86_64-apple-darwin/release/listent \
  target/aarch64-apple-darwin/release/listent
```

## Testing

### Test Categories

#### Unit Tests
Located in individual module files (`#[cfg(test)]` blocks):
```bash
# Run all unit tests
cargo test unit::

# Run specific module tests
cargo test models::tests::
cargo test entitlements::tests::
```

#### Integration Tests
Located in `tests/integration/`:
```bash
# Run all integration tests
cargo test --test integration

# Run specific integration test files
cargo test --test test_basic_monitoring
cargo test --test test_entitlement_filters
```

#### Contract Tests
Located in `tests/contract/` - validate CLI behavior:
```bash
# Run all contract tests
cargo test --test contract

# Run specific contract tests
cargo test --test test_cli_args_validation
cargo test --test test_json_schema
```

#### Functional Tests
End-to-end tests with real system interaction:
```bash
# Run simple functional tests
cargo test --test simple_functional

# Run comprehensive functional tests
cargo test --test functional_comprehensive

# Run with output for debugging
cargo test --test simple_functional -- --nocapture
```

### Running Test Suites
```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run tests in single-threaded mode (for debugging)
cargo test -- --test-threads=1

# Run only fast tests (excluding slow integration tests)
cargo test --exclude-pattern slow
```

### Test Environment Setup
Some tests require controlled environments:
```bash
# Build release binary for functional tests
cargo build --release

# Create test binaries (done automatically by test framework)
# Tests use Swift compiler to create signed binaries with known entitlements
```

## Code Quality

### Formatting
```bash
# Format all code
cargo fmt --all

# Check formatting without applying
cargo fmt --all -- --check
```

### Linting
```bash
# Run clippy for all targets
cargo clippy --all-targets -- -D warnings

# Fix clippy suggestions automatically
cargo clippy --fix --allow-staged --all-targets

# Run clippy for specific features
cargo clippy --all-features --all-targets -- -D warnings
```

### Security Auditing
```bash
# Check for known vulnerabilities
cargo audit

# Check dependency licenses and security policies
cargo deny check
```

### Performance Profiling
```bash
# Build with debug symbols for profiling
cargo build --release --debug

# Profile with Instruments (macOS)
xcrun xctrace record --template "Time Profiler" --launch -- ./target/release/listent /Applications

# Profile with Flamegraph
cargo install flamegraph
sudo flamegraph ./target/release/listent /Applications
```

## Project Structure

### Source Code Layout
```
src/
├── main.rs                    # Application entry point and CLI coordination
├── constants.rs               # Application-wide constants
├── cli/
│   └── mod.rs                # Command-line argument parsing with clap
├── models/
│   └── mod.rs                # Data structures and type definitions
├── scan/
│   └── mod.rs                # Directory traversal and binary discovery
├── entitlements/
│   ├── mod.rs                # Entitlement extraction coordination  
│   ├── native.rs             # Native codesign integration
│   └── pattern_matcher.rs    # Glob pattern matching for filtering
├── output/
│   ├── mod.rs                # Output formatting coordination
│   └── progress.rs           # Progress indicators and status updates
├── monitor/
│   ├── mod.rs                # Process monitoring coordination
│   ├── polling.rs            # Polling loop and process detection
│   ├── process_tracker.rs    # Process state management
│   └── unified_logging.rs    # macOS ULS logging integration
└── daemon/
    ├── mod.rs                # Daemon coordination and lifecycle
    ├── config.rs             # Configuration file management
    ├── ipc.rs                # Inter-process communication
    ├── launchd.rs            # LaunchD plist generation and management
    └── logging.rs            # Enhanced daemon logging
```

### Test Structure
```
tests/
├── simple_functional.rs      # Fast end-to-end tests
├── functional_*.rs           # Comprehensive functional test suites
├── test_*.rs                 # Individual component tests
├── contract/
│   └── test_*.rs            # CLI contract and behavior tests
├── integration/
│   └── test_*.rs            # Integration tests with system components
├── unit/
│   └── test_*.rs            # Unit tests for individual modules
└── helpers/
    ├── mod.rs               # Test utilities and shared code
    └── reliable_runner.rs   # Test execution framework
```

### Configuration Files
```
├── Cargo.toml               # Rust package configuration and dependencies
├── Cargo.lock               # Dependency version lock file
├── rustfmt.toml             # Code formatting configuration
├── .gitignore               # Git ignore patterns
├── build.rs                 # Build script for compile-time configuration
└── specs/                   # Feature specifications and documentation
    ├── 001-macos-rust-cli/
    ├── 002-add-monitor-switch/
    └── 003-add-launchd-daemon/
```

## Contributing

### Development Process
1. **Create Feature Branch**: Always work on the `listent` branch or feature branches
2. **Follow Conventions**: Use existing code patterns and architectural principles
3. **Write Tests**: Add tests for new functionality (TDD preferred)
4. **Update Documentation**: Keep README.md and code comments current
5. **Check Quality**: Ensure `cargo test`, `cargo clippy`, and `cargo fmt` pass

### Constitutional Principles
Before adding complexity, check `CONSTITUTION.md` for "Expansion Triggers":
- Single binary CLI tool targeting macOS
- Minimal dependencies, prefer std library
- No unsafe code without justification
- Test-driven development with comprehensive coverage
- Clear error handling with structured messages

### Coding Standards
```rust
// Use Result types for fallible operations
pub fn extract_entitlements(path: &Path) -> Result<Vec<String>, ExtractError> {
    // Implementation
}

// Custom error types with context
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// Document public APIs
/// Extracts code signing entitlements from a binary file.
/// 
/// This function uses the macOS `codesign` utility to extract entitlements
/// from the specified binary file. Returns an empty vector if the file has
/// no entitlements or is not code-signed.
///
/// # Arguments
/// * `path` - Path to the binary file to analyze
///
/// # Returns
/// * `Ok(Vec<String>)` - List of entitlement keys
/// * `Err(ExtractError)` - Error if file cannot be processed
pub fn extract_entitlements(path: &Path) -> Result<Vec<String>, ExtractError> {
    // Implementation
}
```

### Testing Guidelines
1. **Unit Tests**: Test individual functions and modules in isolation
2. **Integration Tests**: Test component interactions and system integration
3. **Contract Tests**: Validate CLI behavior and output formats
4. **Functional Tests**: End-to-end testing with real system interaction
5. **Performance Tests**: Verify performance characteristics under load

### Pull Request Process
1. Ensure all tests pass: `cargo test`
2. Verify code quality: `cargo clippy --all-targets -- -D warnings`
3. Format code: `cargo fmt --all`
4. Update documentation if needed
5. Create descriptive commit messages
6. Submit pull request with clear description

## Debugging

### Logging and Tracing
```bash
# Enable debug logging
RUST_LOG=debug ./target/debug/listent /Applications

# Enable specific module logging
RUST_LOG=listent::monitor=debug ./target/debug/listent --monitor

# Use backtrace for panic debugging
RUST_BACKTRACE=1 ./target/debug/listent /Applications
```

### Development Tools
```bash
# Use cargo-expand to see macro expansions
cargo install cargo-expand
cargo expand --lib entitlements

# Use cargo-tree to inspect dependencies
cargo tree

# Use cargo-outdated to check for dependency updates
cargo install cargo-outdated
cargo outdated
```

### Performance Analysis
```bash
# Create flamegraph for performance analysis
sudo flamegraph ./target/release/listent /Applications

# Use time for basic performance measurement
time ./target/release/listent /Applications

# Memory usage analysis with Activity Monitor or instruments
```

## Release Process

### Version Management
```bash
# Update version in Cargo.toml
# Update CHANGELOG.md with release notes
# Create git tag for version
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

### Pre-Release Checklist
1. All tests pass: `cargo test`
2. Code quality passes: `cargo clippy`
3. Documentation is current
4. Security audit clean: `cargo audit`
5. Performance benchmarks acceptable
6. Manual testing of key scenarios

For detailed release procedures, see [`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md).

## Troubleshooting

### Common Development Issues

#### Compilation Errors
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check for conflicting dependencies
cargo tree --duplicates
```

#### Test Failures
```bash
# Run tests with full output
cargo test -- --nocapture

# Run specific failing test
cargo test test_name -- --exact --nocapture

# Run tests in single thread for debugging
cargo test -- --test-threads=1
```

#### Permission Issues
```bash
# Some tests require codesign access
# Ensure Xcode command line tools are installed
xcode-select --install

# For daemon tests, may need to run with sudo
sudo cargo test --test test_daemon_integration
```

### IDE Setup

#### VS Code
Recommended extensions:
- `rust-analyzer`: Language server for Rust
- `CodeLLDB`: Debugger for Rust
- `Better TOML`: TOML file support
- `Error Lens`: Inline error display

#### Vim/Neovim
```vim
" Use rust.vim plugin
" Configure ALE or coc.nvim for language server support
```

#### IntelliJ/CLion
- Install Rust plugin
- Configure Cargo integration
- Set up debugger configuration

## Resources

### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [macOS Security Documentation](https://developer.apple.com/security/)
- [Code Signing Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/)

### Tools and Libraries
- [clap](https://docs.rs/clap/) - Command-line argument parsing
- [serde](https://docs.rs/serde/) - Serialization framework
- [tokio](https://docs.rs/tokio/) - Async runtime for daemon mode
- [anyhow](https://docs.rs/anyhow/) - Error handling
- [thiserror](https://docs.rs/thiserror/) - Error derive macros

### Community
- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Discord](https://discord.gg/rust-lang)
- [macOS Security Research Community](https://github.com/topics/macos-security)