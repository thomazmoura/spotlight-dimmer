/**
 * SpotlightDimmer - Focus Tracker
 *
 * Tracks focus changes and window position changes using GNOME Shell signals.
 * Emits signals when focus or window geometry changes.
 */

import GObject from 'gi://GObject';
import GLib from 'gi://GLib';

/**
 * FocusTracker monitors window focus and position changes.
 * Emits 'focus-changed' and 'window-geometry-changed' signals.
 */
export const FocusTracker = GObject.registerClass({
    Signals: {
        // Emitted when focus changes to a different window
        // Parameters: window (Meta.Window or null), monitorIndex (int)
        'focus-changed': {
            param_types: [GObject.TYPE_OBJECT, GObject.TYPE_INT],
        },
        // Emitted when the focused window's geometry changes
        // Parameters: window (Meta.Window), monitorIndex (int)
        'window-geometry-changed': {
            param_types: [GObject.TYPE_OBJECT, GObject.TYPE_INT],
        },
        // Emitted when the focused window's title changes (terminals change
        // titles on tmux attach/detach and tab switches without any focus or
        // geometry event; the daemon uses this to requery wezterm/tmux)
        // Parameters: window (Meta.Window)
        'window-title-changed': {
            param_types: [GObject.TYPE_OBJECT],
        },
        // Emitted when the set of always-on-top windows, or any of their
        // rects, changed. Parameter: the rects as a JSON array string, in
        // stacking order with the topmost last.
        'floating-changed': {
            param_types: [GObject.TYPE_STRING],
        },
    },
}, class FocusTracker extends GObject.Object {
    _init() {
        super._init();

        this._focusSignalId = null;
        this._currentWindow = null;
        this._windowSignalIds = [];
        this._wmSizeChangeId = null;

        // Always-on-top tracking stays off until the daemon asks for it, so
        // the default configuration pays nothing for this feature.
        this._trackFloating = false;
        this._restackedId = null;
        this._floatingSignalIds = [];
        this._lastFloatingJson = '[]';

        this._connectDisplaySignals();

        // Track initial focused window
        this._trackCurrentWindow(global.display.focus_window);
    }

    /**
     * Connect to global display signals.
     * @private
     */
    _connectDisplaySignals() {
        // Focus change signal - fires when a different window gains focus
        this._focusSignalId = global.display.connect(
            'notify::focus-window',
            this._onFocusChanged.bind(this)
        );
        this._wmSizeChangeId = global.window_manager.connect('size-change',
             (wm, actor, change) => {
                 // Use timeout to wait for animation to complete
                 GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                     this._onWindowGeometryChanged();
                     return GLib.SOURCE_REMOVE;
                 });
             }
         );
    }

    /**
     * Handle focus change event.
     * @private
     */
    _onFocusChanged() {
        const newWindow = global.display.focus_window;

        // Disconnect from old window's signals
        this._untrackCurrentWindow();

        // Connect to new window's signals
        this._trackCurrentWindow(newWindow);

        // Emit focus changed signal
        const monitor = newWindow ? newWindow.get_monitor() : 0;
        this.emit('focus-changed', newWindow, monitor);
    }

    /**
     * Turn always-on-top tracking on or off. Driven by `track_floating` in
     * the daemon's overlays payload.
     * @param {boolean} enabled - Whether to report always-on-top windows
     */
    setTrackFloating(enabled) {
        const wanted = !!enabled;
        if (wanted === this._trackFloating) {
            return;
        }

        this._trackFloating = wanted;

        if (!wanted) {
            if (this._restackedId) {
                global.display.disconnect(this._restackedId);
                this._restackedId = null;
            }
            this._disconnectFloatingWindows();
            // Tell the daemon to drop whatever it still holds.
            if (this._lastFloatingJson !== '[]') {
                this._lastFloatingJson = '[]';
                this.emit('floating-changed', '[]');
            }
            return;
        }

        // 'restacked' is the only signal that covers a window being raised,
        // lowered, added, removed or having its above state toggled.
        this._restackedId = global.display.connect('restacked', () => {
            this._refreshFloating(true);
        });
        this._refreshFloating(true);
    }

    /**
     * Re-send the current always-on-top rects even if unchanged. Used after
     * the daemon restarts, since it comes back with empty state.
     */
    resendFloating() {
        if (!this._trackFloating) {
            return;
        }

        this._lastFloatingJson = null;
        this._refreshFloating(true);
    }

    /**
     * Collect the rects of every visible always-on-top window, in stacking
     * order. `global.get_window_actors()` is already bottom-first, which is
     * the order the daemon expects.
     * @returns {Array<Object>} Rects as {x, y, width, height}
     * @private
     */
    _collectFloating() {
        const rects = [];

        let actors;
        try {
            actors = global.get_window_actors();
        } catch (e) {
            console.warn(`SpotlightDimmer: Error listing window actors: ${e.message}`);
            return rects;
        }

        for (const actor of actors) {
            try {
                const window = actor.meta_window ?? actor.get_meta_window();
                if (!window || window.minimized || !window.is_above()) {
                    continue;
                }

                // Frame rect, like the focus path: get_buffer_rect() would
                // include CSD shadows and overshoot the visible window.
                const rect = window.get_frame_rect();
                if (!rect || rect.width <= 0 || rect.height <= 0) {
                    continue;
                }

                rects.push({
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                });
            } catch (e) {
                // Window destroyed mid-enumeration; skip it.
            }
        }

        return rects;
    }

    /**
     * Re-enumerate always-on-top windows and emit only when something
     * actually changed. The diff matters: 'restacked' fires on every raise,
     * and re-sending an unchanged set would put needless traffic on D-Bus.
     * @param {boolean} resubscribe - Also rebuild per-window geometry signals
     * @private
     */
    _refreshFloating(resubscribe) {
        if (!this._trackFloating) {
            return;
        }

        if (resubscribe) {
            this._subscribeFloatingWindows();
        }

        const json = JSON.stringify(this._collectFloating());
        if (json === this._lastFloatingJson) {
            return;
        }

        this._lastFloatingJson = json;
        this.emit('floating-changed', json);
    }

    /**
     * Follow position and size on every always-on-top window, so dragging
     * one moves its exemption with it.
     * @private
     */
    _subscribeFloatingWindows() {
        this._disconnectFloatingWindows();

        let actors;
        try {
            actors = global.get_window_actors();
        } catch (e) {
            return;
        }

        for (const actor of actors) {
            try {
                const window = actor.meta_window ?? actor.get_meta_window();
                if (!window || !window.is_above()) {
                    continue;
                }

                for (const signal of ['position-changed', 'size-changed']) {
                    const id = window.connect(signal, () => this._refreshFloating(false));
                    this._floatingSignalIds.push({ window, id });
                }
            } catch (e) {
                // Window destroyed mid-enumeration; skip it.
            }
        }
    }

    /**
     * Drop the per-window geometry signals from the always-on-top set.
     * @private
     */
    _disconnectFloatingWindows() {
        for (const { window, id } of this._floatingSignalIds) {
            try {
                window.disconnect(id);
            } catch (e) {
                // Window already destroyed - normal.
            }
        }
        this._floatingSignalIds = [];
    }

    /**
     * Track a window for position/size changes.
     * @param {Meta.Window|null} window - Window to track
     * @private
     */
    _trackCurrentWindow(window) {
        this._currentWindow = window;

        if (!window) {
            return;
        }

        try {
            // Connect to position changes (window dragged)
            const positionId = window.connect('position-changed', () => {
                this._onWindowGeometryChanged();
            });
            this._windowSignalIds.push({ window, id: positionId });

            // Connect to size changes (window resized)
            const sizeId = window.connect('size-changed', () => {
                this._onWindowGeometryChanged();
            });
            this._windowSignalIds.push({ window, id: sizeId });

            // Connect to maximized state changes (keyboard snapping: Super+Left/Right)
            const maximizedHId = window.connect('notify::maximized-horizontally', () => {
                this._onWindowGeometryChangedDeferred();
            });
            this._windowSignalIds.push({ window, id: maximizedHId });

            const maximizedVId = window.connect('notify::maximized-vertically', () => {
                this._onWindowGeometryChangedDeferred();
            });
            this._windowSignalIds.push({ window, id: maximizedVId });

            // Connect to fullscreen changes (F11 or fullscreen toggle)
            const fullscreenId = window.connect('notify::fullscreen', () => {
                this._onWindowGeometryChangedDeferred();
            });
            this._windowSignalIds.push({ window, id: fullscreenId });

            // Connect to title changes (forwarded to the daemon, which
            // debounces and requeries the wezterm/tmux integration)
            const titleId = window.connect('notify::title', () => {
                this.emit('window-title-changed', window);
            });
            this._windowSignalIds.push({ window, id: titleId });
        } catch (e) {
            // Window may have been destroyed during connection
            console.warn(`SpotlightDimmer: Error tracking window: ${e.message}`);
        }
    }

    /**
     * Handle window geometry change event.
     * @private
     */
    _onWindowGeometryChanged() {
        if (!this._currentWindow) {
            return;
        }

        try {
            const monitor = this._currentWindow.get_monitor();
            this.emit('window-geometry-changed', this._currentWindow, monitor);
        } catch (e) {
            // Window may have been destroyed
            console.warn(`SpotlightDimmer: Error getting window geometry: ${e.message}`);
        }
    }

    /**
     * Handle window geometry change with deferral.
     * Used for property changes (maximized, fullscreen) where we need to wait
     * for GNOME's animation to complete before reading the final geometry.
     * @private
     */
    _onWindowGeometryChangedDeferred() {
        // Defer update to next frame to allow GNOME animation to complete
        GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            this._onWindowGeometryChanged();
            return GLib.SOURCE_REMOVE;
        });
    }

    /**
     * Disconnect from current window's signals.
     * @private
     */
    _untrackCurrentWindow() {
        for (const { window, id } of this._windowSignalIds) {
            try {
                window.disconnect(id);
            } catch (e) {
                // Window may have been destroyed - this is normal
            }
        }
        this._windowSignalIds = [];
        this._currentWindow = null;
    }

    /**
     * Get current focused window information.
     * @returns {Object|null} {window, monitor, rect} or null if no focused window
     */
    getCurrentFocus() {
        const window = global.display.focus_window;
        if (!window) {
            return null;
        }

        try {
            const rect = window.get_frame_rect();
            return {
                window,
                monitor: window.get_monitor(),
                rect: {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                },
            };
        } catch (e) {
            // Window may be in an invalid state
            console.warn(`SpotlightDimmer: Error getting focus info: ${e.message}`);
            return null;
        }
    }

    /**
     * Force emit a focus changed event for the current window.
     * Useful for initial setup or refreshing state.
     */
    emitCurrentFocus() {
        const window = global.display.focus_window;
        const monitor = window ? window.get_monitor() : 0;
        this.emit('focus-changed', window, monitor);
    }

    /**
     * Clean up resources.
     */
    destroy() {
        // Disconnect from display signals
        if (this._focusSignalId) {
            global.display.disconnect(this._focusSignalId);
            this._focusSignalId = null;
        }

        if (this._wmSizeChangeId) {
            global.window_manager.disconnect(this._wmSizeChangeId);
            this._wmSizeChangeId = null;
        }

        // Disconnect from always-on-top tracking
        if (this._restackedId) {
            global.display.disconnect(this._restackedId);
            this._restackedId = null;
        }

        this._disconnectFloatingWindows();

        // Disconnect from window signals
        this._untrackCurrentWindow();
    }
});
