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
#
# Two modes:
#
#   spotlight-dimmer-tmux-report.sh
#       Report the focused pane. This is what the hooks call.
#
#   spotlight-dimmer-tmux-report.sh --rect <left> <top> <width> <height>
#       Report an arbitrary rectangle given in CELLS, absolute in the client
#       grid (origin = top-left of the terminal's cell grid, status bar
#       included). Used by spotlight-dimmer-tmux-popup.sh to spotlight a
#       display-popup, which is an overlay rather than a pane and so has no
#       #{pane_*} geometry of its own.
#
# Options, combinable with either mode:
#
#   --client <tty>   Query that tmux client instead of the current one. Used
#                    by the daemon, which runs outside any tmux context.
#   --print          Print "<tty> <x> <y> <width> <height>" instead of sending
#                    it over D-Bus (the daemon applies the answer itself).

set -u

is_uint() {
    case "$1" in
        ''|*[!0-9]*) return 1 ;;
        *) return 0 ;;
    esac
}

# --- Argument parsing ------------------------------------------------------
#
# The rectangle is validated up front: a caller that computed nonsense must not
# push it to the daemon, and (like every other failure here) must not fail the
# hook or binding it came from either.
rect_mode=0
client=""
print=0
while [ $# -gt 0 ]; do
    case "$1" in
        --rect)
            [ $# -ge 5 ] || exit 0
            rect_mode=1
            rect_left=$2
            rect_top=$3
            rect_width=$4
            rect_height=$5
            shift 5
            ;;
        --client)
            [ $# -ge 2 ] || exit 0
            client=$2
            shift 2
            ;;
        --print)
            print=1
            shift
            ;;
        *)
            exit 0
            ;;
    esac
done
if [ "$rect_mode" = 1 ]; then
    for value in "$rect_left" "$rect_top" "$rect_width" "$rect_height"; do
        is_uint "$value" || exit 0
    done
    [ "$rect_width" -gt 0 ] || exit 0
    [ "$rect_height" -gt 0 ] || exit 0
fi

# Everything needed in a single tmux call:
# pane geometry (cells), cell pixel size, client tty, status bar layout,
# client grid size (to detect pane borders at the window-area edges), and the
# focused neovim split published by the SpotlightDimmer neovim plugin
# (SpotlightDimmer.NeovimPlugin), if any, plus whether a tmux mode covers it.
# The plugin publishes the split in the @spotlight_dimmer_nvim pane option
# when it runs inside this tmux, and in its pane title (sd-nvim=...) when it
# runs over ssh, where it cannot reach this tmux. The pane title comes last
# because it is free text: `read` hands the last variable the rest of the
# line, '|' characters included.
# --rect uses only the cell size and the tty; the query is left whole so both
# modes stay a single round trip to the tmux server.
target=()
[ -n "$client" ] && target=(-c "$client")
info="$(tmux display-message "${target[@]}" -p '#{pane_left}|#{pane_top}|#{pane_width}|#{pane_height}|#{client_cell_width}|#{client_cell_height}|#{client_tty}|#{status}|#{status-position}|#{client_width}|#{client_height}|#{pane_in_mode}|#{@spotlight_dimmer_nvim}|#{pane_title}' 2>/dev/null)" || exit 0

IFS='|' read -r pane_left pane_top pane_width pane_height cell_w cell_h tty status status_position client_width client_height pane_in_mode nvim_rect pane_title <<< "$info"

# The title is untrusted (any program in the pane can set it): the regex only
# extracts digits and commas, and the value is validated like the option.
if [ -z "${nvim_rect:-}" ] && [[ "${pane_title:-}" =~ sd-nvim=([0-9,]+) ]]; then
    nvim_rect=${BASH_REMATCH[1]}
fi

# Without a client (detached context) or cell pixel info (terminal does not
# report pixel sizes) there is nothing useful to report.
[ -n "${tty:-}" ] || exit 0
[ "${cell_w:-0}" -gt 0 ] 2>/dev/null || exit 0
[ "${cell_h:-0}" -gt 0 ] 2>/dev/null || exit 0

if [ "$rect_mode" = 1 ]; then
    # Neither of the pane-specific adjustments below applies to a popup:
    # its coordinates are already absolute over the whole client grid (so no
    # status-bar offset), and its border is drawn inside its own width and
    # height (so no edge extension).
    pane_left=$rect_left
    pane_top=$rect_top
    pane_width=$rect_width
    pane_height=$rect_height
    y_offset=0
else
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

    # Neovim split: when the pane runs neovim with the SpotlightDimmer plugin,
    # narrow the pane to the focused split. The option holds
    # "<grid_cols>,<grid_rows>,<col>,<row>,<width>,<height>" in cells relative to
    # the pane; it is honoured only while it is well-formed, fits the pane, and
    # was computed for the pane's current size (a resize leaves it stale until
    # neovim re-reports). A tmux mode (copy-mode, choose-tree) covers the whole
    # pane, so the split is ignored while one is active.
    # The touch_* flags record which inner edges lie on the pane edge: only those
    # can border another tmux pane; interior edges are framed by neovim's own
    # separators, which the plugin already includes.
    touch_left=1 touch_top=1 touch_right=1 touch_bottom=1
    if [ -n "${nvim_rect:-}" ] && [ "${pane_in_mode:-0}" = "0" ]; then
        IFS=',' read -r grid_cols grid_rows n_col n_row n_width n_height n_extra <<< "$nvim_rect"
        if [ -z "${n_extra:-}" ] \
            && is_uint "${grid_cols:-}" && is_uint "${grid_rows:-}" \
            && is_uint "${n_col:-}" && is_uint "${n_row:-}" \
            && is_uint "${n_width:-}" && is_uint "${n_height:-}" \
            && [ "$grid_cols" -eq "$pane_width" ] && [ "$grid_rows" -eq "$pane_height" ] \
            && [ "$n_width" -gt 0 ] && [ "$n_height" -gt 0 ] \
            && [ $((n_col + n_width)) -le "$pane_width" ] \
            && [ $((n_row + n_height)) -le "$pane_height" ]; then
            [ "$n_col" -eq 0 ] || touch_left=0
            [ "$n_row" -eq 0 ] || touch_top=0
            [ $((n_col + n_width)) -eq "$pane_width" ] || touch_right=0
            [ $((n_row + n_height)) -eq "$pane_height" ] || touch_bottom=0
            pane_left=$((pane_left + n_col))
            pane_top=$((pane_top + n_row))
            pane_width=$n_width
            pane_height=$n_height
        fi
    fi

    # tmux pane coordinates cover only the pane interior: the single-cell border
    # lines drawn between panes belong to no pane. Extend every edge that has an
    # adjacent border (does not touch the window-area edge) by one cell so the
    # border characters are highlighted together with the focused pane.
    window_rows=$(( ${client_height:-0} - status_rows ))
    if [ "$touch_left" -eq 1 ] && [ "$pane_left" -gt 0 ]; then
        pane_left=$((pane_left - 1))
        pane_width=$((pane_width + 1))
    fi
    if [ "$touch_top" -eq 1 ] && [ "$pane_top" -gt 0 ]; then
        pane_top=$((pane_top - 1))
        pane_height=$((pane_height + 1))
    fi
    if [ "$touch_right" -eq 1 ] && [ $((pane_left + pane_width)) -lt "${client_width:-0}" ]; then
        pane_width=$((pane_width + 1))
    fi
    if [ "$touch_bottom" -eq 1 ] && [ $((pane_top + pane_height)) -lt "$window_rows" ]; then
        pane_height=$((pane_height + 1))
    fi
fi

x=$((pane_left * cell_w))
y=$((y_offset + pane_top * cell_h))
width=$((pane_width * cell_w))
height=$((pane_height * cell_h))

if [ "$print" = 1 ]; then
    printf '%s %s %s %s %s\n' "$tty" "$x" "$y" "$width" "$height"
    exit 0
fi

# Silent no-op when the extension (and its D-Bus service) is not running.
gdbus call --session \
    --dest org.spotlightdimmer.PaneTracker \
    --object-path /org/spotlightdimmer/PaneTracker \
    --method org.spotlightdimmer.PaneTracker.UpdatePaneGeometry \
    "$tty" "$x" "$y" "$width" "$height" \
    >/dev/null 2>&1 || true

exit 0
