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
    },
}, class FocusTracker extends GObject.Object {
    _init() {
        super._init();

        this._focusSignalId = null;
        this._currentWindow = null;
        this._windowSignalIds = [];
        this._wmSizeChangeId = null;

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

        // Disconnect from window signals
        this._untrackCurrentWindow();
    }
});
