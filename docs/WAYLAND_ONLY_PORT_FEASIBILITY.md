# SpotlightDimmer Wayland-Only Port Feasibility Analysis

**Project:** SpotlightDimmer
**Analysis Date:** 2025-11-30
**Last Updated:** 2025-12-09
**Target Platform:** Linux (Wayland ONLY - No X11 Support)
**Current Platform:** Windows 10/11 (.NET 10)

---

## Executive Summary

This analysis evaluates the feasibility of porting SpotlightDimmer to **Wayland exclusively**, without X11 support or fallback. This constraint significantly increases technical risk due to Wayland's compositor fragmentation and protocol limitations.

### Critical Assessment

| Aspect | Wayland-Only Feasibility | Risk Level |
|--------|-------------------------|------------|
| **Semi-Transparent Overlays** | ⚠️ 70% - Compositor dependent | HIGH |
| **Event-Driven Focus Tracking** | ❌ 40% - No universal protocol | CRITICAL |
| **Multi-Monitor Support** | ✅ 85% - Well supported | MEDIUM |
| **Configuration Hot-Reload** | ✅ 100% - Platform agnostic | LOW |
| **Config GUI** | ✅ 90% - GTK4 native to Wayland | LOW |
| **System Tray** | ⚠️ 70% - DE dependent | MEDIUM |
| **Auto-Start** | ✅ 95% - Standard mechanism | LOW |

### Overall Verdict

**⚠️ FEASIBLE BUT HIGH RISK** - Critical features depend on compositor-specific implementations with no universal fallback. Success rate varies dramatically by desktop environment:

- **GNOME 45+:** ⚠️ 60-70% feasibility (via GNOME Shell Extension - see Section 3.1)
- **KDE Plasma 5.27+:** ✅ 85% feasibility (good protocol support)
- **Sway/wlroots:** ✅ 90% feasibility (excellent protocol support)
- **Hyprland:** ✅ 85% feasibility (IPC available)
- **GNOME 44 and earlier:** ⚠️ 60% feasibility (deprecated APIs)

**Expected User Coverage:** ~85-95% of Wayland users with GNOME Shell Extension approach

> **⚠️ Update (Dec 2025):** With a dedicated GNOME Shell Extension (JavaScript), GNOME support increases from 30% to 60-70% feasibility. This requires a separate JavaScript codebase but enables GNOME Wayland users to use SpotlightDimmer. See updated GNOME Extension section below.

---

## 1. Core Feature Analysis (Wayland-Only Context)

### 1.1 Semi-Transparent Overlays - CRITICAL FEATURE ⚠️

**Windows Implementation:**
- `WS_EX_LAYERED` + `WS_EX_TRANSPARENT` for click-through transparency
- `SetLayeredWindowAttributes()` for opacity control
- `DeferWindowPos()` for atomic batch updates

**Wayland Equivalent - Multiple Protocol Challenges:**

#### Protocol Option 1: Layer Shell Protocol (zwlr_layer_shell_v1) ✅ RECOMMENDED

**What it is:**
- Wayland protocol extension for compositor overlays (panels, notifications, overlays)
- Provides layering system: Background, Bottom, Top, Overlay
- Designed specifically for this use case

**Compositor Support:**

| Compositor | Support Level | Notes |
|------------|---------------|-------|
| **Sway** | ✅ Excellent | Native wlroots implementation |
| **Wayfire** | ✅ Excellent | wlroots-based |
| **River** | ✅ Excellent | wlroots-based |
| **Hyprland** | ✅ Good | Custom implementation, fully compatible |
| **KDE Plasma** | ⚠️ Partial | Added in 5.27, some edge cases |
| **GNOME** | ❌ Not supported | Rejected protocol, no plans to implement |
| **Cosmic** | ✅ Planned | wlroots-based, should work |

**Implementation:**

```c
// Create layer surface
struct zwlr_layer_shell_v1 *layer_shell = /* bind from registry */;
struct zwlr_layer_surface_v1 *layer_surface = zwlr_layer_shell_v1_get_layer_surface(
    layer_shell,
    wl_surface,
    wl_output,           // NULL for all outputs
    ZWLR_LAYER_SHELL_V1_LAYER_OVERLAY,  // Top layer
    "spotlight-dimmer"
);

// Configure layer properties
zwlr_layer_surface_v1_set_size(layer_surface, width, height);
zwlr_layer_surface_v1_set_anchor(layer_surface, 0);  // No anchoring (free positioning)
zwlr_layer_surface_v1_set_exclusive_zone(layer_surface, -1);  // Don't affect layout
zwlr_layer_surface_v1_set_keyboard_interactivity(layer_surface, 0);  // No keyboard

// Set input region to none (click-through)
struct wl_region *empty_region = wl_compositor_create_region(compositor);
// Don't add any rectangles - empty region
wl_surface_set_input_region(wl_surface, empty_region);

// Commit
wl_surface_commit(wl_surface);
```

**Rendering with Cairo:**

```c
// Create shared memory buffer
int fd = create_shm_buffer(width * height * 4);  // ARGB32
void *data = mmap(NULL, width * height * 4, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);

struct wl_shm_pool *pool = wl_shm_create_pool(shm, fd, width * height * 4);
struct wl_buffer *buffer = wl_shm_pool_create_buffer(
    pool, 0, width, height, width * 4,
    WL_SHM_FORMAT_ARGB8888
);

// Render with Cairo
cairo_surface_t *surface = cairo_image_surface_create_for_data(
    data, CAIRO_FORMAT_ARGB32, width, height, width * 4
);
cairo_t *cr = cairo_create(surface);

// Draw semi-transparent overlay
cairo_set_source_rgba(cr, r, g, b, opacity);  // e.g., 0.0, 0.0, 0.0, 0.6
cairo_paint(cr);

cairo_destroy(cr);
cairo_surface_destroy(surface);

// Attach and commit
wl_surface_attach(wl_surface, buffer, 0, 0);
wl_surface_damage_buffer(wl_surface, 0, 0, width, height);
wl_surface_commit(wl_surface);
```

**Critical Issues:**

1. **GNOME Exclusion (40% of Wayland Users):**
   - GNOME developers explicitly rejected layer-shell protocol
   - Philosophy: "Third-party apps shouldn't create overlays"
   - No alternative protocol offered
   - **Impact:** SpotlightDimmer fundamentally incompatible with GNOME Wayland

2. **KDE Partial Support:**
   - Layer shell added relatively recently (Plasma 5.27, late 2022)
   - Some bugs with positioning and multi-monitor setups
   - Input region handling occasionally inconsistent
   - **Impact:** Requires extensive testing, may need workarounds

3. **Input Region Reliability:**
   - Protocol specifies `wl_surface_set_input_region(NULL)` should make surface click-through
   - Some compositors ignore empty regions or have bugs
   - **Impact:** Users may not be able to click through overlays

#### Protocol Option 2: XDG Shell with Positioning Extensions ⚠️ FALLBACK

**What it is:**
- Standard desktop window protocol (xdg_shell)
- Use xdg_positioner or absolute positioning hacks
- Mark as utility window type

**Why it's problematic:**
- Not designed for overlays - designed for desktop windows
- No guaranteed always-on-top behavior
- Compositors may add decorations (title bars)
- Positioning is "request" not "demand" (compositor can override)
- Full-screen apps may cover overlays

**Compositor Support:**
- ✅ Universal (all compositors implement xdg_shell)
- ⚠️ Behavior varies wildly

**Implementation Challenges:**

```c
// Create xdg surface
struct xdg_surface *xdg_surface = xdg_wm_base_get_xdg_surface(xdg_shell, wl_surface);
struct xdg_toplevel *xdg_toplevel = xdg_surface_get_toplevel(xdg_surface);

// Request utility window (may be ignored)
xdg_toplevel_set_app_id(xdg_toplevel, "spotlight-dimmer-overlay");

// Try to disable decorations (not guaranteed)
struct zxdg_toplevel_decoration_v1 *decoration = /* ... */;
zxdg_toplevel_decoration_v1_set_mode(decoration, ZXDG_TOPLEVEL_DECORATION_V1_MODE_CLIENT_SIDE);

// Position (compositor may ignore!)
// NOTE: Wayland has no "set absolute position" - only relative positioning
// This is the fundamental problem with xdg_shell for overlays
```

**Fatal Flaw:** Wayland's xdg_shell protocol **intentionally lacks absolute positioning**. The compositor decides where windows go. This makes pixel-perfect overlay alignment impossible.

**Verdict on xdg_shell:** ❌ Not viable for SpotlightDimmer's use case

#### Protocol Option 3: Subsurfaces 🔬 EXPERIMENTAL

**What it is:**
- wl_subsurface allows creating child surfaces with relative positioning
- Could attach overlay subsurfaces to focused window

**Why it's problematic:**
- Requires permission from parent window (can't overlay arbitrary windows)
- SpotlightDimmer doesn't own the focused window
- **Fatal:** Cannot attach subsurfaces to external windows

**Verdict on subsurfaces:** ❌ Architecturally incompatible

---

### Overlay Feasibility Verdict:

**Layer Shell Protocol is the ONLY viable approach.**

**Compositor Coverage:**
- ✅ **Supported (60%):** Sway, Wayfire, River, Hyprland, KDE Plasma 5.27+
- ❌ **Not Supported (40%):** GNOME (all versions), older KDE, niche compositors

**Risk Assessment:**
- **Technical:** MEDIUM - Protocol is well-designed where supported
- **Coverage:** HIGH - Excludes largest desktop environment (GNOME)
- **User Impact:** CRITICAL - 40% of Wayland users cannot use app

**Mitigation:**
- Detect compositor on startup
- Show error message: "SpotlightDimmer requires layer-shell protocol (not supported on GNOME)"
- Provide fallback recommendation: Use X11 session (but you said no X11 support!)

---

### 1.2 Event-Driven Focus Tracking - CRITICAL FEATURE ❌

**Windows Implementation:**
- `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` - 0ms latency
- `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE)` - instant window movement detection
- Zero polling, pure event-driven

**Wayland Challenge:**

Wayland's security model **intentionally prevents global window queries**. Apps cannot know:
- What windows exist
- Which window is focused
- Window positions/sizes (except their own)

This is not a bug - it's a fundamental security design decision.

#### Protocol Option 1: Foreign Toplevel Management (wlr_foreign_toplevel_management_unstable_v1) ⚠️

**What it is:**
- Protocol extension providing window list and focus state
- Designed for taskbars/docks

**Compositor Support:**

| Compositor | Support Level | API Quality |
|------------|---------------|-------------|
| **Sway** | ✅ Excellent | Full implementation |
| **Wayfire** | ✅ Good | wlroots-based |
| **River** | ✅ Good | wlroots-based |
| **Hyprland** | ✅ Excellent | Custom, very detailed |
| **KDE Plasma** | ⚠️ Partial | Limited, buggy in older versions |
| **GNOME** | ❌ Not supported | No plans to implement |

**What Information is Provided:**

```c
// Events from foreign toplevel protocol
toplevel_listener.title = (data, toplevel, title) => {
    // Window title changed
};

toplevel_listener.app_id = (data, toplevel, app_id) => {
    // Application ID (e.g., "firefox")
};

toplevel_listener.state = (data, toplevel, state) => {
    // State array - check for ZWLR_FOREIGN_TOPLEVEL_HANDLE_V1_STATE_ACTIVATED
    if (state_contains_activated) {
        // This window is focused!
    }
};

// CRITICAL LIMITATION: No geometry/position information!
// The protocol provides NO x, y, width, height data
```

**Fatal Limitation:**

The foreign toplevel protocol **does not provide window geometry**. You know:
- ✅ Which window is focused
- ✅ Window title and app ID
- ❌ Window position (x, y)
- ❌ Window size (width, height)

**Why this breaks SpotlightDimmer:**

SpotlightDimmer's core feature is positioning overlays around the focused window:
- Top overlay: Above window
- Bottom overlay: Below window
- Left/Right overlays: Beside window
- Center overlay: On window (PartialWithActive mode)

Without window position/size, **you cannot calculate overlay positions**.

#### Protocol Option 2: Hyprland IPC ✅ COMPOSITOR-SPECIFIC

**What it is:**
- Hyprland provides IPC socket with detailed window information
- Includes position, size, focus state, workspace, etc.

**Example:**

```bash
$ hyprctl activewindow -j
{
  "address": "0x5639f0d7a950",
  "at": [100, 50],
  "size": [1920, 1080],
  "workspace": {"id": 1, "name": "1"},
  "class": "firefox",
  "title": "Mozilla Firefox",
  "monitor": 0
}
```

**Implementation:**

```csharp
// Connect to Hyprland IPC socket
var socketPath = Environment.GetEnvironmentVariable("HYPRLAND_INSTANCE_SIGNATURE");
var socket = new UnixDomainSocketEndPoint($"/tmp/hypr/{socketPath}/.socket2.sock");

// Subscribe to events
socket.Send("[[BATCH]]events/activewindow,workspace");

// Event format: "activewindow>>firefox,Mozilla Firefox"
// Then query full geometry:
var response = SendCommand("j/activewindow");  // JSON output
var window = JsonSerializer.Deserialize<HyprlandWindow>(response);
// Now you have: window.at (position), window.size
```

**Coverage:** ✅ Works perfectly on Hyprland (but only Hyprland)

#### Protocol Option 3: Sway IPC ✅ COMPOSITOR-SPECIFIC

**What it is:**
- Sway provides IPC socket (similar to i3)
- Detailed window tree with positions

**Example:**

```bash
$ swaymsg -t get_tree
{
  "nodes": [
    {
      "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080},
      "focused": true,
      "name": "Firefox",
      "app_id": "firefox"
    }
  ]
}
```

**Implementation:**

```csharp
// Connect to Sway IPC socket
var socketPath = Environment.GetEnvironmentVariable("SWAYSOCK");
var socket = new UnixDomainSocketEndPoint(socketPath);

// Subscribe to window events
SendIPCMessage(2, "[\"window\"]");  // Subscribe to window events

// Event received when focus changes:
// { "change": "focus", "container": { "rect": {...}, "focused": true } }

// Or poll:
var tree = SendIPCMessage(4, "");  // GET_TREE
var focusedWindow = FindFocusedNode(tree);
// Now you have: focusedWindow.rect.x, .y, .width, .height
```

**Coverage:** ✅ Works perfectly on Sway (but only Sway/wlroots compositors with IPC)

#### Protocol Option 4: KDE KWin D-Bus API ⚠️ PARTIAL

**What it is:**
- KDE exposes window information via D-Bus

**Example:**

```bash
$ qdbus org.kde.KWin /KWin org.kde.KWin.activeWindow
# Returns window ID

$ qdbus org.kde.KWin /Windows/<id> org.kde.KWin.geometry
# Returns QRect (x, y, width, height)
```

**Coverage:** ⚠️ Works on KDE Plasma but API has changed across versions

#### Protocol Option 5: GNOME - ❌ NO SOLUTION

**Historical Context:**
- GNOME 3.x: Had `org.gnome.Shell` D-Bus interface (deprecated)
- GNOME 40-44: Limited shell extensions API
- GNOME 45+: Removed most window introspection APIs for security

**Current State:**
- ❌ No D-Bus API for window positions
- ❌ Extensions cannot access window geometry reliably
- ❌ Wayland protocol explicitly avoided

**GNOME Philosophy:**
"Applications should not be able to inspect or manipulate other applications' windows. This is a security boundary."

**Verdict:** ❌ **Fundamentally incompatible with GNOME Wayland**

---

### Focus Tracking Feasibility Verdict:

**No universal Wayland protocol provides window geometry.**

**Compositor-Specific Solutions Required:**

| Compositor | Solution | Coverage | Maintenance Burden |
|------------|----------|----------|-------------------|
| **Hyprland** | ✅ IPC socket | Excellent | Medium (IPC format may change) |
| **Sway** | ✅ IPC socket | Excellent | Low (stable i3-compatible IPC) |
| **River** | ✅ IPC (similar to Sway) | Good | Medium |
| **KDE Plasma** | ⚠️ D-Bus API | Partial | High (API changes frequently) |
| **GNOME** | ❌ No solution | None | N/A |
| **Wayfire** | ⚠️ wf-info plugin | Limited | High (plugin-dependent) |

**Risk Assessment:**
- **Technical:** CRITICAL - Requires maintaining 4-5 different implementations
- **Coverage:** 60% of Wayland users (excludes GNOME entirely)
- **User Impact:** CRITICAL - Core feature unavailable on largest DE

**Latency Comparison:**

| Platform | Method | Latency | Polling? |
|----------|--------|---------|----------|
| Windows | SetWinEventHook | 0-5ms | No |
| Hyprland IPC | Event subscription | 10-30ms | No |
| Sway IPC | Event subscription | 10-30ms | No |
| KDE D-Bus | Property monitoring | 20-50ms | Semi (PropertyChanged signals) |
| GNOME | ❌ N/A | N/A | N/A |

---

### 1.3 Multi-Monitor Support - UTILITY FEATURE ✅

**Windows Implementation:**
- `EnumDisplayMonitors()` + `GetMonitorInfo()`
- `MonitorFromWindow()` finds monitor containing window

**Wayland Equivalent:**

#### wl_output Protocol ✅ UNIVERSAL

**What it is:**
- Core Wayland protocol for output (monitor) information
- All compositors implement this

**Implementation:**

```c
// Bind to wl_output globals
registry_listener.global = (data, registry, name, interface, version) => {
    if (strcmp(interface, "wl_output") == 0) {
        struct wl_output *output = wl_registry_bind(registry, name, &wl_output_interface, 3);
        wl_output_add_listener(output, &output_listener, NULL);
    }
};

// Output events
output_listener.geometry = (data, output, x, y, physical_width, physical_height, ...) => {
    // x, y: Position in compositor space
    // physical_width/height: Physical dimensions (mm)
};

output_listener.mode = (data, output, flags, width, height, refresh) => {
    if (flags & WL_OUTPUT_MODE_CURRENT) {
        // Current resolution: width x height @ refresh Hz
    }
};

output_listener.scale = (data, output, scale_factor) => {
    // HiDPI scale factor (1, 2, 3, etc.)
};

output_listener.done = (data, output) => {
    // All output properties received, safe to use now
};
```

**XDG Output Extension (Recommended):**

```c
// Provides logical size/position (accounting for DPI scaling)
xdg_output_listener.logical_position = (data, xdg_output, x, y) => {
    // Logical position (e.g., 0, 0 for primary, 1920, 0 for second monitor)
};

xdg_output_listener.logical_size = (data, xdg_output, width, height) => {
    // Logical size (e.g., 1920x1080 even if physical is 3840x2160)
};

xdg_output_listener.name = (data, xdg_output, name) => {
    // Output name (e.g., "HDMI-A-1", "eDP-1")
};

xdg_output_listener.description = (data, xdg_output, description) => {
    // Human-readable (e.g., "Dell Inc. DELL U2720Q")
};
```

**Hot-Plug Detection:**

```c
// wl_output lifecycle
registry_listener.global = (data, registry, name, interface, version) => {
    // New output added
};

registry_listener.global_remove = (data, registry, name) => {
    // Output removed
};
```

**Finding Monitor for Window:**

**PROBLEM:** Without window geometry from foreign toplevel, you **cannot determine which monitor contains the focused window**.

**Workaround for Compositor-Specific Solutions:**
- Hyprland IPC provides `"monitor": 0` field
- Sway IPC provides monitor in window tree
- KDE D-Bus: Query window screen

**Verdict:** ✅ **FEASIBLE** - 85% confidence
- Multi-monitor enumeration is well-supported
- Hot-plug detection works universally
- **CAVEAT:** Requires window geometry to map window→monitor (see 1.2)

---

### 1.4 Configuration Hot-Reload - UTILITY FEATURE ✅

**Identical to X11 analysis - FileSystemWatcher works on Linux.**

**Verdict:** ✅ **TRIVIAL** - 100% confidence

---

### 1.5 Configuration GUI - USER INTERFACE ✅

**Windows Forms → GTK4 (Wayland-native)**

**GTK4 Advantages on Wayland:**
- Native Wayland backend (no X11 compatibility layer)
- Full touch/gesture support
- HiDPI scaling handled automatically

**Recommended Stack:**
- **GTK 4.x** (current stable)
- **libadwaita** (modern GNOME-style widgets)
- **GtkSharp bindings** (C# bindings for GTK)

**Migration Example:**

```csharp
// Windows Forms (old)
var slider = new TrackBar {
    Minimum = 0,
    Maximum = 255,
    Value = 153
};
slider.ValueChanged += (s, e) => config.InactiveOpacity = slider.Value;

// GTK4 (new)
var slider = Gtk.Scale.NewWithRange(Gtk.Orientation.Horizontal, 0, 255, 1);
slider.Value = 153;
slider.OnValueChanged += (s, e) => config.InactiveOpacity = (byte)slider.Value;
```

**Effort:** 2-3 weeks for feature parity

**Verdict:** ✅ **STRAIGHTFORWARD** - 90% confidence

---

### 1.6 System Tray - NICE-TO-HAVE ⚠️

**StatusNotifier Protocol (D-Bus-based)**

**Compositor Support:**
- ✅ KDE Plasma: Excellent
- ✅ Sway: Via waybar/swaybar
- ⚠️ GNOME: Requires extension (AppIndicator extension)
- ✅ Hyprland: Via waybar
- ⚠️ Minimal compositors: May not have tray at all

**Verdict:** ⚠️ **FEASIBLE** - 70% confidence (DE-dependent)

---

### 1.7 Auto-Start - CONVENIENCE FEATURE ✅

**Identical to X11 analysis - .desktop files work universally.**

**Verdict:** ✅ **TRIVIAL** - 95% confidence

---

## 2. Compositor Compatibility Matrix

### 2.1 Detailed Compatibility Analysis

| Compositor | Layer Shell | Window Geometry | Overall Feasibility | Market Share |
|------------|-------------|-----------------|---------------------|--------------|
| **GNOME 45+** | ❌ No | ❌ No | ❌ 0% | ~35-40% |
| **GNOME 44 and earlier** | ❌ No | ⚠️ Deprecated APIs | ⚠️ 20% | ~5-10% (declining) |
| **KDE Plasma 5.27+** | ⚠️ Partial | ⚠️ D-Bus (unstable) | ⚠️ 60% | ~25-30% |
| **KDE Plasma <5.27** | ❌ No | ⚠️ D-Bus (unstable) | ❌ 20% | ~5% (legacy) |
| **Sway** | ✅ Excellent | ✅ IPC | ✅ 95% | ~10-15% |
| **Hyprland** | ✅ Excellent | ✅ IPC | ✅ 95% | ~5-10% |
| **River** | ✅ Excellent | ✅ IPC | ✅ 90% | ~1-2% |
| **Wayfire** | ✅ Good | ⚠️ Plugin-dependent | ⚠️ 70% | ~2-3% |
| **Other wlroots** | ✅ Good | ⚠️ Varies | ⚠️ 60% | ~3-5% |

### 2.2 User Coverage Calculation

**Best Case Scenario:**
- ✅ Sway/Hyprland/River/wlroots: 20-25% of Wayland users
- ⚠️ KDE Plasma 5.27+: 25-30% (partial support)
- ❌ GNOME: 40-45% (no support)

**Realistic Coverage:** ~45-55% of Wayland users

**Pessimistic Scenario (if KDE bugs severe):** ~20-25% of Wayland users

---

## 3. Critical Blockers & Showstoppers

### 3.1 GNOME Incompatibility - SHOWSTOPPER ⚠️

**Problem:**
- GNOME is the largest Linux desktop (~40% of all Linux desktop users)
- GNOME Wayland has **no protocol** for:
  - Creating overlay windows (rejected layer-shell)
  - Querying window positions (intentional security model)
  - Monitoring focus changes with geometry

**Options:**

#### Option A: Accept GNOME Exclusion
- Show error message: "SpotlightDimmer requires layer-shell protocol (not available on GNOME)"
- **Impact:** Lose 40% of potential Linux users
- **Benefit:** Clean architecture, no hacks

#### Option B: GNOME Shell Extension ✅ UPDATED RECOMMENDATION

> **Updated Dec 2025:** A dedicated GNOME Shell Extension is the recommended approach for GNOME Wayland support. While it requires a separate JavaScript codebase, it provides full access to window geometry and overlay capabilities.

**Why This Works:**

GNOME Shell extensions run **inside** the compositor and have access to:
- `Meta.Window.get_frame_rect()` - Window position and size
- `global.display` signals - Focus change notifications
- Clutter actors - Overlay rendering with transparency
- `St.Widget` with `reactive: false` - Click-through overlays

**Key Implementation Details:**

```javascript
// extension.js (GNOME 45+ ESModules syntax)
import Meta from 'gi://Meta';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

export default class SpotlightDimmerExtension {
    enable() {
        this._focusId = global.display.connect('notify::focus-window', () => {
            const window = global.display.focus_window;
            if (window) {
                const rect = window.get_frame_rect();
                // rect.x, rect.y, rect.width, rect.height available!
                this._updateOverlays(rect);
            }
        });
    }

    _createOverlay(x, y, width, height, opacity) {
        const overlay = new St.Widget({
            style: `background-color: rgba(0, 0, 0, ${opacity});`,
            x, y, width, height,
            reactive: false,  // Click-through!
        });
        Main.layoutManager.addTopChrome(overlay);
        return overlay;
    }

    disable() {
        global.display.disconnect(this._focusId);
        // Cleanup overlays
    }
}
```

**Architecture:**

```
spotlight-dimmer-gnome/
├─ extension.js          (Main entry, focus tracking)
├─ metadata.json         (Shell versions: 45, 46, 47, 48)
├─ overlayManager.js     (Clutter/St overlay actors)
├─ configBridge.js       (Read ~/.config/SpotlightDimmer/config.json)
├─ stylesheet.css        (Optional styling)
└─ prefs.js             (Extension preferences UI)
```

**Configuration Sharing:**
- Extension reads same `config.json` as C# app
- Uses GLib.FileMonitor for hot-reload
- Shares color/opacity/mode settings

**Pros:**
- Full window geometry access
- Click-through overlays via `reactive: false`
- Uses same config file as C# app
- Can be distributed via extensions.gnome.org

**Cons:**
- Separate JavaScript codebase (not C#)
- Must update for each GNOME major version (45→46→47)
- GNOME 45 migrated to ESModules (breaking change)
- Manual installation required

**GNOME Version Compatibility:**

| Version | Status | Notes |
|---------|--------|-------|
| GNOME 45+ | ✅ Target | ESModules, `import` syntax |
| GNOME 44 | ⚠️ Limited | Legacy imports |
| GNOME 43- | ❌ Not supported | Too old |

**Feasibility:** ⚠️ **60-70%** (viable with maintenance commitment)

See `docs/GNOME_SHELL_EXTENSION.md` for detailed implementation guide.

#### Option C: Wait for GNOME to Support Layer Shell
- **Status:** GNOME developers have explicitly rejected this
- **Timeline:** Never
- **Feasibility:** ❌ 0%

**Recommendation:** **Option B** - GNOME Shell Extension (updated Dec 2025)

With X11 deprecation accelerating (GNOME 50 removes it entirely), GNOME Wayland users need a native solution. The GNOME Shell Extension approach provides this while sharing configuration with the C# Wayland client.

---

### 3.2 No Universal Window Geometry Protocol - CRITICAL ❌

**Problem:**
- Foreign toplevel provides focus state but NOT position/size
- Each compositor requires different IPC mechanism
- No fallback for unsupported compositors

**Impact:**
- Must maintain 4-5 different implementations (Sway, Hyprland, KDE, etc.)
- High maintenance burden as APIs evolve
- Code complexity increases significantly

**Mitigation:**
- Create abstraction layer: `IWindowGeometryProvider`
- Implement per-compositor:
  - `HyprlandGeometryProvider`
  - `SwayGeometryProvider`
  - `KWinGeometryProvider`
  - `WaylandPollingProvider` (fallback - poll window tree if available)

**Code Example:**

```csharp
public interface IWindowGeometryProvider
{
    event Action<WindowGeometry>? FocusedWindowChanged;
    WindowGeometry? GetFocusedWindow();
}

public record WindowGeometry(int X, int Y, int Width, int Height, int MonitorIndex);

public class CompositorDetector
{
    public static IWindowGeometryProvider CreateProvider()
    {
        var compositor = Environment.GetEnvironmentVariable("XDG_CURRENT_DESKTOP");
        var hyprlandSig = Environment.GetEnvironmentVariable("HYPRLAND_INSTANCE_SIGNATURE");
        var swaysock = Environment.GetEnvironmentVariable("SWAYSOCK");

        if (hyprlandSig != null)
            return new HyprlandGeometryProvider(hyprlandSig);
        else if (swaysock != null)
            return new SwayGeometryProvider(swaysock);
        else if (compositor?.Contains("KDE") == true)
            return new KWinGeometryProvider();
        else
            throw new PlatformNotSupportedException(
                $"Compositor '{compositor}' not supported. " +
                "SpotlightDimmer requires Hyprland, Sway, or KDE Plasma.");
    }
}
```

---

### 3.3 Input Region Reliability - MEDIUM RISK ⚠️

**Problem:**
- `wl_surface_set_input_region(NULL)` should make surface click-through
- Some compositors have bugs or don't honor this

**Testing Required:**
- Test on each compositor with different window managers
- Document which compositors have working click-through

**Fallback:**
- If click-through doesn't work, overlays will intercept clicks
- **User Impact:** Unable to interact with windows beneath overlays (broken UX)

---

## 4. Recommended Architecture (Wayland-Only)

### 4.1 Project Structure

```
SpotlightDimmer.Core/
├─ AppState.cs                  ✅ No changes
├─ OverlayDefinition.cs         ✅ No changes
├─ AppConfig.cs                 ⚠️ Minor path changes
└─ ConfigurationManager.cs      ⚠️ Minor path changes

SpotlightDimmer.Platform/       🆕 NEW - Abstraction layer
├─ IMonitorManager.cs
├─ IWindowGeometryProvider.cs   🆕 NEW - Window position tracking
├─ IOverlayRenderer.cs
├─ ISystemTrayManager.cs
└─ IAutoStartManager.cs

SpotlightDimmer.WaylandClient/  🆕 NEW - Wayland implementation
├─ WaylandBindings/
│   ├─ WaylandApi.cs            (wayland-client, layer-shell, xdg-output protocols)
│   ├─ WaylandMonitorManager.cs (wl_output enumeration)
│   ├─ WaylandOverlayRenderer.cs(layer-shell surfaces + Cairo)
│   └─ WaylandEventLoop.cs      (wl_display_dispatch)
│
├─ GeometryProviders/           🆕 NEW - Compositor-specific
│   ├─ HyprlandGeometryProvider.cs  (Unix socket IPC)
│   ├─ SwayGeometryProvider.cs      (i3 IPC protocol)
│   ├─ KWinGeometryProvider.cs      (D-Bus API)
│   └─ CompositorDetector.cs        (Auto-detect compositor)
│
├─ SystemTrayManager.cs         (StatusNotifier D-Bus)
├─ AutoStartManager.cs          (.desktop files)
└─ Program.cs                   (Compositor detection + initialization)

SpotlightDimmer.GtkConfig/      🆕 NEW - GTK4 GUI
├─ ConfigWindow.cs              (GTK4 widgets)
├─ ConfigWindow.ui              (GTK4 UI definition)
└─ Program.cs                   (Gtk.Application entry)
```

### 4.2 Compositor Detection Flow

```
1. Read environment variables:
   - XDG_CURRENT_DESKTOP
   - HYPRLAND_INSTANCE_SIGNATURE
   - SWAYSOCK
   - XDG_SESSION_TYPE (must be "wayland")

2. Detect compositor:
   - If HYPRLAND_INSTANCE_SIGNATURE → HyprlandGeometryProvider
   - Else if SWAYSOCK → SwayGeometryProvider
   - Else if XDG_CURRENT_DESKTOP contains "KDE" → KWinGeometryProvider
   - Else if XDG_CURRENT_DESKTOP contains "GNOME" → Error + exit
   - Else → Error: Unsupported compositor

3. Initialize:
   - WaylandMonitorManager (always)
   - WaylandOverlayRenderer with layer-shell (test availability)
   - Compositor-specific geometry provider
   - Wire up event handlers

4. Runtime checks:
   - Verify layer-shell protocol available
   - Test input region (click-through)
   - Show warnings if features unavailable
```

---

## 5. Implementation Roadmap (Wayland-Only)

### Phase 1: Core Wayland Infrastructure (Weeks 1-3)

**Goals:**
- Wayland connection and event loop
- wl_output enumeration
- Basic surface creation

**Deliverables:**
- [ ] Wayland P/Invoke bindings (wayland-client.h)
- [ ] Event loop integration
- [ ] Monitor enumeration with xdg-output
- [ ] Test: Print all connected monitors

**Risk:** Low
**Blockers:** None

---

### Phase 2: Layer Shell Overlays (Weeks 4-7)

**Goals:**
- Layer shell protocol implementation
- Semi-transparent RGBA rendering with Cairo
- Input region configuration (click-through)

**Deliverables:**
- [ ] Layer shell P/Invoke bindings
- [ ] WaylandOverlayRenderer implementation
- [ ] Cairo rendering (solid colors)
- [ ] Test on Sway: Red overlay on all monitors
- [ ] Test click-through functionality
- [ ] Test on KDE Plasma 5.27+ for bugs

**Risk:** Medium
**Blockers:**
- Layer shell not available (GNOME)
- Input region bugs on specific compositors

---

### Phase 3: Window Geometry - Compositor Implementations (Weeks 8-12)

**Goals:**
- Implement geometry providers for major compositors
- Auto-detection mechanism
- Fallback error handling

**Deliverables:**
- [ ] HyprlandGeometryProvider (IPC socket)
  - [ ] Event subscription (activewindow, workspace)
  - [ ] JSON parsing
  - [ ] Position + size extraction
- [ ] SwayGeometryProvider (i3 IPC)
  - [ ] IPC message protocol
  - [ ] Window tree traversal
  - [ ] Find focused node
- [ ] KWinGeometryProvider (D-Bus)
  - [ ] org.kde.KWin binding
  - [ ] activeWindow property monitoring
  - [ ] Geometry queries
- [ ] CompositorDetector
  - [ ] Environment variable detection
  - [ ] Provider factory
  - [ ] Error messages for unsupported compositors
- [ ] Test on each compositor

**Risk:** High
**Blockers:**
- KDE D-Bus API changes
- IPC format changes
- Undocumented IPC protocols

---

### Phase 4: Integration & Overlay Calculation (Weeks 13-14)

**Goals:**
- Wire geometry providers to Core AppState
- Implement all dimming modes
- Test multi-monitor scenarios

**Deliverables:**
- [ ] Connect geometry provider events to AppState.Calculate()
- [ ] Test FullScreen mode (all 3 compositors)
- [ ] Test Partial mode (4-overlay split)
- [ ] Test PartialWithActive mode (5-overlay with center)
- [ ] Multi-monitor testing (2-3 monitors)
- [ ] Configuration hot-reload integration

**Risk:** Medium
**Blockers:**
- Geometry provider bugs
- Race conditions in event handling

---

### Phase 5: GTK4 Configuration GUI (Weeks 15-17)

**Goals:**
- Replace Windows Forms with GTK4
- Feature parity with Windows config app
- Wayland-native (no XWayland)

**Deliverables:**
- [ ] GTK4 main window with libadwaita
- [ ] Profile management UI
- [ ] Mode selection (ComboBox)
- [ ] Color pickers (Gtk.ColorButton)
- [ ] Opacity sliders (Gtk.Scale)
- [ ] Real-time preview
- [ ] Configuration save/load

**Risk:** Low
**Blockers:** Learning curve for GTK4 if unfamiliar

---

### Phase 6: System Integration (Weeks 18-19)

**Goals:**
- System tray (StatusNotifier)
- Auto-start (.desktop files)
- Error handling and diagnostics

**Deliverables:**
- [ ] StatusNotifier D-Bus implementation
- [ ] Tray icon (normal/paused states)
- [ ] Context menu (Pause, Profiles, Settings, Quit)
- [ ] .desktop file auto-start manager
- [ ] Logging to ~/.local/share/SpotlightDimmer/logs/
- [ ] Compositor detection warnings

**Risk:** Low
**Blockers:** None

---

### Phase 7: Testing & Packaging (Weeks 20-22)

**Goals:**
- Extensive testing on supported compositors
- Package creation
- Documentation

**Deliverables:**
- [ ] Test matrix:
  - [ ] Sway (wlroots reference)
  - [ ] Hyprland
  - [ ] KDE Plasma 5.27, 5.28, 6.0
  - [ ] River
  - [ ] Wayfire
- [ ] Performance testing (CPU, memory, GDI equivalent)
- [ ] Multi-monitor edge cases (hot-plug, sleep/wake)
- [ ] HiDPI scaling testing
- [ ] Package creation:
  - [ ] Arch: PKGBUILD
  - [ ] Debian/Ubuntu: .deb
  - [ ] Fedora: .rpm
  - [ ] Universal: Flatpak
- [ ] Documentation:
  - [ ] README with compositor requirements
  - [ ] Installation guide per distro
  - [ ] Troubleshooting (GNOME not supported, etc.)
  - [ ] Configuration reference

**Risk:** Medium
**Blockers:**
- Access to testing environments
- Compositor-specific bugs

---

## 6. Effort Estimation (Wayland-Only)

### 6.1 Development Time Breakdown

| Component | Complexity | Estimated Hours | Weeks (40h) |
|-----------|------------|-----------------|-------------|
| **Wayland Bindings (core)** | Medium | 40 | 1.0 |
| **Monitor Manager** | Low | 24 | 0.6 |
| **Layer Shell Renderer** | High | 80 | 2.0 |
| **Hyprland Geometry Provider** | Medium | 32 | 0.8 |
| **Sway Geometry Provider** | Medium | 32 | 0.8 |
| **KDE Geometry Provider** | High | 48 | 1.2 |
| **Compositor Detection** | Low | 16 | 0.4 |
| **Integration with Core** | Medium | 32 | 0.8 |
| **GTK4 Config GUI** | Medium | 60 | 1.5 |
| **System Tray (StatusNotifier)** | Medium | 40 | 1.0 |
| **Auto-Start Manager** | Low | 8 | 0.2 |
| **Testing (per compositor)** | High | 120 | 3.0 |
| **Bug Fixes & Edge Cases** | High | 80 | 2.0 |
| **Documentation** | Medium | 24 | 0.6 |
| **Packaging** | Medium | 32 | 0.8 |
| **TOTAL** | | **668 hours** | **16.7 weeks** |

### 6.2 Team Size Scenarios

**1 Developer (Experienced with Wayland):**
- Duration: 17-22 weeks (4-5.5 months)
- Requires: C#, Wayland protocol knowledge, compositor IPC knowledge

**2 Developers:**
- Duration: 11-14 weeks (3-3.5 months)
- Dev 1: Wayland bindings + geometry providers
- Dev 2: GTK4 GUI + testing

---

## 7. Risk Assessment & Mitigation

### 7.1 Critical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **GNOME incompatibility loses 40% users** | 100% | CRITICAL | Accept limitation, document clearly in README |
| **KDE D-Bus API breaks across versions** | 60% | HIGH | Version detection, fallback to older APIs |
| **Layer shell input region bugs** | 30% | HIGH | Extensive testing, document broken compositors |
| **Hyprland/Sway IPC format changes** | 40% | MEDIUM | Versioning, parse both old and new formats |
| **Compositor not detectable** | 20% | MEDIUM | Provide manual override flag |

### 7.2 Medium Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Performance worse than Windows** | 40% | MEDIUM | Profile rendering, optimize Cairo usage |
| **HiDPI scaling issues** | 30% | MEDIUM | Use logical coordinates consistently |
| **System tray unavailable** | 25% | LOW | Fallback to CLI controls |

---

## 8. Go/No-Go Decision Criteria

### 8.1 Proceed If:

- ✅ **Acceptable to exclude GNOME (40% of users)**
- ✅ Team has Wayland expertise or 4+ month timeline for learning
- ✅ Willing to maintain 3-4 compositor-specific implementations
- ✅ Layer shell + input regions work reliably on Sway/Hyprland (verified via prototype)
- ✅ Target audience is power users (Sway/Hyprland users already technical)

### 8.2 Do NOT Proceed If:

- ❌ Require broad Linux desktop coverage (>80% users)
- ❌ GNOME support is mandatory
- ❌ Cannot maintain compositor-specific code paths
- ❌ Team lacks Wayland experience and timeline <3 months
- ❌ Layer shell unavailable or unreliable in testing

---

## 9. Alternative Approaches

### 9.1 X11 Support as Fallback

**Recommendation:** Reconsider X11 exclusion

**Rationale:**
- X11 still powers ~40% of Linux desktops (including GNOME X11 sessions)
- X11 has universal protocols (no compositor fragmentation)
- Provides fallback for GNOME users: "Run GNOME on X11 session"
- Easier to implement than Wayland (mature APIs, good docs)

**With X11 Support:**
- GNOME users: Use X11 session (option available on login screen)
- Coverage increases from 50% → 95% of Linux users

**Trade-off:**
- Additional ~3-4 weeks development
- Larger codebase (X11Bindings + WaylandBindings)
- **Benefit:** Near-universal Linux compatibility

### 9.2 GNOME Shell Extension ✅ NOW RECOMMENDED

> **Updated Dec 2025:** With X11 deprecation accelerating, GNOME Shell Extension is now the recommended approach for GNOME Wayland support.

**Why it now works:**
- GNOME Shell extensions have full access to `Meta.Window` APIs
- `global.display.connect('notify::focus-window')` provides instant focus tracking
- `St.Widget` with `reactive: false` creates click-through overlays
- Shares config.json with C# app (no duplication)

**Challenges (manageable):**
- Requires JavaScript codebase (separate from C#)
- Must update for GNOME major versions (45, 46, 47)
- Distribution via extensions.gnome.org

**Implementation:**
- See `docs/GNOME_SHELL_EXTENSION.md` for detailed architecture
- Key APIs: `Meta.Window.get_frame_rect()`, `Main.layoutManager.addTopChrome()`

**Verdict:** ✅ **Viable with maintenance commitment** (60-70% feasibility)

### 9.3 Compositor Plugin System

**For wlroots compositors (Sway, River, etc.):**
- Could write compositor plugin instead of IPC
- **Problem:** Each compositor has different plugin API
- **Effort:** Higher than IPC approach

**Verdict:** ⚠️ Not recommended (IPC is simpler)

---

## 10. Conclusion & Recommendation

### 10.1 Final Verdict

> **Updated Dec 2025:** With the GNOME Shell Extension approach, Wayland-only is now the recommended strategy.

**Wayland-only port is TECHNICALLY FEASIBLE with GNOME Shell Extension.**

**Technical Feasibility:** ✅ 75-80%
- wlr-layer-shell compositors (Sway, Hyprland, KDE): Full C# implementation
- GNOME: JavaScript extension with internal API access
- Compositor-specific geometry providers for window tracking

**User Coverage:** ✅ 85-95% of Wayland users
- Sway/Hyprland/River: 20-25% (C# native)
- KDE Plasma 5.27+: 25-30% (C# native)
- GNOME 45+: 40% (JavaScript extension)

**Maintenance Burden:** ⚠️ MEDIUM-HIGH
- 3-4 compositor-specific geometry providers (C#)
- 1 GNOME Shell extension (JavaScript, update per major version)
- Shared configuration file reduces duplication

---

### 10.2 Strong Recommendation: Add X11 Support

**Why X11 Support is Critical:**

1. **GNOME Fallback:**
   - GNOME users can use X11 session (one click on login screen)
   - Provides path forward instead of "not supported"

2. **Coverage:**
   - X11-only: 40% of Linux (legacy)
   - Wayland-only: 20-25% of Linux (GNOME excluded)
   - **X11 + Wayland: 90-95% of Linux** ✅

3. **Simpler Implementation:**
   - X11 has universal protocols (no compositor fragmentation)
   - Single implementation works everywhere
   - More documentation and examples available

4. **Risk Mitigation:**
   - If Wayland compositor support degrades, X11 remains stable
   - Provides fallback for unsupported Wayland compositors

**Recommended Strategy:**

```
Phase 1 (Months 1-3): X11 Implementation
- Proven technology, universal support
- Achieves 40% Linux coverage quickly
- Validates Core logic works on Linux

Phase 2 (Months 4-5): Wayland Implementation
- Add Wayland for modern compositors
- Increases coverage to 90-95%
- Users can choose X11 as fallback

Phase 3 (Month 6): Polish & Release
- Testing across DEs
- Documentation
- Packaging
```

**With this approach:**
- ✅ GNOME users: Use X11 session (documented)
- ✅ KDE users: Choose X11 or Wayland (both work)
- ✅ Sway/Hyprland users: Wayland (best experience)
- ✅ Legacy systems: X11 works
- ✅ Future-proof: Both protocols supported

---

### 10.3 If X11 Support is Truly Not Acceptable

**Then proceed with Wayland-only under these conditions:**

1. **Accept 50-60% Wayland user coverage** (~20-25% of all Linux users)
2. **Clearly document GNOME incompatibility:**
   ```
   # Compositor Requirements

   SpotlightDimmer requires layer-shell protocol support:

   ✅ Supported:
   - Hyprland
   - Sway
   - River
   - Wayfire
   - KDE Plasma 5.27+
   - Other wlroots-based compositors

   ❌ Not Supported:
   - GNOME (all versions) - No layer-shell support
   - KDE Plasma <5.27
   - Mutter-based DEs without layer-shell

   GNOME users: SpotlightDimmer cannot work on GNOME Wayland
   due to compositor limitations. No workaround available.
   ```

3. **Market as "Power User Tool":**
   - Target Sway/Hyprland users (already technical)
   - Not general-purpose Linux desktop tool

4. **Commit to maintaining 3-4 compositor implementations:**
   - Budget ongoing time for API changes
   - Have access to test environments

5. **Extensive Testing Phase:**
   - Build prototype in first 6 weeks
   - Validate layer shell + input regions work reliably
   - Test on all target compositors before full implementation

---

### 10.4 Final Answer to Your Question

**Can SpotlightDimmer be ported to Wayland-only (no X11)?**

**Technical Answer:** Yes, but with significant limitations.

**Practical Answer:** Not recommended without X11 fallback.

**Coverage:** ~50-60% of Wayland users (excludes GNOME entirely)

**Effort:** 4-5.5 months (single experienced developer)

**My Professional Recommendation:** Add X11 support to achieve 90-95% Linux coverage. The additional 3-4 weeks investment pays for itself in user reach and maintenance simplicity.

**If Wayland-only is a firm requirement:** Proceed with clear understanding of GNOME exclusion and commit to multi-compositor maintenance burden. Target power users on tiling compositors (Sway/Hyprland) rather than general Linux audience.

---

**Report End**

Questions? Need clarification on any technical details? I can provide:
- Detailed P/Invoke signatures for specific protocols
- Code examples for IPC implementations
- Compositor feature comparison matrices
- Testing checklists
