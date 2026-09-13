#!/usr/bin/env bash
# SpotlightDimmer tmux integration: spotlight a tmux popup while it is open.
#
# Usage, as the shell-command of a display-popup binding:
#
#   bind t display-popup -E -w 80% -h 60% -x C -y C \
#     '~/.config/SpotlightDimmer/tools/spotlight-dimmer-tmux-popup.sh -- my-script.sh'
#
# Why this wrapper has to exist at all:
#
#   - tmux fires no hook when display-popup opens or closes, so the hooks in
#     spotlight-dimmer.tmux.conf never learn that anything happened.
#   - A popup is an overlay drawn over the panes, not a pane. #{pane_left} and
#     friends keep describing the pane UNDERNEATH it, and tmux 3.4 has no
#     #{popup_*} formats to ask instead.
#
#   The spotlight therefore stays on the previously focused pane and the popup
#   sits in the dimmed region. There is no way to fix that from the outside;
#   the popup's geometry is only knowable from inside the popup itself.
#
# How the geometry is recovered:
#
#   tmux sizes the popup's pty to the popup INTERIOR, so `stty size` gives the
#   popup's inner rows and columns exactly -- no need to repeat the binding's
#   -w/-h percentages here, and no rounding to replicate. Adding the border
#   back gives the outer size, and the position follows from tmux centring the
#   popup in the client grid.
#
# Assumptions, both of which match tmux's own defaults:
#
#   - The popup is CENTRED (`-x C -y C`, which is also what tmux does when -x
#     and -y are omitted -- the two place a popup identically). A popup pinned
#     elsewhere with an explicit -x/-y is not supported; it would be
#     spotlighted in the wrong place.
#   - The popup has a border. Pass --no-border for a popup opened with -B.
#
# Like the reporter it delegates to, this must never break the command it
# wraps: every SpotlightDimmer failure path is silent, and the wrapped
# command's exit status is passed through untouched so `display-popup -EE`
# keeps working.

set -u

REPORT="$(dirname "$0")/spotlight-dimmer-tmux-report.sh"

border=1
while [ $# -gt 0 ]; do
    case "$1" in
        --no-border) border=0; shift ;;
        --) shift; break ;;
        *) break ;;
    esac
done

if [ $# -eq 0 ]; then
    echo "usage: $(basename "$0") [--no-border] -- <command> [args...]" >&2
    exit 2
fi

# Report the popup's rectangle, best-effort. Anything unexpected here leaves
# the previous (pane) rect in place, which is exactly today's behaviour.
report_popup() {
    [ -x "$REPORT" ] || return 0

    local size rows cols grid client_width client_height w h x y
    size="$(stty size < /dev/tty 2>/dev/null)" || return 0
    read -r rows cols <<< "$size"
    case "${rows:-}${cols:-}" in ''|*[!0-9]*) return 0 ;; esac

    grid="$(tmux display-message -p '#{client_width}|#{client_height}' 2>/dev/null)" || return 0
    IFS='|' read -r client_width client_height <<< "$grid"
    case "${client_width:-}${client_height:-}" in ''|*[!0-9]*) return 0 ;; esac

    w=$((cols + 2 * border))
    h=$((rows + 2 * border))

    # tmux's own centring, which is NOT the symmetric (size - popup) / 2 the two
    # axes look like they should share: the vertical rounds the half-height up
    # where the horizontal truncates, so a popup with an odd height sits one row
    # higher than the naive formula predicts. Measured against tmux 3.4 by
    # reading the cursor-positioning it writes to a captured client, over both
    # parities of client and popup size; `-x C -y C` and the default position
    # (no -x/-y) place a popup identically.
    x=$(((client_width - 1) / 2 - w / 2))
    y=$(((client_height - 1) / 2 - (h + 1) / 2))
    [ "$x" -ge 0 ] || x=0
    [ "$y" -ge 0 ] || y=0

    "$REPORT" --rect "$x" "$y" "$w" "$h"
}

# Restore the focused pane's rect on the way out. Armed BEFORE the command
# runs, and on signals as well as a normal exit, so the spotlight comes back
# however the popup ends -- including the abort path (fzf cancelled with Esc),
# which runs no tmux command at all and so fires no hook either.
restore_pane() {
    [ -x "$REPORT" ] && "$REPORT"
    return 0
}
trap restore_pane EXIT INT TERM HUP

report_popup

"$@"
exit $?
