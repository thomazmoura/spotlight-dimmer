#!/usr/bin/env bash
# Install SpotlightDimmer on Ubuntu (GNOME) or Kubuntu (KDE Plasma 6) from
# this repository checkout, replacing whatever was installed before.
#
# Usage:
#   SpotlightDimmer.LinuxDaemon/tools/install-local.sh [--gnome|--kde] [--no-config-gui] [--yes]
#
# The result is a clean install of the code in this checkout:
#   - Everything is built first, so a failed build leaves the current install alone.
#   - The release packages (spotlight-dimmer-gnome/-kde/-config) are purged.
#   - Every file of a previous source install is deleted before the new copies
#     go in, so files that no longer exist in the repo do not linger.
# Only your settings carry over: ~/.config/SpotlightDimmer/config.json, the
# GNOME enabled-extensions list, and KDE shortcut bindings.
#
# Run it as your normal user: apt runs through sudo, while the per-user steps
# (extension enabling, KDE shortcuts, daemon restart) need your session.
set -euo pipefail

EXT_UUID="spotlightdimmer@thomazmoura.github.io"
RELEASE_PACKAGES=(spotlight-dimmer-gnome spotlight-dimmer-kde spotlight-dimmer-config)

variant=""
with_config=1
assume_yes=0

log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mWARNING:\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'EOF'
Usage: install-local.sh [options]

Builds and installs SpotlightDimmer from this repository checkout. Removes the
release packages and any previous source install first; only your settings
(~/.config/SpotlightDimmer/config.json, enabled extensions, KDE shortcuts) are
kept.

  --gnome / --kde    Skip desktop detection and install this variant
  --no-config-gui    Do not build or install the spotlight-dimmer-config settings window
  -y, --yes          Do not ask apt for confirmation when purging the packages
EOF
    exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --gnome) variant=gnome ;;
        --kde) variant=kde ;;
        --no-config-gui) with_config=0 ;;
        -y|--yes) assume_yes=1 ;;
        -h|--help) usage 0 ;;
        *) warn "unknown option: $1"; usage 1 ;;
    esac
    shift
done

[[ $EUID -ne 0 ]] || die "run this as your normal user, not root (sudo is used for apt only)"
for cmd in cargo make python3; do
    command -v "$cmd" >/dev/null || die "'$cmd' is required (Rust comes from https://rustup.rs)"
done

daemon_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo_root="$(cd "$daemon_dir/.." && pwd)"
[[ -f "$daemon_dir/Cargo.toml" && -d "$repo_root/SpotlightDimmer.GnomeShellExtension" ]] \
    || die "run this script from a SpotlightDimmer repository checkout"

# --- Detect the desktop ------------------------------------------------------
# Same detection as install-release.sh: /etc/os-release reports "Ubuntu" on
# Kubuntu too, so look at the desktop instead.
pkg_installed() {
    dpkg-query -W -f='${db:Status-Status}' "$1" 2>/dev/null | grep -qx installed
}

detect_variant() {
    case "${XDG_CURRENT_DESKTOP:-}${DESKTOP_SESSION:-}" in
        *KDE*|*kde*|*plasma*) echo kde; return ;;
        *GNOME*|*gnome*|*ubuntu*) echo gnome; return ;;
    esac
    if pkg_installed kubuntu-desktop; then echo kde; return; fi
    if pkg_installed ubuntu-desktop || pkg_installed ubuntu-desktop-minimal; then echo gnome; return; fi
    if command -v plasmashell >/dev/null; then echo kde; return; fi
    if command -v gnome-shell >/dev/null; then echo gnome; return; fi
}

if [[ -z "$variant" ]]; then
    variant="$(detect_variant)"
    [[ -n "$variant" ]] || die "could not detect GNOME or KDE; pass --gnome or --kde"
    log "Detected desktop: $variant ($([[ $variant == kde ]] && echo Kubuntu || echo Ubuntu))"
fi

# --- Say what is being installed ---------------------------------------------
version="$(awk -F'"' '/^\[workspace.package\]/ { ws = 1 } ws && /^version/ { print $2; exit }' "$daemon_dir/Cargo.toml")"
revision="not a git checkout"
if git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1; then
    revision="$(git -C "$repo_root" rev-parse --abbrev-ref HEAD) @ $(git -C "$repo_root" rev-parse --short HEAD)"
    [[ -n "$(git -C "$repo_root" status --porcelain)" ]] && revision+=" + uncommitted changes"
fi

log "Installing SpotlightDimmer from the repository, NOT from a release"
echo "   Source:   $repo_root"
echo "   Version:  $version ($revision)"
echo "   Desktop:  $variant"
echo "   Replaces: the release packages and any previous source install"
echo "   Keeps:    ~/.config/SpotlightDimmer/config.json and your desktop settings"

# --- Build -------------------------------------------------------------------
# Before anything is removed, so a failed build leaves the current install in
# place. GNOME renders the overlays in the extension, so its daemon is built
# without the GTK layer-shell renderer, as in the spotlight-dimmer-gnome package.
features=""
[[ "$variant" == gnome ]] && features="--no-default-features"

log "Building the daemon"
make -C "$daemon_dir" --no-print-directory build-daemon FEATURES="$features"

if (( with_config )); then
    if pkg-config --exists 'gtk4 >= 4.12' 2>/dev/null; then
        log "Building the settings window"
        make -C "$daemon_dir" --no-print-directory build-config-gui
    else
        warn "skipping the settings window: GTK4 >= 4.12 headers not found (sudo apt install libgtk-4-dev)"
        with_config=0
    fi
fi

# Sampled before anything is removed from under a running daemon.
daemon_active=0
systemctl --user is-active --quiet spotlight-dimmer-daemon.service 2>/dev/null && daemon_active=1

# --- Remove the release packages ---------------------------------------------
installed_packages=()
for pkg in "${RELEASE_PACKAGES[@]}"; do
    pkg_installed "$pkg" && installed_packages+=("$pkg")
done
if [[ ${#installed_packages[@]} -gt 0 ]]; then
    apt_flags=()
    (( assume_yes )) && apt_flags+=(-y)
    log "Purging the release packages (sudo): ${installed_packages[*]}"
    sudo apt purge "${apt_flags[@]}" "${installed_packages[@]}"
fi

# --- Remove the previous source install --------------------------------------
# Every per-user file `make install-linux-*` (or this script) has ever put in
# place. The tools directory goes too: it is rebuilt below from the repo, and
# ~/.tmux.conf keeps sourcing the same path.
previous_install=(
    "$HOME/.local/bin/spotlight-dimmer-daemon"
    "$HOME/.config/systemd/user/spotlight-dimmer-daemon.service"
    "$HOME/.local/share/dbus-1/services/org.spotlightdimmer.Daemon.service"
    "$HOME/.local/share/applications/org.spotlightdimmer.toggle.desktop"
    "$HOME/.local/share/gnome-shell/extensions/$EXT_UUID"
    "$HOME/.local/share/kwin/scripts/spotlightdimmer"
    "$HOME/.local/bin/spotlight-dimmer-config"
    "$HOME/.local/share/applications/org.spotlightdimmer.Config.desktop"
    "$HOME/.local/share/applications/org.spotlightdimmer.ConfigToggle.desktop"
    "$HOME/.local/share/icons/hicolor/scalable/apps/org.spotlightdimmer.Config.svg"
    "$HOME/.config/SpotlightDimmer/tools"
)
removed_any=0
for path in "${previous_install[@]}"; do
    [[ -e "$path" || -L "$path" ]] || continue
    (( removed_any )) || log "Removing the previous source install"
    removed_any=1
    if [[ "$path" == */kwin/scripts/* ]] && command -v kpackagetool6 >/dev/null; then
        kpackagetool6 --type KWin/Script --remove spotlightdimmer >/dev/null 2>&1 || true
    fi
    rm -rf "$path"
    echo "   removed $path"
done

# --- Install from the repository ---------------------------------------------
targets=(install-daemon install-config install-tools)
(( with_config )) && targets+=(install-config-gui)
if [[ "$variant" == gnome ]]; then
    targets+=(install-gnome)
else
    targets+=(install-kwin)
fi

log "Installing from the repository"
make -C "$daemon_dir" --no-print-directory "${targets[@]}" FEATURES="$features"

dbus-send --session --type=method_call --dest=org.freedesktop.DBus \
    /org/freedesktop/DBus org.freedesktop.DBus.ReloadConfig >/dev/null 2>&1 || true

# install-daemon restarts a daemon that is still active; one whose unit
# disappeared with the packages may not be, so start it again here.
if (( daemon_active )) && ! systemctl --user is-active --quiet spotlight-dimmer-daemon.service 2>/dev/null; then
    log "Starting the daemon"
    systemctl --user start spotlight-dimmer-daemon.service || warn "daemon start failed"
fi

# --- Per-user setup ----------------------------------------------------------
if [[ "$variant" == kde ]]; then
    # The toggle launcher only; `make install-kde-shortcut` would also
    # overwrite a shortcut the user rebound.
    apps="$HOME/.local/share/applications"
    install -Dm644 "$daemon_dir/data/org.spotlightdimmer.toggle.desktop" "$apps/org.spotlightdimmer.toggle.desktop"
    command -v update-desktop-database >/dev/null && update-desktop-database "$apps" 2>/dev/null || true

    # Bind the shortcuts only when unset, so a user's own rebinding survives.
    if command -v kwriteconfig6 >/dev/null && command -v kreadconfig6 >/dev/null; then
        bind_shortcut() {
            local desktop="$1" keys="$2"
            [[ -e "$apps/$desktop" ]] || return 0
            local current
            current="$(kreadconfig6 --file kglobalshortcutsrc --group services --group "$desktop" --key _launch)"
            if [[ -z "$current" ]]; then
                kwriteconfig6 --file kglobalshortcutsrc --group services --group "$desktop" --key _launch "$keys"
                log "Bound $keys to $desktop (log out/in if it does not fire)"
            fi
        }
        bind_shortcut org.spotlightdimmer.toggle.desktop "Meta+Shift+D"
        bind_shortcut org.spotlightdimmer.ConfigToggle.desktop "Meta+Alt+Shift+D"
    fi
else
    if command -v gsettings >/dev/null; then
        enabled="$(gsettings get org.gnome.shell enabled-extensions 2>/dev/null || true)"
        if [[ -n "$enabled" && "$enabled" != *"$EXT_UUID"* ]]; then
            new_list="$(python3 -c 'import ast,sys; l=ast.literal_eval(sys.argv[1].removeprefix("@as ")); l.append(sys.argv[2]); print(repr(l))' "$enabled" "$EXT_UUID")"
            gsettings set org.gnome.shell enabled-extensions "$new_list"
            log "Enabled the GNOME Shell extension"
        fi
    fi
fi

log "SpotlightDimmer $version installed from the repository ($revision)"
if [[ "$variant" == gnome ]]; then
    echo "   Log out and back in so GNOME Shell loads the extension from this checkout."
fi
echo "   To go back to a release, run tools/install-release.sh (it removes this source install)."
