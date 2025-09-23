#!/bin/bash
# Quick test script to verify CTRL-C works in monitor mode
echo "Starting monitor mode test..."
echo "This should start monitoring and show the CTRL-C instructions."
echo "The process should exit cleanly when interrupted."
echo ""

# Start monitor mode for a brief test
./target/release/listent --monitor --interval 10.0 &
MONITOR_PID=$!

# Give it a moment to start
sleep 2

# Send SIGINT (same as CTRL-C)
echo "Sending SIGINT to test CTRL-C handling..."
kill -INT $MONITOR_PID

# Wait for graceful shutdown
wait $MONITOR_PID
echo "Monitor process exited with status: $?"
echo "Test complete!"