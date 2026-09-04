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
| Integrations | `AppIntegrations[]` | `WmClass`, `Provider`, `TtySource`, `ContentOffsetX`, `ContentOffsetY` |

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

The terminal pane spotlight: match a terminal window by its `WM_CLASS` so the
spotlight follows the focused pane instead of the whole window. The `tmux`
provider needs the tmux hooks as well — see
[TMUX_INTEGRATION.md](TMUX_INTEGRATION.md); the `herdr` provider reads the
layout from the Herdr session socket — see
[HERDR_INTEGRATION.md](HERDR_INTEGRATION.md).

- **WM_CLASS** — matched case-sensitively. To find a window's:
  - KDE: `qdbus6 org.kde.KWin /KWin org.kde.KWin.queryWindowInfo`, then click
    the window.
  - GNOME: Alt+F2, `lg`, Windows tab, read `wm_class`.
  - Known-good values: `org.wezfurlong.wezterm`, `com.mitchellh.ghostty`.
- **Provider** — `tmux` (pane geometry pushed by the tmux hooks) or `herdr`
  (layout read from the Herdr session socket). The rows below adapt: the tty
  source belongs to `tmux`, the socket path to `herdr`.
- **Tty source** (`tmux`) — how the focused pane's tty is discovered:
  - *WezTerm CLI* (`wezterm`): asks `wezterm cli` which pane is focused.
  - *Window title* (`title`): reads it from the window title, where tmux
    publishes it via `set-titles-string`. Use this for terminals with no
    pane-query CLI, such as Ghostty.
- **Herdr socket** (`herdr`) — leave empty for the default session
  (`$HERDR_SOCKET_PATH`, then `~/.config/herdr/herdr.sock`). A named session
  listens on `~/.config/herdr/sessions/<name>/herdr.sock`.
- **Content offset X / Y** — pixels from the window's client area edge to the
  terminal cell grid (padding for X, padding plus tab bar height for Y).
  Window decorations are reported separately by the compositor adapter and
  must not be folded in here.

## Daemon controls

When the daemon is already running, the header bar switch mirrors its
`Enabled` property (the same state as the Meta+Shift+D / Super+Shift+D
toggle), and the footer reads `daemon running · protocol vN`. Flipping the
switch elsewhere updates the window, and vice versa.

When the daemon is not running the switch is disabled and the footer reads
`daemon not running` — **the window never starts the daemon**. The daemon is
D-Bus-activatable, so it checks whether the bus name already has an owner
before talking to it; merely opening the settings window will not launch a
daemon you deliberately stopped. Everything else in the window keeps working:
it is a config file editor first, and a daemon front-end second.

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

`make install-linux-gnome` and `make install-linux-kde` include it already.

## Notes

- The window follows the system GTK theme (Breeze on Kubuntu, Adwaita on
  Ubuntu); libadwaita is deliberately not used, because it ignores the system
  theme and would look foreign on KDE.
- Under Breeze-GTK, GTK may print `GtkGizmo (slider) reported min width -2` on
  startup. It is a theme quirk (it does not appear under Adwaita) and is
  harmless.
