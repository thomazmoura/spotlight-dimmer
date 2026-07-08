# SpotlightDimmer

A utility that creates semi-transparent overlays to dim inactive displays or regions, creating a "spotlight" effect on the active window. Available for **Windows** (built with .NET 10 and native Windows APIs) and **Linux/Wayland** (a shared Rust daemon with adapters for GNOME Shell and KDE Plasma 6).

## Features

✅ **Multi-monitor support** - Automatically detects all connected monitors, including hot-plug
✅ **Three dimming modes** - FullScreen, Partial, and PartialWithActive on all platforms
✅ **Configurable overlays** - Customize colors and opacity for both inactive and active regions independently
✅ **Click-through overlays** - Overlays don't capture mouse input
✅ **100% event-driven** - Uses window-manager events instead of polling for zero CPU usage when idle
✅ **No admin privileges** - Runs as a regular user process
✅ **Hot-reloadable configuration** - Changes apply instantly without restart
✅ **Toggle shortcut** - Pause/resume all dimming with a keyboard shortcut
✅ **tmux pane spotlight (Linux)** - Optionally follow the focused tmux pane inside WezTerm instead of the whole terminal window

Windows-specific:

✅ **Multiple rendering backends** - Choose between:
  - **LayeredWindow**: Extremely lightweight (< 10MB RAM usage)
  - **CompositeOverlay**: Better visual quality during window dragging (~50MB RAM for dual monitor setup with partial overlays)
✅ **Small footprint** - Less than 50MB installed, installer under 10MB
✅ **Native AOT compilation** - Fast startup and minimal runtime dependencies

## Installation

### Windows

Install via winget:

```powershell
winget install ThomazMoura.SpotlightDimmer
```

Or download the installer from the [latest GitHub release](https://github.com/thomazmoura/spotlight-dimmer/releases/latest).

To build from source instead, see [Building](#building) below.

### Linux (Ubuntu / Kubuntu, Wayland)

The Linux version consists of a shared daemon (`spotlight-dimmer-daemon`) plus a thin adapter for your desktop: a GNOME Shell extension on Ubuntu, or a KWin script on Kubuntu. See [docs/LINUX_DAEMON.md](docs/LINUX_DAEMON.md) for architecture details.

**Requirements:** a Wayland session (the default on recent Ubuntu and Kubuntu) with GNOME Shell 45–48 (Ubuntu 24.04 or newer) **or** KDE Plasma 6 (Kubuntu 24.10 or newer).

#### .deb packages (recommended)

Download the package for your desktop and architecture from the newest Linux release on the [releases page](https://github.com/thomazmoura/spotlight-dimmer/releases) (Linux release tags end in `-linux`), then install it with apt so runtime dependencies are resolved automatically:

```bash
# Ubuntu (GNOME) — use the _arm64.deb files on ARM devices
sudo apt install ./spotlight-dimmer-gnome_<version>_amd64.deb

# Kubuntu (KDE Plasma 6)
sudo apt install ./spotlight-dimmer-kde_<version>_amd64.deb
```

The two packages intentionally conflict with each other — install the one matching your desktop.

**On GNOME**, log out and back in (Wayland cannot reload GNOME Shell in place), then enable the extension:

```bash
gnome-extensions enable spotlightdimmer@thomazmoura.github.io
```

**On KDE**, KWin loads the script automatically — no logout needed. To get the dimming toggle shortcut, open System Settings → Shortcuts, add the "SpotlightDimmer Toggle" application and bind it to **Meta+Shift+D**.

**First run**: packages cannot write to your home directory, so copy the starter configuration once (dimming uses FullScreen mode otherwise, which shows nothing on a single monitor):

```bash
mkdir -p ~/.config/SpotlightDimmer
cp /usr/share/doc/spotlight-dimmer-gnome/examples/config.example.json ~/.config/SpotlightDimmer/config.json
```

(Use `spotlight-dimmer-kde` in the path if you installed the KDE package. The tmux integration tools land in `/usr/share/spotlight-dimmer/tools/`.)

To uninstall: `sudo apt remove spotlight-dimmer-gnome` (or `spotlight-dimmer-kde`).

> **Upgrading from a source install?** Remove the per-user daemon first (see [Uninstalling](#uninstalling-linux-source-installs)) — a leftover unit in `~/.config/systemd/user/` shadows the packaged one in `/usr/lib/systemd/user/`.

#### Installing from source

Building from source additionally requires [Rust via rustup](https://rustup.rs). Clone the repository first (the `make` commands below run from the repository root):

```bash
git clone https://github.com/thomazmoura/spotlight-dimmer.git
cd spotlight-dimmer
```

##### Ubuntu (GNOME)

On GNOME the daemon computes the overlays and the GNOME Shell extension renders them, so the GTK build dependencies are not needed:

```bash
make install-linux-gnome FEATURES=--no-default-features
```

Then log out and back in (Wayland cannot reload GNOME Shell in place) and enable the extension:

```bash
gnome-extensions enable spotlightdimmer@thomazmoura.github.io
```

Dimming starts as soon as the extension is enabled — the daemon is D-Bus activated automatically; there is nothing to start manually. Toggle dimming on/off with **Super+Shift+D**.

> If you also want the daemon-side layer-shell renderer built (not used by GNOME), run `sudo apt install libgtk-4-dev libgtk4-layer-shell-dev` and use plain `make install-linux-gnome`.

##### Kubuntu (KDE Plasma 6)

On KDE the daemon renders the overlays itself via the layer-shell protocol, so the GTK development packages are required:

```bash
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev
make install-linux-kde
```

This builds and installs the daemon, installs and enables the KWin script (which feeds focus and monitor events to the daemon), and binds **Meta+Shift+D** to the dimming toggle. Everything takes effect immediately — no logout needed. If the shortcut doesn't fire right away, log out/in once or rebind it in System Settings → Shortcuts.

#### First run and troubleshooting (Linux)

The source-install targets seed a starter configuration at `~/.config/SpotlightDimmer/config.json` (PartialWithActive mode) if none exists — edits to it apply instantly (for .deb installs, copy it manually as shown above). Note that **FullScreen mode only dims inactive monitors**, so on a single-monitor setup it shows nothing; use Partial or PartialWithActive there.

The daemon runs as a systemd **user** service, so status and logs need the `--user` flag:

```bash
systemctl --user status spotlight-dimmer-daemon
journalctl --user -u spotlight-dimmer-daemon -f
```

On KDE, `qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.isScriptLoaded spotlightdimmer` confirms the adapter script is loaded.

#### Optional: tmux pane spotlight

Source installs copy the tmux integration tools to `~/.config/SpotlightDimmer/tools/`; the .deb packages ship them at `/usr/share/spotlight-dimmer/tools/` (adjust the paths below accordingly). To have the spotlight follow the focused tmux pane inside WezTerm, **two** pieces of configuration are needed:

1. Map WezTerm to the tmux provider by adding an `AppIntegrations` section to `~/.config/SpotlightDimmer/config.json` (the starter config doesn't include it):

```json
"AppIntegrations": [
  {
    "WmClass": "org.wezfurlong.wezterm",
    "Provider": "tmux",
    "ContentOffsetX": 0,
    "ContentOffsetY": 0
  }
]
```

2. Load the reporting hooks by adding to your `~/.tmux.conf` (then reload with `tmux source-file ~/.tmux.conf`):

```
source-file ~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf
```

Without both, dimming falls back to highlighting the whole terminal window. See [docs/TMUX_INTEGRATION.md](docs/TMUX_INTEGRATION.md) for the full setup guide, including the content offsets for terminal padding and tab bars.

#### Uninstalling (Linux, source installs)

For .deb installs, use `sudo apt remove spotlight-dimmer-gnome` (or `spotlight-dimmer-kde`) instead. The commands below undo a `make install-linux-*` source install:

```bash
# Daemon and tools
rm ~/.local/bin/spotlight-dimmer-daemon
systemctl --user disable --now spotlight-dimmer-daemon.service 2>/dev/null
rm ~/.config/systemd/user/spotlight-dimmer-daemon.service
rm ~/.local/share/dbus-1/services/org.spotlightdimmer.Daemon.service
rm -r ~/.config/SpotlightDimmer/tools

# GNOME extension
gnome-extensions disable spotlightdimmer@thomazmoura.github.io
rm -r ~/.local/share/gnome-shell/extensions/spotlightdimmer@thomazmoura.github.io

# KDE script and shortcut
kpackagetool6 --type KWin/Script --remove spotlightdimmer
rm ~/.local/share/applications/org.spotlightdimmer.toggle.desktop
```

## How It Works

### Windows

The application uses a **dual event hook system** for comprehensive window tracking:

#### Event Hooks (100% Event-Driven - No Polling!)
- **EVENT_SYSTEM_FOREGROUND** - Instant detection when switching between applications
- **EVENT_OBJECT_LOCATIONCHANGE** - Real-time detection of window movement:
  - Detects windows being dragged between monitors with the mouse
  - Detects Win+Arrow and Win+Shift+Arrow keyboard shortcuts
  - Filters out cursor/caret events using `OBJID_WINDOW` check

#### Window Management APIs
- **EnumDisplayMonitors** - Detects all connected monitors
- **CreateWindowEx** - Creates overlay windows with layered and transparent styles
- **SetLayeredWindowAttributes** - Sets the semi-transparent appearance with configurable opacity

### Linux

Wayland's security model only lets compositor-side components observe other windows, so a small adapter runs inside the compositor and reports focus, geometry, and monitor changes to the shared daemon over D-Bus:

- **GNOME**: the Shell extension reports events *and* renders the overlays (GNOME doesn't support the layer-shell protocol), receiving computed overlay rectangles back from the daemon.
- **KDE Plasma 6**: the KWin script only reports events; the daemon renders click-through overlays itself via `gtk4-layer-shell`.

The daemon is D-Bus activated and supervised by a systemd user unit, and everything is event-driven — no polling on any platform. See [docs/LINUX_DAEMON.md](docs/LINUX_DAEMON.md) for the full architecture and D-Bus contract.

## Configuration

All platforms share the same JSON configuration schema, hot-reloaded on change:

- **Windows**: `%AppData%\SpotlightDimmer\config.json`
- **Linux**: `~/.config/SpotlightDimmer/config.json`

See [CONFIGURATION.md](CONFIGURATION.md) for detailed configuration options.

## Building

### Windows

```bash
# Regular build
dotnet build

# Run
dotnet run

# AOT build (requires Visual Studio C++ tools)
dotnet publish -c Release -r win-x64
```

### Linux

From `SpotlightDimmer.LinuxDaemon/`:

```bash
# Full build (requires libgtk-4-dev libgtk4-layer-shell-dev)
cargo build --release

# Headless build without GTK (no layer-shell renderer; enough for GNOME)
cargo build --release --no-default-features

# Core unit tests
cargo test -p spotlight-dimmer-core
```

## Architecture

### Windows (`SpotlightDimmer.WindowsClient`)

- **WinApi.cs** - P/Invoke declarations for Windows APIs
- **MonitorManager.cs** - Multi-monitor detection and management
- **OverlayWindow.cs** - Semi-transparent, click-through overlay windows
- **FocusTracker.cs** - Event-driven focus tracking using Windows hooks
- **Program.cs** - Main application logic and message loop

### Linux

- **SpotlightDimmer.LinuxDaemon/core** - Pure Rust crate: overlay calculator, config parsing, state
- **SpotlightDimmer.LinuxDaemon/daemon** - Daemon binary: D-Bus services, event loop, tmux/wezterm integration, layer-shell renderer
- **SpotlightDimmer.GnomeShellExtension** - Thin GNOME adapter + St.Widget renderer
- **SpotlightDimmer.KwinScript** - KWin (Plasma 6) adapter script

## Performance Notes

- **100% event-driven** - No polling whatsoever!
- **Zero CPU usage when idle** - Only activates on actual window changes
- **Instant response** - Event hooks provide immediate notification of window changes
- **Efficient movement detection** - Tracks window position and focus changes in real-time
- Native Windows API calls for maximum performance (Windows)
- AOT compilation eliminates JIT overhead and reduces startup time (Windows)
- Overlay windows with no visible content are unmapped, handing direct scanout back to fullscreen apps (Linux/KDE)
