# tmux Pane Integration

SpotlightDimmer can highlight the **focused tmux pane** inside a terminal
window instead of the whole window. When the focused window is a configured
terminal (e.g. WezTerm) whose visible content is a tmux client, the spotlight
shrinks to the focused pane: sibling panes, the rest of the window, and the
rest of the display are all dimmed. Whenever pane information is unavailable,
SpotlightDimmer automatically falls back to the normal whole-window spotlight.

> **Platform**: Linux (GNOME and KDE Plasma via the shared daemon — see
> `docs/LINUX_DAEMON.md`). The Windows client does not support app integrations yet.

## How It Works

```
tmux hook fires (pane switch, resize, split, window change, mode/zoom change, ...)
  └─> spotlight-dimmer-tmux-report.sh
        one `tmux display-message` call, converts pane cells -> pixels
        └─> D-Bus: org.spotlightdimmer.PaneTracker.UpdatePaneGeometry(tty, x, y, w, h)

GNOME extension (appIntegrations.js)
  - owns the org.spotlightdimmer.PaneTracker D-Bus name
  - stores the latest pane rect per tmux client tty
  - on focus/title changes, asynchronously runs `wezterm cli` to find the
    focused wezterm pane's tty and verifies (via `tmux list-clients`) that a
    live tmux client is attached to it
  - the tty is the join key between the wezterm side and the tmux side

extension.js
  - if a pane rect is resolved, the Partial/PartialWithActive calculation
    receives the pane rect instead of the window rect; the four edge overlays
    then extend from the pane to the monitor edges, dimming everything else
```

Coordinate conversion relies on tmux ≥ 3.4 reporting the terminal's real cell
pixel size (`client_cell_width`/`client_cell_height`, propagated by the
terminal via TIOCGWINSZ), so no font metrics need to be configured.

## Requirements

- GNOME Shell 45+ with the SpotlightDimmer extension enabled
- tmux **3.4 or newer** (needs `client_cell_width`/`client_cell_height` formats)
- WezTerm with the `wezterm` CLI available in `PATH`
- `gdbus` (part of GLib, present on any GNOME system)

## Setup

### 1. Install the helper script

```bash
mkdir -p ~/.config/SpotlightDimmer/tools
cp SpotlightDimmer.LinuxDaemon/tools/spotlight-dimmer-tmux-report.sh \
   ~/.config/SpotlightDimmer/tools/
cp SpotlightDimmer.LinuxDaemon/tools/spotlight-dimmer.tmux.conf \
   ~/.config/SpotlightDimmer/tools/
chmod +x ~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-report.sh
```

### 2. Install the tmux hooks

Add to `~/.tmux.conf`:

```tmux
source-file ~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf
```

Then reload: `tmux source-file ~/.tmux.conf`

> The snippet uses `set-hook -g`, which **replaces** existing global hooks for
> the same events. If you already define one of these hooks, merge the
> `run-shell` command into your hook instead.

### 3. Configure the integration

Add an `AppIntegrations` section to `~/.config/SpotlightDimmer/config.json`:

```json
{
  "Overlay": {
    "Mode": "PartialWithActive",
    "InactiveColor": "#000000",
    "InactiveOpacity": 96,
    "ActiveColor": "#000000",
    "ActiveOpacity": 0
  },
  "AppIntegrations": [
    {
      "WmClass": "org.wezfurlong.wezterm",
      "Provider": "tmux",
      "ContentOffsetX": 2,
      "ContentOffsetY": 0
    }
  ]
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `WmClass` | string | (required) | Window class to match. Find it with `Alt+F2` → `lg` → `global.display.focus_window.get_wm_class()`, or `xprop WM_CLASS` on X11 |
| `Provider` | string | `"tmux"` | Integration provider. Only `"tmux"` exists today |
| `ContentOffsetX` | integer | `0` | Pixels from the window's left edge to the terminal cell grid (window padding) |
| `ContentOffsetY` | integer | `0` | Pixels from the window's top edge to the terminal cell grid (padding + tab bar height, if any) |

`ContentOffsetX/Y` mirror your terminal's chrome. For WezTerm they come from
`window_padding` and the tab bar: with `enable_tab_bar = false` and
`window_padding = { left = '2px' }`, use `ContentOffsetX: 2, ContentOffsetY: 0`.
A few pixels of error is not visually noticeable in a dimming overlay.

### 4. Reload the extension

On Wayland, log out and back in (or use a nested session during development).
The config file itself hot-reloads without restarting.

## Coordinate System

```
screen_x = frame.x + ContentOffsetX + wezterm_pane_origin_x + tmux_rel_x
screen_y = frame.y + ContentOffsetY + wezterm_pane_origin_y + tmux_rel_y

where (computed by the helper script, pixels relative to the content origin):
  tmux_rel_x = pane_left * cell_width
  tmux_rel_y = status_bar_rows_at_top * cell_height + pane_top * cell_height

and (computed by the extension from `wezterm cli list`):
  wezterm_pane_origin_* = the wezterm-native pane's cell origin, for setups
  that split wezterm itself; 0 when tmux fills the whole tab
```

The final rect is clamped to the window frame, so a misconfigured offset can
never highlight outside the window.

## Fallback Behavior

The whole-window spotlight (normal behavior) is used whenever any of these
holds:

- The focused window's `WmClass` has no `AppIntegrations` entry
- `wezterm cli` is unavailable or reports no focused pane
- The focused wezterm pane's tty has no live tmux client attached
  (e.g. a wezterm tab running a plain shell, or the tmux server exited)
- No pane geometry has been reported yet for that tty
- The computed rect is empty after clamping to the window

tmux exiting or detaching is detected because the terminal window title
changes (tmux sets the title), which triggers re-verification.

## Troubleshooting

**Check the D-Bus service is up** (extension enabled):

```bash
gdbus introspect --session --dest org.spotlightdimmer.PaneTracker \
  --object-path /org/spotlightdimmer/PaneTracker
```

**Push a fake pane rect** (overlays should snap to it while the terminal is
focused; switch tmux panes to overwrite it with real data):

```bash
gdbus call --session --dest org.spotlightdimmer.PaneTracker \
  --object-path /org/spotlightdimmer/PaneTracker \
  --method org.spotlightdimmer.PaneTracker.UpdatePaneGeometry \
  "$(tmux display-message -p '#{client_tty}')" 100 100 800 600
```

**Verify the hooks are installed** (session-scope and window-scope hooks are
listed separately):

```bash
tmux show-hooks -g | grep spotlight   # client/session hooks
tmux show-hooks -gw | grep spotlight  # pane/window hooks (mode, layout, ...)
```

**Run the helper by hand** from inside tmux with `bash -x` to see the
computed geometry:

```bash
bash -x ~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-report.sh
```

**Watch extension logs**:

```bash
journalctl --user -f -o cat /usr/bin/gnome-shell | grep SpotlightDimmer
```

**Highlight is misaligned**: adjust `ContentOffsetX/Y` in the config (they
hot-reload). Vertical misalignment by exactly one cell height usually means a
status bar / tab bar assumption is off.
