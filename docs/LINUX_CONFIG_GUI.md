# SpotlightDimmer Settings (Linux)

`spotlight-dimmer-config` is a GTK4 settings window for the Linux daemon. It
edits `~/.config/SpotlightDimmer/config.json` — the same file the daemon
hot-reloads — so a change made in the window is visible on screen roughly a
quarter of a second later, with no restart and no hand-editing of JSON.

It is the Linux counterpart of the Windows `SpotlightDimmer.Config` app, and
it works on both GNOME and KDE Plasma.

## Launching it

From the application menu, as **Spotlight Dimmer Settings**, or from a
terminal:

```bash
spotlight-dimmer-config
```

Launching it a second time raises the window that is already open rather than
opening a second one, so two copies can never fight over the same file.

## What it edits

Only the two sections the Linux daemon consumes:

| Tab | Section | Keys |
|---|---|---|
| General | `Overlay` | `Mode`, `InactiveColor`, `InactiveOpacity`, `ActiveColor`, `ActiveOpacity` |
| Integrations | `AppIntegrations[]` | Built-in WezTerm/Ghostty entries only (on/off, `ContentOffsetX`, `ContentOffsetY`) |

**Every other key in the file is preserved byte-for-byte.** `System`,
`Profiles`, `CurrentProfile`, `ConfigVersion`, `$schema` and
`Overlay.ExcludeFromScreenCapture` are Windows-side settings that the Linux
daemon ignores; the window does not show them, and saving never rewrites or
drops them. Edit those by hand or from the Windows configuration app.

### General

- **Mode** — `FullScreen`, `Partial` or `PartialWithActive`. If the file
  contains a mode string that is not one of the three, it is shown as a fourth
  `Unknown (…)` entry rather than being silently corrected: that is a real
  daemon behaviour (dim unfocused monitors, leave the focused one alone), and
  overwriting a value you typed on purpose would be worse than displaying it.
- **Inactive overlay** — colour and opacity of the dimming applied to
  unfocused monitors, and to the area around the active window in the two
  Partial modes.
- **Active overlay** — colour and opacity of the tint over the focused window.
  Used only in `PartialWithActive`, but editable in every mode so the values
  can be set up before switching.
- **Preview** — a mock focused monitor beside a mock unfocused one, redrawn on
  every change. Opacity is stored as `0-255`; the readout also shows the
  percentage, which is what the numbers actually mean.

### Integrations

The terminal pane spotlight: when a supported terminal is focused, the
spotlight follows the focused tmux pane (and, inside it, the focused neovim
split) instead of the whole window. This needs the tmux hooks as well — see
[TMUX_INTEGRATION.md](TMUX_INTEGRATION.md).

Integrations are code in the daemon, not plugins, so the tab lists every one
it supports with an **Enabled** checkbox — nothing to type or look up:

| Integration | Matches `WmClass` | Writes `TtySource` | Default offsets |
|---|---|---|---|
| WezTerm | `org.wezfurlong.wezterm` | `wezterm` (asks `wezterm cli`) | 0, 0 |
| Ghostty | `com.mitchellh.ghostty` | `title` (reads the window title) | 2, 2 |

Checking a box adds that terminal's `AppIntegrations` entry; unchecking
removes it. Ghostty additionally needs the Ghostty block in
`spotlight-dimmer.tmux.conf` uncommented, so tmux publishes the pane in the
window title.

- **Content offset X / Y** — editable while the integration is enabled:
  pixels from the window's client area edge to the terminal cell grid (padding
  for X, padding plus tab bar height for Y). Window decorations are reported
  separately by the compositor adapter and must not be folded in here.

Other `AppIntegrations` entries — a terminal configured by hand in JSON, or
the Windows client's `ProcessName` entries — are not shown in the tab and are
never modified or removed by it. Re-checking an integration whose entry
already exists keeps that entry's offsets and extra keys as they are.

## Daemon controls

When the daemon is already running, the header bar switch mirrors its
`Enabled` property (the same state as the Meta+Shift+D / Super+Shift+D
toggle), and the footer reads `daemon running · protocol vN`. Flipping the
switch elsewhere updates the window, and vice versa. The daemon saves the
state as `Overlay.Enabled` in `config.json`, so it survives restarts.

When the daemon is not running the footer says so (`daemon not running`), and the
switch edits `Overlay.Enabled` directly, choosing the state the daemon will
start in. **The window never starts the daemon.** The daemon is
D-Bus-activatable, so it checks whether the bus name already has an owner
before talking to it; merely opening the settings window will not launch a
daemon you deliberately stopped. Everything else in the window keeps working:
it is a config file editor first, and a daemon front-end second.

## Opening and closing from a shortcut

`spotlight-dimmer-config --toggle` opens the window, raises it when it is
open behind other windows, and closes it when it already has focus. It is
bound to Super+Alt+Shift+D on GNOME and Meta+Alt+Shift+D on KDE (see
[LINUX_DAEMON.md](LINUX_DAEMON.md#settings-window-shortcut)).

## Safety behaviours

- **Debounced atomic writes.** Dragging a slider produces a handful of writes,
  not one per pixel. Each write goes to a temporary file in the same directory
  and is renamed into place, so the daemon never reads a half-written file.
- **External edits are picked up.** Editing `config.json` in another editor
  while the window is open updates the controls, and the window will not
  clobber that edit with a stale in-flight save.
- **An unparseable file is never overwritten.** If `config.json` has a syntax
  error, the window shows a banner naming the parse error and refuses to save
  anything until you either fix the file or press **Replace the file**.

## Installing

Packaged separately as `spotlight-dimmer-config`, because it pulls in GTK4 and
the GNOME daemon package is deliberately built without it:

```bash
sudo apt install ./spotlight-dimmer-config_<version>-1_amd64.deb
```

It installs alongside either `spotlight-dimmer-gnome` or
`spotlight-dimmer-kde` and does not conflict with them.

From source, in `SpotlightDimmer.LinuxDaemon/`:

```bash
sudo apt install libgtk-4-dev       # Ubuntu/Debian
make install-config-gui
```

`make install-linux-gnome` and `make install-linux-kde` include it already when
the GTK4 headers are installed, and skip it with a warning when they are not.

## Notes

- The window follows the system GTK theme (Breeze on Kubuntu, Adwaita on
  Ubuntu); libadwaita is deliberately not used, because it ignores the system
  theme and would look foreign on KDE.
- Under Breeze-GTK, GTK may print `GtkGizmo (slider) reported min width -2` on
  startup. It is a theme quirk (it does not appear under Adwaita) and is
  harmless.
