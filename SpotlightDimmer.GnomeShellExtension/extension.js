/**
 * SpotlightDimmer - GNOME Shell Extension
 *
 * Main extension entry point that orchestrates all components.
 * Creates semi-transparent overlays to dim inactive displays/regions
 * around the focused window.
 */

import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

import { OverlayCalculator } from './calculator.js';
import { ConfigBridge } from './configBridge.js';
import { OverlayManager } from './overlayManager.js';
import { FocusTracker } from './focusTracker.js';

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

        // Initialize components
        this._calculator = new OverlayCalculator();
        this._configBridge = new ConfigBridge();
        this._overlayManager = new OverlayManager();
        this._focusTracker = new FocusTracker();

        // Signal IDs for cleanup
        this._focusChangedId = null;
        this._geometryChangedId = null;
        this._configChangedId = null;
        this._monitorsChangedId = null;
        this._fullscreenChangedId = null;
        this._overlaysPaused = false;

        // Create overlays for all monitors
        this._createOverlaysForAllMonitors();

        // Connect focus tracker signals
        this._focusChangedId = this._focusTracker.connect(
            'focus-changed',
            this._onFocusOrGeometryChanged.bind(this)
        );

        this._geometryChangedId = this._focusTracker.connect(
            'window-geometry-changed',
            this._onFocusOrGeometryChanged.bind(this)
        );

        // Connect config changes
        this._configChangedId = this._configBridge.connect(
            'config-changed',
            this._onConfigChanged.bind(this)
        );

        // Connect monitor changes (hot-plug support)
        this._monitorsChangedId = Main.layoutManager.connect(
            'monitors-changed',
            this._onMonitorsChanged.bind(this)
        );

        // Connect to fullscreen state changes (system-wide)
        // This ensures overlays update when ANY window enters/exits fullscreen
        this._fullscreenChangedId = global.display.connect(
            'in-fullscreen-changed',
            this._onFullscreenChanged.bind(this)
        );

        // Initial overlay update
        this._updateAllOverlays();

        // Register global keyboard shortcut (Super+Shift+D)
        this._settings = this.getSettings();
        Main.wm.addKeybinding(
            'toggle-dimming',
            this._settings,
            Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
            this._onToggleShortcut.bind(this)
        );

        console.log('SpotlightDimmer: Extension enabled');
    }

    /**
     * Called when the extension is disabled.
     */
    disable() {
        console.log('SpotlightDimmer: Disabling extension');

        // Remove global keyboard shortcut
        Main.wm.removeKeybinding('toggle-dimming');
        this._settings = null;

        // Re-enable compositor unredirect to restore default behavior
        try {
            global.compositor.enable_unredirect();
            console.log('SpotlightDimmer: Re-enabled compositor unredirect');
        } catch (e) {
            console.warn(`SpotlightDimmer: Could not enable unredirect: ${e.message}`);
        }

        // Disconnect focus tracker signals
        if (this._focusChangedId) {
            this._focusTracker.disconnect(this._focusChangedId);
            this._focusChangedId = null;
        }

        if (this._geometryChangedId) {
            this._focusTracker.disconnect(this._geometryChangedId);
            this._geometryChangedId = null;
        }

        // Disconnect config signals
        if (this._configChangedId) {
            this._configBridge.disconnect(this._configChangedId);
            this._configChangedId = null;
        }

        // Disconnect monitor signals
        if (this._monitorsChangedId) {
            Main.layoutManager.disconnect(this._monitorsChangedId);
            this._monitorsChangedId = null;
        }

        // Disconnect fullscreen signal
        if (this._fullscreenChangedId) {
            global.display.disconnect(this._fullscreenChangedId);
            this._fullscreenChangedId = null;
        }

        // Destroy components
        this._focusTracker?.destroy();
        this._overlayManager?.destroy();
        this._configBridge?.destroy();

        this._focusTracker = null;
        this._overlayManager = null;
        this._configBridge = null;
        this._calculator = null;

        console.log('SpotlightDimmer: Extension disabled');
    }

    /**
     * Create overlays for all connected monitors.
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
            // Get active workspace
            const workspace = global.workspace_manager.get_active_workspace();

            // Get work area (excludes dock/panel struts)
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
     * Handle focus or geometry change events.
     * @param {FocusTracker} tracker - The focus tracker
     * @param {Meta.Window|null} window - The focused window
     * @param {number} monitorIndex - The monitor index
     * @private
     */
    _onFocusOrGeometryChanged(tracker, window, monitorIndex) {
        this._updateAllOverlays();
    }

    /**
     * Handle configuration change events.
     * @private
     */
    _onConfigChanged() {
        console.log('SpotlightDimmer: Config changed, updating overlays');
        this._updateAllOverlays();
    }

    /**
     * Handle fullscreen state changes.
     * When any window enters/exits fullscreen, update overlays to ensure
     * other monitors continue to show dimming.
     * @private
     */
    _onFullscreenChanged() {
        // Defer update to allow fullscreen transition animation to complete
        GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            this._updateAllOverlays();
            return GLib.SOURCE_REMOVE;
        });
    }

    /**
     * Handle monitor configuration changes (hot-plug).
     * @private
     */
    _onMonitorsChanged() {
        console.log('SpotlightDimmer: Monitors changed, recreating overlays');

        // Destroy existing overlays
        this._overlayManager.destroy();
        this._overlayManager = new OverlayManager();

        // Recreate for new monitor configuration
        this._createOverlaysForAllMonitors();
        this._updateAllOverlays();
    }

    /**
     * Handle the toggle-dimming keyboard shortcut.
     * Pauses/resumes overlay rendering without disconnecting signals.
     * @private
     */
    _onToggleShortcut() {
        this._overlaysPaused = !this._overlaysPaused;

        if (this._overlaysPaused) {
            console.log('SpotlightDimmer: Overlays paused via shortcut');
            this._overlayManager.hideAll();
        } else {
            console.log('SpotlightDimmer: Overlays resumed via shortcut');
            this._updateAllOverlays();
        }
    }

    /**
     * Update all overlays based on current focus and configuration.
     * @private
     */
    _updateAllOverlays() {
        if (this._overlaysPaused) return;

        const config = this._configBridge.getConfig();
        const focus = this._focusTracker.getCurrentFocus();
        const nMonitors = global.display.get_n_monitors();

        const focusedMonitor = focus ? focus.monitor : -1;
        const windowRect = focus ? focus.rect : null;

        for (let i = 0; i < nMonitors; i++) {
            const monitorGeometry = this._getMonitorWorkArea(i);
            const isFocused = (i === focusedMonitor);

            // Calculate overlay definitions for this monitor
            const definitions = this._calculator.calculate(
                config,
                {
                    x: monitorGeometry.x,
                    y: monitorGeometry.y,
                    width: monitorGeometry.width,
                    height: monitorGeometry.height,
                },
                isFocused ? windowRect : null,
                isFocused
            );

            // If definitions is null, calculator signaled to keep existing state
            // This happens for 0x0 windows during transitions
            if (definitions !== null) {
                this._overlayManager.updateMonitor(i, definitions);
            }
        }
    }
}
