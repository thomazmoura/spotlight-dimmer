/**
 * SpotlightDimmer - GNOME Shell Extension
 *
 * Main extension entry point that orchestrates all components.
 * Creates semi-transparent overlays to dim inactive displays/regions
 * around the focused window.
 */

import GLib from 'gi://GLib';
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

        // Initial overlay update
        this._updateAllOverlays();

        console.log('SpotlightDimmer: Extension enabled');
    }

    /**
     * Called when the extension is disabled.
     */
    disable() {
        console.log('SpotlightDimmer: Disabling extension');

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
            const geometry = global.display.get_monitor_geometry(i);
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
     * Update all overlays based on current focus and configuration.
     * @private
     */
    _updateAllOverlays() {
        const config = this._configBridge.getConfig();
        const focus = this._focusTracker.getCurrentFocus();
        const nMonitors = global.display.get_n_monitors();

        const focusedMonitor = focus ? focus.monitor : -1;
        const windowRect = focus ? focus.rect : null;

        for (let i = 0; i < nMonitors; i++) {
            const monitorGeometry = global.display.get_monitor_geometry(i);
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
