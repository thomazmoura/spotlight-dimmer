# SpotlightDimmer GNOME Shell Extension

**Project:** SpotlightDimmer
**Created:** 2025-12-09
**Target:** GNOME Shell 45, 46, 47, 48+
**Language:** JavaScript (GJS)

---

## Overview

This document describes the architecture for a GNOME Shell extension that provides SpotlightDimmer functionality on GNOME Wayland. Since GNOME does not support the `wlr-layer-shell` protocol, a shell extension is the **only way** to create overlay windows on GNOME Wayland.

### Why an Extension is Required

GNOME Wayland intentionally does not support:
- `wlr-layer-shell` protocol (for creating overlay windows)
- `ext-foreign-toplevel-list-v1` with geometry (for querying window positions)
- Any external protocol for window introspection

A GNOME Shell extension runs **inside** the compositor and has full access to:
- `Meta.Window` API - Window geometry, focus state, monitor info
- `global.display` - Focus change signals
- `Clutter` actors - Rendering overlays
- `St` widgets - UI components with click-through support

---

## Architecture

### File Structure

```
spotlight-dimmer-gnome/
├─ extension.js          # Main extension entry point
├─ metadata.json         # Extension metadata
├─ overlayManager.js     # Overlay actor management
├─ focusTracker.js       # Focus change tracking
├─ configBridge.js       # Configuration file reading
├─ calculator.js         # Overlay geometry calculation
├─ stylesheet.css        # Optional CSS styling
└─ prefs.js             # Extension preferences UI
```

### metadata.json

```json
{
  "name": "SpotlightDimmer",
  "description": "Dims inactive displays and regions around the focused window",
  "uuid": "spotlightdimmer@example.com",
  "shell-version": ["45", "46", "47", "48"],
  "version": 1,
  "url": "https://github.com/your-repo/spotlight-dimmer-gnome",
  "settings-schema": "org.gnome.shell.extensions.spotlightdimmer"
}
```

---

## Core Components

### 1. extension.js - Main Entry Point

```javascript
// GNOME 45+ ESModules syntax
import Meta from 'gi://Meta';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';

import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import { OverlayManager } from './overlayManager.js';
import { FocusTracker } from './focusTracker.js';
import { ConfigBridge } from './configBridge.js';
import { OverlayCalculator } from './calculator.js';

export default class SpotlightDimmerExtension extends Extension {
    enable() {
        this._configBridge = new ConfigBridge();
        this._overlayManager = new OverlayManager();
        this._calculator = new OverlayCalculator();
        this._focusTracker = new FocusTracker();

        // Create overlays for each monitor
        this._createOverlaysForAllMonitors();

        // Connect to focus changes
        this._focusTracker.connect('focus-changed', (_, window, monitor) => {
            this._updateOverlays(window, monitor);
        });

        // Connect to config changes
        this._configBridge.connect('config-changed', () => {
            this._reloadConfig();
        });

        // Initial update
        this._updateOverlays(
            global.display.focus_window,
            this._getMonitorForWindow(global.display.focus_window)
        );
    }

    disable() {
        this._focusTracker?.destroy();
        this._overlayManager?.destroy();
        this._configBridge?.destroy();

        this._focusTracker = null;
        this._overlayManager = null;
        this._configBridge = null;
        this._calculator = null;
    }

    _createOverlaysForAllMonitors() {
        const nMonitors = global.display.get_n_monitors();
        for (let i = 0; i < nMonitors; i++) {
            const geometry = global.display.get_monitor_geometry(i);
            this._overlayManager.createOverlaysForMonitor(i, geometry);
        }
    }

    _updateOverlays(focusedWindow, focusedMonitor) {
        const config = this._configBridge.getConfig();
        const nMonitors = global.display.get_n_monitors();

        let windowRect = null;
        if (focusedWindow) {
            const rect = focusedWindow.get_frame_rect();
            windowRect = { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
        }

        for (let i = 0; i < nMonitors; i++) {
            const monitorGeometry = global.display.get_monitor_geometry(i);
            const isFocused = i === focusedMonitor;

            const overlays = this._calculator.calculate(
                config.mode,
                monitorGeometry,
                isFocused ? windowRect : null,
                isFocused,
                config.inactiveColor,
                config.inactiveOpacity,
                config.activeColor,
                config.activeOpacity
            );

            this._overlayManager.updateMonitor(i, overlays);
        }
    }

    _getMonitorForWindow(window) {
        if (!window) return 0;
        return window.get_monitor();
    }

    _reloadConfig() {
        const focusedWindow = global.display.focus_window;
        const focusedMonitor = this._getMonitorForWindow(focusedWindow);
        this._updateOverlays(focusedWindow, focusedMonitor);
    }
}
```

### 2. focusTracker.js - Focus Change Detection

```javascript
import GObject from 'gi://GObject';
import Meta from 'gi://Meta';

export const FocusTracker = GObject.registerClass({
    Signals: {
        'focus-changed': { param_types: [Meta.Window, GObject.TYPE_INT] },
    },
}, class FocusTracker extends GObject.Object {
    _init() {
        super._init();
        this._focusId = null;
        this._positionId = null;
        this._currentWindow = null;

        this._connectSignals();
    }

    _connectSignals() {
        // Focus change signal
        this._focusId = global.display.connect('notify::focus-window', () => {
            this._onFocusChanged();
        });

        // Window position change - connect to current window
        this._trackCurrentWindow();
    }

    _onFocusChanged() {
        const window = global.display.focus_window;

        // Disconnect from old window
        this._untrackCurrentWindow();

        // Connect to new window
        this._currentWindow = window;
        this._trackCurrentWindow();

        // Emit signal
        const monitor = window ? window.get_monitor() : 0;
        this.emit('focus-changed', window, monitor);
    }

    _trackCurrentWindow() {
        if (!this._currentWindow) return;

        this._positionId = this._currentWindow.connect('position-changed', () => {
            const monitor = this._currentWindow.get_monitor();
            this.emit('focus-changed', this._currentWindow, monitor);
        });
    }

    _untrackCurrentWindow() {
        if (this._currentWindow && this._positionId) {
            this._currentWindow.disconnect(this._positionId);
            this._positionId = null;
        }
        this._currentWindow = null;
    }

    destroy() {
        if (this._focusId) {
            global.display.disconnect(this._focusId);
            this._focusId = null;
        }
        this._untrackCurrentWindow();
    }
});
```

### 3. overlayManager.js - Overlay Actor Management

```javascript
import St from 'gi://St';
import Clutter from 'gi://Clutter';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

export class OverlayManager {
    constructor() {
        // 6 overlays per monitor: FullScreen, Top, Bottom, Left, Right, Center
        this._monitors = new Map(); // Map<monitorIndex, OverlaySet>
    }

    createOverlaysForMonitor(monitorIndex, geometry) {
        const overlays = [];

        // Create 6 pre-allocated overlays
        for (let i = 0; i < 6; i++) {
            const overlay = new St.Widget({
                style_class: 'spotlight-dimmer-overlay',
                reactive: false,  // CRITICAL: Click-through!
                visible: false,
                x: geometry.x,
                y: geometry.y,
                width: 0,
                height: 0,
            });

            // Add to top chrome (above all windows)
            Main.layoutManager.addTopChrome(overlay);
            overlays.push(overlay);
        }

        this._monitors.set(monitorIndex, overlays);
    }

    updateMonitor(monitorIndex, overlayDefinitions) {
        const overlays = this._monitors.get(monitorIndex);
        if (!overlays) return;

        for (let i = 0; i < overlayDefinitions.length; i++) {
            const def = overlayDefinitions[i];
            const overlay = overlays[i];

            if (def.visible) {
                overlay.visible = true;
                overlay.x = def.x;
                overlay.y = def.y;
                overlay.width = def.width;
                overlay.height = def.height;

                // Set color and opacity via inline style
                const r = def.color.r;
                const g = def.color.g;
                const b = def.color.b;
                const a = def.opacity / 255;
                overlay.style = `background-color: rgba(${r}, ${g}, ${b}, ${a});`;
            } else {
                overlay.visible = false;
            }
        }

        // Hide unused overlays
        for (let i = overlayDefinitions.length; i < 6; i++) {
            overlays[i].visible = false;
        }
    }

    hideAll() {
        for (const overlays of this._monitors.values()) {
            for (const overlay of overlays) {
                overlay.visible = false;
            }
        }
    }

    destroy() {
        for (const overlays of this._monitors.values()) {
            for (const overlay of overlays) {
                Main.layoutManager.removeChrome(overlay);
                overlay.destroy();
            }
        }
        this._monitors.clear();
    }
}
```

### 4. configBridge.js - Configuration File Reading

```javascript
import GObject from 'gi://GObject';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';

const CONFIG_PATH = GLib.build_filenamev([
    GLib.get_user_config_dir(),
    'SpotlightDimmer',
    'config.json'
]);

export const ConfigBridge = GObject.registerClass({
    Signals: {
        'config-changed': {},
    },
}, class ConfigBridge extends GObject.Object {
    _init() {
        super._init();
        this._config = this._getDefaultConfig();
        this._monitor = null;

        this._loadConfig();
        this._watchConfig();
    }

    _getDefaultConfig() {
        return {
            mode: 'FullScreen',
            inactiveColor: { r: 0, g: 0, b: 0 },
            inactiveOpacity: 153,
            activeColor: { r: 0, g: 0, b: 0 },
            activeOpacity: 102,
        };
    }

    _loadConfig() {
        try {
            const file = Gio.File.new_for_path(CONFIG_PATH);
            const [success, contents] = file.load_contents(null);

            if (success) {
                const json = new TextDecoder().decode(contents);
                const data = JSON.parse(json);
                this._parseConfig(data);
            }
        } catch (e) {
            log(`SpotlightDimmer: Failed to load config: ${e.message}`);
        }
    }

    _parseConfig(data) {
        // Parse Overlay settings
        if (data.Overlay) {
            this._config.mode = data.Overlay.Mode || 'FullScreen';
            this._config.inactiveOpacity = data.Overlay.InactiveOpacity ?? 153;
            this._config.activeOpacity = data.Overlay.ActiveOpacity ?? 102;

            // Parse hex colors
            if (data.Overlay.InactiveColor) {
                this._config.inactiveColor = this._parseHexColor(data.Overlay.InactiveColor);
            }
            if (data.Overlay.ActiveColor) {
                this._config.activeColor = this._parseHexColor(data.Overlay.ActiveColor);
            }
        }
    }

    _parseHexColor(hex) {
        // Remove # prefix if present
        hex = hex.replace('#', '');
        return {
            r: parseInt(hex.substring(0, 2), 16),
            g: parseInt(hex.substring(2, 4), 16),
            b: parseInt(hex.substring(4, 6), 16),
        };
    }

    _watchConfig() {
        try {
            const file = Gio.File.new_for_path(CONFIG_PATH);
            this._monitor = file.monitor_file(Gio.FileMonitorFlags.NONE, null);
            this._monitor.connect('changed', (monitor, file, otherFile, eventType) => {
                if (eventType === Gio.FileMonitorEvent.CHANGED ||
                    eventType === Gio.FileMonitorEvent.CREATED) {
                    // Debounce
                    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                        this._loadConfig();
                        this.emit('config-changed');
                        return GLib.SOURCE_REMOVE;
                    });
                }
            });
        } catch (e) {
            log(`SpotlightDimmer: Failed to watch config: ${e.message}`);
        }
    }

    getConfig() {
        return this._config;
    }

    destroy() {
        if (this._monitor) {
            this._monitor.cancel();
            this._monitor = null;
        }
    }
});
```

### 5. calculator.js - Overlay Geometry Calculation

```javascript
// Port of Core/AppState.cs overlay calculation logic

export class OverlayCalculator {
    calculate(mode, monitorGeometry, windowRect, isFocused, inactiveColor, inactiveOpacity, activeColor, activeOpacity) {
        const overlays = [];

        if (!isFocused) {
            // Non-focused monitor: full screen overlay
            overlays.push({
                visible: true,
                x: monitorGeometry.x,
                y: monitorGeometry.y,
                width: monitorGeometry.width,
                height: monitorGeometry.height,
                color: inactiveColor,
                opacity: inactiveOpacity,
            });
            return overlays;
        }

        // Focused monitor - depends on mode
        switch (mode) {
            case 'FullScreen':
                // No overlays on focused monitor
                return [];

            case 'Partial':
                return this._calculatePartialOverlays(
                    monitorGeometry, windowRect, inactiveColor, inactiveOpacity
                );

            case 'PartialWithActive':
                const partial = this._calculatePartialOverlays(
                    monitorGeometry, windowRect, inactiveColor, inactiveOpacity
                );
                // Add center overlay
                if (windowRect) {
                    partial.push({
                        visible: true,
                        x: windowRect.x,
                        y: windowRect.y,
                        width: windowRect.width,
                        height: windowRect.height,
                        color: activeColor,
                        opacity: activeOpacity,
                    });
                }
                return partial;

            default:
                return [];
        }
    }

    _calculatePartialOverlays(monitor, window, color, opacity) {
        if (!window) return [];

        const overlays = [];

        // Clamp window to monitor bounds
        const clampedWindow = {
            x: Math.max(window.x, monitor.x),
            y: Math.max(window.y, monitor.y),
            width: Math.min(window.x + window.width, monitor.x + monitor.width) - Math.max(window.x, monitor.x),
            height: Math.min(window.y + window.height, monitor.y + monitor.height) - Math.max(window.y, monitor.y),
        };

        if (clampedWindow.width <= 0 || clampedWindow.height <= 0) {
            // Window not visible on this monitor
            return [];
        }

        // Top overlay
        const topHeight = clampedWindow.y - monitor.y;
        if (topHeight > 0) {
            overlays.push({
                visible: true,
                x: monitor.x,
                y: monitor.y,
                width: monitor.width,
                height: topHeight,
                color, opacity,
            });
        }

        // Bottom overlay
        const bottomY = clampedWindow.y + clampedWindow.height;
        const bottomHeight = (monitor.y + monitor.height) - bottomY;
        if (bottomHeight > 0) {
            overlays.push({
                visible: true,
                x: monitor.x,
                y: bottomY,
                width: monitor.width,
                height: bottomHeight,
                color, opacity,
            });
        }

        // Left overlay (between top and bottom)
        const leftWidth = clampedWindow.x - monitor.x;
        if (leftWidth > 0) {
            overlays.push({
                visible: true,
                x: monitor.x,
                y: clampedWindow.y,
                width: leftWidth,
                height: clampedWindow.height,
                color, opacity,
            });
        }

        // Right overlay (between top and bottom)
        const rightX = clampedWindow.x + clampedWindow.width;
        const rightWidth = (monitor.x + monitor.width) - rightX;
        if (rightWidth > 0) {
            overlays.push({
                visible: true,
                x: rightX,
                y: clampedWindow.y,
                width: rightWidth,
                height: clampedWindow.height,
                color, opacity,
            });
        }

        return overlays;
    }
}
```

---

## GNOME Version Compatibility

### Breaking Changes by Version

| Version | Change | Migration |
|---------|--------|-----------|
| **GNOME 45** | ESModules migration | Use `import` instead of `imports` |
| **GNOME 45** | Extension class required | Extend `Extension` base class |
| **GNOME 46** | Clutter.cairo helpers removed | Use `cairo.Context` directly |
| **GNOME 46** | `Clutter.Container` removed | Use `add_child()` instead |
| **GNOME 47** | Minor API adjustments | See [Port to GNOME 47](https://gjs.guide/extensions/upgrading/gnome-shell-47.html) |

### GNOME 45+ Import Syntax

```javascript
// OLD (GNOME 44 and earlier)
const { Meta, St, Clutter } = imports.gi;
const Main = imports.ui.main;

// NEW (GNOME 45+)
import Meta from 'gi://Meta';
import St from 'gi://St';
import Clutter from 'gi://Clutter';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
```

---

## Configuration Sharing

The GNOME extension reads the same configuration file as the C# app:

**Location:** `~/.config/SpotlightDimmer/config.json`

**Format:**
```json
{
  "$schema": "../config.schema.json",
  "Overlay": {
    "Mode": "FullScreen",
    "InactiveColor": "#000000",
    "InactiveOpacity": 153,
    "ActiveColor": "#000000",
    "ActiveOpacity": 102
  }
}
```

The extension uses `GLib.FileMonitor` to watch for changes, identical to how the C# `ConfigurationManager` uses `FileSystemWatcher`.

---

## Key GNOME Shell APIs

### Meta.Window

```javascript
// Get focused window
const window = global.display.focus_window;

// Window geometry (includes decorations)
const frameRect = window.get_frame_rect();
// frameRect.x, frameRect.y, frameRect.width, frameRect.height

// Window geometry (client area only)
const bufferRect = window.get_buffer_rect();

// Which monitor
const monitorIndex = window.get_monitor();

// Window state
const maximized = window.is_fullscreen() || window.get_maximized();
```

### global.display (Meta.Display)

```javascript
// Focus change signal
global.display.connect('notify::focus-window', () => {
    const window = global.display.focus_window;
});

// Number of monitors
const nMonitors = global.display.get_n_monitors();

// Monitor geometry
const geometry = global.display.get_monitor_geometry(0);
// geometry.x, geometry.y, geometry.width, geometry.height

// Primary monitor index
const primary = global.display.get_primary_monitor();
```

### Main.layoutManager

```javascript
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

// Add overlay above all windows
Main.layoutManager.addTopChrome(widget);

// Remove overlay
Main.layoutManager.removeChrome(widget);

// Monitor info
const monitors = Main.layoutManager.monitors;
```

### St.Widget (Click-Through)

```javascript
const overlay = new St.Widget({
    reactive: false,  // CRITICAL: Makes widget click-through
    visible: true,
    x: 0,
    y: 0,
    width: 100,
    height: 100,
    style: 'background-color: rgba(0, 0, 0, 0.6);',
});
```

---

## Distribution

### Local Installation

```bash
# Create extension directory
mkdir -p ~/.local/share/gnome-shell/extensions/spotlightdimmer@example.com

# Copy files
cp extension.js metadata.json *.js stylesheet.css \
   ~/.local/share/gnome-shell/extensions/spotlightdimmer@example.com/

# Restart GNOME Shell (X11) or log out/in (Wayland)
# X11: Alt+F2, type 'r', press Enter
# Wayland: Log out and log back in

# Enable extension
gnome-extensions enable spotlightdimmer@example.com
```

### extensions.gnome.org

1. Create account on https://extensions.gnome.org
2. Package extension as ZIP (all files in root)
3. Submit for review
4. Users install via web interface or `gnome-extensions install`

---

## Sources

- [GNOME Shell Extensions Guide](https://gjs.guide/extensions/)
- [Meta.Window Documentation](https://mutter.gnome.org/meta/class.Window.html)
- [Meta.Display Documentation](https://mutter.gnome.org/meta/class.Display.html)
- [Meta.Display::focus-window Signal](https://mutter.gnome.org/meta/signal.Display.focus-window.html)
- [Port to GNOME Shell 45](https://gjs.guide/extensions/upgrading/gnome-shell-45.html)
- [Port to GNOME Shell 46](https://gjs.guide/extensions/upgrading/gnome-shell-46.html)
