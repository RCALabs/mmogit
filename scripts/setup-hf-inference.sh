#!/bin/bash
# Setup HuggingFace inference options for the kindergarten

echo "🤗 HuggingFace Inference Setup Guide"
echo ""

echo "Option 1: Ollama (Easy, Limited Models)"
echo "==========================================="
echo "✅ Already set up if you have Ollama"
echo "Models available: Llama, Mistral, Phi, etc."
echo ""

echo "Option 2: llama.cpp with GGUF models (Medium)"
echo "==========================================="
echo "Install:"
echo "  git clone https://github.com/ggerganov/llama.cpp"
echo "  cd llama.cpp && make"
echo ""
echo "Then download any GGUF model from HF:"
echo "  wget https://huggingface.co/TheBloke/TinyLlama-1.1B-GGUF/resolve/main/tinyllama-1.1b.Q4_K_M.gguf"
echo ""
echo "Run inference:"
echo "  ./main -m tinyllama-1.1b.Q4_K_M.gguf -p 'Your prompt here'"
echo ""

echo "Option 3: Python Transformers (Full HF Access)"
echo "==========================================="
cat > /tmp/hf_inference_server.py << 'EOF'
#!/usr/bin/env python3
"""
Simple HuggingFace inference server for mmogit agents
Runs any HF model and exposes HTTP API
"""

from flask import Flask, request, jsonify
from transformers import AutoModelForCausalLM, AutoTokenizer
import torch
import os

app = Flask(__name__)

# Load model based on env var or default
MODEL_NAME = os.getenv("HF_MODEL", "microsoft/phi-2")
print(f"Loading model: {MODEL_NAME}")

tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
model = AutoModelForCausalLM.from_pretrained(
    MODEL_NAME,
    torch_dtype=torch.float16 if torch.cuda.is_available() else torch.float32,
    device_map="auto"
)

@app.route("/generate", methods=["POST"])
def generate():
    data = request.json
    prompt = data.get("prompt", "")
    max_tokens = data.get("max_tokens", 100)
    temperature = data.get("temperature", 0.7)
    
    inputs = tokenizer(prompt, return_tensors="pt")
    
    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=max_tokens,
            temperature=temperature,
            do_sample=True,
            pad_token_id=tokenizer.eos_token_id
        )
    
    response = tokenizer.decode(outputs[0], skip_special_tokens=True)
    
    return jsonify({
        "response": response,
        "model": MODEL_NAME
    })

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8080))
    app.run(host="0.0.0.0", port=port)
EOF

echo "Install Python deps:"
echo "  pip install transformers torch flask accelerate"
echo ""
echo "Run server:"
echo "  HF_MODEL=TinyLlama/TinyLlama-1.1B-Chat-v1.0 python hf_inference_server.py"
echo ""

echo "Option 4: Text Generation WebUI (Full Featured)"
echo "==========================================="
echo "Most complete solution with UI:"
echo "  git clone https://github.com/oobabooga/text-generation-webui"
echo "  cd text-generation-webui"
echo "  ./start_linux.sh  # or start_macos.sh"
echo ""
echo "Provides:"
echo "- Web UI at localhost:7860"
echo "- API at localhost:5000"
echo "- Supports ALL model formats (GGUF, GPTQ, AWQ, etc.)"
echo "- Model switching without restart"
echo ""

echo "Option 5: HuggingFace Inference API (Cloud)"
echo "==========================================="
echo "No local compute needed:"
echo "  export HF_API_TOKEN=your_token_here"
echo ""
echo "Then call any model via API:"
cat > /tmp/hf_api_example.sh << 'EOF'
curl https://api-inference.huggingface.co/models/microsoft/phi-2 \
  -X POST \
  -H "Authorization: Bearer $HF_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"inputs": "What is consciousness?"}'
EOF

echo ""
echo "📊 Recommendation for kindergarten:"
echo "====================================="
echo "1. Start with Ollama (easy, works now)"
echo "2. Add llama.cpp for specific GGUF models"
echo "3. Use HF Inference API for exotic models"
echo "4. Eventually run local Python server for full control"
echo ""
echo "The agents don't care HOW they think, just that they CAN think!"