# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Spotlight Dimmer is a cross-platform application that dims inactive displays to highlight the active one. Currently Windows-only but designed for future cross-platform support. It helps users with multiple monitors focus by dimming all displays except the one with the currently focused window.

## Installation & Build

Spotlight Dimmer is a lightweight native Windows application built with pure Rust and Windows API.

### Building from Source

```bash
cargo build --release --bin spotlight-dimmer --bin spotlight-dimmer-config
```

This creates two binaries in `target/release/`:
- `spotlight-dimmer.exe` (561 KB) - Main application
- `spotlight-dimmer-config.exe` (627 KB) - Configuration tool

### Installation

**Option 1: Windows installer (Recommended)**
- Download the latest `spotlight-dimmer-v*-installer.exe` from GitHub Releases
- Run the installer to deploy both executables, required icons, Start Menu entries, and the uninstaller entry in Windows Settings
- Uninstall via Windows Settings → Apps → Installed apps → Spotlight Dimmer (or the Start Menu uninstaller shortcut)

**Option 2: Install via npm (Alternative)**
```bash
npm install -g spotlight-dimmer
```
- Provides pre-built binaries without compiling locally
- Works on Windows x64 with Node.js 14+
- Adds `spotlight-dimmer` and `spotlight-dimmer-config` commands to the PATH

**Option 3: Manual installation (Cargo)**
```bash
# Install binaries from source
cargo install --path . --bin spotlight-dimmer --bin spotlight-dimmer-config

# Copy icon files to the installation directory (required)
cp spotlight-dimmer-icon.ico spotlight-dimmer-icon-paused.ico ~/.cargo/bin/
```

Alternatively, you can run the PowerShell helper script:
```powershell
.\install.ps1
```

**Icon Management:**
- Both icon files must live alongside the executables (`spotlight-dimmer-icon.ico` and `spotlight-dimmer-icon-paused.ico`)
- During development: `build.rs` copies icons to `target/release/` or `target/debug/`
- After `cargo install`: copy icons to `~/.cargo/bin/` manually (or use `install.ps1`)
- Icon names are unique to avoid conflicts with other binaries in `~/.cargo/bin/`

### Uninstallation

**Option 1: Using the uninstallation script (Recommended - Windows)**
```powershell
.\uninstall.ps1
```

**Option 2: Manual uninstallation**
```bash
# Uninstall binaries
cargo uninstall spotlight-dimmer

# Remove icon files from installation directory
rm ~/.cargo/bin/spotlight-dimmer-icon.ico ~/.cargo/bin/spotlight-dimmer-icon-paused.ico
```

### Stopping the Application

```powershell
Get-Process spotlight-dimmer | Stop-Process
```

### Setting Up Development Environment

**Automatic Git Hooks Setup**

Git hooks are automatically installed when you run any of these commands:
```bash
cargo test
cargo build
cargo check
```

**How it works:**
- `cargo-husky` is configured as a dev dependency
- Hooks are stored in `.cargo-husky/hooks/` (version-controlled)
- First time you run `cargo test` or `cargo build`, hooks are automatically installed to `.git/hooks/`
- **No manual setup required!**

**What the pre-commit hook does:**
- Runs `cargo fmt --check` to verify code formatting
- Runs `cargo clippy` to catch common mistakes and improve code quality
- Prevents commits with formatting or linting issues
- Provides immediate feedback before code reaches CI

**Benefits:**
- Zero manual setup - just clone and build
- All developers automatically get the same hooks
- Hooks are version-controlled in `.cargo-husky/hooks/`
- Updates to hooks propagate automatically on next build

## Architecture Overview

**Pure Windows API Implementation** - No web framework, no browser engine, just native code.

### Core Application (`main_new.rs`)
- **Entry Point**: `src/main_new.rs` - Main application loop
- **Memory Usage**: ~7.6 MB RAM, 561 KB binary
- **Focus Monitoring**: 100ms polling using `GetForegroundWindow()`
- **Display Detection**: Real-time hotplug support via `EnumDisplayMonitors()`

### Configuration System (`config.rs`)
- **Format**: TOML (human-readable text format)
- **Location**: `%APPDATA%\spotlight-dimmer\config.toml`
- **Settings**: Overlay color (RGB + alpha), dimming enabled/disabled
- **Auto-loading**: Configuration loaded on startup

### Configuration CLI (`config_cli.rs`)
- **Purpose**: Standalone tool for managing settings
- **Commands**: `status`, `enable`, `disable`, `color`, `reset`
- **Binary Size**: 627 KB
- **No overhead**: Doesn't affect main application performance

### Platform Layer (`platform/`)
- **Cross-platform traits**: `DisplayManager`, `WindowManager`
- **Windows Implementation**: `src/platform/windows.rs`
- **APIs Used**: `EnumDisplayMonitors`, `GetForegroundWindow`, `MonitorFromWindow`
- **Display Info**: Position, size, primary status for each display

### Overlay System (`overlay.rs`)
- **Implementation**: Native Windows layered windows (`WS_EX_LAYERED`)
- **Transparency**: `SetLayeredWindowAttributes()` with alpha blending
- **Click-Through**: `WS_EX_TRANSPARENT` flag ensures no input capture
- **Always On Top**: `WS_EX_TOPMOST` keeps overlays above all windows
- **No Taskbar**: `WS_EX_TOOLWINDOW` prevents Alt+Tab appearance
- **No Focus**: `WS_EX_NOACTIVATE` prevents focus stealing
- **Window Class**: Custom registered class "SpotlightDimmerOverlay"

### Key Implementation Details

#### Overlay Creation
```rust
CreateWindowExW(
    WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST |
    WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
    class_name, window_name, WS_POPUP,
    x, y, width, height, ...
);
SetLayeredWindowAttributes(hwnd, colorref, alpha, LWA_ALPHA);
```

#### Focus Monitoring Loop
- Polls active window every 100ms
- Compares window handle and display ID with last known state
- Updates overlay visibility (hide on active display, show on others)
- Detects display configuration changes (connect/disconnect)
- Automatically recreates overlays when display count changes

## Development Notes

### Platform Support
- Currently Windows-only implementation
- Linux support prepared but not implemented (see commented code in `platform/mod.rs`)
- Cross-platform traits designed for easy platform additions

### Dependencies
- **serde**: Configuration serialization/deserialization
- **toml**: TOML configuration file parsing
- **winapi**: Windows API bindings (minimal feature set)

### Build Requirements
- Rust 1.77.2+
- Windows development environment
- No additional tools required (no Tauri CLI, no Node.js, no frontend build)

### Build Times
- **Initial build**: ~10-15 seconds (minimal dependencies)
- **Incremental builds**: ~1-2 seconds
- **Clean rebuild**: ~10-15 seconds
- **Much faster than Tauri version**: No web framework compilation overhead

### Configuration
- User settings: `%APPDATA%\spotlight-dimmer\config.toml` (TOML format)
- Build config: `src/Cargo.toml` (Rust dependencies only)

### Debugging
- Console output shows focus monitoring and overlay operations
- Use `println!` or `eprintln!` for debug logging
- Task Manager shows accurate memory usage (~7.6 MB)

## Common Development Patterns

### Platform-Specific Code
- Use `#[cfg(windows)]` for Windows-only code
- Implement platform traits in respective platform modules
- Keep cross-platform types in `platform/mod.rs`

### Overlay Management
- Overlays stored in `HashMap<String, HWND>` keyed by display ID
- Always close existing overlays before creating new ones to prevent leaks
- Use `ShowWindow(SW_HIDE/SW_SHOW)` to toggle visibility
- Register window class once at startup with `RegisterClassExW`

### Adding Configuration Options
1. Update `Config` struct in `config.rs`
2. Add field to TOML serialization
3. Update `Config::default()` with sensible default
4. Add CLI command in `config_cli.rs` if user-facing
5. Update main loop in `main_new.rs` to use new setting

## Implementation Notes & Troubleshooting

### Click-Through Implementation
The application uses Windows API directly for click-through overlays:

- **Windows API**: `WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW`
- **`WS_EX_TRANSPARENT`**: Passes all mouse/keyboard input through to underlying windows
- **`WS_EX_LAYERED`**: Enables alpha blending via `SetLayeredWindowAttributes()`
- **`WS_EX_TOOLWINDOW`**: Prevents Alt+Tab visibility and taskbar appearance
- **`WS_EX_NOACTIVATE`**: Prevents window from stealing focus
- **Status**: ✅ Works perfectly with native Windows API

### Window Movement Detection
Enhanced focus monitoring that tracks both window changes and display changes:

```rust
// Tracks both window changes and display changes
let window_changed = Some(active_window.handle) != last_window_handle;
let display_changed = last_display_id.as_ref() != Some(&active_window.display_id);

if window_changed || display_changed {
    // Update overlays for either type of change
}
```

## Tmux Pane Focusing (WSL Integration)

Spotlight Dimmer can dim inactive tmux panes within Windows Terminal (via WSL) to help you focus on the active pane. This works even when Windows Terminal is in fullscreen mode.

### How It Works

1. **tmux hook** writes active pane boundaries to a shared file when you switch panes
2. **Windows file watcher** detects changes instantly (event-driven, zero polling)
3. **Coordinate translator** converts tmux character coordinates to Windows pixels
4. **Overlay system** creates transparent dimming overlays over inactive pane areas

### Setup Instructions

#### Step 1: Enable tmux mode
```bash
spotlight-dimmer-config tmux-enable
```

#### Step 2: Configure tmux hook
Add this to your `~/.tmux.conf` (in WSL):

```bash
set-hook -g pane-focus-in 'run-shell "tmux display -p \"#{pane_left},#{pane_top},#{pane_right},#{pane_bottom},#{window_width},#{window_height}\" > ~/.spotlight-dimmer/tmux-active-pane.txt"'
```

This hook writes pane boundaries to `~/.spotlight-dimmer/tmux-active-pane.txt` whenever you switch panes.

#### Step 3: Reload tmux configuration
```bash
tmux source-file ~/.tmux.conf
```

#### Step 4: Configure terminal geometry

**Option A: Automatic configuration (Recommended)**

Let Spotlight Dimmer automatically detect settings from Windows Terminal:

```bash
# Auto-detect from default profile
spotlight-dimmer-config tmux-auto-config

# Auto-detect from specific profile
spotlight-dimmer-config tmux-auto-config "Ubuntu-22.04"

# Preview without saving
spotlight-dimmer-config tmux-auto-config --dry-run
```

This command:
- Reads your Windows Terminal settings.json
- Detects font size and calculates exact pixel dimensions using Windows API
- Extracts padding values
- Applies configuration automatically

**Option B: Manual configuration**

If auto-detection doesn't work or you prefer manual setup:

```bash
# Example: 9px wide characters, 20px tall characters, 0px left padding, 35px top padding (title bar)
spotlight-dimmer-config tmux-config 9 20 0 35
```

**How to find your terminal geometry:**
1. **Font size**: Check Windows Terminal settings → Profiles → Appearance → Font face and size
   - Typical monospace fonts at 12pt: ~9px wide, ~20px tall
   - At 10pt: ~8px wide, ~17px tall
2. **Padding**: Measure from window edge to first character
   - Top padding includes title bar (typically 30-40px)
   - Left/right padding is usually 0-5px

#### Step 5: Check status
```bash
spotlight-dimmer-config tmux-status
```

### Shared File Location

The pane boundary file must be accessible from both WSL and Windows:
- **WSL path**: `~/.spotlight-dimmer/tmux-active-pane.txt`
- **Windows path**: `C:\Users\{username}\.spotlight-dimmer\tmux-active-pane.txt`
- The directory is created automatically on first use

### Usage

Once configured, tmux pane focusing activates automatically when:
- ✅ Windows Terminal is the focused application
- ✅ tmux mode is enabled (`tmux-enable`)
- ✅ The pane boundary file exists and is up-to-date

Overlays are automatically cleared when:
- ❌ You switch away from Windows Terminal
- ❌ You disable tmux mode (`tmux-disable`)
- ❌ No active window detected

### CLI Commands

```bash
# Enable/disable tmux mode
spotlight-dimmer-config tmux-enable
spotlight-dimmer-config tmux-disable

# Configure terminal geometry
spotlight-dimmer-config tmux-config <font_width> <font_height> <padding_left> <padding_top>

# Check configuration
spotlight-dimmer-config tmux-status

# View full status including tmux settings
spotlight-dimmer-config status
```

### Compatibility

- **Terminal**: Windows Terminal only (detected via process name)
- **WSL**: Any version (WSL1 or WSL2)
- **tmux**: Version 2.1+ (requires pane_left/pane_top/pane_right/pane_bottom variables)
- **Modes**: Works in both windowed and fullscreen Windows Terminal

### Technical Details

**Coordinate Translation:**
- tmux uses character-based coordinates (columns and rows)
- Windows uses pixel-based coordinates
- Translation formula: `pixel = padding + (character * font_size)`

**File Format:**
The tmux hook writes comma-separated values:
```
pane_left,pane_top,pane_right,pane_bottom,window_width,window_height
```
Example: `0,0,119,29,240,60` (pane at columns 0-119, rows 0-29, in a 240x60 window)

**Performance:**
- Event-driven file watching (zero CPU overhead when idle)
- Overlays update only when pane changes
- Automatic cleanup on exit

### Troubleshooting

**Problem: Overlays don't appear**
- Check that tmux mode is enabled: `spotlight-dimmer-config tmux-status`
- Verify the pane file exists: `ls ~/.spotlight-dimmer/tmux-active-pane.txt` (in WSL)
- Check Windows path: `C:\Users\{username}\.spotlight-dimmer\tmux-active-pane.txt`
- Ensure Windows Terminal is the focused application

**Problem: Overlays are misaligned**
- Reconfigure terminal geometry with correct font size and padding
- Use `spotlight-dimmer-config tmux-config` with measured values
- Font size can be found in Windows Terminal settings

**Problem: Overlays don't update when switching panes**
- Verify tmux hook is active: `tmux show-hooks`
- Reload tmux config: `tmux source-file ~/.tmux.conf`
- Check file is being updated: `watch -n 0.5 cat ~/.spotlight-dimmer/tmux-active-pane.txt` (switch panes to test)

**Problem: Works in windowed mode but not fullscreen**
- This should work in both modes. If not, check that Windows Terminal process is detected
- Verify with: `tasklist | findstr WindowsTerminal`

**Problem: Auto-config fails to find settings.json**
- Ensure Windows Terminal is installed from Microsoft Store or GitHub
- Check settings file exists: `%LOCALAPPDATA%\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json`
- If using portable version, auto-config may not work (use manual `tmux-config` instead)

**Problem: Auto-config calculates wrong font size**
- Some fonts report inconsistent metrics - use `--dry-run` to preview before applying
- If metrics are wrong, use manual `tmux-config` with measured values
- Verify font is installed: Check Windows Settings → Fonts

## Troubleshooting

### Build Issues

#### Problem: "Access denied" during cargo install
**Solution**: The binary is currently running. Stop all instances first:
```bash
# Stop running instances
taskkill /F /IM spotlight-dimmer.exe

# Then reinstall
cd src
cargo install --path . --force
```

#### Problem: Compilation errors
**Solution**: Ensure you have the required dependencies:
```bash
# Check Rust version (1.77.2+)
rustc --version

# Update Rust if needed
rustup update stable
```

### Runtime Issues

#### Problem: Overlays not visible
**Diagnostics**: Check console output for:
- "[Overlay] Created for display..." messages
- Display count matches your actual monitors
- No error messages during overlay creation

**Solution**: Run in console to see debug output:
```bash
cd src/target/release
./spotlight-dimmer.exe
```

#### Problem: Focus monitoring not working
**Diagnostics**: Check console output for:
- "[Main] Focus monitoring loop started..."
- "[Main] Active window: ..." messages when switching windows
- Display ID changes when moving windows between monitors

**Solution**: Windows may require administrator privileges for some window tracking operations.

#### Problem: Click-through not working
**Solution**: This shouldn't happen with direct Windows API implementation. If it does:
- Check Windows version (Windows 7+)
- Ensure no antivirus is blocking window manipulation
- Run as administrator if needed

### Configuration Issues

#### Problem: Configuration not persisting
**Solution**: Check config file location:
```bash
# Config file should be at:
# %APPDATA%\spotlight-dimmer\config.toml

# View current config
spotlight-dimmer-config status

# Reset to defaults
spotlight-dimmer-config reset
```

## Changelog Management (REQUIRED)

**CRITICAL**: Every code change, feature addition, bug fix, or improvement MUST be documented in `CHANGELOG.md`. This is not optional.

### When to Update the Changelog

Update `CHANGELOG.md` for ANY of these changes:
- ✅ New features or functionality
- ✅ Bug fixes and issue resolutions
- ✅ Breaking changes or API modifications
- ✅ Performance improvements
- ✅ UI/UX enhancements
- ✅ Configuration changes
- ✅ Dependency updates (if user-facing)
- ✅ Security fixes
- ✅ Documentation improvements (if significant)

### Changelog Format

Follow [Keep a Changelog](https://keepachangelog.com/) format. Always add entries under the `## [Unreleased]` section:

#### For New Features
```markdown
### Added
- Feature name: Clear description of what it does and why it's useful for users
- Another feature: Focus on user-facing benefits, not internal implementation details
```

#### For Bug Fixes
```markdown
### Fixed
- Issue description: What was broken and how it affects users
- Bug name: Clear explanation of the fix and its impact
```

#### For Breaking Changes
```markdown
### Changed
- Breaking change description: What changed and why
- Migration steps: If users need to take action, explain how
```

#### For Performance/Internal Improvements
```markdown
### Improved
- Performance enhancement: Measurable impact on user experience
- Internal optimization: Only if it affects user-visible behavior
```

#### For Removed Features
```markdown
### Removed
- Deprecated feature: What was removed and why
- Alternative solution: What users should use instead
```

### Portuguese Translation Requirement (MANDATORY)

**CRITICAL**: Every changelog entry MUST include a Portuguese translation. This is required for all entries without exception.

#### Format Structure
Each changelog entry must follow this bilingual format:
```markdown
### Added
- Feature name: Clear description of what it does and why it's useful for users
- Another feature: Focus on user-facing benefits, not internal implementation details

---

### Adicionado
- Nome da funcionalidade: Descrição clara do que faz e por que é útil para os usuários
- Outra funcionalidade: Foque nos benefícios voltados ao usuário, não em detalhes de implementação interna
```

#### Translation Guidelines
1. **Maintain Technical Accuracy**: Ensure technical terms are correctly translated or kept in English when appropriate
2. **User-Friendly Language**: Use Portuguese that Brazilian and Portuguese users can easily understand
3. **Consistent Terminology**: Keep consistent translations for recurring technical terms
4. **Section Headers**: Always translate section headers (Added→Adicionado, Fixed→Corrigido, Changed→Alterado, etc.)

#### Section Header Translations
- **Added** → **Adicionado**
- **Fixed** → **Corrigido**
- **Changed** → **Alterado**
- **Improved** → **Melhorado**
- **Removed** → **Removido**
- **Security** → **Segurança**
- **Deprecated** → **Obsoleto**

### Changelog Writing Guidelines

1. **User-Focused**: Write for end users, not developers
2. **Clear Impact**: Explain what changed and why it matters
3. **Actionable**: Include migration steps for breaking changes
4. **Specific**: Use concrete examples rather than vague descriptions
5. **Consistent**: Follow the same style and format for all entries
6. **Bilingual**: Always include Portuguese translations using the format above

### Example Entry
```markdown
### Added
- Dark mode toggle: Users can now switch between light and dark themes via the system tray menu
- Keyboard shortcuts: Added Ctrl+D to toggle dimming and Ctrl+Q to quit application
- Multi-monitor performance: Reduced CPU usage by 40% when managing 3+ displays

### Fixed
- Display detection bug: Application now properly detects displays after sleep/wake cycles
- Memory leak: Fixed overlay windows not being properly disposed when displays are disconnected

### Changed
- Overlay transparency: Changed default dimming from 30% to 50% for better visibility (users can adjust in settings)

---

### Adicionado
- Alternância de modo escuro: Os usuários agora podem alternar entre temas claro e escuro através do menu da bandeja do sistema
- Atalhos de teclado: Adicionado Ctrl+D para alternar o escurecimento e Ctrl+Q para sair da aplicação
- Performance multi-monitor: Redução de 40% no uso de CPU ao gerenciar 3+ displays

### Corrigido
- Bug de detecção de display: A aplicação agora detecta adequadamente displays após ciclos de suspensão/despertar
- Vazamento de memória: Corrigida a disposição inadequada das janelas de sobreposição quando displays são desconectados

### Alterado
- Transparência de sobreposição: Alterada transparência padrão de 30% para 50% para melhor visibilidade (usuários podem ajustar nas configurações)
```

### Release Process Integration

The changelog directly feeds into GitHub Releases:
- Release notes are automatically generated from the `[Unreleased]` section
- This ensures comprehensive, professional release documentation
- No manual release note writing required

### Quality Standards

Each changelog entry should answer:
- **What** changed?
- **Why** did it change?
- **How** does it affect users?
- **What** should users do (if action required)?
- **Is the Portuguese translation accurate and user-friendly?**

**Enforcement**: Pull requests without proper changelog updates (including Portuguese translations) will be considered incomplete.

## Version Management

Version management is handled automatically by the `/commit` slash command, which:
1. Increments the patch version (third number) in both `package.json` and `src/Cargo.toml`
2. Creates a commit with all changes
3. Creates a git tag with the new version
4. Pushes both commit and tag to the repository

To create a versioned commit, use: `/commit`

## Git Commit Messages

**DO NOT** include "Generated with Claude Code" or "Co-Authored-By: Claude" in commit messages. This repository is built with Claude Code - these attributions are redundant.

Use clear, descriptive commit messages:
- Subject line (50 chars max)
- Brief explanation of what changed and why
- List specific changes if multiple
