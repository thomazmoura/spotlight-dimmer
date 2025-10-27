# Spotlight Dimmer .NET PoC - Dev Container Development Environment

This repository includes a complete dev container configuration that provides a .NET development environment for the SpotlightDimmer .NET 10 PoC.

## What's Included

### Development Tools
- **.NET 10.0 SDK** - Complete .NET SDK for C# development
- **Node.js 20.x LTS** - Required for Claude Code CLI
- **Claude Code** - AI-powered development assistant
- **GitHub CLI** - For managing issues, PRs, and releases
- **Git** - Version control with Oh My Zsh integration

### VS Code Extensions
- `ms-dotnettools.csharp` - C# language support and IntelliSense
- `ms-dotnettools.csdevkit` - C# Dev Kit for enhanced development
- `ms-dotnettools.vscode-dotnet-runtime` - .NET runtime support
- `ms-vscode.vscode-json` - JSON language support

### System Dependencies
- Essential utilities (curl, wget, jq, zip/unzip)
- tmux for terminal multiplexing

## Getting Started

### 1. Open in Dev Container

**Using VS Code:**
1. Install the "Dev Containers" extension
2. Open the repository in VS Code
3. Press `F1` and select "Dev Containers: Reopen in Container"

**Using GitHub Codespaces:**
1. Navigate to the repository on GitHub
2. Click the green "Code" button
3. Select "Codespaces" tab
4. Click "Create codespace on dotnet-10"

### 2. Wait for Setup
The initial setup takes 2-3 minutes and includes:
- Container build with .NET SDK
- Node.js and Claude Code setup
- NuGet package restoration
- Project dependency caching

### 3. Start Developing!
Once setup completes, you'll see:
```
✅ Development environment setup complete!

🛠️  Available commands:
  build                    - Build .NET project (Debug)
  build-release            - Build .NET project (Release)
  run                      - Run the application
  test                     - Run tests
  publish                  - Publish AOT-compiled binary for Windows
```

## Development Workflow

### Building the Project

```bash
# Quick debug build
build

# Release build (optimized)
build-release

# Full restore and build
cd dotnet
dotnet restore
dotnet build
```

### Running the Application

```bash
# Run using the alias
run

# Or manually
cd dotnet
dotnet run
```

### Publishing AOT Binary

```bash
# Publish with Native AOT (requires VS C++ tools on Windows)
publish

# Or manually
cd dotnet
dotnet publish -c Release -r win-x64
```

## Project Structure

```
/workspaces/spotlight-dimmer/
├── SpotlightDimmer.sln              # Main solution file
├── SpotlightDimmer.Core/            # Core business logic (platform-agnostic)
│   ├── AppState.cs                  # State management
│   ├── OverlayDefinition.cs         # Overlay calculations
│   └── SpotlightDimmer.Core.csproj
├── SpotlightDimmer.WindowsClient/   # Windows-specific application
│   ├── Program.cs                   # Main entry point
│   ├── FocusTracker.cs              # Event-driven focus tracking
│   ├── OverlayRenderer.cs           # Overlay window management
│   └── SpotlightDimmer.WindowsClient.csproj
├── SpotlightDimmer.Config/          # Configuration GUI utility
│   └── SpotlightDimmer.Config.csproj
├── SpotlightDimmer.Tests/           # Unit tests
│   └── SpotlightDimmer.Tests.csproj
└── .devcontainer/                   # Dev container configuration
```

## Available Aliases

The post-create script sets up convenient aliases:

| Alias | Command | Description |
|-------|---------|-------------|
| `build` | `cd dotnet && dotnet build` | Build Debug configuration |
| `build-release` | `cd dotnet && dotnet build -c Release` | Build Release configuration |
| `run` | `cd dotnet && dotnet run` | Run the application |
| `test` | `cd dotnet && dotnet test` | Run tests |
| `publish` | `cd dotnet && dotnet publish -c Release -r win-x64` | Publish AOT binary |

## Troubleshooting

### Container Won't Build
- Ensure Docker is running
- Check Docker has enough disk space (container needs ~2GB)
- Try: "Dev Containers: Rebuild Container"

### Claude Code Not Working
Claude Code should be automatically installed. If not:
```bash
sudo npm install -g @anthropic-ai/claude-code
claude --version
```

### .NET SDK Issues
Verify .NET is installed correctly:
```bash
dotnet --version
dotnet --list-sdks
```

Expected output: `10.0.x` or similar

### IntelliSense Not Working
- Wait for initial project indexing (can take 30-60 seconds)
- Restart OmniSharp: `F1` → "OmniSharp: Restart OmniSharp"
- Check Output panel: "View" → "Output" → Select "OmniSharp Log"

## Features vs Production Rust Version

This dev container is configured for the .NET 10 PoC:
- ✅ Fully event-driven window tracking (no polling)
- ✅ Dual event hooks (EVENT_SYSTEM_FOREGROUND + EVENT_OBJECT_LOCATIONCHANGE)
- ✅ Cleaner P/Invoke vs Rust FFI
- ✅ Faster development iteration
- ⚠️ PoC only - production version is Rust (see `/rust` folder)

## SSH Access

The container exposes SSH on port 22 for remote development:
- User: `vscode`
- Authentication: SSH key (configured by container runtime)

## Environment Variables

The container sets:
- `DOTNET_CLI_TELEMETRY_OPTOUT=1` - Disable .NET telemetry
- Development shell defaults to zsh with Oh My Zsh

## Resources

- [.NET Documentation](https://learn.microsoft.com/en-us/dotnet/)
- [C# Programming Guide](https://learn.microsoft.com/en-us/dotnet/csharp/)
- [Native AOT Deployment](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
- [Dev Containers](https://containers.dev/)
