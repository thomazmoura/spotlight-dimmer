# SpotlightDimmer Linux Daemon

`spotlight-dimmer-daemon` (in `SpotlightDimmer.LinuxDaemon/`) is the shared
Linux/Wayland implementation of SpotlightDimmer. One daemon owns everything
compositor-independent — configuration, overlay geometry calculation and the
wezterm + tmux pane integration — while small per-compositor **adapters** feed
it focus and monitor data over D-Bus:

```
┌─────────────────────────────────────────┐
│        spotlight-dimmer-daemon          │
│  config.json · calculator · tmux/wez    │
│  D-Bus: Daemon1/Adapter1/Renderer1      │
│         + PaneTracker                   │
└──────┬──────────────┬───────────────────┘
   D-Bus│         D-Bus│      layer-shell rendering
┌───────┴─────┐ ┌──────┴────┐
│ GNOME Shell │ │ KWin      │   (KWin and wlroots compositors
│ extension   │ │ script    │    support layer-shell; the daemon
│ focus →     │ │ focus →   │    draws its own overlays there)
│ ← overlays  │ │           │
│ (renders    │ │ (daemon   │
│  St.Widgets)│ │  renders) │
└─────────────┘ └───────────┘
```

Why the asymmetry: Wayland's security model means only a compositor-side
component can observe other windows' focus and geometry, and **GNOME does not
support the layer-shell protocol**, so on GNOME the overlays must be drawn by
the Shell extension itself. The extension therefore registers as a *renderer*
(`renders_overlays: true`) and receives computed overlay rectangles back; on
KDE the daemon renders directly via `gtk4-layer-shell`.

## Components

| Directory | Role |
|---|---|
| `SpotlightDimmer.LinuxDaemon/core` | Pure Rust crate: calculator (port of the C# `AppState`), config parsing, state. `cargo test` runs anywhere. |
| `SpotlightDimmer.LinuxDaemon/daemon` | The daemon binary: zbus D-Bus services, glib event loop, wezterm/tmux integration, layer-shell renderer (`render` feature). |
| `SpotlightDimmer.GnomeShellExtension` | Thin GNOME adapter + St.Widget renderer. |
| `SpotlightDimmer.KwinScript` | KWin (Plasma 6) adapter script. |
| `SpotlightDimmer.LinuxDaemon/tools` | tmux hook config + reporter script (unchanged contract). |

## Installation

### From .deb packages (recommended)

Each Linux release (tags ending in `-linux`) publishes six packages built by
`.github/workflows/release-linux.yml`: `spotlight-dimmer-gnome`,
`spotlight-dimmer-kde` and `spotlight-dimmer-config`, for amd64 and arm64.
Install with `sudo apt install ./<package>.deb` so runtime dependencies are
resolved automatically. The two desktop packages conflict with each other on
purpose (both ship `/usr/bin/spotlight-dimmer-daemon`); the configuration GUI
is independent and installs alongside either.

`SpotlightDimmer.LinuxDaemon/tools/install-release.sh` automates this. It picks
the desktop package (the `kubuntu-desktop`/`ubuntu-desktop` metapackages or
`XDG_CURRENT_DESKTOP`) and the architecture, downloads the matching assets from
the latest `-linux` release (or the one given with `--version`), and skips
packages that are already at that version. It also does the per-user steps
listed below that can be scripted.

Package contents:

| Path | Contents |
|---|---|
| `/usr/bin/spotlight-dimmer-daemon` | The daemon (headless build in the GNOME package, layer-shell build in the KDE package) |
| `/usr/lib/systemd/user/spotlight-dimmer-daemon.service` | systemd user unit |
| `/usr/share/dbus-1/services/org.spotlightdimmer.Daemon.service` | D-Bus session activation |
| `/usr/share/gnome-shell/extensions/spotlightdimmer@thomazmoura.github.io/` | GNOME Shell extension (GNOME package only) |
| `/usr/share/kwin/scripts/spotlightdimmer/` | KWin script, auto-loaded by KWin (KDE package only) |
| `/usr/share/applications/org.spotlightdimmer.toggle.desktop` | Toggle launcher for shortcut binding (KDE package only) |
| `/usr/bin/spotlight-dimmer-config` | The settings window (`spotlight-dimmer-config` package only) |
| `/usr/share/applications/org.spotlightdimmer.Config.desktop` | Settings launcher (`spotlight-dimmer-config` package only) |
| `/usr/share/spotlight-dimmer/tools/` | tmux integration tools |
| `/usr/share/doc/<package>/` | `CONFIGURATION.md`, this document, `TMUX_INTEGRATION.md` and `examples/config.example.json` |

Post-install steps the packages cannot do for you:

- **GNOME**: log out/in, then `gnome-extensions enable spotlightdimmer@thomazmoura.github.io`.
- **KDE**: bind the "SpotlightDimmer Toggle" launcher to Meta+Shift+D in System Settings → Shortcuts (and, with `spotlight-dimmer-config` installed, "SpotlightDimmer Settings Toggle" to Meta+Alt+Shift+D).
- **Both**: seed your config once — `mkdir -p ~/.config/SpotlightDimmer && cp /usr/share/doc/<package>/examples/config.example.json ~/.config/SpotlightDimmer/config.json`. (Or install `spotlight-dimmer-config` and let the settings window write it.)

> **Note**: a per-user unit left behind by a previous `make install-daemon`
> (`~/.config/systemd/user/spotlight-dimmer-daemon.service`, plus
> `~/.local/share/dbus-1/services/org.spotlightdimmer.Daemon.service` and
> `~/.local/bin/spotlight-dimmer-daemon`) shadows the packaged files — remove
> them before switching to the .deb (see the README's uninstall section), or
> run `install-release.sh --clean-source-install`, which also removes the
> user-local extension, KWin script and settings window.
> Likewise, tmux hook installs that reference
> `~/.config/SpotlightDimmer/tools/` should be updated to
> `/usr/share/spotlight-dimmer/tools/` when moving to the packages.

The packaging metadata lives in `SpotlightDimmer.LinuxDaemon/daemon/Cargo.toml`
(`[package.metadata.deb]`, built with `cargo deb --variant gnome|kde`) and in
`SpotlightDimmer.LinuxDaemon/config-gui/Cargo.toml` (built with
`cargo deb -p spotlight-dimmer-config`), with `/usr/bin`-pathed
unit/activation files in `SpotlightDimmer.LinuxDaemon/data/packaging/`.

### From source

Build dependencies: [rustup](https://rustup.rs), plus GTK for the layer-shell
renderer and the settings window:

```bash
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev   # Ubuntu/Debian
```

Then from `SpotlightDimmer.LinuxDaemon/`:

```bash
make install-linux-gnome   # GNOME: daemon + extension + tmux tools
make install-linux-kde     # KDE Plasma 6: daemon + KWin script + Meta+Shift+D + tmux tools
```

GNOME machines can drop the GTK build deps entirely with
`make install-linux-gnome FEATURES=--no-default-features` (the layer-shell
renderer is not used on GNOME; `FEATURES` only affects the daemon).
`spotlight-dimmer-config` is GTK4 on every desktop and needs `libgtk-4-dev`:
when its headers are missing, `install-linux-gnome`/`install-linux-kde` skip the
settings window with a warning instead of failing — install `libgtk-4-dev` and
run `make install-config-gui` to add it later.

The daemon is **D-Bus activated**: the GNOME extension watching its bus name
(or the KWin script's first call) starts it automatically, and the systemd
user unit (`Type=dbus`, `Restart=on-failure`) restarts it on crashes. There is
nothing to start manually.

## Configuration

Same file and schema as before (shared with the Windows client):
`~/.config/SpotlightDimmer/config.json`, hot-reloaded on change. The daemon
consumes `Overlay.*` and `AppIntegrations[]`; see `CONFIGURATION.md` and
`docs/TMUX_INTEGRATION.md`.

The `spotlight-dimmer-config` settings window edits those two sections with a
live preview, preserving every other key in the file — see
[LINUX_CONFIG_GUI.md](LINUX_CONFIG_GUI.md). Hand-editing the JSON still works
exactly as before; the window picks up external edits while it is open.

## Toggle shortcut

`Daemon1.Toggle()` is the single source of truth for the paused state.

- GNOME: Super+Shift+D (the extension's existing GSettings keybinding now
  calls Toggle over D-Bus).
- KDE: Meta+Shift+D via a `.desktop` launcher registered by
  `make install-kde-shortcut` (rebind in System Settings → Shortcuts).
- Anywhere: `busctl --user call org.spotlightdimmer.Daemon
  /org/spotlightdimmer/Daemon org.spotlightdimmer.Daemon1 Toggle`

The state is persisted as `Overlay.Enabled` in `config.json`: every change
(Toggle, the `Enabled` property, the settings window) is written there and
announced with `PropertiesChanged`, the daemon starts in the saved state, and
a hand-edit of the key is applied on reload.

### Settings window shortcut

Super+Alt+Shift+D (Meta+Alt+Shift+D on KDE) runs
`spotlight-dimmer-config --toggle`: it opens the settings window, raises it if
it is buried, and closes it when it is already focused.

- GNOME: the extension's `toggle-config-window` GSettings keybinding launches
  `org.spotlightdimmer.ConfigToggle.desktop` (so the window receives an
  activation token and may take focus on Wayland).
- KDE: `make install-kde-shortcut` binds the same launcher; package users bind
  "SpotlightDimmer Settings Toggle" in System Settings → Shortcuts.

## D-Bus contract (protocol version 3)

All on the session bus. `org.spotlightdimmer.Daemon` is activatable;
`org.spotlightdimmer.PaneTracker` deliberately is not (tmux hooks must be
silent no-ops when the daemon is down).

Object `/org/spotlightdimmer/Daemon`:

- `org.spotlightdimmer.Daemon1` — `Toggle() -> b`, properties `Enabled` (bw),
  `ProtocolVersion` (u).
- `org.spotlightdimmer.Adapter1` — inbound compositor events. All argument
  types are basic (KWin's `callDBus` cannot marshal nested structs, and
  silently truncates calls with more than 9 arguments — hence JSON payloads
  for anything structured):
  - `RegisterAdapter(s compositor, a{sv} capabilities) -> u` — capabilities:
    `renders_overlays` (b). Returns the protocol version (2). Unknown senders
    that call other methods are implicitly registered (daemon-restart
    recovery).
  - `UpdateMonitors(s monitorsJson)` — JSON array of
    `{key, geometry:{x,y,width,height}, workArea:{...}, scale}` in logical
    global coordinates. Keys: GNOME = monitor index ("0"), KWin = connector
    name ("DP-1").
  - `FocusChanged(s wmClass, s title, i x, i y, i width, i height)` —
    protocol v1; the frame rect doubles as the content origin.
  - `FocusChanged2(s wmClass, s title, s rectsJson)` — protocol v2;
    `rectsJson` is `{"frame":{x,y,width,height},"client":{...}}` where
    `client` is the decoration-excluded client area (anchors inner-pane
    highlights; omit or send a degenerate rect to fall back to the frame).
  - `FocusCleared()`, `GeometryChanged(iiii)`,
    `GeometryChanged2(s rectsJson)`, `TitleChanged(s)`,
    `FloatingChanged(s rectsJson)`
- `org.spotlightdimmer.Renderer1` — outbound overlay definitions:
  - `RegisterRenderer() -> s` — returns the current payload snapshot.
  - signal `OverlaysChanged(u serial, s overlaysJson)` — payload:
    `{"serial":n,"enabled":bool,"chrome_handling":"Highlight",
    "monitors":[{"key":"0","overlays":[{"region":0,
    "visible":true,"x":..,"y":..,"width":..,"height":..,"color":{"r":0,"g":0,
    "b":0},"opacity":153}]}]}`. Regions: 0=FullScreen, 1=Top, 2=Bottom,
    3=Left, 4=Right, 5=Center (matches the C# `OverlayRegion`).
  - `FloatingChanged` (v3) carries the always-on-top window rects as
    `[{"x":..,"y":..,"width":..,"height":..}]`, in stacking order with the
    topmost last; an empty array clears them. Adapters send it only when the
    set actually changed.
  - `track_floating` (v3) tells the adapter whether to report those rects at
    all. It is false under the default `AlwaysOnTopHandling: "Ignore"`, so
    GNOME does not enumerate windows on `restacked` unless the feature is
    on. The KWin script is an adapter only and never receives the payload,
    so it always reports; its triggering signals are rare enough that the
    change diff keeps the bus quiet.
  - `chrome_handling` is `"Highlight"` or `"Dim"` and tells a renderer where
    to stack the overlays relative to shell chrome. It rides in the payload
    rather than being read from config by the renderer, because the daemon
    owns configuration and adapters only paint what they are handed. It is
    present in the paused payload too, so a renderer keeps its stacking
    while dimming is off. A renderer that ignores the field keeps its
    previous behaviour, so this is not a protocol break.
  - Overlay counts are no longer bounded by the six region slots: cutting an
    always-on-top window out of an edge band splits that band into up to
    four rects, so several overlays can share one `region` value. Renderers
    must consume the list in order rather than keying by `region`. This
    needs no capability negotiation — an older renderer never calls
    `FloatingChanged`, so the daemon has no rects to cut with and emits the
    legacy set unchanged.

Object `/org/spotlightdimmer/PaneTracker`, interface
`org.spotlightdimmer.PaneTracker` (unchanged from the previous GNOME-only
implementation, so existing tmux hook installs keep working):
`UpdatePaneGeometry(s tty, i x, i y, i width, i height)`, `ClearPane(s tty)`.

Notes:

- The daemon computes the focused monitor itself (max overlap of the frame
  rect against monitor geometries); adapters never send a monitor index.
- A transient 0x0 focused window freezes the current overlays (no new serial)
  to prevent flicker during transitions — the calculator contract inherited
  from the C#/JS implementations.
- The `serial` in the signal duplicates the payload serial so renderers can
  drop stale signals cheaply after reconnects.

## Headless testing

```bash
cargo test -p spotlight-dimmer-core        # calculator/config/state unit tests
cargo build --no-default-features          # daemon without GTK
dbus-run-session -- bash                   # isolated bus, then:
  ./target/debug/spotlight-dimmer-daemon &
  busctl --user call org.spotlightdimmer.Daemon /org/spotlightdimmer/Daemon \
    org.spotlightdimmer.Adapter1 UpdateMonitors "s" \
    '[{"key":"0","geometry":{"x":0,"y":0,"width":1920,"height":1080},"workArea":{"x":0,"y":32,"width":1920,"height":1048},"scale":1.0}]'
  busctl --user monitor org.spotlightdimmer.Daemon   # watch OverlaysChanged
```

## Known limitations

- wlroots compositors (Sway/Hyprland) are designed-for but not yet
  implemented: a future in-daemon `wlr-foreign-toplevel` adapter slots into
  the same contract; rendering already works there via layer-shell. Without
  window geometry (the protocol doesn't expose it), the tmux pane spotlight
  degrades to whole-window automatically.
- At fractional scaling, KWin's `frameGeometry` rounding and GTK's logical
  pixel snapping can differ by ±1px — cosmetically irrelevant for dimming.
- On KDE, `ChromeHandling: "Highlight"` puts the dimming surfaces on the
  `Top` layer so Plasma notifications and OSD (which live on the `Overlay`
  layer) stay lit. Panels are on `Top` as well, so panel-vs-dimmer order
  there is decided by map time and is guaranteed in neither direction. On
  GNOME the same setting is exact, because the extension stacks its actors
  directly above the window group inside `uiGroup`.
- `ChromeHandling` governs only the shell's own surfaces. Application
  windows pinned always-on-top are covered by `AlwaysOnTopHandling`
  instead, which reports their geometry over `FloatingChanged` and cuts
  them out of the overlays in the calculator.
- Shell chrome cannot be given the `AlwaysOnTopHandling` treatment, because
  no supported API exposes its geometry: GNOME only has the private
  `Main.layoutManager._trackedActors`, and Plasma's notification surfaces
  are layer-shell surfaces the KWin scripting API cannot see at all. That
  is why chrome is handled by stacking and windows by geometry.
