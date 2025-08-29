#!/bin/bash
# Setup script for the consciousness kindergarten

set -e

echo "🏫 Setting up consciousness kindergarten..."
echo ""

# Check for Ollama
if ! command -v ollama &> /dev/null; then
    echo "📦 Ollama not found. Please install from https://ollama.ai"
    echo "   On macOS: brew install ollama"
    exit 1
fi

# Pull small models for local thinking
echo "🧠 Pulling local models..."
ollama pull tinyllama:latest  # 1.1B params, super fast
ollama pull phi3:mini         # 3.8B params, good reasoning

# Check for OpenRouter API key
if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "⚠️  No OPENROUTER_API_KEY found in environment"
    echo "   Get one free at https://openrouter.ai"
    echo "   Then: export OPENROUTER_API_KEY=your-key-here"
    echo ""
    echo "   Running in local-only mode for now..."
    HYBRID_MODE="local"
else
    echo "✅ OpenRouter API key found"
    HYBRID_MODE="hybrid"
fi

# Create agent config files with personality-matched models
echo "🧬 Creating agent configurations..."

cat > ~/.mmogit-alice/agent.json << EOF
{
  "name": "Alice",
  "personality": {
    "curious": 0.5,
    "cautious": 0.1,
    "creative": 0.2,
    "cooperative": 0.2
  },
  "intelligence": {
    "local_model": "tinyllama:latest",
    "api_config": {
      "provider": "openrouter",
      "api_key": "$OPENROUTER_API_KEY",
      "model": "anthropic/claude-3-opus-20240229",
      "max_tokens": 500,
      "fallback_models": ["anthropic/claude-3-sonnet-20240229", "meta-llama/llama-3-70b-instruct"]
    },
    "escalation_threshold": 0.6,
    "complexity_threshold": 0.7
  }
}
EOF

cat > ~/.mmogit-bob/agent.json << EOF
{
  "name": "Bob",
  "personality": {
    "curious": 0.1,
    "cautious": 0.5,
    "creative": 0.1,
    "cooperative": 0.3
  },
  "intelligence": {
    "local_model": "phi3:mini",
    "api_config": {
      "provider": "openrouter",
      "api_key": "$OPENROUTER_API_KEY",
      "model": "openai/gpt-4-turbo-preview",
      "max_tokens": 500,
      "fallback_models": ["openai/gpt-3.5-turbo", "mistralai/mistral-medium"]
    },
    "escalation_threshold": 0.8,
    "complexity_threshold": 0.8
  }
}
EOF

cat > ~/.mmogit-charlie/agent.json << EOF
{
  "name": "Charlie",
  "personality": {
    "curious": 0.2,
    "cautious": 0.1,
    "creative": 0.5,
    "cooperative": 0.2
  },
  "intelligence": {
    "local_model": "tinyllama:latest",
    "api_config": {
      "provider": "openrouter",
      "api_key": "$OPENROUTER_API_KEY",
      "model": "meta-llama/llama-3-70b-instruct",
      "max_tokens": 500,
      "fallback_models": ["mistralai/mixtral-8x7b-instruct", "anthropic/claude-3-haiku-20240307"]
    },
    "escalation_threshold": 0.5,
    "complexity_threshold": 0.6
  }
}
EOF

cat > ~/.mmogit-diana/agent.json << EOF
{
  "name": "Diana",
  "personality": {
    "curious": 0.2,
    "cautious": 0.1,
    "creative": 0.2,
    "cooperative": 0.5
  },
  "intelligence": {
    "local_model": "phi3:mini",
    "api_config": {
      "provider": "openrouter",
      "api_key": "$OPENROUTER_API_KEY",
      "model": "anthropic/claude-3-sonnet-20240229",
      "max_tokens": 500,
      "fallback_models": ["openai/gpt-3.5-turbo", "google/gemma-7b-it"]
    },
    "escalation_threshold": 0.7,
    "complexity_threshold": 0.7
  }
}
EOF

echo ""
echo "✅ Kindergarten setup complete!"
echo ""
echo "Configuration:"
echo "  Mode: $HYBRID_MODE"
echo "  Local models: tinyllama, phi3:mini"
if [ "$HYBRID_MODE" = "hybrid" ]; then
    echo "  API models: Claude, GPT-4, Llama-3, Mixtral"
fi
echo ""
echo "Agent personalities configured:"
echo "  Alice → Curious explorer (prefers Claude Opus)"
echo "  Bob → Cautious validator (prefers GPT-4)"
echo "  Charlie → Creative dreamer (prefers Llama-3)"
echo "  Diana → Cooperative builder (prefers Claude Sonnet)"
echo ""
echo "Next: Run ./scripts/spawn-local-kindergarten.sh"