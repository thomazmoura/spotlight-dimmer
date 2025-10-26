#!/bin/bash

# Post-create script for Spotlight Dimmer .NET development environment
set -e

echo "🚀 Setting up Spotlight Dimmer .NET development environment..."

# Ensure we're in the right directory
cd /workspaces/spotlight-dimmer

# Source the appropriate shell configuration
if [ -f ~/.zshrc ]; then
    source ~/.zshrc
elif [ -f ~/.bashrc ]; then
    source ~/.bashrc
fi

echo "📦 Restoring .NET project dependencies..."

# Restore .NET dependencies
echo "Restoring NuGet packages..."
cd dotnet
dotnet restore
cd ..

# Verify all tools are working
echo "🔍 Verifying development tools..."

echo "Node.js version: $(node --version)"
echo "npm version: $(npm --version)"
echo ".NET SDK version: $(dotnet --version)"

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
echo "alias build='cd dotnet && dotnet build'" >> ~/.bashrc
echo "alias build-release='cd dotnet && dotnet build -c Release'" >> ~/.bashrc
echo "alias run='cd dotnet && dotnet run'" >> ~/.bashrc
echo "alias test='cd dotnet && dotnet test'" >> ~/.bashrc
echo "alias publish='cd dotnet && dotnet publish -c Release -r win-x64'" >> ~/.bashrc

if [ -f ~/.zshrc ]; then
    echo "alias build='cd dotnet && dotnet build'" >> ~/.zshrc
    echo "alias build-release='cd dotnet && dotnet build -c Release'" >> ~/.zshrc
    echo "alias run='cd dotnet && dotnet run'" >> ~/.zshrc
    echo "alias test='cd dotnet && dotnet test'" >> ~/.zshrc
    echo "alias publish='cd dotnet && dotnet publish -c Release -r win-x64'" >> ~/.zshrc

    # Add tmux auto-start logic with Claude integration
    echo "" >> ~/.zshrc
    echo "# Check if we're already inside tmux" >> ~/.zshrc
    echo 'if [ -z "$TMUX" ]; then' >> ~/.zshrc
    echo "  # Not inside tmux, try to attach or create new session" >> ~/.zshrc
    echo "  tmux a || tmux" >> ~/.zshrc
    echo "else" >> ~/.zshrc
    echo "  # Already inside tmux, run claude" >> ~/.zshrc
    echo "  claude --continue || claude" >> ~/.zshrc
    echo "fi" >> ~/.zshrc
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
echo "  build                    - Build .NET project (Debug)"
echo "  build-release            - Build .NET project (Release)"
echo "  run                      - Run the application"
echo "  test                     - Run tests"
echo "  publish                  - Publish AOT-compiled binary for Windows"
echo ""
echo "📝 Note: This is a .NET 10 PoC for SpotlightDimmer."
echo "   The Rust version is available in the ./rust folder."
echo "   Built binaries will be in: dotnet/bin/Debug/ or dotnet/bin/Release/"
echo ""
echo "🚀 You can now start developing the .NET PoC!"
echo "   Run 'build' to compile the project."
echo "   Run 'run' to execute the application."