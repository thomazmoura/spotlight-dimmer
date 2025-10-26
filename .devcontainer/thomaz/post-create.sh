#!/bin/bash

# Post-create script for Spotlight Dimmer with Thomaz Dev Environment
set -e

echo "🚀 Setting up Spotlight Dimmer .NET development environment (Thomaz)..."

# Ensure we're in the right directory
cd /workspaces/spotlight-dimmer

# Source the appropriate shell configuration (thomazmoura/dev-environment uses PowerShell by default)
if [ -f ~/.config/powershell/Microsoft.PowerShell_profile.ps1 ]; then
    echo "✅ PowerShell profile detected"
fi

echo "📦 Restoring .NET solution dependencies..."

# Restore .NET dependencies for the entire solution
echo "Restoring NuGet packages..."
dotnet restore SpotlightDimmer.sln

# Build to verify everything works
echo "🔨 Building solution..."
dotnet build SpotlightDimmer.sln --configuration Debug

# Verify all tools are working
echo "🔍 Verifying development tools..."

if command -v dotnet &> /dev/null; then
    echo "✅ .NET SDK version: $(dotnet --version)"
else
    echo "⚠️  .NET SDK not found!"
fi

if command -v pwsh &> /dev/null; then
    echo "✅ PowerShell version: $(pwsh --version)"
fi

if command -v gh &> /dev/null; then
    echo "✅ GitHub CLI version: $(gh --version | head -1)"
fi

if command -v tmux &> /dev/null; then
    echo "✅ tmux version: $(tmux -V)"
fi

# Create convenient development aliases for bash/zsh
if [ -f ~/.bashrc ]; then
    echo "📝 Adding development aliases to ~/.bashrc..."
    echo "" >> ~/.bashrc
    echo "# SpotlightDimmer development aliases" >> ~/.bashrc
    echo "alias build='dotnet build SpotlightDimmer.sln'" >> ~/.bashrc
    echo "alias build-release='dotnet build SpotlightDimmer.sln -c Release'" >> ~/.bashrc
    echo "alias run='dotnet run --project SpotlightDimmer/SpotlightDimmer.csproj'" >> ~/.bashrc
    echo "alias test='dotnet test SpotlightDimmer.sln'" >> ~/.bashrc
    echo "alias publish='dotnet publish SpotlightDimmer/SpotlightDimmer.csproj -c Release -r win-x64'" >> ~/.bashrc
    echo "alias clean='dotnet clean SpotlightDimmer.sln'" >> ~/.bashrc
fi

if [ -f ~/.zshrc ]; then
    echo "📝 Adding development aliases to ~/.zshrc..."
    echo "" >> ~/.zshrc
    echo "# SpotlightDimmer development aliases" >> ~/.zshrc
    echo "alias build='dotnet build SpotlightDimmer.sln'" >> ~/.zshrc
    echo "alias build-release='dotnet build SpotlightDimmer.sln -c Release'" >> ~/.zshrc
    echo "alias run='dotnet run --project SpotlightDimmer/SpotlightDimmer.csproj'" >> ~/.zshrc
    echo "alias test='dotnet test SpotlightDimmer.sln'" >> ~/.zshrc
    echo "alias publish='dotnet publish SpotlightDimmer/SpotlightDimmer.csproj -c Release -r win-x64'" >> ~/.zshrc
    echo "alias clean='dotnet clean SpotlightDimmer.sln'" >> ~/.zshrc
fi

# Set up git configuration for Codespaces if not already set
if [ -z "$(git config --global user.name)" ]; then
    echo "📝 Git configuration reminder..."
    echo "Please run the following commands to set up your Git identity:"
    echo "  git config --global user.name 'Your Name'"
    echo "  git config --global user.email 'your.email@example.com'"
fi

echo ""
echo "✅ Development environment setup complete!"
echo ""
echo "🛠️  Available commands:"
echo "  build                    - Build the entire solution (Debug)"
echo "  build-release            - Build the entire solution (Release)"
echo "  run                      - Run SpotlightDimmer application"
echo "  test                     - Run all tests"
echo "  clean                    - Clean build artifacts"
echo "  publish                  - Publish AOT-compiled binary for Windows"
echo ""
echo "📂 Project structure:"
echo "  SpotlightDimmer.sln              - Main solution file"
echo "  SpotlightDimmer/                 - Main Windows application"
echo "  SpotlightDimmer.Core/            - Core business logic (platform-agnostic)"
echo "  SpotlightDimmer.WindowsClient/   - Windows-specific bindings"
echo "  SpotlightDimmer.Config/          - Configuration management"
echo "  SpotlightDimmer.Tests/           - Unit tests"
echo ""
echo "🎯 Thomaz Dev Environment features:"
echo "  • NeoVim configured for development"
echo "  • PowerShell as default shell"
echo "  • tmux for terminal multiplexing"
echo "  • Azure CLI and development tools pre-installed"
echo ""
echo "🚀 Start developing!"
