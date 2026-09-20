/**
 * SpotlightDimmer - Overlay Manager
 *
 * Manages St.Widget overlays for each monitor.
 * Pre-allocates 6 overlays per monitor (matching the C# DisplayOverlayState pattern).
 */

import GLib from 'gi://GLib';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

// Overlays pre-allocated per monitor. Six covers every region the legacy
// calculator emits (FullScreen, Top, Bottom, Left, Right, Center), which is
// the steady state; always-on-top windows carve bands into more pieces and
// grow the pool beyond this on demand.
const OVERLAYS_PER_MONITOR = 6;

// Overlays stack below all shell chrome, leaving notifications, OSD and
// panel menus fully lit.
const CHROME_HIGHLIGHT = 'Highlight';
// Overlays stack at the top of uiGroup, dimming chrome along with windows.
const CHROME_DIM = 'Dim';

/**
 * OverlayManager creates and manages St.Widget overlays for dimming.
 */
export class OverlayManager {
    /**
     * @param {string} [chromeHandling] - Initial stacking policy, preserved
     *   across the monitor hot-plug rebuild so a momentarily absent daemon
     *   cannot silently revert it.
     */
    constructor(chromeHandling = CHROME_HIGHLIGHT) {
        // Map<monitorIndex, St.Widget[]>
        this._monitors = new Map();
        this._chromeHandling = chromeHandling;
        this._restackId = 0;
        this._warnedNoWindowGroup = false;

        // Chrome added after us (another extension enabling later, modal
        // dialogs) lands on top of our overlays, so the stacking has to be
        // re-asserted rather than set once at creation.
        this._childAddedId = Main.layoutManager.uiGroup.connect(
            'child-added',
            () => this._queueRestack()
        );
    }

    /**
     * Current stacking policy, so a rebuilt manager can inherit it.
     * @returns {string} 'Highlight' or 'Dim'
     */
    get chromeHandling() {
        return this._chromeHandling;
    }

    /**
     * Set how overlays stack relative to shell chrome (notifications, OSD,
     * panel, dock). Pushed by the daemon in every overlays payload.
     * @param {string} value - 'Highlight' or 'Dim'
     */
    setChromeHandling(value) {
        const handling = value === CHROME_DIM ? CHROME_DIM : CHROME_HIGHLIGHT;
        if (handling === this._chromeHandling) {
            return;
        }

        this._chromeHandling = handling;
        this._restackAll();
    }

    /**
     * The direct uiGroup child that holds application windows. Overlays are
     * stacked immediately above it to sit over every window but under every
     * piece of chrome. Walks up from global.window_group rather than assuming
     * it is a direct child, since that nesting is a shell internal.
     * @returns {Clutter.Actor|null} Sibling to stack above, null if not found
     * @private
     */
    _windowLayerSibling() {
        const uiGroup = Main.layoutManager.uiGroup;
        let actor = global.window_group;

        while (actor && actor.get_parent() && actor.get_parent() !== uiGroup) {
            actor = actor.get_parent();
        }

        return actor && actor.get_parent() === uiGroup ? actor : null;
    }

    /**
     * Apply the current stacking policy to one overlay.
     * @param {St.Widget} overlay - Overlay to restack
     * @private
     */
    _applyStacking(overlay) {
        const uiGroup = Main.layoutManager.uiGroup;
        if (overlay.get_parent() !== uiGroup) {
            return;
        }

        if (this._chromeHandling === CHROME_DIM) {
            uiGroup.set_child_above_sibling(overlay, null);
            return;
        }

        const sibling = this._windowLayerSibling();
        if (!sibling) {
            // Unexpected shell layout: staying on top still dims correctly,
            // it just keeps the chrome-over-spotlight artifact.
            if (!this._warnedNoWindowGroup) {
                console.warn(
                    'SpotlightDimmer: window group is not under uiGroup; ' +
                    'keeping overlays on top (chrome will be dimmed)'
                );
                this._warnedNoWindowGroup = true;
            }
            uiGroup.set_child_above_sibling(overlay, null);
            return;
        }

        uiGroup.set_child_above_sibling(overlay, sibling);
    }

    /**
     * Restack every overlay on every monitor.
     * @private
     */
    _restackAll() {
        for (const { overlays } of this._monitors.values()) {
            for (const overlay of overlays) {
                this._applyStacking(overlay);
            }
        }
    }

    /**
     * Coalesce restacks to an idle callback: 'child-added' fires once per
     * added actor (including our own six per monitor), and restacking from
     * inside the emission would reorder the children mid-iteration.
     * @private
     */
    _queueRestack() {
        if (this._restackId) {
            return;
        }

        this._restackId = GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            this._restackId = 0;
            this._restackAll();
            return GLib.SOURCE_REMOVE;
        });
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

        // The geometry is kept so overlays created later (when the daemon
        // sends more rects than the pool holds) start off-screen-safe rather
        // than at the origin of the leftmost monitor.
        const entry = { geometry, overlays: [] };
        this._monitors.set(monitorIndex, entry);
        this._growPool(entry, OVERLAYS_PER_MONITOR);

        console.log(`SpotlightDimmer: Created ${OVERLAYS_PER_MONITOR} overlays for monitor ${monitorIndex}`);
    }

    /**
     * Grow a monitor's overlay pool to at least `count` widgets. Widgets are
     * never destroyed on shrink: reusing them keeps the update path
     * allocation-free once the high-water mark is reached.
     * @param {Object} entry - {geometry, overlays}
     * @param {number} count - Required pool size
     * @private
     */
    _growPool(entry, count) {
        while (entry.overlays.length < count) {
            const overlay = new St.Widget({
                style_class: 'spotlight-dimmer-overlay',
                reactive: false, // CRITICAL: Click-through - users can interact with windows below
                can_focus: false,
                track_hover: false,
                visible: false,
                x: entry.geometry.x,
                y: entry.geometry.y,
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
            this._applyStacking(overlay);
            entry.overlays.push(overlay);
        }
    }

    /**
     * Update overlays for a monitor based on calculated definitions.
     * @param {number} monitorIndex - Monitor index
     * @param {Array} definitions - Array of overlay definitions from calculator
     */
    updateMonitor(monitorIndex, definitions) {
        const entry = this._monitors.get(monitorIndex);
        if (!entry) {
            console.warn(`SpotlightDimmer: No overlays found for monitor ${monitorIndex}`);
            return;
        }

        // Definitions are consumed in order rather than by def.region: with
        // always-on-top windows carving holes, a single region can produce
        // several rects and region stops being a slot index.
        const visible = definitions ? definitions.filter(def => def.visible) : [];
        this._growPool(entry, visible.length);

        const overlays = entry.overlays;
        for (let i = 0; i < overlays.length; i++) {
            const overlay = overlays[i];
            const def = visible[i];

            if (!def) {
                // Hide unused overlay
                overlay.visible = false;
                continue;
            }

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
        }
    }

    /**
     * Hide all overlays on all monitors.
     */
    hideAll() {
        for (const { overlays } of this._monitors.values()) {
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
        const entry = this._monitors.get(monitorIndex);
        if (!entry) {
            return;
        }

        for (const overlay of entry.overlays) {
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
        if (this._restackId) {
            GLib.source_remove(this._restackId);
            this._restackId = 0;
        }

        if (this._childAddedId) {
            Main.layoutManager.uiGroup.disconnect(this._childAddedId);
            this._childAddedId = 0;
        }

        for (const { overlays } of this._monitors.values()) {
            for (const overlay of overlays) {
                Main.layoutManager.removeChrome(overlay);
                overlay.destroy();
            }
        }

        this._monitors.clear();
        console.log('SpotlightDimmer: All overlays destroyed');
    }
}
