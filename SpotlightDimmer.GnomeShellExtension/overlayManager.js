/**
 * SpotlightDimmer - Overlay Manager
 *
 * Manages St.Widget overlays for each monitor.
 * Pre-allocates 6 overlays per monitor (matching the C# DisplayOverlayState pattern).
 */

import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

// Number of overlay slots per monitor (matches C# OverlayRegion enum)
// FullScreen=0, Top=1, Bottom=2, Left=3, Right=4, Center=5
const OVERLAYS_PER_MONITOR = 6;

/**
 * OverlayManager creates and manages St.Widget overlays for dimming.
 */
export class OverlayManager {
    constructor() {
        // Map<monitorIndex, St.Widget[]>
        this._monitors = new Map();
    }

    /**
     * Create pre-allocated overlays for a monitor.
     * @param {number} monitorIndex - Monitor index
     * @param {Object} geometry - Monitor geometry {x, y, width, height}
     */
    createOverlaysForMonitor(monitorIndex, geometry) {
        if (this._monitors.has(monitorIndex)) {
            console.warn(`SpotlightDimmer: Overlays already exist for monitor ${monitorIndex}`);
            return;
        }

        const overlays = [];

        for (let i = 0; i < OVERLAYS_PER_MONITOR; i++) {
            const overlay = new St.Widget({
                style_class: 'spotlight-dimmer-overlay',
                reactive: false, // CRITICAL: Click-through - users can interact with windows below
                can_focus: false,
                track_hover: false,
                visible: false,
                x: geometry.x,
                y: geometry.y,
                width: 0,
                height: 0,
            });

            // Add to top chrome with fullscreen tracking disabled
            // This ensures overlays remain visible on OTHER monitors when a fullscreen
            // window is present on ONE monitor, enabling proper multi-monitor dimming
            Main.layoutManager.addTopChrome(overlay, {
                trackFullscreen: false,
                affectsInputRegion: false,  // Allow click-through for dock/panel interactions
            });
            overlays.push(overlay);
        }

        this._monitors.set(monitorIndex, overlays);
        console.log(`SpotlightDimmer: Created ${OVERLAYS_PER_MONITOR} overlays for monitor ${monitorIndex}`);
    }

    /**
     * Update overlays for a monitor based on calculated definitions.
     * @param {number} monitorIndex - Monitor index
     * @param {Array} definitions - Array of overlay definitions from calculator
     */
    updateMonitor(monitorIndex, definitions) {
        const overlays = this._monitors.get(monitorIndex);
        if (!overlays) {
            console.warn(`SpotlightDimmer: No overlays found for monitor ${monitorIndex}`);
            return;
        }

        // Build a map of region -> definition for quick lookup
        const defByRegion = new Map();
        if (definitions) {
            for (const def of definitions) {
                defByRegion.set(def.region, def);
            }
        }

        // Update each overlay slot
        for (let i = 0; i < OVERLAYS_PER_MONITOR; i++) {
            const overlay = overlays[i];
            const def = defByRegion.get(i);

            if (def && def.visible) {
                // Apply position and size
                overlay.x = def.x;
                overlay.y = def.y;
                overlay.width = def.width;
                overlay.height = def.height;

                // Apply color and opacity via inline style
                // Convert opacity from 0-255 to 0-1 for CSS rgba
                const r = def.color.r;
                const g = def.color.g;
                const b = def.color.b;
                const a = (def.opacity / 255).toFixed(3);
                overlay.style = `background-color: rgba(${r}, ${g}, ${b}, ${a});`;

                // Show the overlay
                overlay.visible = true;
            } else {
                // Hide unused overlay
                overlay.visible = false;
            }
        }
    }

    /**
     * Hide all overlays on all monitors.
     */
    hideAll() {
        for (const overlays of this._monitors.values()) {
            for (const overlay of overlays) {
                overlay.visible = false;
            }
        }
    }

    /**
     * Remove overlays for a specific monitor.
     * @param {number} monitorIndex - Monitor index
     */
    removeMonitor(monitorIndex) {
        const overlays = this._monitors.get(monitorIndex);
        if (!overlays) {
            return;
        }

        for (const overlay of overlays) {
            Main.layoutManager.removeChrome(overlay);
            overlay.destroy();
        }

        this._monitors.delete(monitorIndex);
        console.log(`SpotlightDimmer: Removed overlays for monitor ${monitorIndex}`);
    }

    /**
     * Check if overlays exist for a monitor.
     * @param {number} monitorIndex - Monitor index
     * @returns {boolean} True if overlays exist
     */
    hasMonitor(monitorIndex) {
        return this._monitors.has(monitorIndex);
    }

    /**
     * Get number of monitors with overlays.
     * @returns {number} Number of monitors
     */
    get monitorCount() {
        return this._monitors.size;
    }

    /**
     * Clean up all overlays.
     */
    destroy() {
        for (const [monitorIndex, overlays] of this._monitors) {
            for (const overlay of overlays) {
                Main.layoutManager.removeChrome(overlay);
                overlay.destroy();
            }
        }

        this._monitors.clear();
        console.log('SpotlightDimmer: All overlays destroyed');
    }
}
