# KDE Plasma Implementation Research Report

> **Status**: Research Complete | **Feasibility**: ACHIEVABLE (Medium-High Complexity)
>
> This document provides a comprehensive analysis of implementing SpotlightDimmer for KDE Plasma on Wayland, targeting Kubuntu 24.04 and Ubuntu + KDE Plasma.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Technical Feasibility Analysis](#2-technical-feasibility-analysis)
3. [Implementation Approaches](#3-implementation-approaches)
4. [Recommended Approach](#4-recommended-approach-qt6--layer-shell-qt)
5. [Phased Implementation Plan](#5-phased-implementation-plan)
6. [Technical Deep-Dives](#6-technical-deep-dives)
7. [Risk Assessment](#7-risk-assessment)
8. [Dependencies & Requirements](#8-dependencies--requirements)
9. [File Structure](#9-file-structure)
10. [Success Criteria](#10-success-criteria)
11. [Research Sources](#11-research-sources)

---

## 1. Executive Summary

SpotlightDimmer can be ported to KDE Plasma on Wayland. Unlike Windows where we have full control over window rendering, Wayland's security model requires working within compositor boundaries. Two viable approaches exist:

### Approach A (Recommended): Qt6 Application with layer-shell-qt

- Create Qt6/QML overlay windows using KDE's [layer-shell-qt](https://github.com/KDE/layer-shell-qt) library
- Use KWin scripting API + D-Bus for focus tracking
- Most similar to Windows architecture
- Supports colored overlays (like Windows/GNOME versions)

### Approach B: Native KWin Effect Plugin

- Like KDE's built-in "Dim Inactive" effect
- Compositor-level integration, best performance
- Limited to brightness/saturation manipulation (no colored overlays)
- Requires C++ and KDE development environment

### Target Platforms

| Platform | KDE Plasma Version | Notes |
|----------|-------------------|-------|
| Kubuntu 24.04 LTS | 5.27.10 | Primary target |
| Ubuntu 24.04 + KDE | 5.27.10 | Same as Kubuntu |
| Kubuntu 24.10+ | 6.x | Wayland-only by default |

---

## 2. Technical Feasibility Analysis

### What Makes This Possible

| Requirement | Windows Solution | KDE/Wayland Solution | Status |
|-------------|------------------|----------------------|--------|
| Focus tracking | `SetWinEventHook` | KWin scripting `workspace.windowActivated` + D-Bus | ✅ Available |
| Window geometry | `GetWindowRect` | KWin `EffectWindow.frameGeometry` property | ✅ Available |
| Overlay windows | `CreateWindowEx` + `WS_EX_LAYERED` | layer-shell-qt with Overlay layer | ✅ Available |
| Click-through | `WS_EX_TRANSPARENT` | `wl_surface.set_input_region(empty)` | ✅ Available |
| Topmost z-order | `WS_EX_TOPMOST` | Layer Shell Overlay layer (z=3) | ✅ Available |
| Monitor enumeration | `EnumDisplayMonitors` | QScreen / wl_output | ✅ Available |
| Hot-reload config | `FileSystemWatcher` | inotify / `QFileSystemWatcher` | ✅ Available |
| Colored overlays | GDI `FillRect` + `SetLayeredWindowAttributes` | Qt painting + alpha blending | ✅ Available |

### Key Challenges

#### 1. Wayland Security Model

Applications cannot spy on other windows or grab global input. Focus tracking must go through the compositor (KWin). This is fundamentally different from Windows where `SetWinEventHook` allows any application to monitor system-wide events.

**Solution**: Use KWin scripts (JavaScript running inside the compositor) to track focus and communicate via D-Bus.

#### 2. Click-Through on Wayland

Qt's `Qt::WindowTransparentForInput` flag does **NOT** work on Wayland. The Wayland protocol requires setting an empty input region on the surface directly.

**Solution**: Use native Wayland API via `wl_surface_set_input_region()` with an empty region.

#### 3. No Global Window Enumeration

Unlike Windows, Wayland doesn't allow applications to enumerate other windows. We can only track what the compositor tells us.

**Solution**: Rely on KWin's window management events rather than trying to query window state.

#### 4. Plasma 5 vs Plasma 6

Kubuntu 24.04 ships Plasma 5.27, but Plasma 6 has API differences.

**Solution**: Target both versions by abstracting KWin script APIs and testing on both.

---

## 3. Implementation Approaches

### Approach A: Qt6 + layer-shell-qt + KWin Script (RECOMMENDED)

```
┌──────────────────────────────────────────────────────────────┐
│                    KWin (Compositor)                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ KWin Script (JavaScript)                               │  │
│  │ - Listens to workspace.windowActivated                 │  │
│  │ - Gets EffectWindow.frameGeometry                      │  │
│  │ - Emits D-Bus signals on focus/geometry changes        │  │
│  └────────────────────┬───────────────────────────────────┘  │
└───────────────────────│──────────────────────────────────────┘
                        │ D-Bus: org.spotlight.FocusTracker
                        ▼
┌──────────────────────────────────────────────────────────────┐
│              Qt6/QML Application                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐  │
│  │ D-Bus Listener  │→ │ Core Calculator │→ │ Overlay      │  │
│  │ (Focus events)  │  │ (from Core)     │  │ Windows      │  │
│  └─────────────────┘  └─────────────────┘  └──────────────┘  │
│                                                ↓              │
│                                        layer-shell-qt        │
│                                        (Overlay layer)       │
└──────────────────────────────────────────────────────────────┘
```

**Pros:**
- Colored overlays supported (matches Windows/GNOME behavior)
- Clear separation between focus tracking (KWin) and rendering (Qt)
- Can reuse Core layer calculation logic (port to C++)
- Easier to debug and develop than C++ KWin plugins
- Works on both Plasma 5 and Plasma 6

**Cons:**
- Two components to install (KWin script + Qt app)
- D-Bus latency (minimal, <1ms)
- Slightly more complex deployment

### Approach B: Native KWin Effect Plugin

```
┌──────────────────────────────────────────────────────────────┐
│                    KWin (Compositor)                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ SpotlightDimmer Effect (C++ Plugin)                    │  │
│  │ - Inherits from KWin::Effect                           │  │
│  │ - Connects to windowActivated signal                   │  │
│  │ - paintWindow() applies shader to inactive windows     │  │
│  │ - Draws overlay rectangles via OpenGL                  │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**Pros:**
- Single component (just the KWin plugin)
- Best possible performance (compositor-level)
- Native integration with KDE settings
- No D-Bus communication needed

**Cons:**
- C++ development required
- Must be compiled for each KDE/Qt version
- Harder to do colored overlays (would need custom shaders)
- Different API between Plasma 5 and Plasma 6
- Requires KDE development dependencies

### Approach C: Pure KWin Script (Scripted Effect)

KWin supports "scripted effects" using QML/JavaScript. This would be the simplest but most limited approach.

**Verdict**: ❌ Not viable - scripted effects can only do simple animations, not custom overlay rendering.

---

## 4. Recommended Approach: Qt6 + layer-shell-qt

### Why This Approach

1. **Feature Parity**: Colored overlays match Windows/GNOME versions
2. **Code Reuse**: Core calculation logic can be ported from C#
3. **Maintainability**: Separate components with clear responsibilities
4. **Compatibility**: Works across Plasma 5.27 and Plasma 6.x
5. **Proven Pattern**: Similar to existing GNOME extension architecture

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Focus Tracker | KWin Script (JavaScript) | Detect window focus/geometry changes |
| IPC | D-Bus | Communicate focus events to main app |
| Overlay Renderer | Qt6/QML + layer-shell-qt | Create transparent overlay windows |
| Configuration | JSON + QFileSystemWatcher | Hot-reload configuration |
| System Tray | Qt System Tray API | User interface for control |

### Programming Language Decision

**Research on C#/.NET Options:**

| Option | Verdict | Reason |
|--------|---------|--------|
| Qt6 C# Bindings ([QtSharp](https://github.com/ddobrev/QtSharp)) | ❌ Abandoned | Only supports Qt5, no Qt6 bindings exist |
| [Qml.Net](https://github.com/qmlnet/qmlnet) | ❌ Insufficient | QML-only integration, no full Qt widget access |
| [Qt/.NET](https://github.com/qt-labs/qtdotnet) (Official) | ❌ Wrong direction | Only allows calling .NET FROM C++, not reverse |
| [Avalonia UI](https://avaloniaui.net/) | ❌ No layer-shell | Wayland in "private preview", no layer-shell support, click-through not possible |
| .NET MAUI | ❌ Poor Linux support | No native Wayland APIs |

**Conclusion**: C++ with Qt6 is the only viable option for native KDE/Wayland layer-shell integration.

---

## 5. Phased Implementation Plan

### Phase 1: Environment Setup & Proof of Concept

**Goal**: Verify that all required APIs work on Kubuntu 24.04

**Tasks**:
1. Set up Kubuntu 24.04 development environment
2. Install dependencies: Qt6, layer-shell-qt, KWin development packages
3. Create minimal KWin script that logs window focus changes
4. Create minimal Qt6 app that creates a layer-shell overlay window
5. Verify click-through works with empty input region
6. Test D-Bus communication between KWin script and Qt app

**Key Files to Create**:
- `SpotlightDimmer.KDEScript/` (KWin script package)
- `SpotlightDimmer.KDEClient/` (Qt6 application)

**Critical Verification Points**:
- [ ] KWin script receives `workspace.windowActivated` signal
- [ ] KWin script can access `EffectWindow.frameGeometry`
- [ ] D-Bus message arrives in Qt app within <5ms
- [ ] layer-shell-qt overlay appears above all windows
- [ ] Click-through works (mouse events pass to windows below)

### Phase 2: Focus Tracking Implementation

**Goal**: Reliable focus and geometry tracking via KWin script

**KWin Script Features**:
1. `workspace.windowActivated` signal connection
2. Track window geometry changes (position, size)
3. Handle window maximize/fullscreen transitions
4. Detect monitor for focused window
5. D-Bus interface: `org.spotlight.FocusTracker`
   - Signal: `FocusChanged(windowId, x, y, width, height, monitorIndex)`
   - Signal: `GeometryChanged(windowId, x, y, width, height, monitorIndex)`

**Reference**: Existing KWin scripts like [FocusNotifier](https://github.com/c-massie/FocusNotifier)

### Phase 3: Overlay Rendering Implementation

**Goal**: Create and manage overlay windows using layer-shell-qt

**Qt Application Features**:
1. Initialize layer-shell-qt before creating windows
2. Create overlay windows (6 per monitor, matching Windows architecture)
3. Set Layer::Overlay for z-order above all windows
4. Configure anchors for full-screen coverage
5. Set empty input region for click-through
6. Render solid color rectangles with configurable opacity

**layer-shell-qt Configuration**:
```cpp
// Before creating any windows
LayerShellQt::Shell::useLayerShell();

// For each overlay window
auto *lsWindow = LayerShellQt::Window::get(window);
lsWindow->setLayer(LayerShellQt::Window::LayerOverlay);
lsWindow->setKeyboardInteractivity(LayerShellQt::Window::KeyboardInteractivityNone);
lsWindow->setExclusiveZone(-1);  // Don't reserve space
lsWindow->setAnchors(LayerShellQt::Window::AnchorTop |
                     LayerShellQt::Window::AnchorBottom |
                     LayerShellQt::Window::AnchorLeft |
                     LayerShellQt::Window::AnchorRight);

// Critical: Set empty input region for click-through
// This must be done via Wayland native API
```

### Phase 4: Core Logic Integration

**Goal**: Port overlay calculation logic from SpotlightDimmer.Core to C++

**Files to Port** (~1,400 lines):
| C# File | C++ Equivalent | Purpose |
|---------|----------------|---------|
| `Primitives.cs` | `primitives.h/cpp` | Rectangle, Color structs |
| `DimmingMode.cs` | `dimmingmode.h` | Enum for modes |
| `OverlayDefinition.cs` | `overlaydefinition.h/cpp` | Overlay data structure |
| `DisplayOverlayState.cs` | `displayoverlaystate.h/cpp` | Per-display state |
| `AppState.cs` | `appstate.h/cpp` | Core calculation logic |
| `AppConfig.cs` | `appconfig.h/cpp` | Configuration model |
| `ConfigurationManager.cs` | `configurationmanager.h/cpp` | Config loading/watching |

**Why Porting is Straightforward**:
- Core layer has zero platform dependencies
- Pure calculation logic (no Windows APIs)
- Well-documented algorithm in existing code
- Same JSON schema for configuration

### Phase 5: Configuration & System Integration

**Goal**: Configuration file compatibility and system tray

**Tasks**:
1. Read config from `~/.config/SpotlightDimmer/config.json`
2. Implement QFileSystemWatcher for hot-reload
3. Create system tray icon with context menu
4. Auto-start via XDG autostart desktop file

**Configuration Path**:
- Primary: `$XDG_CONFIG_HOME/SpotlightDimmer/config.json`
- Fallback: `~/.config/SpotlightDimmer/config.json`

**Same JSON Schema** as Windows/GNOME versions:
```json
{
  "Overlay": {
    "Mode": "FullScreen",
    "InactiveColor": "#000000",
    "InactiveOpacity": 153,
    "ActiveColor": "#000000",
    "ActiveOpacity": 102
  }
}
```

### Phase 6: Multi-Monitor Support

**Goal**: Proper handling of multiple displays

**Tasks**:
1. Enumerate monitors via QScreen or Wayland wl_output
2. Create overlay set (6 windows) per monitor
3. Handle monitor hotplug (add/remove)
4. Map window position to correct monitor

**Wayland Monitor Detection**:
```cpp
// Qt approach
QList<QScreen*> screens = QGuiApplication::screens();
for (QScreen *screen : screens) {
    QRect geometry = screen->geometry();
    // Create overlays for this screen
}

// Connect to screen changes
connect(qApp, &QGuiApplication::screenAdded, this, &App::onScreenAdded);
connect(qApp, &QGuiApplication::screenRemoved, this, &App::onScreenRemoved);
```

### Phase 7: Packaging & Distribution

**Goal**: Easy installation for end users

**KWin Script Package** (for KDE Store):
```
spotlight-dimmer-kwin/
├── contents/
│   └── code/
│       └── main.js
├── metadata.json
└── install.sh
```

Installation: `kpackagetool5 --type KWin/Script --install spotlight-dimmer-kwin/`

**Qt Application Package Options**:

| Format | Pros | Cons |
|--------|------|------|
| AppImage | Universal, portable | Larger size, no auto-updates |
| Flatpak | Sandboxed, auto-updates | May have layer-shell issues |
| DEB | Native Ubuntu integration | Ubuntu/Debian only |
| RPM | Native Fedora integration | Fedora/RHEL only |

---

## 6. Technical Deep-Dives

### 6.1 KWin Scripting API Reference

**Key Objects**:
- `workspace` - Main entry point for window management
- `EffectWindow` - Represents a window with geometry properties
- `workspace.activeWindow` - Currently focused window

**Key Signals**:
```javascript
// Focus change detection
workspace.windowActivated.connect(function(window) {
    if (window) {
        var geometry = window.frameGeometry;
        var screenIndex = window.screen;

        // Send to Qt app via D-Bus
        callDBus("org.spotlight.Dimmer", "/FocusTracker",
                 "org.spotlight.FocusTracker", "FocusChanged",
                 window.resourceClass,
                 geometry.x, geometry.y,
                 geometry.width, geometry.height,
                 screenIndex);
    }
});

// Window geometry changes (for tracking movement)
// Connect to individual window's frameGeometryChanged signal
```

**D-Bus Communication from KWin Script**:
```javascript
// Available in KWin scripts
callDBus(service, path, interface, method, ...args);
```

### 6.2 layer-shell-qt API Reference

**Initialization** (must call before any QWindow creation):
```cpp
#include <LayerShellQt/Shell>
#include <LayerShellQt/Window>

int main(int argc, char *argv[]) {
    // CRITICAL: Call before QGuiApplication
    LayerShellQt::Shell::useLayerShell();

    QGuiApplication app(argc, argv);
    // ... rest of application
}
```

**Window Configuration**:
```cpp
#include <LayerShellQt/Window>

void configureOverlayWindow(QWindow *window) {
    auto *lsWindow = LayerShellQt::Window::get(window);

    // Set to overlay layer (highest z-order)
    lsWindow->setLayer(LayerShellQt::Window::LayerOverlay);

    // Anchor to all edges (fullscreen coverage)
    lsWindow->setAnchors(
        LayerShellQt::Window::AnchorTop |
        LayerShellQt::Window::AnchorBottom |
        LayerShellQt::Window::AnchorLeft |
        LayerShellQt::Window::AnchorRight
    );

    // Don't reserve exclusive space
    lsWindow->setExclusiveZone(-1);

    // Disable keyboard focus
    lsWindow->setKeyboardInteractivity(
        LayerShellQt::Window::KeyboardInteractivityNone
    );
}
```

**Click-Through Implementation** (Native Wayland):
```cpp
#include <private/qwaylandwindow_p.h>
#include <wayland-client.h>

void setClickThrough(QWindow *window, wl_compositor *compositor) {
    // Get native Wayland window
    auto *waylandWindow = dynamic_cast<QtWaylandClient::QWaylandWindow*>(
        window->handle()
    );

    if (waylandWindow) {
        wl_surface *surface = waylandWindow->surface();

        // Create empty input region
        wl_region *emptyRegion = wl_compositor_create_region(compositor);

        // Set empty input region (makes surface click-through)
        wl_surface_set_input_region(surface, emptyRegion);

        // Cleanup
        wl_region_destroy(emptyRegion);
        wl_surface_commit(surface);
    }
}
```

### 6.3 Layer Shell Protocol Layers

The [wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) protocol defines four layers:

| Layer | Z-Order | Typical Use |
|-------|---------|-------------|
| Background (0) | Lowest | Desktop wallpaper |
| Bottom (1) | Below windows | Desktop widgets |
| Top (2) | Above windows | Panels, docks |
| **Overlay (3)** | **Highest** | **Overlays, notifications** |

SpotlightDimmer should use the **Overlay** layer to appear above all application windows.

### 6.4 KDE Dim Inactive Effect Reference

The built-in KDE "Dim Inactive" effect (`src/plugins/diminactive/`) provides useful patterns:

**Focus Detection**:
```cpp
// In constructor
connect(effects, &EffectsHandler::windowActivated,
        this, &DimInactiveEffect::windowActivated);
```

**Dimming Algorithm**:
```cpp
void DimInactiveEffect::dimWindow(EffectWindow *w, qreal strength) {
    qreal dimFactor = 1.0 - strength;

    // Apply to brightness and saturation
    w->setBrightness(dimFactor);
    w->setSaturation(dimFactor);
}
```

**Window Filtering**:
- Excludes active window and its group
- Excludes docks, desktops, popups (configurable)
- Excludes fullscreen windows

---

## 7. Risk Assessment

### High Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| layer-shell-qt not available on Plasma 5.27 | Cannot create overlays | Verify availability in Phase 1; fallback to X11 session if needed |
| Click-through doesn't work | Overlays block interaction | Test early in Phase 1; investigate alternative approaches if needed |
| D-Bus latency too high | Visible lag on focus change | Measure latency; optimize message size; consider shared memory if needed |

### Medium Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Plasma 5 vs 6 API differences | Requires two codebases | Abstract KWin script to support both; test on both versions |
| Multi-monitor edge cases | Overlays on wrong monitor | Thorough testing; handle hotplug events |
| Fullscreen apps interfere | Overlays visible when shouldn't be | Detect fullscreen via KWin; hide overlays appropriately |

### Low Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Configuration file incompatibility | Users must reconfigure | Use same JSON schema; document differences |
| System tray not visible | Users can't control app | Fallback to CLI commands; config file control |

---

## 8. Dependencies & Requirements

### Build Dependencies (Ubuntu/Kubuntu 24.04)

```bash
# Qt6 development
sudo apt install qt6-base-dev qt6-declarative-dev

# layer-shell-qt (check availability)
sudo apt install layer-shell-qt-dev
# If not available, build from source:
# git clone https://invent.kde.org/plasma/layer-shell-qt.git

# KWin development (for script packaging)
sudo apt install kwin-dev

# D-Bus development
sudo apt install libdbus-1-dev

# Build tools
sudo apt install cmake g++ extra-cmake-modules
```

### Runtime Dependencies

```bash
# Core Qt6 runtime
sudo apt install qt6-qpa-plugins

# Wayland integration
sudo apt install qtwayland6

# KDE Plasma Desktop (includes layer-shell-qt)
# Already installed on Kubuntu
```

### Minimum Versions

| Dependency | Minimum Version | Notes |
|------------|-----------------|-------|
| Qt | 6.2 | For layer-shell-qt compatibility |
| KDE Plasma | 5.27 | Kubuntu 24.04 default |
| KWin | 5.27 | Comes with Plasma |
| layer-shell-qt | 5.27 | Comes with Plasma |

---

## 9. File Structure

```
spotlight-dimmer/
├── SpotlightDimmer.Core/                 # Existing (C#, portable)
├── SpotlightDimmer.WindowsClient/        # Existing (Windows)
├── SpotlightDimmer.GnomeShellExtension/  # Existing (GNOME)
│
├── SpotlightDimmer.KDEScript/            # NEW: KWin script
│   ├── contents/
│   │   └── code/
│   │       └── main.js                   # Focus tracking script
│   ├── metadata.json                     # KDE package metadata
│   └── install.sh                        # Installation helper
│
├── SpotlightDimmer.KDEClient/            # NEW: Qt6 application
│   ├── CMakeLists.txt                    # CMake build config
│   ├── src/
│   │   ├── main.cpp                      # Application entry point
│   │   ├── focustracker.h/cpp            # D-Bus listener
│   │   ├── overlaymanager.h/cpp          # layer-shell overlays
│   │   ├── appstate.h/cpp                # Ported from Core
│   │   ├── overlaydefinition.h/cpp       # Ported from Core
│   │   ├── primitives.h                  # Rectangle, Color
│   │   ├── dimmingmode.h                 # DimmingMode enum
│   │   ├── configmanager.h/cpp           # JSON config
│   │   └── systemtray.h/cpp              # System tray icon
│   ├── qml/
│   │   └── Overlay.qml                   # Overlay window template
│   └── resources/
│       └── spotlight-dimmer.png          # Tray icon
│
└── SpotlightDimmer.Documentation/        # NEW: This document
    └── KDE-PLASMA-IMPLEMENTATION.md
```

---

## 10. Success Criteria

The KDE Plasma implementation is complete when:

| Criterion | Description | Test Method |
|-----------|-------------|-------------|
| **Focus Tracking** | Window focus changes detected within <5ms | Measure D-Bus latency |
| **Overlay Rendering** | Colored overlays appear correctly on all monitors | Visual inspection |
| **Click-Through** | Mouse/keyboard events pass through overlays | Click test on windows |
| **Multi-Monitor** | Correct behavior across 2+ monitors | Test with 2-3 displays |
| **Dimming Modes** | All three modes work (FullScreen, Partial, PartialWithActive) | Test each mode |
| **Configuration** | Hot-reload from config.json works | Modify config, verify change |
| **System Tray** | Users can pause/resume and switch profiles | UI testing |
| **Kubuntu 24.04** | Tested and working on target platform | End-to-end testing |
| **Plasma 5/6** | Works on both Plasma 5.27 and Plasma 6.x | Test on both versions |

---

## 11. Research Sources

### KWin Scripting & Focus Tracking
- [KWin Scripting API](https://develop.kde.org/docs/plasma/kwin/api/) - Official KDE developer documentation
- [KWin Scripting Tutorial](https://develop.kde.org/docs/plasma/kwin/) - Getting started guide
- [FocusNotifier](https://github.com/c-massie/FocusNotifier) - Example KWin script for focus tracking via D-Bus
- [window_signal](https://github.com/bouteillerAlan/window_signal) - KWin script that emits signals on focus changes

### Layer Shell & Wayland Overlays
- [layer-shell-qt](https://github.com/KDE/layer-shell-qt) - KDE Qt component for wlr-layer-shell protocol
- [wlr-layer-shell protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) - Protocol specification
- [gtk-layer-shell](https://github.com/wmww/gtk-layer-shell) - GTK equivalent (for reference)
- [Qt Forum: layer-shell-qt](https://discuss.kde.org/t/best-way-to-use-layer-shell-qt/9185) - Community discussion

### KDE Dim Inactive Effect (Reference Implementation)
- [Dim Inactive Effect Rewrite](https://phabricator.kde.org/D13720) - KDE review for the rewritten effect
- [KWin GitHub](https://github.com/KDE/kwin) - Source code for KWin compositor and effects

### C#/.NET Research (NOT VIABLE)
- [QtSharp](https://github.com/ddobrev/QtSharp) - Abandoned Qt5-only C# bindings
- [Qml.Net](https://github.com/qmlnet/qmlnet) - QML-only .NET integration
- [Qt/.NET](https://github.com/qt-labs/qtdotnet) - Official Qt Labs .NET integration (C++→.NET only)
- [Avalonia Wayland Support](https://avaloniaui.net/blog/bringing-wayland-support-to-avalonia) - In private preview
- [Avalonia Click-Through Discussion](https://github.com/AvaloniaUI/Avalonia/discussions/13827) - Not possible on Wayland

### General KDE/Plasma
- [KDE Wayland Future](https://blogs.kde.org/2025/11/26/going-all-in-on-a-wayland-future/) - KDE's Wayland direction
- [Wayland and Qt](https://doc.qt.io/qt-6/wayland-and-qt.html) - Official Qt documentation
- [KWin Effects Development](https://develop.kde.org/docs/plasma/kwineffect/) - Creating KWin effects

---

## Appendix A: Comparison with Existing Implementations

### Windows Client vs KDE Client

| Aspect | Windows (C#) | KDE (C++) |
|--------|-------------|-----------|
| **Focus Tracking** | `SetWinEventHook` + `EVENT_SYSTEM_FOREGROUND` | KWin Script + `workspace.windowActivated` |
| **Geometry Tracking** | `EVENT_OBJECT_LOCATIONCHANGE` | `window.frameGeometryChanged` signal |
| **Overlay Windows** | Win32 `CreateWindowEx` (6/display) | layer-shell-qt `QWindow` (6/display) |
| **Click-Through** | `WS_EX_TRANSPARENT` | `wl_surface_set_input_region(empty)` |
| **Topmost** | `WS_EX_TOPMOST` | Layer Shell Overlay layer |
| **Configuration** | `%AppData%\SpotlightDimmer\config.json` | `~/.config/SpotlightDimmer/config.json` |
| **Hot-Reload** | `FileSystemWatcher` | `QFileSystemWatcher` / inotify |
| **System Tray** | Win32 `Shell_NotifyIcon` | Qt System Tray API |

### GNOME Extension vs KDE Client

| Aspect | GNOME (JavaScript) | KDE (C++) |
|--------|-------------------|-----------|
| **Focus Tracking** | `global.display.notify::focus-window` | KWin Script D-Bus |
| **Overlay Windows** | GNOME Shell `St.Widget` | layer-shell-qt `QWindow` |
| **Calculation Logic** | `calculator.js` (JS port) | `appstate.cpp` (C++ port) |
| **Configuration** | `Gio.FileMonitor` | `QFileSystemWatcher` |
| **Integration** | Native Shell extension | KWin Script + Qt app |

---

## Appendix B: Quick Start Guide (For Developers)

### Setting Up Development Environment

```bash
# 1. Install Kubuntu 24.04 (or Ubuntu 24.04 + KDE)

# 2. Install development dependencies
sudo apt update
sudo apt install -y \
    qt6-base-dev \
    qt6-declarative-dev \
    cmake \
    g++ \
    extra-cmake-modules \
    kwin-dev

# 3. Clone the repository
git clone https://github.com/your-repo/spotlight-dimmer.git
cd spotlight-dimmer

# 4. Create KDE-specific directories
mkdir -p SpotlightDimmer.KDEScript/contents/code
mkdir -p SpotlightDimmer.KDEClient/src

# 5. Start with Phase 1: Proof of Concept
# See Phase 1 tasks in this document
```

### Testing KWin Script

```bash
# Install the script
kpackagetool5 --type KWin/Script --install SpotlightDimmer.KDEScript/

# Reload KWin to activate
qdbus org.kde.KWin /KWin reconfigure

# View logs
journalctl -f | grep -i spotlight
```

### Testing Qt Application

```bash
# Build
cd SpotlightDimmer.KDEClient
mkdir build && cd build
cmake ..
make

# Run
./spotlight-dimmer-kde
```
