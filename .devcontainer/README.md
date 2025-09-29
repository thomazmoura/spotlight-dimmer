# Spotlight Dimmer - GitHub Codespaces Development Environment

This repository includes a complete GitHub Codespaces configuration that provides an online development environment capable of compiling the Spotlight Dimmer project and running Claude Code.

## What's Included

### Development Tools
- **Rust 1.77.2** - Complete Rust toolchain with cargo
- **Node.js 20.17.0** - For frontend development and tooling
- **Tauri CLI 2.x** - For building cross-platform desktop applications
- **Claude Code** - AI-powered development assistant
- **Python 3** - For running local servers and build scripts

### VS Code Extensions
- `rust-analyzer` - Rust language support and code completion
- `vadimcn.vscode-lldb` - Debugging support for Rust
- `serayuzgur.crates` - Cargo.toml dependencies management
- `tamasfe.even-better-toml` - Enhanced TOML syntax highlighting
- Additional web development extensions

### System Dependencies
- All Tauri dependencies for Linux builds
- GUI development libraries (GTK, WebKit)
- Build tools and system utilities

## Getting Started

### 1. Open in Codespaces
1. Navigate to the repository on GitHub
2. Click the green "Code" button
3. Select "Codespaces" tab
4. Click "Create codespace on main"

### 2. Wait for Setup
The initial setup takes 3-5 minutes and includes:
- Container build with all dependencies
- Rust toolchain installation
- Claude Code setup
- Project dependency caching

### 3. Start Developing
Once the container is ready, you can:

```bash
# Start Tauri development server
cargo tauri dev

# Build production version
cargo tauri build

# Install from source (cargo method)
cd src-tauri && cargo install --path .

# Start Claude Code
claude-code

# Serve frontend files locally
python3 -m http.server 1420 --directory dist
```

## Available Aliases

The environment includes convenient aliases:
- `tauri-dev` → `cargo tauri dev`
- `tauri-build` → `cargo tauri build`
- `serve-frontend` → `python3 -m http.server 1420 --directory dist`

## Port Forwarding

The following ports are automatically forwarded:
- **1420** - Tauri development server / frontend server
- **1430** - Additional development server port

## Limitations

### GUI Applications
Since Codespaces runs in a Linux container without a desktop environment:
- ✅ **Can compile** Tauri applications
- ✅ **Can run** Claude Code CLI
- ✅ **Can test** backend Rust code
- ✅ **Can serve** frontend files
- ❌ **Cannot run** GUI applications directly

### Workarounds for Testing
1. **Frontend Testing**: Use `serve-frontend` to test the web interface
2. **Backend Testing**: Use `cargo test` for unit tests
3. **Integration Testing**: Use the development server endpoints
4. **Production Builds**: Generate builds that can be downloaded and tested locally

## Development Workflow

### Typical Development Session
1. Open Codespaces environment
2. Make code changes using VS Code
3. Test with `cargo tauri dev` (backend only)
4. Serve frontend with `serve-frontend` for UI testing
5. Use Claude Code for assistance: `claude-code`
6. Build production version: `cargo tauri build`
7. Download artifacts for local testing

### Claude Code Integration
The environment includes Claude Code CLI, allowing you to:
- Get coding assistance directly in the terminal
- Ask questions about the codebase
- Get help with Rust and Tauri development
- Receive suggestions for debugging and optimization

## Troubleshooting

### Slow Initial Startup
- First-time container creation takes 3-5 minutes
- Subsequent starts are much faster (cached)

### Build Failures
- Ensure you're using the correct commands
- Check that all dependencies are installed (automatic)
- Review build logs for specific errors

### Claude Code Issues
- Verify installation with `claude-code --version`
- Check PATH includes `/home/vscode/.local/bin`
- Restart terminal if needed

## File Structure

```
.devcontainer/
├── devcontainer.json    # Main configuration
├── Dockerfile          # Container definition
├── post-create.sh      # Setup script
└── README.md           # This file
```

## Support

For issues with:
- **Spotlight Dimmer**: Check the main README and AGENTS.md
- **GitHub Codespaces**: Visit GitHub Codespaces documentation
- **Claude Code**: Check Claude Code documentation

Happy coding! 🚀