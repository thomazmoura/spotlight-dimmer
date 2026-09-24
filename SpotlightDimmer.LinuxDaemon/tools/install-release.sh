#!/usr/bin/env bash
# Install or update SpotlightDimmer on Ubuntu (GNOME) or Kubuntu (KDE Plasma 6)
# from the .deb packages published on the GitHub releases page.
#
# Usage:
#   install-release.sh [--gnome|--kde] [--version X.Y.Z] [--no-config-gui]
#                      [--prerelease] [--force] [--clean-source-install] [--yes]
#
# Or straight from GitHub:
#   curl -fsSL https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/main/SpotlightDimmer.LinuxDaemon/tools/install-release.sh | bash
#
# Run it as your normal user: apt runs through sudo, while the per-user steps
# (config seeding, extension enabling, KDE shortcuts, daemon restart) need
# your session.
set -euo pipefail

REPO="thomazmoura/spotlight-dimmer"
EXT_UUID="spotlightdimmer@thomazmoura.github.io"

variant=""
version=""
with_config=1
allow_prerelease=0
force=0
clean_source=0
assume_yes=0

log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mWARNING:\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
    # Not read from $0: under `curl ... | bash` there is no script file.
    cat <<'EOF'
Usage: install-release.sh [options]

  --gnome / --kde    Skip desktop detection and install this variant
  --version X.Y.Z    Install a specific Linux release instead of the latest
  --no-config-gui    Do not install the spotlight-dimmer-config settings window
  --prerelease       Allow the latest release to be a pre-release
  --force            Reinstall even when the installed version matches
  --clean-source-install
                     Remove the per-user files of a `make install-linux-*`
                     source install (~/.local, ~/.config/systemd/user), which
                     otherwise shadow the packaged daemon, extension, KWin
                     script and settings window
  -y, --yes          Do not ask apt for confirmation
EOF
    exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --gnome) variant=gnome ;;
        --kde) variant=kde ;;
        --version) [[ $# -ge 2 ]] || die "--version needs a value"; version="${2#v}"; version="${version%-linux}"; shift ;;
        --no-config-gui) with_config=0 ;;
        --prerelease) allow_prerelease=1 ;;
        --force) force=1 ;;
        --clean-source-install) clean_source=1 ;;
        -y|--yes) assume_yes=1 ;;
        -h|--help) usage 0 ;;
        *) warn "unknown option: $1"; usage 1 ;;
    esac
    shift
done

[[ $EUID -ne 0 ]] || die "run this as your normal user, not root (sudo is used for apt only)"
for cmd in curl python3 dpkg apt; do
    command -v "$cmd" >/dev/null || die "'$cmd' is required"
done

# --- Detect the desktop ------------------------------------------------------
# /etc/os-release reports "Ubuntu" on Kubuntu too, so look at the desktop
# instead: the session variable first (absent over ssh), then the flavour
# metapackages, then whichever shell binary is installed.
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

arch="$(dpkg --print-architecture)"
[[ "$arch" == amd64 || "$arch" == arm64 ]] || die "unsupported architecture: $arch (releases ship amd64 and arm64)"

# --- Resolve the release -----------------------------------------------------
# Linux and Windows releases share one releases page; Linux tags end in -linux.
log "Looking up the Linux release on github.com/$REPO"
workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT
curl -fsSL -H 'Accept: application/vnd.github+json' -o "$workdir/releases.json" \
    "https://api.github.com/repos/$REPO/releases?per_page=50" \
    || die "could not reach the GitHub API"

# Prints: tag, then one "name<TAB>url" line per asset.
release_info="$(python3 - "$workdir/releases.json" "$version" "$allow_prerelease" <<'PY'
import json, sys
wanted, allow_pre = sys.argv[2], sys.argv[3] == "1"
for r in json.load(open(sys.argv[1])):
    tag = r["tag_name"]
    if not tag.endswith("-linux") or r.get("draft"):
        continue
    if wanted:
        if tag != f"v{wanted}-linux":
            continue
    elif r.get("prerelease") and not allow_pre:
        continue
    print(tag)
    for a in r["assets"]:
        print(f'{a["name"]}\t{a["browser_download_url"]}')
    break
PY
)"
[[ -n "$release_info" ]] || die "no matching Linux release found${version:+ for version $version}"

tag="$(head -n1 <<<"$release_info")"
rel_version="${tag#v}"; rel_version="${rel_version%-linux}"
asset_url() { awk -F'\t' -v n="$1" '$1 == n { print $2 }' <<<"$release_info"; }

packages=("spotlight-dimmer-$variant")
(( with_config )) && packages+=("spotlight-dimmer-config")

# --- Skip what is already current -------------------------------------------
installed_version() {
    dpkg-query -W -f='${db:Status-Status} ${Version}' "$1" 2>/dev/null \
        | awk '$1 == "installed" { print $2 }' || true
}

to_install=()
previously_installed=0
for pkg in "${packages[@]}"; do
    deb="${pkg}_${rel_version}-1_${arch}.deb"
    url="$(asset_url "$deb")"
    if [[ -z "$url" ]]; then
        [[ "$pkg" == spotlight-dimmer-config ]] || die "$tag has no $deb asset"
        warn "$tag does not ship the settings window ($deb); skipping it"
        continue
    fi
    current="$(installed_version "$pkg")"
    [[ -n "$current" ]] && previously_installed=1
    if [[ "$current" == "${rel_version}-1" && $force -eq 0 ]]; then
        log "$pkg is already at $rel_version"
        continue
    fi
    log "$pkg: ${current:-not installed} -> ${rel_version}-1"
    to_install+=("$deb"$'\t'"$url")
done

# --- Source install leftovers ------------------------------------------------
# `make install-linux-*` installs per-user copies (see the README's uninstall
# section). Each one takes precedence over its packaged counterpart in /usr,
# so an old build keeps running however current the packages are: the user
# unit shadows /usr/lib/systemd/user, ~/.local/bin comes first in PATH, and
# GNOME Shell and KWin prefer a user-local extension or script with the same
# id. The tmux tools in ~/.config/SpotlightDimmer/tools are left alone, as
# ~/.tmux.conf may still source them.
source_leftovers() {
    local paths=(
        "$HOME/.local/bin/spotlight-dimmer-daemon"
        "$HOME/.config/systemd/user/spotlight-dimmer-daemon.service"
        "$HOME/.local/share/dbus-1/services/org.spotlightdimmer.Daemon.service"
        "$HOME/.local/share/applications/org.spotlightdimmer.toggle.desktop"
        "$HOME/.local/share/gnome-shell/extensions/$EXT_UUID"
        "$HOME/.local/share/kwin/scripts/spotlightdimmer"
    )
    # Without the packaged settings window the source-built one is the only
    # copy, so keep it.
    (( with_config )) && paths+=(
        "$HOME/.local/bin/spotlight-dimmer-config"
        "$HOME/.local/share/applications/org.spotlightdimmer.Config.desktop"
        "$HOME/.local/share/applications/org.spotlightdimmer.ConfigToggle.desktop"
        "$HOME/.local/share/icons/hicolor/scalable/apps/org.spotlightdimmer.Config.svg"
    )
    local p
    for p in "${paths[@]}"; do
        [[ -e "$p" ]] && printf '%s\n' "$p"
    done
    return 0
}

mapfile -t leftovers < <(source_leftovers)
if [[ ${#leftovers[@]} -gt 0 && $clean_source -eq 0 ]]; then
    warn "a source install's per-user files shadow the packaged ones, so the old build keeps running:"
    printf '   %s\n' "${leftovers[@]}" >&2
    warn "rerun with --clean-source-install to remove them"
fi
(( clean_source )) || leftovers=()

if [[ ${#to_install[@]} -eq 0 && ${#leftovers[@]} -eq 0 ]]; then
    log "Everything is up to date (use --force to reinstall)"
    exit 0
fi

other="spotlight-dimmer-$([[ $variant == kde ]] && echo gnome || echo kde)"
if [[ ${#to_install[@]} -gt 0 && -n "$(installed_version "$other")" ]]; then
    warn "$other is installed and will be replaced by spotlight-dimmer-$variant (they conflict)"
fi

# --- Download and install ----------------------------------------------------
if [[ ${#to_install[@]} -gt 0 ]]; then
    debs=()
    for entry in "${to_install[@]}"; do
        deb="${entry%%$'\t'*}"; url="${entry#*$'\t'}"
        log "Downloading $deb"
        curl -fL --progress-bar -o "$workdir/$deb" "$url"
        debs+=("$workdir/$deb")
    done

    # apt (not dpkg -i) resolves the runtime dependencies. The _apt sandbox user
    # must be able to read the files, which mktemp's 0700 directory prevents.
    chmod 755 "$workdir"; chmod 644 "${debs[@]}"
    apt_flags=()
    (( assume_yes )) && apt_flags+=(-y)
    log "Installing with apt (sudo)"
    sudo apt install "${apt_flags[@]}" "${debs[@]}"
fi

# Sampled before the cleanup below, which swaps the unit under a running daemon.
daemon_active=0
systemctl --user is-active --quiet spotlight-dimmer-daemon.service 2>/dev/null && daemon_active=1

# --- Remove the source install -----------------------------------------------
# Runs after apt so the packaged copies are already in place when the
# user-local ones disappear.
if [[ ${#leftovers[@]} -gt 0 ]]; then
    log "Removing the source install's per-user files"
    for path in "${leftovers[@]}"; do
        if [[ "$path" == */kwin/scripts/* ]] && command -v kpackagetool6 >/dev/null; then
            kpackagetool6 --type KWin/Script --remove spotlightdimmer >/dev/null 2>&1 || true
        fi
        rm -rf "$path"
        echo "   removed $path"
    done
    # Pick up the packaged unit and D-Bus activation file in place of the
    # removed ones; the daemon restart below then runs the packaged binary.
    systemctl --user daemon-reload 2>/dev/null || true
    dbus-send --session --type=method_call --dest=org.freedesktop.DBus \
        /org/freedesktop/DBus org.freedesktop.DBus.ReloadConfig >/dev/null 2>&1 || true
    command -v update-desktop-database >/dev/null \
        && update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi

# --- Per-user setup ----------------------------------------------------------
config_dir="$HOME/.config/SpotlightDimmer"
if [[ ! -e "$config_dir/config.json" ]]; then
    example="/usr/share/doc/spotlight-dimmer-$variant/examples/config.example.json"
    if [[ -r "$example" ]]; then
        mkdir -p "$config_dir"
        cp "$example" "$config_dir/config.json"
        log "Seeded $config_dir/config.json from the example configuration"
    fi
fi

# The daemon is D-Bus activated; restart a running one so it uses the new
# binary. If it is not running, the adapter starts it on demand.
if (( daemon_active )); then
    log "Restarting the daemon"
    systemctl --user restart spotlight-dimmer-daemon.service || warn "daemon restart failed"
fi

if [[ "$variant" == kde ]]; then
    # KWin keeps an already-running script's old code until it is unloaded.
    if command -v qdbus6 >/dev/null || command -v dbus-send >/dev/null; then
        qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript spotlightdimmer >/dev/null 2>&1 \
            || dbus-send --session --print-reply --dest=org.kde.KWin /Scripting \
                org.kde.kwin.Scripting.unloadScript string:spotlightdimmer >/dev/null 2>&1 || true
        qdbus6 org.kde.KWin /KWin reconfigure >/dev/null 2>&1 \
            || dbus-send --session --dest=org.kde.KWin /KWin org.kde.KWin.reconfigure >/dev/null 2>&1 || true
    fi

    # Bind the shortcuts only when unset, so a user's own rebinding survives updates.
    if command -v kwriteconfig6 >/dev/null && command -v kreadconfig6 >/dev/null; then
        bind_shortcut() {
            local desktop="$1" keys="$2"
            [[ -e "/usr/share/applications/$desktop" ]] || return 0
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
    # Enabling works even before GNOME Shell has seen the extension: it is
    # just a GSettings list, applied at the next login.
    if command -v gsettings >/dev/null; then
        enabled="$(gsettings get org.gnome.shell enabled-extensions 2>/dev/null || true)"
        if [[ -n "$enabled" && "$enabled" != *"$EXT_UUID"* ]]; then
            new_list="$(python3 -c 'import ast,sys; l=ast.literal_eval(sys.argv[1].removeprefix("@as ")); l.append(sys.argv[2]); print(repr(l))' "$enabled" "$EXT_UUID")"
            gsettings set org.gnome.shell enabled-extensions "$new_list"
            log "Enabled the GNOME Shell extension"
        fi
    fi
fi

log "SpotlightDimmer $rel_version installed"
if [[ "$variant" == gnome ]]; then
    echo "   Log out and back in so GNOME Shell loads the $([[ $previously_installed -eq 1 ]] && echo updated || echo new) extension."
fi
