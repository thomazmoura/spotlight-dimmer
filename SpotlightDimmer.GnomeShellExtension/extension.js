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

        // Disable compositor unredirect to prevent fullscreen flickering
        // This keeps overlays visible when fullscreen apps are running
        try {
            global.compositor.disable_unredirect();
            console.log('SpotlightDimmer: Disabled compositor unredirect');
        } catch (e) {
            console.warn(`SpotlightDimmer: Could not disable unredirect: ${e.message}`);
        }

        this._overlayManager = new OverlayManager();
        this._focusTracker = new FocusTracker();
        this._daemonBridge = new DaemonBridge();

        // Signal IDs for cleanup
        this._focusChangedId = null;
        this._geometryChangedId = null;
        this._titleChangedId = null;
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
                    this._daemonBridge.geometryChanged(rect);
                }
            }
        );

        this._titleChangedId = this._focusTracker.connect(
            'window-title-changed',
            (tracker, window) => this._daemonBridge.titleChanged(window.title ?? '')
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

        // Register global keyboard shortcut (Super+Shift+D)
        this._settings = this.getSettings();
        Main.wm.addKeybinding(
            'toggle-dimming',
            this._settings,
            Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
            () => this._daemonBridge.toggle()
        );

        console.log('SpotlightDimmer: Extension enabled');
    }

    /**
     * Called when the extension is disabled.
     */
    disable() {
        console.log('SpotlightDimmer: Disabling extension');

        Main.wm.removeKeybinding('toggle-dimming');
        this._settings = null;

        // Re-enable compositor unredirect to restore default behavior
        try {
            global.compositor.enable_unredirect();
            console.log('SpotlightDimmer: Re-enabled compositor unredirect');
        } catch (e) {
            console.warn(`SpotlightDimmer: Could not enable unredirect: ${e.message}`);
        }

        if (this._focusChangedId) {
            this._focusTracker.disconnect(this._focusChangedId);
            this._focusChangedId = null;
        }

        if (this._geometryChangedId) {
            this._focusTracker.disconnect(this._geometryChangedId);
            this._geometryChangedId = null;
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
        this._sendMonitors();
        this._sendFocus(global.display.focus_window);
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

        this._daemonBridge.focusChanged(wmClass, window.title ?? '', rect);
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

        this._overlayManager.destroy();
        this._overlayManager = new OverlayManager();
        this._createOverlaysForAllMonitors();

        this._sendMonitors();
        this._sendFocus(global.display.focus_window);
    }
}
