# Spotlight Dimmer

## Overview

A lightweight Windows application that dims inactive displays to highlight the active one. Built with pure Rust and Windows API for maximum performance and minimal resource usage.

> Para a versão em português vá para: [LEIAME.md](LEIAME.md)

Spotlight Dimmer is a program for Windows that dims all the monitors other than the monitor that has the currently focused program.

It's intended to help people who use multiple monitores to focus and assist in quickly noticing which window has the current focus while changing focus with shortcuts like `alt + tab`. It's specially useful for users who navigate mainly with the keyboard. It helps to avoid silly situations like typing terminal commands on Teams because you're looking at a screen while the focus is on the other screen.

## Features

- **Ultra-lightweight**: Only ~7.6 MB RAM usage, ~561 KB binary size
- **Native Windows API**: No browser engine overhead, instant startup
- **Perfect transparency**: Smooth 50% dimming (customizable) on inactive displays
- **Click-through overlays**: Overlays don't interfere with mouse/keyboard input
- **Automatic focus tracking**: Detects active window and display changes in real-time (100ms polling)
- **Display hotplug support**: Automatically recreates overlays when displays are connected/disconnected
- **Persistent configuration**: Settings saved in TOML format at `%APPDATA%\spotlight-dimmer\config.toml`
- **CLI configuration tool**: Manage settings without running the main application

## Installation

### From Source

```bash
cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config
```

Binaries will be in `target\release\`:
- `spotlight-dimmer.exe` - Main application
- `spotlight-dimmer-config.exe` - Configuration tool

### Usage

#### Running the Application

Simply run `spotlight-dimmer.exe`:

```cmd
spotlight-dimmer.exe
```

The application will:
1. Load configuration from `%APPDATA%\spotlight-dimmer\config.toml` (or create default)
2. Detect all connected displays
3. Create semi-transparent overlay windows on each display
4. Monitor active window focus and hide overlay on active display
5. Run indefinitely until terminated

#### Stopping the Application

To stop the running application, use PowerShell:

```powershell
Get-Process spotlight-dimmer | Stop-Process
```

Or use Task Manager to end the `spotlight-dimmer.exe` process.

#### Configuration Tool

Use `spotlight-dimmer-config.exe` to manage settings:

```cmd
# Show current configuration
spotlight-dimmer-config status

# Enable/disable dimming
spotlight-dimmer-config enable
spotlight-dimmer-config disable

# Set overlay color (RGB 0-255, alpha 0.0-1.0)
spotlight-dimmer-config color 0 0 0 0.7      # 70% black overlay
spotlight-dimmer-config color 50 50 50 0.3   # 30% gray overlay

# Reset to defaults
spotlight-dimmer-config reset
```

**Note**: Configuration changes are automatically detected and reloaded within 2 seconds. No restart needed!

## Configuration File

Configuration is stored at `%APPDATA%\spotlight-dimmer\config.toml`:

```toml
is_dimming_enabled = true

[overlay_color]
r = 0
g = 0
b = 0
a = 0.5
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

## Comparison with Tauri Version

| Metric | Tauri v0.1.8 | WinAPI v0.1.9 | Improvement |
|--------|--------------|---------------|-------------|
| Binary Size | 10.1 MB | 561 KB | ~95% reduction |
| Memory Usage | ~200 MB | ~7.6 MB | ~96% reduction |
| Startup Time | ~400ms | Instant | N/A |
| Dependencies | 30+ crates | 3 crates | Minimal |
| Runtime Deps | WebView2 | None | Self-contained |

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

## Known Limitations

### Window Dragging Behavior

When dragging windows between monitors with the mouse, overlays are temporarily hidden to prevent system instability:

- **During drag**: All overlays disappear when the left mouse button is pressed
- **After drag**: Overlays reappear with correct visibility when the mouse button is released
- **Why**: Windows' drag-and-drop message loop conflicts with overlay visibility updates, causing system instability if overlays remain visible
- **Workaround**: Use keyboard shortcuts (Win+Arrow keys) for instant overlay updates without hiding

This is a Windows API limitation, not a bug. Focus changes and keyboard-based window movement work instantly without hiding overlays.

## Roadmap

- [ ] System tray icon (optional, using `trayicon` crate)
- [x] Hot reload configuration without restart (2-second detection window)
- [ ] Per-display color customization
- [ ] Linux support (using X11/Wayland)

## License

MIT

## Credits

Built with Rust and the Windows API for maximum performance.
