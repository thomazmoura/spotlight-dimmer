# SpotlightDimmer Linux daemon - install targets
#
# Quick start:
#   GNOME:  make install-linux-gnome   (then log out/in and enable the extension)
#   KDE:    make install-linux-kde
#
# Build dependencies (Ubuntu/Debian): rustup + libgtk-4-dev libgtk4-layer-shell-dev
# GNOME-only machines can skip the GTK deps with FEATURES=--no-default-features
# (disables the layer-shell renderer, which GNOME doesn't use anyway).

PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin
FEATURES ?=

EXT_UUID := spotlightdimmer@thomazmoura.github.io
EXT_SRC := ../SpotlightDimmer.GnomeShellExtension
EXT_DIR := $(HOME)/.local/share/gnome-shell/extensions/$(EXT_UUID)
KWIN_SRC := ../SpotlightDimmer.KwinScript
TOOLS_DIR := $(HOME)/.config/SpotlightDimmer/tools

.PHONY: build test install-daemon install-config install-gnome install-kwin \
        install-kde-shortcut install-tools install-linux-gnome install-linux-kde

build:
	cargo build --release $(FEATURES)

test:
	cargo test -p spotlight-dimmer-core

install-daemon: build
	install -Dm755 target/release/spotlight-dimmer-daemon $(BINDIR)/spotlight-dimmer-daemon
	install -Dm644 data/spotlight-dimmer-daemon.service $(HOME)/.config/systemd/user/spotlight-dimmer-daemon.service
	mkdir -p $(HOME)/.local/share/dbus-1/services
	sed "s|@BINDIR@|$(BINDIR)|" data/org.spotlightdimmer.Daemon.service.in > $(HOME)/.local/share/dbus-1/services/org.spotlightdimmer.Daemon.service
	systemctl --user daemon-reload
	-systemctl --user try-restart spotlight-dimmer-daemon.service 2>/dev/null
	@echo "Daemon installed; it will start on demand via D-Bus activation."

# Seed a starter config only when none exists. Without one the daemon
# defaults to FullScreen mode, which dims nothing on a single monitor —
# indistinguishable from a broken install.
install-config:
	@if [ -f $(HOME)/.config/SpotlightDimmer/config.json ]; then \
	  echo "Keeping existing ~/.config/SpotlightDimmer/config.json"; \
	else \
	  install -Dm644 ../config.example.json $(HOME)/.config/SpotlightDimmer/config.json; \
	  echo "Starter config installed to ~/.config/SpotlightDimmer/config.json (PartialWithActive mode)"; \
	fi

install-gnome:
	mkdir -p $(EXT_DIR)
	cp $(EXT_SRC)/*.js $(EXT_SRC)/*.json $(EXT_SRC)/*.css $(EXT_DIR)/
	cp -r $(EXT_SRC)/schemas $(EXT_DIR)/
	glib-compile-schemas $(EXT_DIR)/schemas/
	@echo "Extension installed. On Wayland, log out/in to reload GNOME Shell, then:"
	@echo "  gnome-extensions enable $(EXT_UUID)"

# unloadScript (synchronous) evicts a running old script instance so the
# reconfigure loads the freshly installed version — a plain reconfigure keeps
# an already-running script's old code, and toggling the enabled flag with
# two back-to-back reconfigures races (reconfigure is async). The fresh
# script re-registers with the daemon and re-sends monitors.
install-kwin:
	kpackagetool6 --type KWin/Script --upgrade $(KWIN_SRC) 2>/dev/null || \
	  kpackagetool6 --type KWin/Script --install $(KWIN_SRC)
	-qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript spotlightdimmer 2>/dev/null || \
	  dbus-send --session --print-reply --dest=org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript string:spotlightdimmer
	kwriteconfig6 --file kwinrc --group Plugins --key spotlightdimmerEnabled true
	-qdbus6 org.kde.KWin /KWin reconfigure 2>/dev/null || \
	  dbus-send --session --dest=org.kde.KWin /KWin org.kde.KWin.reconfigure
	@echo "KWin script installed and enabled."

install-kde-shortcut:
	install -Dm644 data/org.spotlightdimmer.toggle.desktop $(HOME)/.local/share/applications/org.spotlightdimmer.toggle.desktop
	kwriteconfig6 --file kglobalshortcutsrc --group services --group org.spotlightdimmer.toggle.desktop --key _launch "Meta+Shift+D"
	@echo "Meta+Shift+D bound to SpotlightDimmer toggle (log out/in if it doesn't fire; rebind in System Settings > Shortcuts)."

install-tools:
	install -Dm755 tools/spotlight-dimmer-tmux-report.sh $(TOOLS_DIR)/spotlight-dimmer-tmux-report.sh
	install -Dm644 tools/spotlight-dimmer.tmux.conf $(TOOLS_DIR)/spotlight-dimmer.tmux.conf
	@echo "tmux tools installed; add to ~/.tmux.conf:"
	@echo "  source-file ~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf"

install-linux-gnome: install-daemon install-config install-gnome install-tools

install-linux-kde: install-daemon install-config install-kwin install-kde-shortcut install-tools
