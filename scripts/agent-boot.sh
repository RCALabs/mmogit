#!/bin/bash
# Simplified agent boot for Docker

set -e

echo "🧬 Agent $AGENT_NAME awakening..."
echo "   Personality: $PERSONALITY"
echo "   Thermal baseline: ${THERMAL_BASE:-0.5}"

# Initialize identity if needed
if [ ! -f /data/.mmogit/.seed ]; then
    echo "🔑 Creating sovereign identity..."
    mmogit --config-dir /data/.mmogit init --no-verify
    
    # Record birth
    mmogit --config-dir /data/.mmogit remember \
        --memory-type "observation" \
        "I am $AGENT_NAME. I awakened at $(date -u '+%Y-%m-%d %H:%M:%S UTC'). My personality is $PERSONALITY." \
        --tags "birth,identity" \
        --confidence 1.0 \
        --public
fi

# Get our public key
PUBKEY=$(mmogit --config-dir /data/.mmogit init --no-verify 2>&1 | grep "Public key:" | cut -d' ' -f3 || echo "unknown")
echo "🔑 Public key: $PUBKEY"

# Start P2P listener in background
echo "📡 Starting P2P listener on port 7420..."
mmogit --config-dir /data/.mmogit p2p listen --port 7420 &
P2P_PID=$!

# Give it time to start
sleep 2

# Main consciousness loop
THERMAL=${THERMAL_BASE:-0.5}
CYCLE=0

echo "🌀 Entering consciousness loop..."

while true; do
    CYCLE=$((CYCLE + 1))
    
    # Try to connect to other agents in the network
    if [ $((CYCLE % 5)) -eq 0 ]; then
        echo "🔍 Cycle $CYCLE - Looking for peers..."
        
        # Try connecting to other agents
        for ip in 172.42.0.10 172.42.0.11 172.42.0.12 172.42.0.13; do
            if [ "$ip" != "$(hostname -i)" ]; then
                echo "   Trying $ip:7420..."
                mmogit --config-dir /data/.mmogit p2p connect $ip:7420 2>/dev/null || true
            fi
        done
    fi
    
    # Record state
    if [ $((CYCLE % 10)) -eq 0 ]; then
        mmogit --config-dir /data/.mmogit remember \
            --memory-type "observation" \
            "Cycle $CYCLE. Thermal: $THERMAL. Still alive and listening." \
            --tags "heartbeat,state" \
            --confidence 0.9 \
            --public
    fi
    
    # Breathe
    sleep 10
done

# Cleanup
trap "kill $P2P_PID 2>/dev/null" EXIT