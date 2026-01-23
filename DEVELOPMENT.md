# Development
_JCD was vibe coded by Mark Russinovich, Mario Hewardt with Github Copilot Agent and Claude Sonnet 4.5_
## Build
1. **Clone and Build**:
   ```bash
   git clone https://github.com/microsoft/sysinternals-listent.git
   cd sysinternals-listent
   cargo build --release
   ```
2. **Build Package**:

   ```bash
   # Replace 1.0.0 with the actual version from Cargo.toml
   ./makePackages.sh . target/release listent 1.0.0 0 brew ""
   ```

## Test
The project includes a comprehensive test suite located in the `tests/` directory:

