# tmux Pane Integration

SpotlightDimmer can highlight the **focused tmux pane** inside a terminal
window instead of the whole window. When the focused window is a configured
terminal (e.g. WezTerm or Ghostty) whose visible content is a tmux client, the spotlight
shrinks to the focused pane: sibling panes, the rest of the window, and the
rest of the display are all dimmed. Whenever pane information is unavailable,
SpotlightDimmer automatically falls back to the normal whole-window spotlight.

> **Platform**: Linux (GNOME and KDE Plasma via the shared daemon — see
> `docs/LINUX_DAEMON.md`). For the Windows equivalent (Windows Terminal panes,
> tmux in WSL2, WezTerm) see `docs/WINDOWS_TERMINAL_INTEGRATION.md`.

## How It Works

```
tmux hook fires (pane switch, resize, split, window change, mode/zoom change, ...)
  └─> spotlight-dimmer-tmux-report.sh
        one `tmux display-message` call, converts pane cells -> pixels
        └─> D-Bus: org.spotlightdimmer.PaneTracker.UpdatePaneGeometry(tty, x, y, w, h)

daemon (integrations/)
  - owns the org.spotlightdimmer.PaneTracker D-Bus name
  - stores the latest pane rect per tmux client tty
  - on focus/title changes, resolves the focused pane's tty and verifies
    (via `tmux list-clients`) that a live tmux client is attached to it
  - the tty is the join key between the terminal side and the tmux side
  - how the tty is resolved depends on the terminal (`TtySource`):
      "wezterm" -> asynchronously run `wezterm cli` and read the focused
                   pane's tty_name (also yields wezterm-native split origins)
      "title"   -> read the tty tmux published in the window title, for
                   terminals with no pane-query CLI (Ghostty)

overlay calculation
  - if a pane rect is resolved, the Partial/PartialWithActive calculation
    receives the pane rect instead of the window rect; the four edge overlays
    then extend from the pane to the monitor edges, dimming everything else
```

Coordinate conversion relies on tmux ≥ 3.4 reporting the terminal's real cell
pixel size (`client_cell_width`/`client_cell_height`, propagated by the
terminal via TIOCGWINSZ), so no font metrics need to be configured.

## Requirements

- GNOME Shell 45+ with the SpotlightDimmer extension enabled, or KDE Plasma
  with the KWin script loaded
- tmux **3.4 or newer** (needs `client_cell_width`/`client_cell_height` formats)
- A supported terminal:
  - **WezTerm**, with the `wezterm` CLI available in `PATH`, or
  - **Ghostty** (see [Ghostty](#ghostty) below — no CLI needed)
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

> The snippet registers its hooks with `set-hook -ag` (append), and probes each
> event first so re-sourcing your config never stacks up duplicates. Hooks
> belonging to other tools on the same events are left untouched, so there is
> nothing to merge by hand.

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
| `TtySource` | string | `"wezterm"` | How the focused pane's tty is found: `"wezterm"` (query the `wezterm` CLI) or `"title"` (read it from the window title — see [Ghostty](#ghostty)) |
| `ContentOffsetX` | integer | `0` | Pixels from the window content area's left edge to the terminal cell grid (window padding) |
| `ContentOffsetY` | integer | `0` | Pixels from the window content area's top edge to the terminal cell grid (padding + tab bar height, if any) |

`ContentOffsetX/Y` mirror your terminal's *internal* chrome, measured from the
client area (window content, decorations excluded — adapters report it
separately from the decorated frame, so title bars and borders are accounted
for automatically in both windowed and maximized states). For WezTerm they
come from `window_padding` and the tab bar: with `enable_tab_bar = false` and
`window_padding = { left = '2px' }`, use `ContentOffsetX: 2, ContentOffsetY: 0`.
A few pixels of error is not visually noticeable in a dimming overlay.

### 4. Reload the extension

On Wayland, log out and back in (or use a nested session during development).
The config file itself hot-reloads without restarting.

## Ghostty

Ghostty has no CLI that can report which surface is focused or which tty it
owns, so the tty cannot be pulled from the terminal. tmux publishes it
instead: `set-titles-string` appends the client tty to the window title, which
the daemon already receives from the compositor on every title change.

### 1. Let tmux publish the tty

Uncomment the Ghostty block at the end of
`~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf`:

```tmux
set -g set-titles on
set -g set-titles-string '#S:#I:#W - "#T" #{session_alerts}⁣sd:#{client_tty}'
```

Then reload: `tmux source-file ~/.tmux.conf`.

The marker between `#{session_alerts}` and `sd:` is U+2063 INVISIBLE
SEPARATOR, which renders as nothing — the title still looks normal in the
window list and Alt-Tab. The rest of the string is tmux's own default; if you
already customize `set-titles-string`, keep your format and append the marker
plus `#{client_tty}` at the end.

### 2. Configure the integration

```json
{
  "AppIntegrations": [
    {
      "WmClass": "com.mitchellh.ghostty",
      "Provider": "tmux",
      "TtySource": "title",
      "ContentOffsetX": 2,
      "ContentOffsetY": 2
    }
  ]
}
```

`ContentOffsetX/Y` mirror Ghostty's `window-padding-x` / `window-padding-y`
(both default to `2`). When the window has client-side decorations, add the
titlebar/tab-bar height to `ContentOffsetY` — the window frame the compositor
reports starts above it.

### Limitation: Ghostty-native splits

Only setups where **tmux fills the whole Ghostty surface** are supported. If
you split in Ghostty itself and run tmux inside one of those splits, the split's
pixel origin is unknowable without a Ghostty API, so the highlight would land in
the wrong place. (With WezTerm this works, because `wezterm cli list` reports
each pane's cell origin.)

## Coordinate System

```
screen_x = client.x + ContentOffsetX + wezterm_pane_origin_x + tmux_rel_x
screen_y = client.y + ContentOffsetY + wezterm_pane_origin_y + tmux_rel_y

where:
  client = the window's client-area rect (decorations excluded), as reported
  by the compositor adapter; falls back to the decorated frame rect when the
  adapter doesn't provide one (protocol v1)

and (computed by the helper script, pixels relative to the content origin):
  tmux_rel_x = pane_left * cell_width
  tmux_rel_y = status_bar_rows_at_top * cell_height + pane_top * cell_height

and (computed by the daemon from `wezterm cli list`):
  wezterm_pane_origin_* = the wezterm-native pane's cell origin, for setups
  that split wezterm itself; 0 when tmux fills the whole tab, and always 0
  for the "title" tty source
```

Basing the origin on the client area (not the decorated frame) keeps the
highlight aligned in both windowed and maximized states: decorations appear
and disappear with the window state, while the terminal's internal chrome
covered by `ContentOffsetX/Y` does not.

tmux pane coordinates cover only the pane interior — the single-cell border
lines drawn between panes belong to no pane. Each edge that has an adjacent
border (i.e. does not touch the tmux window-area edge) is extended by one
cell, so the border characters framing the focused pane are highlighted with
it instead of being left as a dimmed strip.

The final rect is clamped to the client area, so a misconfigured offset can
never highlight outside the window content (or onto the title bar).

## Fallback Behavior

The whole-window spotlight (normal behavior) is used whenever any of these
holds:

- The focused window's `WmClass` has no `AppIntegrations` entry
- `"TtySource": "wezterm"`: `wezterm cli` is unavailable or reports no focused
  pane
- `"TtySource": "title"`: the window title carries no `sd:/dev/...` marker
  (tmux is not attached in that window, or the snippet is not installed)
- The resolved tty has no live tmux client attached (e.g. a terminal tab
  running a plain shell, or the tmux server exited)
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

**Ghostty: check the tty reaches the window title** (`Alt+F2` → `lg`, or
`xprop WM_NAME` on X11) and compare it with what tmux reports:

```bash
tmux display-message -p '#{client_tty}'
```

If the title never changes, tmux is not setting it: confirm the terminfo entry
is resolvable (`infocmp xterm-ghostty | grep tsl`, which Ghostty ships), and
that `set -g set-titles on` was actually sourced (`tmux show -g set-titles`).

**Watch extension logs**:

```bash
journalctl --user -f -o cat /usr/bin/gnome-shell | grep SpotlightDimmer
```

**Highlight is misaligned**: adjust `ContentOffsetX/Y` in the config (they
hot-reload). Vertical misalignment by exactly one cell height usually means a
status bar / tab bar assumption is off. Misalignment that only happens in
windowed (not maximized) mode means the adapter isn't reporting the client
area — make sure the KWin script / GNOME extension version matches the
daemon (adapter protocol v2).
