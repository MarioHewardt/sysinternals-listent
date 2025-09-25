# Assets Directory

This directory contains media assets for the README and documentation.

## Files

- `listent-demo.gif` - Animated demonstration of listent usage (placeholder - to be created)

## Creating Demo GIF

To create the demo GIF:

1. Use terminal recording tool like `asciinema` or `ttygif`
2. Record a typical listent session showing:
   - Basic scanning with progress
   - Entitlement filtering
   - Monitor mode demonstration
   - Output formats (human + JSON)
3. Convert to optimized GIF
4. Update README.md with actual GIF path

Example recording session:
```bash
# Show basic scan
./target/release/listent /Applications --quiet

# Show filtering
./target/release/listent -e "*network*" /usr/bin

# Show JSON output  
./target/release/listent -e "*security*" /usr/bin --json

# Show monitor mode (brief)
./target/release/listent --monitor --interval 2.0
# (Ctrl+C after a few seconds)
```