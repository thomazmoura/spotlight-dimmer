#!/usr/bin/env bash
# SpotlightDimmer tmux integration: report the focused pane's geometry to the
# GNOME Shell extension via D-Bus.
#
# Invoked by tmux hooks (see spotlight-dimmer.tmux.conf). It must be fast and
# must never fail the hook, so every failure path exits 0 silently.
#
# Coordinates sent are PIXELS relative to the terminal content origin (the
# top-left corner of the terminal's cell grid). The extension adds the window
# position and any configured content offsets.

set -u

# Everything needed in a single tmux call:
# pane geometry (cells), cell pixel size, client tty, status bar layout.
info="$(tmux display-message -p '#{pane_left}|#{pane_top}|#{pane_width}|#{pane_height}|#{client_cell_width}|#{client_cell_height}|#{client_tty}|#{status}|#{status-position}' 2>/dev/null)" || exit 0

IFS='|' read -r pane_left pane_top pane_width pane_height cell_w cell_h tty status status_position <<< "$info"

# Without a client (detached context) or cell pixel info (terminal does not
# report pixel sizes) there is nothing useful to report.
[ -n "${tty:-}" ] || exit 0
[ "${cell_w:-0}" -gt 0 ] 2>/dev/null || exit 0
[ "${cell_h:-0}" -gt 0 ] 2>/dev/null || exit 0

# Rows occupied by the status bar: "off" = 0, "on" = 1, "2".."5" = that many.
case "$status" in
    off) status_rows=0 ;;
    on)  status_rows=1 ;;
    *)   status_rows=$status ;;
esac

# tmux pane coordinates are relative to the window area, which sits BELOW the
# status bar when status-position is top.
y_offset=0
if [ "$status_position" = "top" ]; then
    y_offset=$((status_rows * cell_h))
fi

x=$((pane_left * cell_w))
y=$((y_offset + pane_top * cell_h))
width=$((pane_width * cell_w))
height=$((pane_height * cell_h))

# Silent no-op when the extension (and its D-Bus service) is not running.
gdbus call --session \
    --dest org.spotlightdimmer.PaneTracker \
    --object-path /org/spotlightdimmer/PaneTracker \
    --method org.spotlightdimmer.PaneTracker.UpdatePaneGeometry \
    "$tty" "$x" "$y" "$width" "$height" \
    >/dev/null 2>&1 || true

exit 0
