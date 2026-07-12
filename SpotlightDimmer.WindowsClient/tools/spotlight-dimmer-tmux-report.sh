#!/usr/bin/env bash
# SpotlightDimmer tmux integration (Windows/WSL): report the focused pane's
# geometry to the SpotlightDimmer Windows client.
#
# Invoked by tmux hooks (see spotlight-dimmer.tmux.conf) inside WSL. It calls
# SpotlightDimmer.PaneReport.exe through WSL's Windows interop, which forwards
# the message to the client over the \\.\pipe\SpotlightDimmer.PaneTracker
# named pipe. It must be fast and must never fail the hook, so every failure
# path exits 0 silently.
#
# Unlike the Linux helper, coordinates are sent in CELLS (plus the total
# client grid size): ConPTY does not propagate pixel cell sizes into WSL, so
# the Windows side derives pixels from the terminal's content rectangle.
# Because of that, tmux >= 3.0 suffices (no 3.4 pixel-format requirement).
#
# Wire protocol v1 (one line, max 1024 bytes):
#   v1|update|tty=<pts>|cells=<l>,<t>,<w>,<h>|grid=<cols>,<rows>|status=<rows>,<top|bottom>[|wt=<WT_SESSION>][|wz=<WEZTERM_PANE>][|sid=<session>]
#   v1|clear|tty=<pts>
#
# The Windows exe is located via (first match wins):
#   1. $SPOTLIGHT_DIMMER_REPORT_EXE
#   2. ~/.cache/spotlight-dimmer/report-exe (cached probe result)
#   3. The default install locations under /mnt/c

set -u

mode=update
[ "${1:-}" = "--clear" ] && mode=clear

# ----------------------------------------------------------------------------
# Locate SpotlightDimmer.PaneReport.exe
# ----------------------------------------------------------------------------
find_exe() {
    if [ -n "${SPOTLIGHT_DIMMER_REPORT_EXE:-}" ] && [ -x "$SPOTLIGHT_DIMMER_REPORT_EXE" ]; then
        printf '%s' "$SPOTLIGHT_DIMMER_REPORT_EXE"
        return 0
    fi

    local cache="$HOME/.cache/spotlight-dimmer/report-exe"
    if [ -f "$cache" ]; then
        local cached
        cached="$(cat "$cache" 2>/dev/null)"
        if [ -n "$cached" ] && [ -x "$cached" ]; then
            printf '%s' "$cached"
            return 0
        fi
    fi

    local candidate
    for candidate in \
        /mnt/c/Users/*/AppData/Local/Programs/"Spotlight Dimmer"/SpotlightDimmer.PaneReport.exe \
        "/mnt/c/Program Files/Spotlight Dimmer/SpotlightDimmer.PaneReport.exe"; do
        if [ -x "$candidate" ]; then
            mkdir -p "$(dirname "$cache")" 2>/dev/null
            printf '%s' "$candidate" > "$cache" 2>/dev/null
            printf '%s' "$candidate"
            return 0
        fi
    done

    return 1
}

exe="$(find_exe)" || exit 0

# ----------------------------------------------------------------------------
# Gather geometry - everything needed in a single tmux call
# ----------------------------------------------------------------------------
info="$(tmux display-message -p '#{pane_left}|#{pane_top}|#{pane_width}|#{pane_height}|#{client_width}|#{client_height}|#{status}|#{status-position}|#{client_tty}|#{session_id}' 2>/dev/null)" || exit 0

IFS='|' read -r pane_left pane_top pane_width pane_height client_width client_height status status_position tty session_id <<< "$info"

# Without a client (detached context) there is nothing useful to report.
[ -n "${tty:-}" ] || exit 0

if [ "$mode" = "clear" ]; then
    "$exe" "v1|clear|tty=$tty" >/dev/null 2>&1 || true
    exit 0
fi

[ "${pane_width:-0}" -gt 0 ] 2>/dev/null || exit 0
[ "${client_width:-0}" -gt 0 ] 2>/dev/null || exit 0

# Rows occupied by the status bar: "off" = 0, "on" = 1, "2".."5" = that many.
case "$status" in
    off) status_rows=0 ;;
    on)  status_rows=1 ;;
    *)   status_rows=$status ;;
esac
[ "$status_position" = "top" ] && status_pos=top || status_pos=bottom

# ----------------------------------------------------------------------------
# Join hints: the tmux session environment first (kept fresh across attaches
# by 'set -ga update-environment' in spotlight-dimmer.tmux.conf), then this
# process's environment.
# ----------------------------------------------------------------------------
session_env() {
    local value
    value="$(tmux show-environment "$1" 2>/dev/null)"
    case "$value" in
        "$1"=*) printf '%s' "${value#*=}" ;;
        *) printf '' ;;
    esac
}

wt="$(session_env WT_SESSION)"
[ -n "$wt" ] || wt="${WT_SESSION:-}"
wz="$(session_env WEZTERM_PANE)"
[ -n "$wz" ] || wz="${WEZTERM_PANE:-}"

# ----------------------------------------------------------------------------
# Compose and send (empty-valued keys are omitted)
# ----------------------------------------------------------------------------
line="v1|update|tty=$tty|cells=$pane_left,$pane_top,$pane_width,$pane_height|grid=$client_width,$client_height|status=$status_rows,$status_pos"
[ -n "$wt" ] && line="$line|wt=$wt"
[ -n "$wz" ] && line="$line|wz=$wz"
[ -n "$session_id" ] && line="$line|sid=$session_id"

"$exe" "$line" >/dev/null 2>&1 || true

exit 0
