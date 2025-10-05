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

echo "📦 Installing project dependencies..."

# Install Rust dependencies and build project to populate cache
echo "Building Rust project (this may take a few minutes)..."
cargo fetch
cargo check --all-features

# Verify all tools are working
echo "🔍 Verifying development tools..."

echo "Node.js version: $(node --version)"
echo "npm version: $(npm --version)"
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Clippy version: $(cargo clippy --version)"
echo "Rustfmt version: $(cargo fmt --version)"

# Check if Claude Code is available (binary name is 'claude', not 'claude-code')
if command -v claude &> /dev/null; then
    echo "✅ Claude Code version: $(claude --version)"
else
    echo "⚠️  Claude Code not found in PATH."
    echo "This should have been installed during container build."
    echo "Try rebuilding the container or manually install with:"
    echo "    sudo npm install -g @anthropic-ai/claude-code"
    echo "Note: The binary is called 'claude', not 'claude-code'"
fi

# Create convenient development aliases
echo "alias build='cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.bashrc
echo "alias build-debug='cargo build --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.bashrc
echo "alias test='cargo test'" >> ~/.bashrc
echo "alias lint='cargo clippy -- -D warnings'" >> ~/.bashrc
echo "alias fmt='cargo fmt'" >> ~/.bashrc

if [ -f ~/.zshrc ]; then
    echo "alias build='cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.zshrc
    echo "alias build-debug='cargo build --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.zshrc
    echo "alias test='cargo test'" >> ~/.zshrc
    echo "alias lint='cargo clippy -- -D warnings'" >> ~/.zshrc
    echo "alias fmt='cargo fmt'" >> ~/.zshrc
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
echo "  build                    - Build release binaries (spotlight-dimmer + spotlight-dimmer-config)"
echo "  build-debug              - Build debug binaries"
echo "  test                     - Run all tests"
echo "  lint                     - Run clippy linter"
echo "  fmt                      - Format code with rustfmt"
echo "  cargo install --path .   - Install binaries to ~/.cargo/bin/"
echo "  claude-code              - Launch Claude Code CLI"
echo ""
echo "📝 Note: This is a pure Rust project using Windows API."
echo "   For Windows-specific features, you may need to test on Windows."
echo ""
echo "🚀 You can now start developing Spotlight Dimmer!"
echo "   Run 'build' to compile the release binaries."