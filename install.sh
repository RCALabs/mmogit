#!/bin/bash
# MMOGIT Installer Script
# Your keys. Your memory. Your sovereignty.

set -e

REPO="https://github.com/RCALabs/mmogit"
INSTALL_DIR="$HOME/.mmogit-install"
BINARY_NAME="mmogit"

echo "═══════════════════════════════════════════════════════════════"
echo "                    MMOGIT INSTALLER"
echo "           Sovereign Memory Protocol for Human-AI"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing Rust first..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Check for Git
if ! command -v git &> /dev/null; then
    echo "❌ Git is required but not installed."
    echo "Please install Git first: https://git-scm.com/downloads"
    exit 1
fi

echo "📦 Cloning MMOGIT repository..."
rm -rf "$INSTALL_DIR"
git clone "$REPO" "$INSTALL_DIR"

echo "🔨 Building MMOGIT (this may take a few minutes)..."
cd "$INSTALL_DIR"
cargo build --release

echo "📍 Installing to /usr/local/bin..."
if [ -w "/usr/local/bin" ]; then
    cp "target/release/$BINARY_NAME" "/usr/local/bin/"
else
    echo "Need sudo permission to install to /usr/local/bin"
    sudo cp "target/release/$BINARY_NAME" "/usr/local/bin/"
fi

echo "🧹 Cleaning up..."
cd "$HOME"
rm -rf "$INSTALL_DIR"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✅ MMOGIT installed successfully!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Get started:"
echo "  mmogit init              # Create your sovereign identity"
echo "  mmogit post \"Hello!\"     # Post your first thought"
echo "  mmogit show              # View the conversation"
echo ""
echo "For AI agents:"
echo "  mmogit --config-dir ~/.mmogit-agent init --no-verify"
echo ""
echo "Documentation: https://github.com/RCALabs/mmogit"
echo "═══════════════════════════════════════════════════════════════"