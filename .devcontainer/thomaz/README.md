# SpotlightDimmer - Thomaz Dev Environment Configuration

This is an alternative dev container configuration for SpotlightDimmer that uses the [thomazmoura/dev-environment](https://hub.docker.com/r/thomazmoura/dev-environment) Docker image. This provides a feature-rich development environment with NeoVim, PowerShell, tmux, and other productivity tools.

## What's Included

### From thomazmoura/dev-environment Base Image
- **NeoVim** - Fully configured for modern development
- **PowerShell** - Default shell with extensive scripting capabilities
- **tmux** - Terminal multiplexer for efficient workflow
- **Azure CLI** - For cloud development and deployment
- **Git** - Version control with advanced configurations
- **Docker CLI** - Container management tools

### Added for SpotlightDimmer Development
- **.NET 10.0 SDK** - Complete .NET SDK for C# development
- **GitHub CLI** - For managing issues, PRs, and releases
- **VS Code Extensions**:
  - `ms-dotnettools.csharp` - C# language support and IntelliSense
  - `ms-dotnettools.csdevkit` - C# Dev Kit for enhanced development
  - `ms-dotnettools.vscode-dotnet-runtime` - .NET runtime support
  - `ms-vscode.vscode-json` - JSON language support
  - `anthropic.claude-dev` - AI-powered development assistant

## How to Use This Configuration

### Option 1: VS Code - Select Configuration Manually

1. Install the "Dev Containers" extension in VS Code
2. Open the repository in VS Code
3. Press `F1` and run: **"Dev Containers: Reopen in Container"**
4. When prompted, select **".devcontainer/thomaz/devcontainer.json"**
5. VS Code will rebuild the container using the Thomaz environment

### Option 2: VS Code - Set as Default

To always use this configuration, rename the directories:
```bash
# Rename current default to 'standard'
mv .devcontainer .devcontainer-standard

# Rename thomaz to default
mv .devcontainer-thomaz .devcontainer
```

### Option 3: GitHub Codespaces

When creating a new Codespace:
1. Click the green "Code" button on GitHub
2. Select "Codespaces" tab
3. Click the "..." menu → "New with options"
4. Under "Dev container configuration", select **"thomaz"**
5. Click "Create codespace"

Alternatively, you can specify the configuration in `.devcontainer/devcontainer.json` in your branch.

## Development Workflow

### Shell Environment

The container uses **PowerShell** as the default shell, providing:
- Rich scripting capabilities
- Cross-platform compatibility
- Object-based pipeline (vs text-based in bash)
- Extensive cmdlet library

To switch shells:
```bash
# Switch to bash
bash

# Switch to zsh (if configured)
zsh

# Return to PowerShell
pwsh
```

### Using tmux

tmux is pre-installed for terminal multiplexing:

```bash
# Start tmux session
tmux

# Create new window: Ctrl+b c
# Switch windows: Ctrl+b n (next) or Ctrl+b p (previous)
# Split pane horizontally: Ctrl+b "
# Split pane vertically: Ctrl+b %
# Detach session: Ctrl+b d
# Reattach session: tmux attach
```

### Using NeoVim

The image includes a fully configured NeoVim setup:

```bash
# Open file in NeoVim
nvim Program.cs

# Or use the 'vi' alias
vi Program.cs
```

### Building and Running SpotlightDimmer

The post-create script sets up convenient aliases:

```bash
# Build the entire solution
build

# Build in Release mode
build-release

# Run the application
run

# Run tests
test

# Clean build artifacts
clean

# Publish AOT-compiled binary
publish
```

Or use dotnet commands directly:
```bash
# Restore dependencies
dotnet restore SpotlightDimmer.sln

# Build
dotnet build SpotlightDimmer.sln

# Run specific project
dotnet run --project SpotlightDimmer/SpotlightDimmer.csproj

# Run tests
dotnet test SpotlightDimmer.sln
```

## Project Structure

```
/workspaces/spotlight-dimmer/
├── SpotlightDimmer.sln              # Main solution file
├── SpotlightDimmer/                 # Main Windows application
│   ├── Program.cs                   # Entry point
│   └── SpotlightDimmer.csproj
├── SpotlightDimmer.Core/            # Core business logic (platform-agnostic)
│   ├── AppState.cs                  # State management
│   ├── OverlayDefinition.cs         # Overlay calculations
│   └── SpotlightDimmer.Core.csproj
├── SpotlightDimmer.WindowsClient/   # Windows-specific bindings
│   ├── FocusTracker.cs              # Event-driven focus tracking
│   ├── OverlayRenderer.cs           # Overlay window management
│   └── SpotlightDimmer.WindowsClient.csproj
├── SpotlightDimmer.Config/          # Configuration management
│   └── SpotlightDimmer.Config.csproj
└── SpotlightDimmer.Tests/           # Unit tests
    └── SpotlightDimmer.Tests.csproj
```

## Differences from Default Configuration

| Feature | Default Config | Thomaz Config |
|---------|---------------|---------------|
| Base Image | Custom Dockerfile | thomazmoura/dev-environment |
| Default Shell | Zsh + Oh My Zsh | PowerShell |
| Text Editor | VS Code only | NeoVim + VS Code |
| Terminal Multiplexer | None | tmux pre-configured |
| Azure Tools | None | Azure CLI included |
| Container Size | ~2 GB | ~3-4 GB (more tools) |
| Startup Time | Faster | Slightly slower (larger image) |

## Tips and Tricks

### PowerShell Productivity

```powershell
# List all dotnet projects
Get-ChildItem -Recurse -Filter *.csproj

# Find files containing specific text
Get-ChildItem -Recurse -Filter *.cs | Select-String "FocusTracker"

# Quick directory navigation
cd $HOME  # Or just: ~
```

### Git with PowerShell

```powershell
# PowerShell aliases for git
git status   # Works as expected
git log --oneline -10

# Or use GitHub CLI
gh pr list
gh issue create
```

### NeoVim + VS Code Workflow

- Use VS Code for heavy refactoring and debugging
- Use NeoVim for quick edits and terminal-based development
- Both editors share the same file system and git repository

## Troubleshooting

### Container Won't Build
- Ensure Docker is running and has sufficient resources
- The thomazmoura/dev-environment image is ~3-4 GB
- Try: "Dev Containers: Rebuild Container Without Cache"

### .NET SDK Not Found
The post-create script installs .NET 10.0. If missing:
```bash
# Check available SDKs
dotnet --list-sdks

# Manually install .NET 10 if needed
curl -sSL https://dot.net/v1/dotnet-install.sh | bash /dev/stdin --version 10.0
```

### PowerShell Not Default
If bash is the default shell instead of PowerShell:
```bash
# Temporarily switch
pwsh

# Permanently set in VS Code
# Edit .devcontainer/thomaz/devcontainer.json
# "terminal.integrated.defaultProfile.linux": "pwsh"
```

### tmux Not Working
tmux should be pre-installed in the base image. If missing:
```bash
sudo apt update && sudo apt install -y tmux
```

## Resources

- [thomazmoura/dev-environment Docker Hub](https://hub.docker.com/r/thomazmoura/dev-environment)
- [thomazmoura/dev-environment GitHub](https://github.com/thomazmoura/dev-environment)
- [.NET Documentation](https://learn.microsoft.com/en-us/dotnet/)
- [PowerShell Documentation](https://learn.microsoft.com/en-us/powershell/)
- [NeoVim Documentation](https://neovim.io/doc/)
- [tmux Cheat Sheet](https://tmuxcheatsheet.com/)
- [Dev Containers](https://containers.dev/)

## Why Use This Configuration?

Choose the **Thomaz configuration** if you:
- Prefer PowerShell over bash/zsh
- Want to use NeoVim for fast terminal-based editing
- Use tmux for terminal multiplexing
- Need Azure CLI for cloud development
- Want a more feature-rich development environment

Choose the **default configuration** if you:
- Prefer a minimal, fast-starting container
- Use Zsh with Oh My Zsh
- Don't need the extra tools
- Want faster container rebuild times
