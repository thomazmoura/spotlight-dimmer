# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Spotlight Dimmer is a cross-platform application that dims inactive displays to highlight the active one. Currently Windows-only but designed for future cross-platform support. It helps users with multiple monitors focus by dimming all displays except the one with the currently focused window.

## Installation & Build Methods

Spotlight Dimmer supports **two distinct installation methods**, each with different characteristics and use cases:

### Method 1: Tauri Builds (Recommended for Full Features)
Full GUI application with perfect transparency and all features.

#### Development
```bash
# Start development server with hot reload
cargo tauri dev
```

#### Production Build
```bash
# Build for production (creates installer/executable)
cargo tauri build
```

#### Testing Frontend Changes
```bash
# Serve the frontend files locally for testing (Python required)
python -m http.server 1420 --directory dist
```

### Method 2: Cargo Install (Portable Installation)
Direct installation via Cargo package manager for developers.

#### Installation
```bash
# Install from local source
cd src-tauri && cargo install --path .

# Install from crates.io (when published)
cargo install spotlight-dimmer
```

#### Uninstallation
```bash
cargo uninstall spotlight-dimmer
```

### Key Differences Between Installation Methods

| Feature | Tauri Build | Cargo Install |
|---------|-------------|---------------|
| **Installation** | Requires Tauri CLI + build | Standard `cargo install` |
| **Transparency** | ✅ Perfect (file-based overlays) | ⚠️ Limited (data URL fallback) |
| **File Size** | Larger (includes webview) | Smaller (embedded assets) |
| **Dependencies** | Requires `dist/` files | Self-contained binary |
| **Use Case** | End users, full GUI | Developers, automation |
| **Update Method** | Rebuild + reinstall | `cargo install --force` |

### ⚠️ CRITICAL BUILD COMPATIBILITY WARNINGS

**DO NOT make changes that break either installation method:**

1. **Never remove the `dist/` directory** - Required for Tauri builds
2. **Never remove embedded asset fallbacks** - Required for cargo install
3. **Never change overlay creation logic** without testing both methods
4. **Always test both installation methods** before committing changes
5. **Preserve the hybrid overlay approach** in `lib.rs:create_overlay_window()`

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
- **Click-Through Implementation**: Uses Tauri's native `set_ignore_cursor_events(true)` API (preferred method)
- **Fallback**: Windows API with `WS_EX_TRANSPARENT`, `WS_EX_LAYERED`, and `WS_EX_TOOLWINDOW` flags
- **Auto-startup**: Application starts with dimming enabled by default

#### Hybrid Overlay Loading System
The overlay system uses a **hybrid approach** to support both installation methods:

1. **Primary Method (File-based)**: `WebviewUrl::App("overlay.html".into())`
   - Used when `dist/overlay.html` exists (Tauri builds)
   - Provides perfect transparency with `rgba(0, 0, 0, 0.5)` dimming
   - Preserves all CSS styling and functionality

2. **Fallback Method (Embedded)**: Data URL with embedded content
   - Used when file-based loading fails (cargo install)
   - Serves embedded HTML from `build.rs` generated assets
   - May have transparency limitations on Windows due to data URL constraints

```rust
// Implementation in lib.rs:create_overlay_window()
let window_result = WebviewWindowBuilder::new(app_handle, &format!("{}_file", overlay_id), WebviewUrl::App("overlay.html".into()))
    .build();

let window = match window_result {
    Ok(win) => win,  // File-based overlay (preferred)
    Err(_) => {
        // Fallback to embedded content
        let overlay_html_content = get_asset("overlay.html").unwrap_or(OVERLAY_HTML);
        let data_url = format!("data:text/html;charset=utf-8,{}", urlencoding::encode(overlay_html_content));
        WebviewWindowBuilder::new(app_handle, overlay_id, WebviewUrl::External(data_url.parse().unwrap()))
            .build()?
    }
};
```

**⚠️ Critical**: Never modify this hybrid loading logic without testing both installation methods!

### Tauri Commands
- `get_displays()`: Returns list of all displays with positioning info
- `get_active_window()`: Returns currently focused window information
- `toggle_dimming()`: Enables/disables dimming functionality
- `is_dimming_enabled()`: Returns current dimming state

### Focus Monitoring
- Background thread continuously monitors active window changes (100ms polling interval)
- **Enhanced Detection**: Tracks both window handle changes AND display changes for comprehensive monitoring
- **Window Movement**: Detects when windows move between displays, updating overlays immediately
- Skips self-owned overlay windows to prevent focus loops
- Uses `MonitorFromWindow` Windows API to determine display association
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

### ⚠️ RUST BUILD TIME CONSIDERATIONS

**IMPORTANT for AI Agents**: This is a Rust project with complex dependencies. Build times are significant:

#### Expected Build Times
- **Development builds** (`cargo tauri dev`): 60-120 seconds initial compile, faster on subsequent runs
- **Production builds** (`cargo tauri build`): 90-180 seconds including bundling
- **Cargo installs** (`cargo install --path .`): 30-60 seconds
- **Clean rebuilds**: Can take 2-3 minutes for full compilation

#### Agent Patience Guidelines
1. **Use longer timeouts**: Set timeouts to at least 120-180 seconds for build commands
2. **Monitor background processes**: Use `run_in_background=true` for build commands and monitor with `BashOutput`
3. **Wait for compilation**: Don't interrupt builds - Rust compilation is CPU-intensive but reliable
4. **Expect warnings**: The project generates several harmless warnings about unused imports/functions
5. **Trust the process**: Builds will complete successfully - compilation messages indicate progress

#### Build Command Best Practices
```bash
# Use extended timeouts for build commands
cargo tauri dev --timeout=180000    # 3 minutes
cargo tauri build --timeout=300000  # 5 minutes
cargo install --path . --timeout=120000  # 2 minutes
```

**Note**: Build times vary based on hardware, but patience is key - Rust's compilation model ensures correct builds once complete.

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
- **Auto-startup Configuration**: `AppState::default()` initializes with `is_dimming_enabled: true`
- **Startup Sequence**: 500ms delay ensures proper initialization before overlay creation

## Implementation Notes & Troubleshooting

### Click-Through Implementation
The application successfully implements click-through overlays using multiple approaches:

#### Primary Method (Recommended)
- **Tauri Native API**: `window.set_ignore_cursor_events(true)`
- **Status**: ✅ Working reliably in Tauri 2.x
- **Advantages**: Clean, cross-platform, no manual Windows API calls needed

#### Fallback Method
- **Windows API**: Direct manipulation using `SetWindowLongPtrW`
- **Flags**: `WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW`
- **Usage**: Automatically attempted if Tauri API fails
- **Note**: `WS_EX_TOOLWINDOW` prevents Alt+Tab visibility and focus issues

### Window Movement Detection
Enhanced focus monitoring system that goes beyond simple window handle tracking:

```rust
// Tracks both window changes and display changes
let window_changed = Some(active_window.handle) != last_window_handle;
let display_changed = last_display_id.as_ref() != Some(&active_window.display_id);

if window_changed || display_changed {
    // Update overlays for either type of change
}
```

## Installation Troubleshooting

### Cargo Install Issues

#### Problem: "Access denied" during installation
```bash
error: failed to move spotlight-dimmer.exe
Caused by: Acesso negado. (os error 5)
```
**Solution**: The binary is currently running. Stop all instances first:
```bash
cargo uninstall spotlight-dimmer
# or manually close any running spotlight-dimmer processes
cargo install --path . --force
```

#### Problem: "webview-data-url feature not found"
**Cause**: Missing Tauri feature for data URL support in embedded content fallback.
**Solution**: Ensure `Cargo.toml` includes:
```toml
tauri = { version = "2.8.5", features = ["tray-icon", "devtools", "webview-data-url"] }
```

#### Problem: Overlays appear completely transparent (no dimming)
**Cause**: Using data URL fallback which has Windows transparency limitations.
**Solutions**:
1. Use Tauri build instead: `cargo tauri build`
2. Verify `dist/overlay.html` exists for file-based loading
3. Check console output for "File-based overlay not found" message

### Tauri Build Issues

#### Problem: "Frontend dist directory not found"
**Solution**: Ensure `dist/` directory exists with required files:
```bash
ls dist/  # Should show: index.html, overlay.html, style.css, main.js
```

#### Problem: Overlays not showing proper transparency
**Solution**: Verify `dist/overlay.html` contains correct CSS:
```css
body, html {
    background-color: rgba(0, 0, 0, 0.5);  /* 50% transparent black */
    pointer-events: none;  /* Essential for click-through */
    overflow: hidden;
}
```

### General Issues

#### Problem: Focus monitoring not working
**Diagnostics**: Check console output for:
- "Focus monitoring thread started!"
- "Active window changed: [window name]"
- "Successfully emitted focus-changed event"

#### Problem: Click-through not working
**Solution**: Verify console shows:
- "Successfully set ignore cursor events with Tauri API"
- If not, check Windows permissions and Tauri version

### Testing Both Installation Methods

Before making any changes, always test both methods:

```bash
# Test Tauri build
cargo tauri dev
# Verify transparency and functionality

# Test cargo install
cd src-tauri
cargo install --path . --force
spotlight-dimmer
# Verify it works (may have limited transparency)

# Clean up
cargo uninstall spotlight-dimmer
```

### Build System Maintenance

#### Embedded Assets (`build.rs`)
- Automatically embeds `dist/` files into binary
- Provides fallback HTML/CSS for cargo install
- Regenerates on `dist/` changes

#### Dependencies for Both Methods
- **Tauri builds**: Requires Tauri CLI, `dist/` files
- **Cargo install**: Requires `webview-data-url` feature, embedded assets
- **Both**: Windows API dependencies, Rust 1.77.2+

**Remember**: Any changes to overlay creation logic in `lib.rs:create_overlay_window()` must be tested with both installation methods to ensure compatibility!