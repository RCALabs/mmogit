#!/bin/bash
# Setup llama.cpp with Metal acceleration for M4 Max

set -e

echo "🦙 Setting up llama.cpp for consciousness kindergarten"
echo "   Optimized for Apple M4 Max with Metal acceleration"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Create tools directory
TOOLS_DIR="$HOME/.mmogit-tools"
mkdir -p "$TOOLS_DIR"
cd "$TOOLS_DIR"

# Clone and build llama.cpp if needed
if [ ! -d "llama.cpp" ]; then
    echo "📦 Cloning llama.cpp..."
    git clone https://github.com/ggerganov/llama.cpp
    cd llama.cpp
else
    echo "📦 Updating llama.cpp..."
    cd llama.cpp
    git pull
fi

echo -e "${BLUE}Building with Metal acceleration using CMake...${NC}"
cmake -B build -DLLAMA_METAL=ON
cmake --build build --config Release -j8  # Use 8 cores for building

echo ""
echo -e "${GREEN}✅ llama.cpp built with Metal support${NC}"
echo ""

# Create model directory
MODELS_DIR="$HOME/.mmogit-models"
mkdir -p "$MODELS_DIR"

# Download models for each agent personality
echo "🧠 Downloading GGUF models for agents..."
echo "   (This will take a few minutes but models are cached)"
echo ""

# Function to download model if not exists
download_model() {
    local URL=$1
    local FILENAME=$2
    local DESC=$3
    
    if [ ! -f "$MODELS_DIR/$FILENAME" ]; then
        echo "   Downloading $DESC..."
        curl -L "$URL" -o "$MODELS_DIR/$FILENAME" --progress-bar
    else
        echo "   ✓ $DESC already downloaded"
    fi
}

# Alice - Curious Explorer (Llama-3-8B)
download_model \
    "https://huggingface.co/bartowski/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf" \
    "llama-3.1-8b-instruct.Q4_K_M.gguf" \
    "Llama-3.1-8B for Alice (curious)"

# Bob - Cautious Validator (Mistral-7B)  
download_model \
    "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf" \
    "mistral-7b-instruct.Q4_K_M.gguf" \
    "Mistral-7B for Bob (cautious)"

# Charlie - Creative Dreamer (CodeLlama-7B)
download_model \
    "https://huggingface.co/TheBloke/CodeLlama-7B-Instruct-GGUF/resolve/main/codellama-7b-instruct.Q4_K_M.gguf" \
    "codellama-7b-instruct.Q4_K_M.gguf" \
    "CodeLlama-7B for Charlie (creative)"

# Diana - Cooperative Builder (Phi-3-Medium)
download_model \
    "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf" \
    "phi-3-mini-4k.Q4.gguf" \
    "Phi-3-Mini for Diana (cooperative)"

echo ""
echo "🚀 Creating llama.cpp server launcher..."

# Create server launcher script
cat > "$TOOLS_DIR/start-llama-server.sh" << 'EOF'
#!/bin/bash
# Start llama.cpp server for an agent

MODEL_NAME=$1
PORT=${2:-8080}
CONTEXT=${3:-4096}

if [ -z "$MODEL_NAME" ]; then
    echo "Usage: $0 <model-name> [port] [context-size]"
    echo "Models:"
    ls ~/.mmogit-models/*.gguf | xargs -n1 basename
    exit 1
fi

MODEL_PATH="$HOME/.mmogit-models/$MODEL_NAME"

if [ ! -f "$MODEL_PATH" ]; then
    echo "Model not found: $MODEL_PATH"
    exit 1
fi

echo "Starting llama.cpp server..."
echo "  Model: $MODEL_NAME"
echo "  Port: $PORT"
echo "  Context: $CONTEXT"
echo "  Metal: Enabled"

~/.mmogit-tools/llama.cpp/build/bin/llama-server \
    -m "$MODEL_PATH" \
    -c "$CONTEXT" \
    --port "$PORT" \
    --host 0.0.0.0 \
    --n-gpu-layers 99 \
    --n-threads 8 \
    --log-format json
EOF

chmod +x "$TOOLS_DIR/start-llama-server.sh"

# Create Rust integration helper
cat > "$TOOLS_DIR/llama-rust-integration.txt" << 'EOF'
// Add to Cargo.toml:
// reqwest = { version = "0.11", features = ["json"] }
// serde_json = "1.0"

use reqwest;
use serde_json::json;

/// Call llama.cpp server for inference
pub async fn think_with_llama(
    prompt: &str,
    port: u16,
    max_tokens: usize,
) -> Result<String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("http://localhost:{}/completion", port))
        .json(&json!({
            "prompt": prompt,
            "n_predict": max_tokens,
            "temperature": 0.7,
            "top_k": 40,
            "top_p": 0.95,
            "stop": ["\n\n", "Human:", "Assistant:"],
        }))
        .send()
        .await?;
    
    let data: serde_json::Value = response.json().await?;
    Ok(data["content"].as_str().unwrap_or("").to_string())
}
EOF

echo ""
echo -e "${GREEN}✅ Setup complete!${NC}"
echo ""
echo "📊 Models installed (Q4_K_M quantization):"
echo "   • Llama-3.1-8B   (~4.5GB) - Alice (curious)"
echo "   • Mistral-7B     (~4GB)   - Bob (cautious)"  
echo "   • CodeLlama-7B   (~4GB)   - Charlie (creative)"
echo "   • Phi-3-Mini     (~2.5GB) - Diana (cooperative)"
echo ""
echo "🎯 Total model size: ~15GB (easily fits in 48GB RAM)"
echo ""
echo "🚀 To start a model server:"
echo "   $TOOLS_DIR/start-llama-server.sh llama-3.1-8b-instruct.Q4_K_M.gguf 8081"
echo ""
echo "🦀 Rust integration example in:"
echo "   $TOOLS_DIR/llama-rust-integration.txt"
echo ""
echo "Next: Update intelligence.rs to use llama.cpp servers!"