# Spotlight Dimmer - Dev Container Development Environment

This repository includes a complete dev container configuration that provides a development environment capable of compiling the Spotlight Dimmer project and running Claude Code.

## What's Included

### Development Tools
- **Rust 1.83.0** - Complete Rust toolchain with cargo, rustfmt, clippy, and rust-analyzer
- **Node.js 20.x LTS** - Required for Claude Code CLI
- **Claude Code** - AI-powered development assistant
- **GitHub CLI** - For managing issues, PRs, and releases
- **Git** - Version control with Oh My Zsh integration

### VS Code Extensions
- `rust-lang.rust-analyzer` - Rust language support and code completion
- `vadimcn.vscode-lldb` - Debugging support for Rust
- `serayuzgur.crates` - Cargo.toml dependencies management
- `tamasfe.even-better-toml` - Enhanced TOML syntax highlighting
- `ms-vscode.vscode-json` - JSON language support

### System Dependencies
- Build tools (GCC, make, etc.)
- **MinGW-w64** - Cross-compiler for building Windows binaries from Linux
- **Windows target** - x86_64-pc-windows-gnu Rust target
- OpenSSL development libraries
- Essential utilities (curl, wget, jq, zip/unzip)

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
4. Click "Create codespace on main"

### 2. Wait for Setup
The initial setup takes 2-4 minutes and includes:
- Container build with all dependencies
- Rust toolchain installation (including Windows cross-compilation target)
- MinGW-w64 cross-compiler installation
- Node.js and Claude Code setup
- Project dependency caching

### 3. Start Developing
Once the container is ready, you can:

```bash
# Build Windows binaries using cross-compilation (recommended)
cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config

# Or use the convenient alias
build

# Run tests (with Windows target)
cargo test --lib --target x86_64-pc-windows-gnu

# Format code
cargo fmt

# Run linter (with Windows target)
cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code

# Or use the convenient alias
lint

# Start Claude Code
claude
```

## Available Aliases

The environment includes convenient aliases for common tasks (all use Windows cross-compilation target):
- `build` → `cargo build --release --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config`
- `build-debug` → `cargo build --target x86_64-pc-windows-gnu --bin spotlight-dimmer --bin spotlight-dimmer-config`
- `test` → `cargo test --lib --target x86_64-pc-windows-gnu`
- `lint` → `cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -W clippy::all -A dead_code`
- `fmt` → `cargo fmt`

## Architecture Overview

Spotlight Dimmer is a **pure Windows API application** built with Rust - no web frameworks, no browser engines, just native code.

**Key Components:**
- **Main Application**: Pure Windows API overlays for dimming inactive displays
- **Config Tool**: CLI tool for managing application settings
- **No GUI in Dev Container**: The app requires Windows to run, but you can build and test in the container

## Development Workflow

### Typical Development Session
1. Open dev container (VS Code or Codespaces)
2. Make code changes using VS Code
3. Use Claude Code for assistance: `claude`
4. Build and test:
   ```bash
   cargo build --release
   cargo test
   cargo clippy
   ```
5. Commit and push changes
6. Download binaries for Windows testing (if needed)

### Claude Code Integration
The environment includes Claude Code CLI, allowing you to:
- Get coding assistance directly in the terminal
- Ask questions about the codebase
- Get help with Rust and Windows API development
- Receive suggestions for debugging and optimization

**Using Claude Code:**
```bash
# Start Claude Code
# IMPORTANT: The binary is called 'claude', not 'claude-code'!
# Package name: @anthropic-ai/claude-code
# Binary name: claude
claude

# Verify installation
claude --version

# Check PATH includes Node.js and Claude Code
echo $PATH
```

## Cross-Compilation Support

### What You Can Do in the Dev Container
Since this dev container includes MinGW-w64 cross-compilation support:
- ✅ **Build Windows binaries** from Linux (`.exe` files)
- ✅ **Validate ALL code** including `#[cfg(windows)]` sections
- ✅ **Run tests** against Windows target
- ✅ **Run linters** on Windows-specific code
- ✅ **Use Claude Code CLI** for AI assistance
- ✅ **Full CI/CD parity** - exactly matches GitHub Actions
- ❌ **Cannot run** the GUI application (requires actual Windows)

### How Cross-Compilation Works
1. **Target**: `x86_64-pc-windows-gnu` (MinGW-based)
2. **Toolchain**: MinGW-w64 cross-compiler
3. **Output**: Windows `.exe` files in `target/x86_64-pc-windows-gnu/release/`
4. **Testing**: Built binaries can be transferred to Windows for execution

### Verification Workflow
1. **Local Validation**: Use `/check` or `lint` alias to validate Windows code
2. **Build Binaries**: Use `build` alias to create Windows `.exe` files
3. **CI Verification**: Push to GitHub - CI uses same cross-compilation process
4. **Windows Testing**: Transfer built binaries to Windows for final testing

## Troubleshooting

### Claude Code Not Found
**Symptoms**: `command not found: claude`

**IMPORTANT**: The npm package is `@anthropic-ai/claude-code`, but the executable binary is named `claude` (not `claude-code`).

**Solution**:
```bash
# Check if Node.js is installed
node --version
npm --version

# Reinstall Claude Code
npm install -g @anthropic-ai/claude-code

# Verify installation
claude --version

# Check PATH
echo $PATH  # Should include /usr/local/bin or Node.js global bin
```

### Slow Initial Startup
- First-time container creation takes 2-4 minutes
- Subsequent starts are much faster (cached layers)
- Rust dependency compilation runs during post-create script

### Build Failures
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version  # Should be 1.83.0

# Review build logs for specific errors
cargo build --verbose
```

### Path Issues
If tools aren't found, ensure PATH is configured:
```bash
# Should be in your PATH:
export PATH="/home/vscode/.cargo/bin:/home/vscode/.local/bin:${PATH}"

# Verify tools are accessible
which cargo
which node
which claude
```

## File Structure

```
.devcontainer/
├── devcontainer.json    # Main configuration
├── Dockerfile          # Container definition with Rust + Node.js
├── post-create.sh      # Setup script (runs after container creation)
└── README.md           # This file
```

## Development Tools Included

### Rust Toolchain
- **rustc 1.83.0** - Rust compiler
- **cargo** - Package manager and build tool
- **rustfmt** - Code formatter
- **clippy** - Linter for catching common mistakes
- **rust-analyzer** - LSP server for IDE support

### Node.js Ecosystem
- **Node.js 20.x** - JavaScript runtime (required for Claude Code)
- **npm** - Package manager for installing Claude Code

### Shell & Terminal
- **Zsh** - Default shell with Oh My Zsh
- **Git** - Version control
- **GitHub CLI** - GitHub operations from terminal

## Support

For issues with:
- **Spotlight Dimmer**: Check the main README.md and AGENTS.md in the repository root
- **Dev Containers**: Visit [VS Code Dev Containers documentation](https://code.visualstudio.com/docs/devcontainers/containers)
- **GitHub Codespaces**: Visit [GitHub Codespaces documentation](https://docs.github.com/en/codespaces)
- **Claude Code**: Run `claude --help` or check the [Claude Code documentation](https://docs.claude.com/claude)

## Quick Reference

```bash
# Build commands (via aliases)
build              # Build release binaries
build-debug        # Build debug binaries
test              # Run all tests
lint              # Run clippy linter
fmt               # Format code with rustfmt

# Claude Code
claude        # Start Claude Code CLI
claude --version  # Check installation

# Cargo commands
cargo build --release  # Build optimized binaries
cargo run --bin spotlight-dimmer-config status  # Run config tool
cargo install --path .  # Install to ~/.cargo/bin/

# Development utilities
cargo doc --open    # Generate and open documentation
cargo tree         # Show dependency tree
cargo outdated     # Check for outdated dependencies (requires cargo-outdated)
```

Happy coding! 🚀
