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
# pane geometry (cells), cell pixel size, client tty, status bar layout,
# client grid size (to detect pane borders at the window-area edges).
info="$(tmux display-message -p '#{pane_left}|#{pane_top}|#{pane_width}|#{pane_height}|#{client_cell_width}|#{client_cell_height}|#{client_tty}|#{status}|#{status-position}|#{client_width}|#{client_height}' 2>/dev/null)" || exit 0

IFS='|' read -r pane_left pane_top pane_width pane_height cell_w cell_h tty status status_position client_width client_height <<< "$info"

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

# tmux pane coordinates cover only the pane interior: the single-cell border
# lines drawn between panes belong to no pane. Extend every edge that has an
# adjacent border (does not touch the window-area edge) by one cell so the
# border characters are highlighted together with the focused pane.
window_rows=$(( ${client_height:-0} - status_rows ))
if [ "$pane_left" -gt 0 ]; then
    pane_left=$((pane_left - 1))
    pane_width=$((pane_width + 1))
fi
if [ "$pane_top" -gt 0 ]; then
    pane_top=$((pane_top - 1))
    pane_height=$((pane_height + 1))
fi
if [ $((pane_left + pane_width)) -lt "${client_width:-0}" ]; then
    pane_width=$((pane_width + 1))
fi
if [ $((pane_top + pane_height)) -lt "$window_rows" ]; then
    pane_height=$((pane_height + 1))
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
