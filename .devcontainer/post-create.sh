#!/bin/bash

# Post-create script for Spotlight Dimmer development environment
set -e

echo "🚀 Setting up Spotlight Dimmer development environment..."

# Ensure we're in the right directory
cd /workspaces/spotlight-dimmer

# Source the appropriate shell configuration
if [ -f ~/.zshrc ]; then
    source ~/.zshrc
elif [ -f ~/.bashrc ]; then
    source ~/.bashrc
fi

# Verify Node.js is available
if [ -f ~/.nvm/nvm.sh ]; then
    source ~/.nvm/nvm.sh
    nvm use default
fi

echo "📦 Installing project dependencies..."

# Install Rust dependencies and build project to populate cache
echo "Building Rust project (this may take a few minutes)..."
cd src-tauri
cargo fetch
cargo check --all-features
cd ..

# Verify all tools are working
echo "🔍 Verifying development tools..."

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Node.js version: $(node --version)"
echo "npm version: $(npm --version)"
echo "Tauri CLI version: $(cargo tauri --version)"

# Check if Claude Code is available
if command -v claude-code &> /dev/null; then
    echo "Claude Code version: $(claude-code --version)"
else
    echo "⚠️  Claude Code not found in PATH. Installing..."
    # Attempt to install Claude Code again
    curl -fsSL https://claude.ai/claude-code/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
    if command -v claude-code &> /dev/null; then
        echo "✅ Claude Code successfully installed: $(claude-code --version)"
    else
        echo "❌ Claude Code installation failed. Please install manually."
    fi
fi

# Create a convenient development alias
echo "alias tauri-dev='cargo tauri dev'" >> ~/.bashrc
echo "alias tauri-build='cargo tauri build'" >> ~/.bashrc
echo "alias serve-frontend='python3 -m http.server 1420 --directory dist'" >> ~/.bashrc

if [ -f ~/.zshrc ]; then
    echo "alias tauri-dev='cargo tauri dev'" >> ~/.zshrc
    echo "alias tauri-build='cargo tauri build'" >> ~/.zshrc
    echo "alias serve-frontend='python3 -m http.server 1420 --directory dist'" >> ~/.zshrc
fi

# Set up git configuration for Codespaces if not already set
if [ -z "$(git config --global user.name)" ]; then
    echo "📝 Setting up Git configuration..."
    echo "Please run the following commands to set up your Git identity:"
    echo "  git config --global user.name 'Your Name'"
    echo "  git config --global user.email 'your.email@example.com'"
fi

echo "✅ Development environment setup complete!"
echo ""
echo "🛠️  Available commands:"
echo "  cargo tauri dev          - Start development server with hot reload"
echo "  cargo tauri build        - Build production version"
echo "  serve-frontend           - Serve frontend files locally"
echo "  claude-code              - Launch Claude Code CLI"
echo ""
echo "🚀 You can now start developing Spotlight Dimmer!"
echo "   Run 'cargo tauri dev' to start the development server."