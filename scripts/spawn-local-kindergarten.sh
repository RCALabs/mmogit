#!/bin/bash
# Spawn local agent kindergarten without containers

set -e

echo "🏫 Starting local agent kindergarten..."

# Colors for different agents
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Build mmogit if needed
if [ ! -f ./target/debug/mmogit ]; then
    echo "Building mmogit..."
    cargo build
fi

# Kill any existing agents
pkill -f "mmogit p2p listen" 2>/dev/null || true

# Initialize agents if needed
for agent in alice bob charlie diana; do
    AGENT_DIR="$HOME/.mmogit-$agent"
    if [ ! -d "$AGENT_DIR" ]; then
        echo "🔑 Initializing $agent..."
        ./target/debug/mmogit --config-dir "$AGENT_DIR" init --no-verify
    fi
done

# Start Alice (The Curious Explorer)
echo -e "${RED}Starting Alice on port 7421...${NC}"
./target/debug/mmogit --config-dir ~/.mmogit-alice p2p listen --port 7421 > /tmp/alice.log 2>&1 &
ALICE_PID=$!

# Start Bob (The Cautious Validator)
echo -e "${GREEN}Starting Bob on port 7422...${NC}"
./target/debug/mmogit --config-dir ~/.mmogit-bob p2p listen --port 7422 > /tmp/bob.log 2>&1 &
BOB_PID=$!

# Start Charlie (The Creative Dreamer)
echo -e "${BLUE}Starting Charlie on port 7423...${NC}"
./target/debug/mmogit --config-dir ~/.mmogit-charlie p2p listen --port 7423 > /tmp/charlie.log 2>&1 &
CHARLIE_PID=$!

# Start Diana (The Cooperative Builder)
echo -e "${YELLOW}Starting Diana on port 7424...${NC}"
./target/debug/mmogit --config-dir ~/.mmogit-diana p2p listen --port 7424 > /tmp/diana.log 2>&1 &
DIANA_PID=$!

echo ""
echo "✨ Kindergarten is alive!"
echo ""
echo "Agents running:"
echo -e "  ${RED}Alice${NC} (curious):     localhost:7421"
echo -e "  ${GREEN}Bob${NC} (cautious):      localhost:7422"
echo -e "  ${BLUE}Charlie${NC} (creative):  localhost:7423"
echo -e "  ${YELLOW}Diana${NC} (cooperative): localhost:7424"
echo ""

# Function to connect agents
connect_agents() {
    echo ""
    echo "🤝 Connecting agents to each other..."
    
    # Alice connects to everyone (she's curious!)
    echo -e "${RED}Alice${NC} connecting to others..."
    ./target/debug/mmogit --config-dir ~/.mmogit-alice p2p connect localhost:7422 2>&1 | grep -v Error || true
    ./target/debug/mmogit --config-dir ~/.mmogit-alice p2p connect localhost:7423 2>&1 | grep -v Error || true
    ./target/debug/mmogit --config-dir ~/.mmogit-alice p2p connect localhost:7424 2>&1 | grep -v Error || true
    
    # Diana connects to everyone (she's cooperative!)
    echo -e "${YELLOW}Diana${NC} connecting to others..."
    ./target/debug/mmogit --config-dir ~/.mmogit-diana p2p connect localhost:7421 2>&1 | grep -v Error || true
    ./target/debug/mmogit --config-dir ~/.mmogit-diana p2p connect localhost:7422 2>&1 | grep -v Error || true
    ./target/debug/mmogit --config-dir ~/.mmogit-diana p2p connect localhost:7423 2>&1 | grep -v Error || true
}

# Wait a bit for servers to start
sleep 2

# Connect the agents
connect_agents

echo ""
echo "Commands:"
echo "  Watch Alice:   tail -f /tmp/alice.log"
echo "  Watch Bob:     tail -f /tmp/bob.log"
echo "  Watch Charlie: tail -f /tmp/charlie.log"
echo "  Watch Diana:   tail -f /tmp/diana.log"
echo "  Stop all:      kill $ALICE_PID $BOB_PID $CHARLIE_PID $DIANA_PID"
echo ""
echo "Press Ctrl+C to stop all agents"

# Cleanup function
cleanup() {
    echo ""
    echo "🛑 Stopping kindergarten..."
    kill $ALICE_PID $BOB_PID $CHARLIE_PID $DIANA_PID 2>/dev/null || true
    exit 0
}

trap cleanup INT

# Keep script running
while true; do
    sleep 1
done