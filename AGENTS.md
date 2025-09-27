# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Spotlight Dimmer is a cross-platform application that dims inactive displays to highlight the active one. Currently Windows-only but designed for future cross-platform support. It helps users with multiple monitors focus by dimming all displays except the one with the currently focused window.

## Build Commands

### Development
```bash
# Start development server with hot reload
cargo tauri dev
```

### Building
```bash
# Build for production
cargo tauri build
```

### Testing Frontend Changes
```bash
# Serve the frontend files locally for testing (Python required)
python -m http.server 1420 --directory dist
```

## Architecture Overview

### Backend (Rust/Tauri)
- **Main Application Logic**: `src-tauri/src/lib.rs` - Contains core dimming functionality, overlay management, and Tauri commands
- **Platform Abstraction**: `src-tauri/src/platform/` - Abstracts platform-specific display and window management
- **Windows Implementation**: `src-tauri/src/platform/windows.rs` - Windows API integration for display enumeration and window tracking
- **Entry Point**: `src-tauri/src/main.rs` - Simple entry point that calls the main lib

### Frontend (HTML/CSS/JS)
- **Main UI**: `dist/index.html` - Primary control interface with status display and toggle controls
- **Overlay**: `dist/overlay.html` - Transparent overlay windows for dimming inactive displays
- **Styling**: `dist/style.css` - All UI styling
- **JavaScript**: `dist/main.js` - Frontend logic for Tauri communication and UI updates

### Key Components

#### State Management (`lib.rs`)
- `AppState` struct manages global application state including overlay windows, dimming status, and active display tracking
- Thread-safe using `Arc<Mutex<>>` for concurrent access
- Focus monitoring runs in background thread with 100ms polling interval

#### Platform Layer (`platform/`)
- `DisplayManager` trait: Cross-platform display enumeration and information
- `WindowManager` trait: Active window detection and display association
- Windows implementation uses WinAPI for display/window management

#### Overlay System
- Creates transparent, click-through overlay windows on each display
- Overlays are hidden on active display, shown on inactive displays
- Uses Windows Extended Window Styles (`WS_EX_TRANSPARENT`, `WS_EX_LAYERED`) for click-through behavior

### Tauri Commands
- `get_displays()`: Returns list of all displays with positioning info
- `get_active_window()`: Returns currently focused window information
- `toggle_dimming()`: Enables/disables dimming functionality
- `is_dimming_enabled()`: Returns current dimming state

### Focus Monitoring
- Background thread continuously monitors active window changes
- Skips self-owned overlay windows to prevent focus loops
- Emits `focus-changed` events to frontend when active display changes
- Updates overlay visibility based on active display

## Development Notes

### Platform Support
- Currently Windows-only implementation
- Linux support prepared but not implemented (see commented code in `platform/mod.rs`)
- Cross-platform traits designed for easy platform additions

### Dependencies
- **Tauri 2.x**: Main framework for desktop app
- **WinAPI**: Windows system integration
- **Tokio**: Async runtime for Tauri commands
- **Serde**: JSON serialization for frontend communication

### Build Requirements
- Rust 1.77.2+
- Tauri CLI (`cargo install tauri-cli --version ^2.0.0`)
- Windows development environment for Windows features

### Configuration
- Tauri config: `src-tauri/tauri.conf.json`
- Cargo config: `src-tauri/Cargo.toml`

### Debugging
- Tauri supports devtools for frontend debugging
- Rust logs via `env_logger` and `tauri-plugin-log`
- Console output for focus monitoring and overlay operations

## Common Development Patterns

### Adding New Tauri Commands
1. Define async function in `lib.rs` with `#[tauri::command]` attribute
2. Add to `invoke_handler` in `run()` function
3. Call from frontend using `window.__TAURI__.core.invoke()`

### Platform-Specific Code
- Use `#[cfg(windows)]` for Windows-only code
- Implement platform traits in respective platform modules
- Keep cross-platform types in `platform/mod.rs`

### Overlay Management
- All overlay operations are async and use the shared `AppState`
- Overlays are keyed by display ID in the global HashMap
- Always close existing overlays before creating new ones to prevent leaks