# SpotlightDimmer Linux daemon - install targets
#
# Quick start:
#   GNOME:  make install-linux-gnome   (then log out/in and enable the extension)
#   KDE:    make install-linux-kde
#
# Build dependencies (Ubuntu/Debian): rustup + libgtk-4-dev libgtk4-layer-shell-dev
# FEATURES applies to the daemon only: FEATURES=--no-default-features drops its
# layer-shell renderer (which GNOME doesn't use anyway) and with it the GTK
# build deps. The settings window is GTK4 on every desktop and needs
# libgtk-4-dev; install-linux-gnome/-kde skip it with a warning when the GTK4
# headers are missing, while `make install-config-gui` always builds it.

PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin
FEATURES ?=

EXT_UUID := spotlightdimmer@thomazmoura.github.io
EXT_SRC := ../SpotlightDimmer.GnomeShellExtension
EXT_DIR := $(HOME)/.local/share/gnome-shell/extensions/$(EXT_UUID)
KWIN_SRC := ../SpotlightDimmer.KwinScript
TOOLS_DIR := $(HOME)/.config/SpotlightDimmer/tools

.PHONY: build build-daemon build-config-gui test install-daemon install-config \
        install-config-gui install-config-gui-if-available install-gnome install-kwin \
        install-kde-shortcut install-tools test-tools install-linux-gnome install-linux-kde

# Each crate is built on its own: a workspace-wide build would always compile
# the GTK4 settings window, whatever FEATURES says about the daemon.
build: build-daemon build-config-gui

build-daemon:
	cargo build --release -p spotlight-dimmer-daemon $(FEATURES)

build-config-gui:
	cargo build --release -p spotlight-dimmer-config

test:
	cargo test -p spotlight-dimmer-core
	cargo test -p spotlight-dimmer-config

install-daemon: build-daemon
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

# The GTK4 settings window. Independent of the desktop: the same binary and
# the same config file serve GNOME and KDE.
install-config-gui: build-config-gui
	install -Dm755 target/release/spotlight-dimmer-config $(BINDIR)/spotlight-dimmer-config
	install -Dm644 data/org.spotlightdimmer.Config.desktop $(PREFIX)/share/applications/org.spotlightdimmer.Config.desktop
	install -Dm644 ../SpotlightDimmer.Assets/icon.svg $(PREFIX)/share/icons/hicolor/scalable/apps/org.spotlightdimmer.Config.svg
	-update-desktop-database $(PREFIX)/share/applications 2>/dev/null
	@echo "Settings window installed; launch it with spotlight-dimmer-config or from the application menu."

# The one-shot targets must not fail on a GNOME host built GTK-free, so the
# settings window is only added when its GTK4 headers are present.
install-config-gui-if-available:
	@if pkg-config --exists 'gtk4 >= 4.12'; then \
	  $(MAKE) install-config-gui; \
	else \
	  echo "WARNING: skipping the settings window (spotlight-dimmer-config): GTK4 >= 4.12 headers not found."; \
	  echo "         Install libgtk-4-dev, then run: make install-config-gui"; \
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
	install -Dm755 tools/spotlight-dimmer-tmux-popup.sh $(TOOLS_DIR)/spotlight-dimmer-tmux-popup.sh
	install -Dm644 tools/spotlight-dimmer.tmux.conf $(TOOLS_DIR)/spotlight-dimmer.tmux.conf
	install -Dm644 ../SpotlightDimmer.NeovimPlugin/lua/spotlight-dimmer/init.lua $(TOOLS_DIR)/nvim/lua/spotlight-dimmer/init.lua
	@echo "tmux tools installed; add to ~/.tmux.conf:"
	@echo "  source-file ~/.config/SpotlightDimmer/tools/spotlight-dimmer.tmux.conf"
	@echo "Optional neovim split spotlight: see docs/TMUX_INTEGRATION.md (Neovim splits)"

# Shell/Lua tests for the tmux report script and the neovim plugin (need
# bash and nvim, not a tmux server or D-Bus session).
test-tools:
	bash tools/tests/test-tmux-report.sh
	nvim --headless -u NONE -l ../SpotlightDimmer.NeovimPlugin/tests/rect_spec.lua

install-linux-gnome: install-daemon install-config install-config-gui-if-available install-gnome install-tools

install-linux-kde: install-daemon install-config install-config-gui-if-available install-kwin install-kde-shortcut install-tools
