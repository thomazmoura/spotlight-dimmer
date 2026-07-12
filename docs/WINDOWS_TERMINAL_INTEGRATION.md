# Terminal Pane Integration (Windows)

SpotlightDimmer can highlight the **focused terminal pane** instead of the
whole terminal window. Three scenarios are supported:

1. **Native Windows Terminal panes** — split panes with `Alt+Shift+D` /
   `Alt+Shift+-` and the spotlight follows the focused pane. No extra setup
   beyond the config entry.
2. **tmux panes (tmux running inside WSL2)** — a Windows Terminal or WezTerm
   tab running tmux gets the spotlight on the focused *tmux* pane, driven by
   tmux hooks (like the Linux integration).
3. **WezTerm native panes** — resolved through the `wezterm` CLI, same
   strategy as the Linux version.

Whenever pane information is unavailable, SpotlightDimmer silently falls back
to the normal whole-window spotlight.

> **Platform**: Windows. For the Linux integration see
> [TMUX_INTEGRATION.md](TMUX_INTEGRATION.md).

## How It Works

```
PUSH side (tmux in WSL2)                     PULL side (Windows client)
────────────────────────                     ──────────────────────────
tmux hook fires (pane switch, resize, ...)   Focus changes to a terminal whose
 └─ spotlight-dimmer-tmux-report.sh            process matches AppIntegrations
     one `tmux display-message` call           └─ TerminalPaneTracker
     (CELL coordinates + total grid size)          ├─ windows-terminal: focused
     └─ SpotlightDimmer.PaneReport.exe             │   pane control located via
        (Windows exe, invoked via WSL              │   accessibility (MSAA)
        interop)                                   ├─ wezterm: `wezterm cli`
        └─ named pipe                              │   query chain
           \\.\pipe\SpotlightDimmer.PaneTracker    └─ title changes trigger a
                                                       debounced re-evaluation

The resolved pane rect substitutes the window rect in the Partial /
PartialWithActive overlay calculation: the four edge overlays extend from the
pane to the monitor edges, dimming everything else.
```

Key difference from Linux: **the wire protocol carries cells, not pixels**.
ConPTY (the Windows pseudo-console that hosts WSL) does not propagate pixel
cell sizes, so tmux's `client_cell_width`/`client_cell_height` read 0 inside
WSL. Instead, the hook script reports the pane rect and the total client size
in *cells*, and the Windows side derives pixels by dividing the terminal's
content rectangle by the cell grid. A side effect: tmux ≥ 3.0 suffices (the
Linux integration needs ≥ 3.4).

## Wire Protocol (v1)

One UTF-8 line per named-pipe connection, max 1024 bytes:

```
v1|update|tty=/dev/pts/3|cells=81,0,80,45|grid=162,46|status=1,bottom|wt=<WT_SESSION>|wz=<WEZTERM_PANE>|sid=$1
v1|clear|tty=/dev/pts/3
```

| Field | Content |
|-------|---------|
| `v1` | Protocol version. Unknown versions are dropped; unknown keys are ignored (forward compatible) |
| `update` / `clear` | Store new geometry / remove stored geometry (detach, exit) |
| `tty` | The tmux client's tty — the storage key |
| `cells` | `#{pane_left},#{pane_top},#{pane_width},#{pane_height}` in cells, relative to the tmux window area |
| `grid` | `#{client_width},#{client_height}` — the full terminal grid in cells (the ConPTY workaround) |
| `status` | Status bar rows (`off`→0, `on`→1, `2`..`5`) and position (`top`/`bottom`) |
| `wt` | `$WT_SESSION` join hint (optional) |
| `wz` | `$WEZTERM_PANE` join hint (optional; the deterministic join key for WezTerm) |
| `sid` | tmux session id (diagnostics only) |

## Requirements

- SpotlightDimmer for Windows with `Overlay.Mode` set to `Partial` or
  `PartialWithActive`
- For tmux: WSL2 with tmux ≥ 3.0
- For WezTerm: the `wezterm` CLI available on the Windows `PATH`

## Setup

### 1. Configure the integration

Add an `AppIntegrations` section to
`%AppData%\SpotlightDimmer\config.json` (JSON-only for now — not yet editable
in the Config GUI):

```json
{
  "Overlay": { "Mode": "PartialWithActive" },
  "AppIntegrations": [
    {
      "ProcessName": "WindowsTerminal.exe",
      "Provider": "windows-terminal",
      "ContentOffsetX": 8,
      "ContentOffsetY": 8
    },
    {
      "ProcessName": "wezterm-gui.exe",
      "Provider": "wezterm"
    }
  ]
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ProcessName` | string | (required) | Executable name to match, case-insensitive |
| `Provider` | string | `"tmux"` | `"windows-terminal"`, `"wezterm"`, or `"tmux"` (generic terminal running tmux full-window) |
| `ContentOffsetX` | integer | `0` | Padding from the content area's left/right/bottom edges to the cell grid |
| `ContentOffsetY` | integer | `0` | Padding from the content area's top edge (padding + tab bar, if any) |

For Windows Terminal the default `padding` is 8, so `ContentOffsetX: 8,
ContentOffsetY: 8` matches a default profile. The offsets refine the tmux
cell math only; a few pixels of error is not visually noticeable, and the
result is always clamped to the window frame.

The config hot-reloads — changes apply without restarting.

**Native Windows Terminal pane spotlighting works at this point.** The steps
below are only needed for tmux-in-WSL.

### 2. Install the WSL-side tools (tmux only)

**Recommended: one command from PowerShell.** The
`Install-WslTmuxIntegration.ps1` script performs steps 2 and 3 in one go —
it copies the tools into the distro, normalizes line endings, records the
`SpotlightDimmer.PaneReport.exe` location for the report script, wires
`~/.tmux.conf` idempotently (re-running never duplicates the line), reloads a
running tmux server, and smoke-tests the forwarder through WSL interop:

```powershell
# Installed copy (default per-user install):
& "$env:LOCALAPPDATA\Programs\Spotlight Dimmer\tools\Install-WslTmuxIntegration.ps1"

# Or from a repository checkout (also finds dev build outputs):
.\SpotlightDimmer.Scripts\Install-WslTmuxIntegration.ps1
```

Options: `-Distribution <name>` targets a specific WSL distro,
`-PaneReportExePath`/`-ToolsSourceDir` override the automatic probing, and
`-SkipTmuxConf` leaves `~/.tmux.conf` untouched. Re-run the script after
moving the exe (e.g. after installing a release over a dev build) — it
refreshes the recorded path.

**Manual alternative.** The installer ships the tools in `<install dir>\tools`
(`%LOCALAPPDATA%\Programs\Spotlight Dimmer\tools` for a default per-user
install). From inside WSL:

```bash
mkdir -p ~/.config/SpotlightDimmer/tools
cp "/mnt/c/Users/<you>/AppData/Local/Programs/Spotlight Dimmer/tools/"* \
   ~/.config/SpotlightDimmer/tools/
chmod +x ~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-report.sh
```

The report script locates `SpotlightDimmer.PaneReport.exe` automatically
(default install locations are probed and cached in
`~/.cache/spotlight-dimmer/report-exe`). For a non-standard location, set:

```bash
export SPOTLIGHT_DIMMER_REPORT_EXE="/mnt/c/path/to/SpotlightDimmer.PaneReport.exe"
```

### 3. Install the tmux hooks (tmux only)

Add to `~/.tmux.conf` inside WSL:

```tmux
source-file ~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf
```

Then reload: `tmux source-file ~/.tmux.conf`

> The snippet uses `set-hook -g`, which **replaces** existing global hooks for
> the same events. If you already define one of these hooks, merge the
> `run-shell` command into your hook instead.

### 4. Propagate join hints into WSL (WezTerm + tmux only)

For the deterministic WezTerm join, `WEZTERM_PANE` must be visible inside
WSL. Add to your Windows environment (System Properties → Environment
Variables, or PowerShell):

```powershell
[Environment]::SetEnvironmentVariable("WSLENV", "$env:WSLENV`:WEZTERM_PANE/u", "User")
```

Windows Terminal propagates `WT_SESSION` on its own in current releases; it
is used only as a soft hint, so nothing breaks if it is absent.

## How the Focused Pane Is Resolved

Stored tmux reports are keyed by tty. Because Windows cannot discover a WSL
pts from the terminal side (the way the Linux daemon matches ttys), selection
uses a fallback ladder — first match wins:

1. **WEZTERM_PANE hint** — the report's `wz` matches the focused wezterm
   pane id from `wezterm cli` (deterministic; WezTerm only).
2. **Single live report** — exactly one tmux client reported (the dominant
   case).
3. **Freshest report** — every pane switch fires a tmux hook, so the most
   recently updated report almost always belongs to the pane the user just
   interacted with. Best-effort for multi-tmux-client setups.
4. **Native pane** — the focused Windows Terminal / WezTerm pane rect without
   tmux sub-resolution.
5. **Whole window** — the normal spotlight.

tmux detach/exit is handled by an explicit `clear` report (the
`client-detached` hook) plus a debounced re-evaluation on terminal title
changes (tmux sets the title, so attach/detach always changes it).

## Implementation Notes

- **Windows Terminal pane location**: WT exposes its panes ("TermControl")
  through UI Automation. Pane switches never change the foreground window,
  but they do raise `EVENT_OBJECT_FOCUS` accessibility winevents, which
  SpotlightDimmer hooks per-process (zero polling). The event's accessible
  object reports the pane's exact screen rect (`IAccessible.accLocation`),
  which is cached so window drags cost one COM call per frame.
  - Validated empirically: focus events fire per pane switch and the rects
    match pane geometry exactly (e.g. a top-left quadrant of a 1113×586
    content area reports (234,275) 555×291).
  - A freshly split pane can fail `accLocation` for a moment
    (`DISP_E_MEMBERNOTFOUND`); the tracker re-probes after the title-change
    debounce (150 ms).
- **Windows Terminal single-pane windows raise no focus events** (there is
  nothing to switch to); the tracker then resolves the focused control via
  the MSAA `accFocus` chain when the terminal gains foreground.
- **No polling anywhere**: winevent hooks, named-pipe async I/O, and
  `SetTimer`-based debouncing only, consistent with the app's event-driven
  architecture.
- The pipe server accepts one line per connection; the helper exe is a dumb
  forwarder (the shell script owns the protocol), AOT-compiled for fast
  startup under WSL interop.

## Troubleshooting

**Check reports are arriving**: run SpotlightDimmer with `--verbose` (or set
`System.LogLevel` to `Debug`) and watch
`%AppData%\SpotlightDimmer\logs` for `[PANE]` lines.

**Push a fake pane rect** (overlays should snap to the left half of the
terminal while it is focused; switch tmux panes to overwrite it):

```powershell
& "$env:LOCALAPPDATA\Programs\Spotlight Dimmer\SpotlightDimmer.PaneReport.exe" `
  "v1|update|tty=/dev/pts/0|cells=0,0,80,45|grid=160,45|status=0,bottom"
```

**Run the helper by hand** from inside tmux in WSL:

```bash
bash -x ~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-report.sh
```

**Verify the hooks are installed**:

```bash
tmux show-hooks -g | grep spotlight
```

**Highlight is misaligned**: adjust `ContentOffsetX/Y` (they hot-reload).
Vertical misalignment by exactly one cell height usually means the status bar
position assumption is off — check `status` in a hand-run of the report
script.

**WezTerm pane not detected**: verify `wezterm cli list-clients --format json`
works from PowerShell, and that `WSLENV` propagates `WEZTERM_PANE` (run
`echo $WEZTERM_PANE` inside the WSL tab).

## Runtime Validation Status

Validated on Windows 11 with Windows Terminal:

- [x] `EVENT_OBJECT_FOCUS` fires per WT pane switch (multi-pane windows)
- [x] `AccessibleObjectFromEvent`/`accLocation` returns exact pane content rects
- [x] Cached `IAccessible` stays valid across time and re-queries
- [x] Title changes (`EVENT_OBJECT_NAMECHANGE`) fire on pane switches and
      shell activity — used as the debounced re-evaluation trigger
- [x] Helper exe → named pipe → parser chain end-to-end
- [x] Transient `accLocation` failure right after `split-pane` (handled via
      debounce re-probe)

Validated on Windows 11 with Windows Terminal + WSL2 (Ubuntu 26.04,
tmux 3.6):

- [x] tmux hook → report script → PaneReport.exe (WSL interop) → named pipe →
      overlay chain end-to-end: split, pane switch, zoom/unzoom, and resize
      all move the spotlight to the focused pane's pixel rect
- [x] `WT_SESSION` is visible inside WSL2 on current Windows Terminal
      releases (WT injects it into `WSLENV` itself)
- [x] End-to-end tmux hook latency through WSL interop: ~165 ms from tmux
      command to report received with a non-AOT (JIT) debug build of
      PaneReport.exe; the AOT release exe is faster. Perceptually fine.
- [x] `client-detached` clear delivers the detached client's tty — via
      `#{hook_client}` in the hook (`#{client_tty}` expands empty because the
      client is already gone when the hook runs; this was a bug found and
      fixed during validation)
- [x] ConPTY cell size inside WSL2: contrary to the original design
      assumption, tmux 3.6 reports real values (`client_cell_width` = 10,
      `client_cell_height` = 20), not 0. The cell math never uses them, so
      behavior is unaffected either way; the cells-based wire protocol stays.

Still needing validation (no WezTerm on the validation machine):

- [ ] WezTerm-on-Windows: `wezterm cli list` reporting non-zero
      `pixel_width`/`pixel_height`, and `WEZTERM_PANE` propagation via `WSLENV`

Known limitation: a report whose clear is never delivered (tmux server
killed, WSL VM torn down, terminal crash) lingers and keeps shrinking the
spotlight of the matched terminal. Detach/attach any tmux client or touch
`config.json` (a config reload clears all stored reports) to recover.
