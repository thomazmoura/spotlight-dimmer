/**
 * SpotlightDimmer - Overlay Geometry Calculator
 *
 * Port of SpotlightDimmer.Core/AppState.cs calculation logic to JavaScript.
 * This module has no GNOME dependencies and can be tested independently.
 */

/**
 * Overlay region enumeration matching C# OverlayRegion.
 * Each display can have up to 6 overlays (one per region).
 */
export const OverlayRegion = Object.freeze({
    FULLSCREEN: 0,
    TOP: 1,
    BOTTOM: 2,
    LEFT: 3,
    RIGHT: 4,
    CENTER: 5,
});

/**
 * Dimming mode enumeration matching C# DimmingMode.
 */
export const DimmingMode = Object.freeze({
    FULLSCREEN: 'FullScreen',
    PARTIAL: 'Partial',
    PARTIAL_WITH_ACTIVE: 'PartialWithActive',
});

/**
 * Calculates overlay definitions for monitors.
 * Port of AppState.Calculate() from C# codebase.
 */
export class OverlayCalculator {
    /**
     * Calculate overlays for a single monitor.
     *
     * @param {Object} config - Configuration object with:
     *   - mode: DimmingMode value
     *   - inactiveColor: {r, g, b} color for inactive areas
     *   - inactiveOpacity: 0-255 opacity for inactive areas
     *   - activeColor: {r, g, b} color for active window (PartialWithActive only)
     *   - activeOpacity: 0-255 opacity for active window
     * @param {Object} monitorGeometry - Monitor bounds {x, y, width, height}
     * @param {Object|null} windowRect - Focused window {x, y, width, height} or null
     * @param {boolean} isFocusedMonitor - Whether this monitor has the focused window
     * @returns {Array|null} Array of overlay definitions, or null to keep existing state
     */
    calculate(config, monitorGeometry, windowRect, isFocusedMonitor) {
        // Skip update if window has invalid 0x0 dimensions (prevents flickering)
        // This happens during window transitions and focus changes
        if (windowRect && (windowRect.width === 0 || windowRect.height === 0)) {
            return null; // Signal to keep existing state
        }

        // Non-focused monitors always get a full-screen overlay
        if (!isFocusedMonitor) {
            return this._createFullScreenOverlay(monitorGeometry, config);
        }

        // Focused monitor behavior depends on mode
        switch (config.mode) {
            case DimmingMode.FULLSCREEN:
                // No overlays on focused monitor in FullScreen mode
                return [];

            case DimmingMode.PARTIAL:
                return this._calculatePartialOverlays(
                    monitorGeometry,
                    windowRect,
                    config.inactiveColor,
                    config.inactiveOpacity
                );

            case DimmingMode.PARTIAL_WITH_ACTIVE:
                return this._calculatePartialWithActiveOverlays(
                    monitorGeometry,
                    windowRect,
                    config
                );

            default:
                // Unknown mode, no overlays
                return [];
        }
    }

    /**
     * Create a single full-screen overlay for a non-focused monitor.
     * @private
     */
    _createFullScreenOverlay(monitor, config) {
        return [{
            region: OverlayRegion.FULLSCREEN,
            visible: true,
            x: monitor.x,
            y: monitor.y,
            width: monitor.width,
            height: monitor.height,
            color: config.inactiveColor,
            opacity: config.inactiveOpacity,
        }];
    }

    /**
     * Calculate the 4 edge overlays (Top, Bottom, Left, Right) around a focused window.
     * Port of AppState.UpdatePartialOverlays() from C#.
     * @private
     */
    _calculatePartialOverlays(monitor, window, color, opacity) {
        if (!window) {
            return [];
        }

        const overlays = [];

        // Clamp window bounds to monitor bounds
        const clamped = this._clampToMonitor(window, monitor);

        // If window is not visible on this monitor, no overlays
        if (clamped.width <= 0 || clamped.height <= 0) {
            return [];
        }

        // Top overlay: Full width, from display top to window top
        const topHeight = clamped.y - monitor.y;
        if (topHeight > 0) {
            overlays.push({
                region: OverlayRegion.TOP,
                visible: true,
                x: monitor.x,
                y: monitor.y,
                width: monitor.width,
                height: topHeight,
                color: color,
                opacity: opacity,
            });
        }

        // Bottom overlay: Full width, from window bottom to display bottom
        const bottomY = clamped.y + clamped.height;
        const bottomHeight = (monitor.y + monitor.height) - bottomY;
        if (bottomHeight > 0) {
            overlays.push({
                region: OverlayRegion.BOTTOM,
                visible: true,
                x: monitor.x,
                y: bottomY,
                width: monitor.width,
                height: bottomHeight,
                color: color,
                opacity: opacity,
            });
        }

        // Left overlay: Window height, from display left to window left
        const leftWidth = clamped.x - monitor.x;
        if (leftWidth > 0) {
            overlays.push({
                region: OverlayRegion.LEFT,
                visible: true,
                x: monitor.x,
                y: clamped.y,
                width: leftWidth,
                height: clamped.height,
                color: color,
                opacity: opacity,
            });
        }

        // Right overlay: Window height, from window right to display right
        const rightX = clamped.x + clamped.width;
        const rightWidth = (monitor.x + monitor.width) - rightX;
        if (rightWidth > 0) {
            overlays.push({
                region: OverlayRegion.RIGHT,
                visible: true,
                x: rightX,
                y: clamped.y,
                width: rightWidth,
                height: clamped.height,
                color: color,
                opacity: opacity,
            });
        }

        return overlays;
    }

    /**
     * Calculate 4 edge overlays plus a center overlay on the focused window.
     * Port of AppState.UpdatePartialWithActiveOverlays() from C#.
     * @private
     */
    _calculatePartialWithActiveOverlays(monitor, window, config) {
        // First, calculate the 4 edge overlays with inactive color
        const overlays = this._calculatePartialOverlays(
            monitor,
            window,
            config.inactiveColor,
            config.inactiveOpacity
        );

        // Add center overlay on the focused window with active color
        if (window) {
            const clamped = this._clampToMonitor(window, monitor);

            if (clamped.width > 0 && clamped.height > 0) {
                overlays.push({
                    region: OverlayRegion.CENTER,
                    visible: true,
                    x: clamped.x,
                    y: clamped.y,
                    width: clamped.width,
                    height: clamped.height,
                    color: config.activeColor,
                    opacity: config.activeOpacity,
                });
            }
        }

        return overlays;
    }

    /**
     * Clamp a rectangle to fit within monitor bounds.
     * Port of AppState.ClampToDisplay() from C#.
     * @private
     */
    _clampToMonitor(rect, monitor) {
        const left = Math.max(rect.x, monitor.x);
        const top = Math.max(rect.y, monitor.y);
        const right = Math.min(rect.x + rect.width, monitor.x + monitor.width);
        const bottom = Math.min(rect.y + rect.height, monitor.y + monitor.height);

        return {
            x: left,
            y: top,
            width: Math.max(0, right - left),
            height: Math.max(0, bottom - top),
        };
    }
}
