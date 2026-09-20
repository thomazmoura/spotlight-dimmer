/**
 * SpotlightDimmer - GNOME Shell Extension
 *
 * Thin compositor adapter for the spotlight-dimmer-daemon. The daemon owns
 * configuration, overlay calculation and the wezterm/tmux integration; this
 * extension:
 * - reports monitors, focus, geometry and title changes over D-Bus
 * - renders the overlay definitions the daemon publishes back (GNOME has no
 *   layer-shell, so overlays must be St.Widgets inside the Shell)
 *
 * Without the daemon installed and activatable, no dimming occurs.
 */

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

import { OverlayManager } from './overlayManager.js';
import { FocusTracker } from './focusTracker.js';
import { DaemonBridge } from './daemonBridge.js';

export default class SpotlightDimmerExtension extends Extension {
    /**
     * Called when the extension is enabled.
     */
    enable() {
        console.log('SpotlightDimmer: Enabling extension');

        // Names actually handed to Main.wm, so disable() removes exactly
        // those: a keybinding may be skipped when the schema lacks its key.
        this._keybindings = [];

        // Nothing below may escape: an exception halfway through enable()
        // would leave overlay widgets, signal handlers and the D-Bus name
        // watch alive with no way to reach them again. GNOME catches what we
        // rethrow and marks the extension as errored, which is recoverable.
        try {
            this._enable();
        } catch (e) {
            console.error(`SpotlightDimmer: enable() failed, rolling back: ${e}`);
            try {
                this.disable();
            } catch (cleanupError) {
                console.error(`SpotlightDimmer: rollback failed: ${cleanupError}`);
            }
            throw e;
        }

        console.log('SpotlightDimmer: Extension enabled');
    }

    /**
     * Body of enable(), wrapped by it so any failure is rolled back.
     * @private
     */
    _enable() {
        // Disable compositor unredirect to prevent fullscreen flickering.
        // This keeps overlays visible when fullscreen apps are running.
        this._setUnredirect(false);

        this._overlayManager = new OverlayManager();
        this._focusTracker = new FocusTracker();
        this._daemonBridge = new DaemonBridge();

        // Signal IDs for cleanup
        this._focusChangedId = null;
        this._geometryChangedId = null;
        this._titleChangedId = null;
        this._floatingChangedId = null;
        this._monitorsChangedId = null;
        this._fullscreenChangedId = null;

        // Create overlay widgets for all monitors
        this._createOverlaysForAllMonitors();

        // Daemon events
        this._daemonBridge.onOverlays = payload => this._applyPayload(payload);
        this._daemonBridge.onDaemonAppeared = () => this._syncWithDaemon();
        this._daemonBridge.onDaemonVanished = () => this._overlayManager.hideAll();

        // Compositor events -> daemon
        this._focusChangedId = this._focusTracker.connect(
            'focus-changed',
            (tracker, window) => this._sendFocus(window)
        );

        this._geometryChangedId = this._focusTracker.connect(
            'window-geometry-changed',
            (tracker, window) => {
                const rect = this._frameRect(window);
                if (rect) {
                    this._daemonBridge.geometryChanged(rect, this._clientRect(window));
                }
            }
        );

        this._titleChangedId = this._focusTracker.connect(
            'window-title-changed',
            (tracker, window) => this._daemonBridge.titleChanged(window.title ?? '')
        );

        this._floatingChangedId = this._focusTracker.connect(
            'floating-changed',
            (tracker, rectsJson) => this._daemonBridge.floatingChanged(rectsJson)
        );

        // Monitor hot-plug: recreate overlay widgets and re-report monitors
        this._monitorsChangedId = Main.layoutManager.connect(
            'monitors-changed',
            this._onMonitorsChanged.bind(this)
        );

        // When ANY window enters/exits fullscreen, refresh geometry so the
        // daemon recalculates (work areas may change with struts)
        this._fullscreenChangedId = global.display.connect(
            'in-fullscreen-changed',
            this._onFullscreenChanged.bind(this)
        );

        // Start talking to the daemon (AUTO_START activates it if installed);
        // onDaemonAppeared then performs registration and initial sync
        this._daemonBridge.init();

        // Keyboard shortcuts. Both are optional: a stale compiled schema
        // must degrade to "shortcut unavailable", never to a dead session.
        this._settings = this._tryGetSettings();

        // Super+Shift+D toggles dimming.
        this._addKeybinding('toggle-dimming', () => this._daemonBridge.toggle());

        // Super+Alt+Shift+D opens the settings window, or closes it when
        // focused. Launched through its .desktop file so the Shell hands the
        // new process an activation token and the window may take focus.
        this._addKeybinding('toggle-config-window', () => this._toggleConfigWindow());
    }

    /**
     * Turn compositor unredirection on or off.
     *
     * A fullscreen window is normally handed straight to the display,
     * bypassing compositing — which also bypasses our overlays, so the
     * dimming flickers or disappears. Switching unredirection off keeps
     * everything composited.
     *
     * The call moved in GNOME 47: `Meta.{disable,enable}_unredirect_for_display()`
     * became `global.compositor.{disable,enable}_unredirect()`. metadata.json
     * declares support for 45 through 48, so both have to work. This
     * feature-detects rather than parsing a shell version, which keeps
     * working if the API moves again.
     *
     * @param {boolean} enabled - true restores unredirection, false suppresses it
     * @private
     */
    _setUnredirect(enabled) {
        try {
            const compositor = global.compositor;
            if (typeof compositor?.disable_unredirect === 'function') {
                // GNOME 47+
                if (enabled) {
                    compositor.enable_unredirect();
                } else {
                    compositor.disable_unredirect();
                }
            } else if (enabled) {
                // GNOME 45/46
                Meta.enable_unredirect_for_display(global.display);
            } else {
                Meta.disable_unredirect_for_display(global.display);
            }

            console.log(`SpotlightDimmer: ${enabled ? 'Re-enabled' : 'Disabled'} compositor unredirect`);
        } catch (e) {
            // Non-fatal: only the fullscreen-flicker workaround is lost.
            console.warn(`SpotlightDimmer: Could not ${enabled ? 'enable' : 'disable'} unredirect: ${e.message}`);
        }
    }

    /**
     * Load the extension's GSettings, or null when the schema is missing or
     * unreadable (half-installed extension, schemas never compiled).
     * @returns {Gio.Settings|null}
     * @private
     */
    _tryGetSettings() {
        try {
            return this.getSettings();
        } catch (e) {
            console.error(`SpotlightDimmer: settings schema unavailable, keyboard shortcuts disabled (run 'make install-gnome' to reinstall): ${e.message}`);
            return null;
        }
    }

    /**
     * Register a keybinding only when the compiled schema really defines it.
     *
     * CRITICAL: Main.wm.addKeybinding() ends in g_settings_get_value(), and
     * GLib answers an unknown key with g_error() — an unconditional abort()
     * inside the library. That is not a JS exception: GJS never sees it, so
     * it cannot be caught, and on Wayland gnome-shell is the session leader,
     * so the abort takes every running application down with it. This happens
     * whenever the installed *.js is newer than the installed compiled
     * gschemas (e.g. JS copied by hand, schemas left stale). The has_key()
     * check below is the only thing standing between that mismatch and a
     * forced logout, so it must stay in front of every addKeybinding call.
     *
     * @param {string} name - Key name in the extension's schema
     * @param {Function} handler - Invoked when the shortcut fires
     * @private
     */
    _addKeybinding(name, handler) {
        if (!this._settings?.settings_schema?.has_key(name)) {
            console.warn(`SpotlightDimmer: schema has no key '${name}', shortcut disabled (reinstall the extension with 'make install-gnome' to recompile its schemas)`);
            return;
        }

        try {
            Main.wm.addKeybinding(
                name,
                this._settings,
                Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
                Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
                handler
            );
            this._keybindings.push(name);
        } catch (e) {
            console.error(`SpotlightDimmer: could not register shortcut '${name}': ${e.message}`);
        }
    }

    /**
     * Run `spotlight-dimmer-config --toggle` via its launcher entry.
     */
    _toggleConfigWindow() {
        const appInfo = Gio.DesktopAppInfo.new('org.spotlightdimmer.ConfigToggle.desktop');
        if (!appInfo) {
            console.warn('SpotlightDimmer: settings window not installed (spotlight-dimmer-config)');
            return;
        }
        try {
            appInfo.launch([], global.create_app_launch_context(0, -1));
        } catch (e) {
            console.warn(`SpotlightDimmer: could not open settings window: ${e.message}`);
        }
    }

    /**
     * Called when the extension is disabled.
     */
    disable() {
        console.log('SpotlightDimmer: Disabling extension');

        for (const name of this._keybindings ?? []) {
            try {
                Main.wm.removeKeybinding(name);
            } catch (e) {
                console.warn(`SpotlightDimmer: could not remove shortcut '${name}': ${e.message}`);
            }
        }
        this._keybindings = [];
        this._settings = null;

        // Re-enable compositor unredirect to restore default behavior
        this._setUnredirect(true);

        if (this._focusChangedId) {
            this._focusTracker.disconnect(this._focusChangedId);
            this._focusChangedId = null;
        }

        if (this._geometryChangedId) {
            this._focusTracker.disconnect(this._geometryChangedId);
            this._geometryChangedId = null;
        }

        if (this._floatingChangedId) {
            this._focusTracker.disconnect(this._floatingChangedId);
            this._floatingChangedId = null;
        }

        if (this._titleChangedId) {
            this._focusTracker.disconnect(this._titleChangedId);
            this._titleChangedId = null;
        }

        if (this._monitorsChangedId) {
            Main.layoutManager.disconnect(this._monitorsChangedId);
            this._monitorsChangedId = null;
        }

        if (this._fullscreenChangedId) {
            global.display.disconnect(this._fullscreenChangedId);
            this._fullscreenChangedId = null;
        }

        this._focusTracker?.destroy();
        this._overlayManager?.destroy();
        this._daemonBridge?.destroy();

        this._focusTracker = null;
        this._overlayManager = null;
        this._daemonBridge = null;

        console.log('SpotlightDimmer: Extension disabled');
    }

    /**
     * Register with the (re)appeared daemon and send the full current state:
     * monitors first, then focus, then render the returned snapshot.
     * @private
     */
    async _syncWithDaemon() {
        await this._daemonBridge.register();
        // The extension may have been disabled while that call was in
        // flight, which tears these down.
        if (!this._daemonBridge) {
            return;
        }
        this._sendMonitors();
        this._sendFocus(global.display.focus_window);
        // A restarted daemon comes back with no floating rects, so resend
        // unconditionally rather than relying on the change diff.
        this._focusTracker?.resendFloating();
    }

    /**
     * Render an overlays payload from the daemon. Monitor keys are Mutter
     * monitor indices as strings ("0", "1"). A paused daemon sends empty
     * overlay lists, which hides every slot.
     * @private
     */
    _applyPayload(payload) {
        if (!this._overlayManager || !Array.isArray(payload.monitors)) {
            return;
        }

        // Stacking policy rides along with every payload: the daemon owns
        // config, the extension only renders what it is handed.
        if (payload.chrome_handling) {
            this._overlayManager.setChromeHandling(payload.chrome_handling);
        }

        // Only enumerate always-on-top windows when the daemon will act on
        // them: 'restacked' fires on every raise, so this is not free.
        this._focusTracker.setTrackFloating(payload.track_floating === true);

        for (const monitor of payload.monitors) {
            const index = parseInt(monitor.key, 10);
            if (Number.isInteger(index) && this._overlayManager.hasMonitor(index)) {
                this._overlayManager.updateMonitor(index, monitor.overlays);
            }
        }
    }

    /**
     * Report all monitors (full geometry + work area + scale) to the daemon.
     * Keys are monitor indices as strings; all rects are Mutter logical
     * global coordinates.
     * @private
     */
    _sendMonitors() {
        const nMonitors = global.display.get_n_monitors();
        const monitors = [];

        for (let i = 0; i < nMonitors; i++) {
            const geometry = global.display.get_monitor_geometry(i);
            const workArea = this._getMonitorWorkArea(i);
            monitors.push({
                key: String(i),
                geometry: {
                    x: geometry.x,
                    y: geometry.y,
                    width: geometry.width,
                    height: geometry.height,
                },
                workArea,
                scale: global.display.get_monitor_scale(i),
            });
        }

        this._daemonBridge.updateMonitors(monitors);
    }

    /**
     * Report the focused window (or its absence) to the daemon. WM_CLASS and
     * title are included so the daemon can match app integrations (tmux).
     * @param {Meta.Window|null} window
     * @private
     */
    _sendFocus(window) {
        const rect = this._frameRect(window);
        if (!window || !rect) {
            this._daemonBridge.focusCleared();
            return;
        }

        let wmClass = '';
        try {
            wmClass = window.get_wm_class() ?? '';
        } catch (e) {
            // Window may have been destroyed
        }

        this._daemonBridge.focusChanged(
            wmClass, window.title ?? '', rect, this._clientRect(window));
    }

    /**
     * Frame rect of a window as a plain object, or null.
     * @private
     */
    _frameRect(window) {
        if (!window) {
            return null;
        }

        try {
            const rect = window.get_frame_rect();
            return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
        } catch (e) {
            console.warn(`SpotlightDimmer: Error getting frame rect: ${e.message}`);
            return null;
        }
    }

    /**
     * Client-area rect (decorations excluded) as a plain object, or null when
     * unavailable — the daemon then falls back to the frame rect. Equals the
     * frame rect for CSD windows; differs for X11 server-side decorations.
     * Deliberately not get_buffer_rect(), which includes CSD shadows.
     * @private
     */
    _clientRect(window) {
        if (!window) {
            return null;
        }

        try {
            const rect = window.frame_rect_to_client_rect(window.get_frame_rect());
            if (!rect || rect.width <= 0 || rect.height <= 0) {
                return null;
            }
            return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
        } catch (e) {
            return null;
        }
    }

    /**
     * Create overlay widgets for all connected monitors.
     * @private
     */
    _createOverlaysForAllMonitors() {
        const nMonitors = global.display.get_n_monitors();

        for (let i = 0; i < nMonitors; i++) {
            const geometry = this._getMonitorWorkArea(i);
            this._overlayManager.createOverlaysForMonitor(i, {
                x: geometry.x,
                y: geometry.y,
                width: geometry.width,
                height: geometry.height,
            });
        }

        console.log(`SpotlightDimmer: Created overlays for ${nMonitors} monitor(s)`);
    }

    /**
     * Get work area for a monitor, excluding dock and panel struts.
     * Falls back to full monitor geometry if work area unavailable.
     * @param {number} monitorIndex - Monitor index
     * @returns {Object} {x, y, width, height}
     * @private
     */
    _getMonitorWorkArea(monitorIndex) {
        try {
            const workspace = global.workspace_manager.get_active_workspace();
            const workArea = workspace.get_work_area_for_monitor(monitorIndex);

            return {
                x: workArea.x,
                y: workArea.y,
                width: workArea.width,
                height: workArea.height,
            };
        } catch (e) {
            console.warn(`SpotlightDimmer: Could not get work area for monitor ${monitorIndex}, using full geometry: ${e.message}`);
            const geometry = global.display.get_monitor_geometry(monitorIndex);
            return {
                x: geometry.x,
                y: geometry.y,
                width: geometry.width,
                height: geometry.height,
            };
        }
    }

    /**
     * Handle fullscreen state changes: defer (animation), then re-report
     * monitors (struts may change) and focus geometry.
     * @private
     */
    _onFullscreenChanged() {
        GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            this._sendMonitors();
            this._sendFocus(global.display.focus_window);
            return GLib.SOURCE_REMOVE;
        });
    }

    /**
     * Handle monitor configuration changes (hot-plug).
     * @private
     */
    _onMonitorsChanged() {
        console.log('SpotlightDimmer: Monitors changed, recreating overlays');

        const chromeHandling = this._overlayManager.chromeHandling;
        this._overlayManager.destroy();
        this._overlayManager = new OverlayManager(chromeHandling);
        this._createOverlaysForAllMonitors();

        this._sendMonitors();
        this._sendFocus(global.display.focus_window);
    }
}
