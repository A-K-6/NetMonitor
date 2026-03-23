#!/bin/bash
set -e

# Isolated Accuracy Suite for NetMonitor
# Requires sudo/capabilities

if [ "$EUID" -ne 0 ]; then
  echo "Error: accuracy verification requires root (ip netns)."
  exit 1
fi

NS_NAME="netmonitor-test-ns"
VETH_HOST="veth-host"
VETH_NS="veth-ns"
ADDR_HOST="10.200.0.1/24"
ADDR_NS="10.200.0.2/24"
TEST_PORT=9999
DATA_MB=1

echo "--- 1. Setting up Isolated Network Namespace ---"
# Clean up if previously interrupted
ip netns delete "$NS_NAME" 2>/dev/null || true
ip link delete "$VETH_HOST" 2>/dev/null || true

ip netns add "$NS_NAME"
ip link add "$VETH_HOST" type veth peer name "$VETH_NS"
ip link set "$VETH_NS" netns "$NS_NAME"

ip addr add "$ADDR_HOST" dev "$VETH_HOST"
ip link set "$VETH_HOST" up

ip netns exec "$NS_NAME" ip addr add "$ADDR_NS" dev "$VETH_NS"
ip netns exec "$NS_NAME" ip link set "$VETH_NS" up
ip netns exec "$NS_NAME" ip link set lo up

echo "--- 2. Starting NetMonitor Headless ---"
# We need to run netmonitor in a way it can see the traffic.
# Since it monitors all PIDs, we'll start a simple sender in the namespace.

# Start a listener on the host
nc -l -p "$TEST_PORT" > /dev/null &
NC_PID=$!

# Run netmonitor in headless mode for 5 seconds
./target/release/netmonitor --headless --verify-accuracy &
NET_PID=$!

# Give it a second to start
sleep 1

echo "--- 3. Generating Traffic ($DATA_MB MB) ---"
# Send data from NS to host with a 5-second timeout and "quit on EOF" flag
ip netns exec "$NS_NAME" head -c "${DATA_MB}M" /dev/urandom | nc -w 5 10.200.0.1 "$TEST_PORT" || echo "Traffic generation ended (timeout or closed)."

# Give NetMonitor a moment to process the final packets
sleep 2

# Kill processes
kill $NC_PID || true
# Netmonitor should ideally exit by itself after verify_accuracy, or we signal it
kill -SIGINT $NET_PID || true

echo "--- 4. Cleanup ---"
ip link delete "$VETH_HOST"
ip netns delete "$NS_NAME"

echo "Accuracy Verification Script Finished."
