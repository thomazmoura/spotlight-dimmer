# Spotlight Dimmer

## Overview

A lightweight Windows application that dims inactive displays to highlight the active one. Built with pure Rust and Windows API for maximum performance and minimal resource usage.

> Para a versão em português vá para: [LEIAME.md](LEIAME.md)

Spotlight Dimmer is a program for Windows that dims all the monitors other than the monitor that has the currently focused program.

It's intended to help people who use multiple monitores to focus and assist in quickly noticing which window has the current focus while changing focus with shortcuts like `alt + tab`. It's specially useful for users who navigate mainly with the keyboard. It helps to avoid silly situations like typing terminal commands on Teams because you're looking at a screen while the focus is on the other screen.

## Features

- **Ultra-lightweight**: Only ~7.6 MB RAM usage, ~561 KB binary size
- **Native Windows API**: No browser engine overhead, instant startup
- **Flexible overlay modes**:
  - **Inactive display dimming**: Dims all non-active displays (traditional mode)
  - **Active display overlay**: Optional customizable overlay on the active display/window
  - **Partial dimming**: Highlights windowed apps by dimming empty areas around the focused window
- **System tray integration**:
  - Double-click to hide/show overlays
  - Right-click for profile switching
  - Exit option available from tray menu
- **Profile system**: Save and switch between custom configurations with different colors and overlay settings
- **Click-through overlays**: Overlays don't interfere with mouse/keyboard input
- **Automatic focus tracking**: Detects active window and display changes in real-time (100ms polling)
- **Display hotplug support**: Automatically recreates overlays when displays are connected/disconnected
- **Persistent configuration**: Settings saved in TOML format at `%APPDATA%\spotlight-dimmer\config.toml`
- **CLI configuration tool**: Manage settings without restarting the application (2-second auto-reload)

## Installation

### Option 1: Windows Installer (Recommended)

1. Download the latest `spotlight-dimmer-v*-installer.exe` from the [GitHub Releases page](https://github.com/thomazmoura/spotlight-dimmer/releases).
2. Run the installer and follow the setup wizard. It will:
   - Install the main app and configuration CLI
   - Copy the required icon files automatically
   - Create Start Menu entries (optional desktop shortcut)
   - Add an uninstaller entry in Windows Settings > Apps

**Uninstall:** Open Windows Settings → Apps → Installed apps, search for "Spotlight Dimmer", and click Uninstall (or run the uninstaller from the Start Menu folder).

### Option 2: Install via npm (Alternative)

The easiest way to install Spotlight Dimmer is through npm:

```bash
npm install -g spotlight-dimmer
```

**Requirements:**
- Node.js 14 or higher
- Windows x64

The package includes pre-built binaries - no compilation needed! After installation, the commands `spotlight-dimmer` and `spotlight-dimmer-config` will be available globally.

**Uninstall:**
```bash
npm uninstall -g spotlight-dimmer
```

### Option 3: Build from Source (Cargo)

If you prefer to build and install manually using Cargo:

```bash
cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config
```

Binaries will be in `target\release\`:
- `spotlight-dimmer.exe` - Main application
- `spotlight-dimmer-config.exe` - Configuration tool

You can also use the PowerShell installation script:
```powershell
.\install.ps1
```

### Usage

#### Running the Application

**If installed via Windows installer:**
- Use the Start Menu shortcut: Start → Spotlight Dimmer
- The application will run with a system tray icon

**If installed via npm, cargo, or portable zip:**
```cmd
spotlight-dimmer.exe
```

The application will:
1. Load configuration from `%APPDATA%\spotlight-dimmer\config.toml` (or create default)
2. Add a system tray icon for quick access
3. Detect all connected displays
4. Create semi-transparent overlay windows on each display
5. Monitor active window focus and update overlay visibility
6. Run indefinitely until closed via system tray or Task Manager

#### Controlling the Application via System Tray

The system tray icon provides quick access to key functions:

- **Double-click**: Hide/show all overlays (pause/resume dimming)
- **Right-click**: Access the context menu with:
  - Profile switching (quick-switch between saved profiles)
  - Exit option to close the application

**Alternative ways to stop the application:**

```powershell
# Using PowerShell
Get-Process spotlight-dimmer | Stop-Process
```

Or use Task Manager to end the `spotlight-dimmer.exe` process.

#### Configuration Tool

Use `spotlight-dimmer-config.exe` to manage all settings:

```cmd
# Show current configuration
spotlight-dimmer-config status

# Inactive overlay commands (dims non-active displays)
spotlight-dimmer-config enable              # Enable inactive display dimming
spotlight-dimmer-config disable             # Disable inactive display dimming
spotlight-dimmer-config color 0 0 0 0.7     # Set inactive overlay color (RGB 0-255, alpha 0.0-1.0)

# Active overlay commands (highlights active display)
spotlight-dimmer-config active-enable       # Enable active display overlay
spotlight-dimmer-config active-disable      # Disable active display overlay
spotlight-dimmer-config active-color 50 100 255 0.15  # Set active overlay color

# Partial dimming commands (dims empty areas around focused window)
spotlight-dimmer-config partial-enable      # Enable partial dimming
spotlight-dimmer-config partial-disable     # Disable partial dimming

# Profile commands
spotlight-dimmer-config list-profiles       # List all saved profiles
spotlight-dimmer-config set-profile dark-mode    # Load and apply a saved profile
spotlight-dimmer-config save-profile my-setup    # Save current settings as a profile
spotlight-dimmer-config delete-profile my-setup  # Delete a saved profile

# General commands
spotlight-dimmer-config reset               # Reset to defaults
spotlight-dimmer-config help                # Show all available commands
```

**Note**: Configuration changes are automatically detected and reloaded within 2 seconds. No restart needed!

## Configuration File

Configuration is stored at `%APPDATA%\spotlight-dimmer\config.toml`:

```toml
is_dimming_enabled = true
is_active_overlay_enabled = false
is_paused = false
is_partial_dimming_enabled = false

# Inactive overlay color (for non-active displays)
[overlay_color]
r = 0
g = 0
b = 0
a = 0.5

# Active overlay color (for active display, optional)
[active_overlay_color]
r = 50
g = 100
b = 255
a = 0.15

# Saved profiles (two default profiles: "light-mode" and "dark-mode")
[profiles.light-mode]
is_dimming_enabled = true
is_active_overlay_enabled = false
is_partial_dimming_enabled = true

[profiles.light-mode.overlay_color]
r = 0
g = 0
b = 0
a = 0.5

[profiles.dark-mode]
is_dimming_enabled = true
is_active_overlay_enabled = true
is_partial_dimming_enabled = true

[profiles.dark-mode.overlay_color]
r = 0
g = 0
b = 0
a = 0.7

[profiles.dark-mode.active_overlay_color]
r = 0
g = 0
b = 0
a = 0.3
```

## Architecture

### Core Application (`spotlight-dimmer.exe`)

- **Memory usage**: ~7.6 MB
- **Binary size**: 561 KB
- **Implementation**: Pure Windows API with Rust `winapi` crate
- **Overlay technology**: Layered windows (`WS_EX_LAYERED`) with alpha blending
- **Focus monitoring**: 100ms polling using `GetForegroundWindow()` and `MonitorFromWindow()`

### Configuration Tool (`spotlight-dimmer-config.exe`)

- **Binary size**: 627 KB
- **Implementation**: CLI tool using `clap` for argument parsing
- **Configuration**: TOML format via `toml` crate

### Key Technical Details

- **Click-through**: `WS_EX_TRANSPARENT` flag ensures overlays don't capture input
- **Always on top**: `WS_EX_TOPMOST` keeps overlays above other windows
- **No taskbar**: `WS_EX_TOOLWINDOW` prevents overlays from appearing in Alt+Tab
- **No focus**: `WS_EX_NOACTIVATE` prevents overlays from stealing focus
- **Transparency**: `SetLayeredWindowAttributes()` with `LWA_ALPHA` for smooth dimming

## Development

### Project Structure

```
.
├── src/
│   ├── main_new.rs          # Main application entry point
│   ├── config_cli.rs        # Configuration CLI tool
│   ├── config.rs            # Configuration system (TOML)
│   ├── overlay.rs           # WinAPI overlay implementation
│   └── platform/
│       ├── mod.rs           # Cross-platform traits
│       └── windows.rs       # Windows display/window management
├── Cargo.toml               # Rust dependencies
└── target/release/          # Build output
```

### Building

```bash
cargo build --release
```

### Dependencies

- `serde` - Configuration serialization
- `toml` - TOML configuration parsing
- `winapi` - Windows API bindings

## License

MIT

## Credits

Developed by **Thomaz Moura** with most of the code generated using AI-assisted development tools:
- [Claude Code](https://claude.ai/code) (Anthropic)
- [Codex CLI](https://github.com/thomazmoura/codex-cli) (OpenAI)

Built with Rust and the Windows API for maximum performance.
