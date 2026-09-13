#!/usr/bin/env bash
# Geometry tests for spotlight-dimmer-tmux-report.sh.
#
# Runs the report script against shim `tmux` and `gdbus` executables placed
# first in PATH: the tmux shim answers `display-message -p` with a canned
# line, the gdbus shim records the UpdatePaneGeometry arguments. No tmux
# server or D-Bus session is needed.
#
#   bash SpotlightDimmer.LinuxDaemon/tools/tests/test-tmux-report.sh

set -u

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$here/../spotlight-dimmer-tmux-report.sh"

shims="$(mktemp -d)"
trap 'rm -rf "$shims"' EXIT

cat > "$shims/tmux" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$FAKE_TMUX_INFO"
EOF
cat > "$shims/gdbus" <<'EOF'
#!/usr/bin/env bash
# Keep the last five args: tty x y width height
args=("$@")
n=${#args[@]}
printf '%s\n' "${args[*]:n-5:5}" > "$FAKE_GDBUS_OUT"
EOF
chmod +x "$shims/tmux" "$shims/gdbus"

failures=0

# Client: 160x41 cells (1-row status bar at the bottom -> 40 window rows),
# 10x20 px cells, tty /dev/pts/3.
#   fields: pane_left|pane_top|pane_width|pane_height|cell_w|cell_h|tty|
#           status|status-position|client_width|client_height|pane_in_mode|nvim
check() {
    local name="$1" pane="$2" mode="$3" nvim="$4" want="$5"
    local out="$shims/out"
    rm -f "$out"

    FAKE_TMUX_INFO="$pane|10|20|/dev/pts/3|on|bottom|160|41|$mode|$nvim" \
    FAKE_GDBUS_OUT="$out" PATH="$shims:$PATH" bash "$script"

    local got
    got="$(cat "$out" 2>/dev/null || echo "<no report>")"
    if [ "$got" = "$want" ]; then
        echo "ok   $name"
    else
        echo "FAIL $name: got '$got', want '$want'"
        failures=$((failures + 1))
    fi
}

# Right-hand tmux pane of a vertical split: cells 81..159, border at col 80
RIGHT="81|0|79|40"

check "full-window pane, no nvim" "0|0|160|40" 0 "" \
    "/dev/pts/3 0 0 1600 800"
check "right pane takes its left border" "$RIGHT" 0 "" \
    "/dev/pts/3 800 0 800 800"

check "nvim single window = whole pane" "$RIGHT" 0 "79,40,0,0,79,40" \
    "/dev/pts/3 800 0 800 800"
check "nvim left split keeps the tmux border" "$RIGHT" 0 "79,40,0,0,41,40" \
    "/dev/pts/3 800 0 420 800"
check "nvim right split stops at the window edge" "$RIGHT" 0 "79,40,39,0,40,40" \
    "/dev/pts/3 1200 0 400 800"
check "nvim top split: no extension below" "0|0|160|40" 0 "160,40,0,0,160,21" \
    "/dev/pts/3 0 0 1600 420"

# Anything stale or malformed falls back to the whole tmux pane
check "grid mismatch (stale after resize)" "$RIGHT" 0 "80,40,0,0,41,40" \
    "/dev/pts/3 800 0 800 800"
check "garbage value" "$RIGHT" 0 "abc" \
    "/dev/pts/3 800 0 800 800"
check "extra field" "$RIGHT" 0 "79,40,0,0,41,40,1" \
    "/dev/pts/3 800 0 800 800"
check "rect outside the pane" "$RIGHT" 0 "79,40,50,0,40,40" \
    "/dev/pts/3 800 0 800 800"
check "empty rect" "$RIGHT" 0 "79,40,0,0,0,40" \
    "/dev/pts/3 800 0 800 800"
check "tmux mode covers the pane" "$RIGHT" 1 "79,40,0,0,41,40" \
    "/dev/pts/3 800 0 800 800"

if [ "$failures" -gt 0 ]; then
    echo "$failures failure(s)"
    exit 1
fi
echo "all passed"
