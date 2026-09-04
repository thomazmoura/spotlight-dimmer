# Herdr Pane Integration

SpotlightDimmer can highlight the **focused [Herdr](https://herdr.dev) pane**
inside a terminal window instead of the whole window. When the focused window
is a configured terminal whose visible content is a Herdr client, the spotlight
shrinks to the focused pane: the sidebar, the tab bar, the sibling panes, the
rest of the window and the rest of the display are all dimmed. Whenever the
layout is unavailable, SpotlightDimmer automatically falls back to the normal
whole-window spotlight.

> **Platform**: Linux (GNOME and KDE Plasma via the shared daemon — see
> `docs/LINUX_DAEMON.md`). There is no Windows equivalent yet. For tmux (on
> either platform) see `docs/TMUX_INTEGRATION.md`.

## How It Works

Herdr has no `set-hook` equivalent that could push geometry the way the tmux
snippet does, but it exposes a JSON socket API with an event stream — so the
daemon subscribes to it directly instead of shipping a hook script:

```
daemon (integrations/herdr.rs), one background thread
  - connects to the Herdr session socket and sends `events.subscribe` for
    pane.focused, tab.focused, workspace.focused, layout.updated and the
    pane created/closed/moved events
  - on every event, re-reads `pane.layout` (the focused layout) and
    `workspace.get` (its label), and publishes the result when it changed
  - reconnects every 5s while Herdr is not running

daemon event loop
  - the focused window's title says which Herdr workspace that window shows
    (the `sd:herdr:` marker below); it is compared against the workspace the
    layout came from, so a plain shell — or a second window showing another
    workspace — keeps the whole-window spotlight

overlay calculation
  - the pane rect arrives in terminal *cells*, so it is mapped proportionally
    onto the window's content box and handed to the Partial/PartialWithActive
    calculation in place of the window rect
```

Unlike tmux, Herdr reports no cell pixel size (`pane.graphics.info` does, but
only with `experimental.kitty_graphics` enabled), so the cell grid is scaled
onto the window instead of multiplied by a font metric. Nothing to configure,
and it follows font, DPI and resize changes on its own.

## Requirements

- GNOME Shell 45+ with the SpotlightDimmer extension enabled, or KDE Plasma
  with the KWin script loaded
- Herdr **0.8.2 or newer** (socket API protocol 20: `events.subscribe`,
  `pane.layout`)
- Any terminal Herdr runs in — no terminal CLI is needed, since the geometry
  comes from Herdr and not from the terminal

## Setup

### 1. Let Herdr publish which workspace the window shows

Herdr writes the terminal window title from a template. Append the marker to
it in `~/.config/herdr/config.toml`:

```toml
[ui]
window_title = "{hostname}: {workspace}⁣sd:herdr:{workspace}"
```

Then reload: `herdr server reload-config`.

`⁣` is U+2063 INVISIBLE SEPARATOR, which renders as nothing — the title
still looks normal in the window list and in Alt-Tab. The part before it is
Herdr's own default; keep whatever you already use and append the marker.

The value after `sd:herdr:` is the join key. `{workspace}` is the right choice
when several windows attach to the same Herdr session, so each window is
matched against the workspace it is actually showing. With a single window,
`sd:herdr:*` also works and skips the check.

### 2. Configure the integration

Add an `AppIntegrations` entry to `~/.config/SpotlightDimmer/config.json`
(or add it in the config GUI's **Integrations** tab):

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
      "WmClass": "com.mitchellh.ghostty",
      "Provider": "herdr",
      "ContentOffsetX": 2,
      "ContentOffsetY": 2
    }
  ]
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `WmClass` | string | (required) | Window class to match. Find it with `Alt+F2` → `lg` → `global.display.focus_window.get_wm_class()`, or `xprop WM_CLASS` on X11 |
| `Provider` | string | `"tmux"` | Set to `"herdr"` for this integration |
| `SocketPath` | string | (empty) | Herdr session socket. Empty uses `$HERDR_SOCKET_PATH`, then `~/.config/herdr/herdr.sock`. A named session (`herdr --session dev`) listens on `~/.config/herdr/sessions/<name>/herdr.sock` |
| `ContentOffsetX` | integer | `0` | Pixels of terminal padding on the left/right of the cell grid |
| `ContentOffsetY` | integer | `0` | Pixels of terminal padding above/below the cell grid |

`ContentOffsetX/Y` are the terminal's *internal* padding, measured from the
client area (window content, decorations excluded — adapters report it
separately, so title bars and borders are handled automatically in both
windowed and maximized states). For Ghostty they are `window-padding-x` /
`window-padding-y`, both `2` by default. Unlike the tmux provider, no tab-bar
height belongs here: Herdr's own tab bar is part of the cell grid it reports.

A window class can carry one entry per provider — a `tmux` entry and a `herdr`
entry for the same terminal coexist, since each provider recognizes its own
windows by the marker in the title. Only one Herdr session is read at a time:
the first `herdr` entry's `SocketPath` is the one the daemon connects to.

The config file hot-reloads; the daemon picks up the provider without a
restart. Changing `SocketPath` afterwards is the exception: the reader thread
is started once, so switching sessions needs
`systemctl --user restart spotlight-dimmer-daemon`. (On Wayland, a *newly installed* GNOME extension still needs a
logout.)

## Coordinate System

Herdr reports its layout in terminal cells, with the pane area already offset
by the sidebar and the tab bar:

```
area   = {x: 26, y: 1, width: 187, height: 55}   # sidebar 26 cols, tab bar 1 row
pane   = {x: 26, y: 1, width:  94, height: 55}   # focused pane, same grid
grid   = (area.x + area.width, area.y + area.height) = (213, 56)
```

which maps onto the window as

```
inner_width  = client.width  - 2 * ContentOffsetX
screen_x     = client.x + ContentOffsetX + round(pane.x * inner_width / grid.cols)
```

and likewise for the vertical axis, both edges being mapped so that adjacent
panes stay flush. `client` is the window's client-area rect (decorations
excluded) as reported by the compositor adapter, falling back to the decorated
frame for protocol v1 adapters. The result is clamped to the client area, so a
misconfigured offset can never highlight outside the window content.

Herdr's pane rects are contiguous — it draws no separate border cells — so,
unlike the tmux provider, no edge is extended by one cell.

## Fallback Behavior

The whole-window spotlight is used whenever any of these holds:

- The focused window's `WmClass` has no `AppIntegrations` entry
- The window title carries no `sd:herdr:` marker (the terminal is not running
  Herdr, or the `window_title` template was not updated)
- The marker names a workspace other than the one Herdr currently has focused
- Herdr is not running, or the socket path is wrong (the daemon retries every
  5 seconds and picks the session up when it comes back)
- Herdr reports no focused pane, or the computed rect is empty after clamping

## Limitations

Herdr's socket API (protocol 20) reports **panes** — nothing else about its
UI. So the spotlight follows the focused pane only:

- **The sidebar (spaces and agents panels) cannot be highlighted.** Nothing in
  the API says the sidebar has focus: when you move into it, into navigate
  mode (`prefix+g`) or into a picker, Herdr keeps reporting the same focused
  pane. The sidebar is dimmed along with everything else outside the pane.
- **Popups are not highlighted.** Herdr has `popup.close` and popup placements
  for plugin panes, but no event announces that a popup is open and nothing
  reports its rect — a `[[keys.command]] type = "popup"` binding produces no
  event at all.

Both would need an upstream Herdr feature: an event carrying the focused UI
target (pane / sidebar section / popup) together with its cell rect. Until
then, those regions are dimmed like the rest of the window.

## Troubleshooting

**Check Herdr's socket answers**:

```bash
printf '{"id":"t","method":"pane.layout","params":{}}\n' \
  | socat - UNIX-CONNECT:"${HERDR_SOCKET_PATH:-$HOME/.config/herdr/herdr.sock}"
```

The same thing without `socat`: `herdr pane layout --current`.

**Check the marker reaches the window title**: `xprop WM_NAME` on X11, or
`Alt+F2` → `lg` → `global.display.focus_window.get_title()` on GNOME Wayland.
The title must end with `sd:herdr:<workspace>`; if it does not, the
`window_title` template was not reloaded (`herdr server reload-config`).

**Watch the daemon**: it logs `herdr provider watching <path>` at startup and
`herdr session connected` / `disconnected` as the session comes and goes.

```bash
journalctl --user -f -u spotlight-dimmer-daemon
```

**Highlight is misaligned**: adjust `ContentOffsetX/Y` (they hot-reload). A
horizontal drift that grows toward the right edge means the padding is wrong,
not the grid; a vertical drift that grows toward the bottom means the terminal
has a row Herdr does not draw on.
