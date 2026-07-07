# Convenience wrapper so Linux install targets work from the repo root.
# The real Makefile lives in SpotlightDimmer.LinuxDaemon/ — see it for
# documentation of each target and the FEATURES variable.

LINUX_DAEMON_DIR := SpotlightDimmer.LinuxDaemon

LINUX_TARGETS := build test install-daemon install-config install-gnome \
                 install-kwin install-kde-shortcut install-tools \
                 install-linux-gnome install-linux-kde

.PHONY: $(LINUX_TARGETS)
$(LINUX_TARGETS):
	$(MAKE) -C $(LINUX_DAEMON_DIR) $@
