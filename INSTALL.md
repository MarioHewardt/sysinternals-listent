# Installation Guide

This document provides detailed installation instructions for `listent` on macOS systems.

## System Requirements

### Minimum Requirements
- **Operating System**: macOS 10.15 (Catalina) or later
- **Architecture**: x86_64 (Intel) or ARM64 (Apple Silicon)
- **Disk Space**: 10 MB for binary, additional space for build dependencies if compiling from source
- **Memory**: 50 MB minimum during operation

### Runtime Dependencies
- **codesign**: macOS code signing utility (included with system)
- **launchd**: System service manager for daemon mode (included with system)

### Administrative Privileges
- **Static Scanning**: No special privileges required for user-accessible directories
- **System Directories**: May require `sudo` for `/System`, `/private`, etc.
- **Daemon Installation**: Requires administrator privileges for LaunchD service registration

## Installation Methods

### Method 1: Homebrew (Recommended)

Homebrew provides the easiest installation and update experience.

#### Install Homebrew
If you don't have Homebrew installed:
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Install listent
```bash
# Add the tap (one-time setup)
brew tap mariohewardt/tools

# Install listent
brew install listent

# Verify installation
listent --version
```

#### Update listent
```bash
# Update to latest version
brew update && brew upgrade listent

# Check for available updates
brew outdated listent
```

#### Uninstall listent
```bash
# Remove listent
brew uninstall listent

# Optionally remove the tap
brew untap mariohewardt/tools
```

### Method 2: Pre-built Binaries

Download pre-built binaries from GitHub releases.

#### Download and Install
```bash
# Download latest release (replace with actual version)
curl -LO https://github.com/mariohewardt/listent/releases/download/v1.0.0/listent-macos-universal.tar.gz

# Extract binary
tar -xzf listent-macos-universal.tar.gz

# Move to system PATH
sudo mv listent /usr/local/bin/

# Verify installation
listent --version

# Clean up
rm listent-macos-universal.tar.gz
```

#### Available Binary Variants
- `listent-macos-universal.tar.gz` - Universal binary (Intel + Apple Silicon)
- `listent-macos-x86_64.tar.gz` - Intel Macs only
- `listent-macos-aarch64.tar.gz` - Apple Silicon Macs only

### Method 3: Compile from Source

Build listent yourself for the latest features or custom configurations.

#### Prerequisites
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Xcode Command Line Tools (for codesign and system libraries)
xcode-select --install

# Verify Rust installation
rustc --version
cargo --version
```

#### Build and Install
```bash
# Clone repository
git clone https://github.com/mariohewardt/listent.git
cd listent

# Switch to main development branch
git checkout listent

# Build optimized release binary
cargo build --release

# Install to system PATH
sudo cp target/release/listent /usr/local/bin/

# Verify installation
listent --version

# Optional: Clean up build directory
cd .. && rm -rf listent
```

#### Development Build
For development or testing purposes:
```bash
# Clone and enter directory
git clone https://github.com/mariohewardt/listent.git
cd listent

# Build debug version (faster compilation)
cargo build

# Run directly from build directory
./target/debug/listent --help

# Or build and run in one command
cargo run -- --help
```

## Post-Installation Setup

### Verify Installation
```bash
# Check version
listent --version

# Display help to verify all features are working
listent --help

# Test basic functionality
listent --help | head -5
```

### Path Configuration
If `listent` is not found after installation:

```bash
# Check if /usr/local/bin is in PATH
echo $PATH | grep -q "/usr/local/bin" && echo "✓ PATH configured" || echo "✗ PATH needs configuration"

# Add to PATH if needed (choose your shell)
# For bash users
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bash_profile
source ~/.bash_profile

# For zsh users (default on macOS Catalina+)
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# For fish users
fish_add_path /usr/local/bin
```

### Shell Completion (Optional)
Enable tab completion for your shell:

```bash
# Generate completion scripts
listent --generate-completion bash > /usr/local/etc/bash_completion.d/listent
listent --generate-completion zsh > /usr/local/share/zsh/site-functions/_listent
listent --generate-completion fish > ~/.config/fish/completions/listent.fish

# Note: Completion generation is planned for future releases
```

## Daemon Mode Setup

### Installation Requirements
Daemon mode requires administrator privileges for LaunchD integration:

```bash
# Install daemon service
sudo listent install-daemon

# Verify daemon installation
listent daemon-status
```

### Configuration Files
Daemon mode uses configuration files in standard locations:

```bash
# System-wide configuration
sudo mkdir -p /etc/listent
sudo listent install-daemon --config /etc/listent/daemon.toml

# User-specific configuration (if supported)
mkdir -p ~/.config/listent
listent install-daemon --config ~/.config/listent/daemon.toml --user-mode
```

### Daemon Management
```bash
# Start daemon service
sudo listent daemon-start

# Stop daemon service
sudo listent daemon-stop

# Check daemon logs
listent logs --follow

# Uninstall daemon
sudo listent uninstall-daemon
```

## Verification and Testing

### Basic Functionality Test
```bash
# Test static scanning
listent /usr/bin --quiet

# Test entitlement filtering
listent -e "*security*" /usr/bin

# Test JSON output
listent /usr/bin --json --quiet | jq '.summary'
```

### Monitor Mode Test
```bash
# Test monitor mode (run for 5 seconds then Ctrl+C)
timeout 5 listent --monitor --interval 2.0 || true

# Test with entitlement filtering
timeout 5 listent --monitor -e "*network*" || true
```

### Performance Test
```bash
# Benchmark scan performance
time listent /Applications --quiet

# Test with progress indicators
listent /usr/bin
```

## Troubleshooting

### Common Installation Issues

#### Homebrew Installation Fails
```bash
# Update Homebrew first
brew update

# Check for conflicts
brew doctor

# Force reinstall if needed
brew uninstall listent && brew install listent
```

#### Permission Denied Errors
```bash
# For system directory installation
sudo mv listent /usr/local/bin/
sudo chmod +x /usr/local/bin/listent

# For daemon installation
sudo listent install-daemon
```

#### Build from Source Fails
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release

# Check for missing dependencies
xcode-select --install
```

#### Command Not Found
```bash
# Verify binary location
which listent
ls -la /usr/local/bin/listent

# Check PATH configuration
echo $PATH
source ~/.zshrc  # or ~/.bash_profile
```

### Runtime Issues

#### codesign Not Found
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Verify codesign is available
which codesign
codesign --version
```

#### Permission Issues with System Directories
```bash
# Use sudo for system directories
sudo listent /System/Library/Frameworks

# Check file permissions
ls -la /path/to/directory
```

#### Monitor Mode Not Working
```bash
# Check process monitoring permissions
listent --monitor --interval 5.0

# Verify no other monitoring tools are conflicting
ps aux | grep -E "(listent|monitor)"
```

### Performance Issues

#### Slow Scanning Performance
```bash
# Use quiet mode to reduce output overhead
listent /large/directory --quiet

# Apply entitlement filters to reduce processing
listent -e "*security*" /Applications

# Check available disk space and memory
df -h
vm_stat
```

#### High Memory Usage
```bash
# Monitor memory usage during scan
top -pid $(pgrep listent)

# Use smaller scan targets
listent /usr/bin instead of /Applications
```

## Platform-Specific Notes

### Intel Macs (x86_64)
- All installation methods fully supported
- Rosetta not required for native binaries
- Performance optimized for Intel architecture

### Apple Silicon Macs (ARM64)
- Native ARM64 binaries available via Homebrew
- Universal binaries support both architectures
- May run x86_64 binaries via Rosetta if needed

### macOS Version Compatibility
- **macOS 10.15 (Catalina)**: Minimum supported version
- **macOS 11.0 (Big Sur)**: Full feature support
- **macOS 12.0 (Monterey)**: Enhanced daemon mode features
- **macOS 13.0 (Ventura)**: Latest optimizations
- **macOS 14.0 (Sonoma)**: Full compatibility tested

## Uninstallation

### Remove listent Binary
```bash
# If installed via Homebrew
brew uninstall listent
brew untap mariohewardt/tools

# If installed manually
sudo rm -f /usr/local/bin/listent
```

### Remove Daemon Services
```bash
# Stop and uninstall daemon
sudo listent daemon-stop
sudo listent uninstall-daemon

# Remove configuration files
sudo rm -rf /etc/listent
rm -rf ~/.config/listent
```

### Remove Build Dependencies (Source Installation)
```bash
# Remove Rust toolchain (optional)
rustup self uninstall

# Remove Xcode Command Line Tools (use with caution)
# sudo rm -rf /Library/Developer/CommandLineTools
```

### Clean Temporary Files
```bash
# Remove any temporary files
rm -rf ~/.cache/listent
sudo rm -rf /tmp/listent*

# Remove logs (if any)
sudo rm -rf /var/log/listent
```

## Support

### Getting Help
- **Command Help**: `listent --help`
- **Version Information**: `listent --version`
- **GitHub Issues**: [https://github.com/mariohewardt/listent/issues](https://github.com/mariohewardt/listent/issues)
- **Documentation**: [README.md](README.md) and [DEVELOPMENT.md](DEVELOPMENT.md)

### Reporting Installation Issues
When reporting installation issues, please include:
1. macOS version: `sw_vers`
2. Architecture: `uname -m`
3. Installation method used
4. Complete error messages
5. Output of `listent --version` (if partially working)

### Community Resources
- **Homebrew Tap**: [mariohewardt/homebrew-tools](https://github.com/mariohewardt/homebrew-tools)
- **Security Documentation**: [SECURITY.md](SECURITY.md)
- **Threat Model**: [docs/threat-model.md](docs/threat-model.md)