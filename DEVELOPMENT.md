# Development
_JCD was vibe coded by Mark Russinovich, Mario Hewardt with Github Copilot Agent and Claude Sonnet 4._
## Build
1. **Clone and Build**:
   ```bash
   git clone https://github.com/microsoft/sysinternals-listent.git
   cd sysinternals-listent
   cargo build --release
   ```
2. **Build Package**:

   Set the VERSION environment variable to the version of listent.

   ```
   ./makePackages.sh . target/release listent $(VERSION) 0 brew ""
   ```

## Test
The project includes a comprehensive test suite located in the `tests/` directory:

