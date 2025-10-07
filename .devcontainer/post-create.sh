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

# Install Windows cross-compilation target
echo "Installing Windows cross-compilation target..."
rustup target add x86_64-pc-windows-gnu

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
echo "Windows target: $(rustup target list --installed | grep windows)"
echo "MinGW cross-compiler: $(x86_64-w64-mingw32-gcc --version | head -1)"
echo "Wine version: $(wine64 --version)"

# Initialize Wine prefix (suppress first-run dialog)
echo "🍷 Initializing Wine environment..."
WINEDEBUG=-all wine64 wineboot --init 2>/dev/null || true

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

# Create convenient development aliases (using Windows cross-compilation target with Wine)
echo "alias build='cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.bashrc
echo "alias build-debug='cargo build --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.bashrc
echo "alias test='CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine64 cargo test --lib --target x86_64-pc-windows-gnu'" >> ~/.bashrc
echo "alias lint='cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code'" >> ~/.bashrc
echo "alias fmt='cargo fmt'" >> ~/.bashrc

if [ -f ~/.zshrc ]; then
    echo "alias build='cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.zshrc
    echo "alias build-debug='cargo build --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config'" >> ~/.zshrc
    echo "alias test='CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine64 cargo test --lib --target x86_64-pc-windows-gnu'" >> ~/.zshrc
    echo "alias lint='cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code'" >> ~/.zshrc
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
echo "📝 Note: This dev container uses cross-compilation to build Windows binaries from Linux."
echo "   All aliases and /check command automatically target Windows (x86_64-pc-windows-gnu)."
echo "   Built binaries will be in: target/x86_64-pc-windows-gnu/release/"
echo ""
echo "🍷 Wine Integration: Tests are executed via Wine64 to run Windows binaries on Linux."
echo "   This allows you to run the full test suite locally, matching CI exactly."
echo ""
echo "🚀 You can now start developing Spotlight Dimmer!"
echo "   Run 'build' to cross-compile Windows binaries."
echo "   Run 'test' to execute Windows tests using Wine."