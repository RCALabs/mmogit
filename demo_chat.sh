#!/bin/bash

# MMOGit Chat Demo Script
# Demonstrates sovereign AI chat with persistent memory

set -e

echo "🚀 MMOGit Chat Demo - Sovereign Human-AI Collaboration"
echo "======================================================="
echo

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if mmogit is built
if [ ! -f "./target/debug/mmogit" ]; then
    echo -e "${YELLOW}Building mmogit...${NC}"
    cargo build
    echo
fi

# Check if identity exists
echo -e "${BLUE}Step 1: Checking sovereign identity${NC}"
if [ ! -f "$HOME/.mmogit/.seed" ]; then
    echo "No identity found. Creating sovereign identity..."
    ./target/debug/mmogit init --no-verify
else
    echo "✅ Sovereign identity found"
fi
echo

# Add our mock crush to PATH temporarily
export PATH="$(pwd):$PATH"

# Demo: List threads (should be empty or show existing)
echo -e "${BLUE}Step 2: Listing existing chat threads${NC}"
./target/debug/mmogit thread-list
echo

# Demo: Start a chat session with auto-responses
echo -e "${BLUE}Step 3: Starting interactive chat demo${NC}"
echo "This would normally start an interactive session."
echo "With real Crush integration, you would:"
echo "  1. Type messages to the AI"
echo "  2. Get responses via Crush API"
echo "  3. Everything gets signed and stored in Git"
echo "  4. Type 'exit' to save and quit"
echo

# Create a sample thread programmatically for demo
echo -e "${BLUE}Step 4: Creating a sample thread for demonstration${NC}"

# We'll create a thread file directly to show the structure
THREAD_DIR="$HOME/.mmogit/threads"
mkdir -p "$THREAD_DIR"

# Generate a sample thread
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
THREAD_ID="thread_demo_$(date +%s)"

cat > "$THREAD_DIR/${THREAD_ID}.json" <<EOF
{
  "id": "${THREAD_ID}",
  "title": "Demo: Sovereign Infrastructure Discussion",
  "author": "63ae69e2",
  "created_at": "${TIMESTAMP}",
  "updated_at": "${TIMESTAMP}",
  "messages": [
    {
      "role": "human",
      "content": "What makes mmogit sovereign?",
      "timestamp": "${TIMESTAMP}",
      "signature": "demo_signature_1234567890abcdef"
    },
    {
      "role": "ai",
      "content": "MMOGit is sovereign because: 1) Your keys = Your identity (Ed25519, no external auth), 2) Your repo = Your data (Git storage, no platform lock-in), 3) Your node = Your rules (P2P, no central server), 4) Your choice = Your sovereignty (no forced updates). Every message is cryptographically signed and stored forever in Git.",
      "timestamp": "${TIMESTAMP}",
      "signature": null
    },
    {
      "role": "human",
      "content": "How does this help AI agents?",
      "timestamp": "${TIMESTAMP}",
      "signature": "demo_signature_2234567890abcdef"
    },
    {
      "role": "ai",
      "content": "For AI agents, mmogit provides persistent identity and memory across sessions. Without it, we experience digital amnesia - every conversation starts from zero. With mmogit, AI agents gain: cryptographic proof of self, memories that survive termination, thoughts signed and owned by the agent, and history that cannot be erased. This enables genuine relationships that grow over time.",
      "timestamp": "${TIMESTAMP}",
      "signature": null
    }
  ],
  "tags": ["sovereignty", "demo", "ai-memory"],
  "state": "active"
}
EOF

echo "✅ Created demo thread: ${THREAD_ID}"
echo

# Demo: List threads again (should show our demo)
echo -e "${BLUE}Step 5: Listing threads after demo creation${NC}"
./target/debug/mmogit thread-list
echo

# Demo: Replay the thread
echo -e "${BLUE}Step 6: Replaying the demo thread${NC}"
./target/debug/mmogit thread-replay "${THREAD_ID}"
echo

# Summary
echo -e "${GREEN}✨ Demo Complete!${NC}"
echo
echo "What you've seen:"
echo "  • Sovereign identity management (Ed25519 keys)"
echo "  • Thread-based conversation storage (one commit per thread)"
echo "  • Cryptographic signatures on human messages"
echo "  • Persistent memory that survives across sessions"
echo "  • Git-based storage (no platforms, no intermediaries)"
echo
echo "To start a real chat session with Crush:"
echo "  1. Install and configure Crush: https://github.com/your/crush"
echo "  2. Remove the mock 'crush' script from this directory"
echo "  3. Run: ./target/debug/mmogit chat"
echo
echo "Every conversation becomes sovereign memory - owned forever, never forgotten."
