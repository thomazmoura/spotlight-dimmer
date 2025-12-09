# SpotlightDimmer Linux/Wayland Port Feasibility Analysis

**Project:** SpotlightDimmer
**Analysis Date:** 2025-11-30
**Last Updated:** 2025-12-09
**Target Platform:** Linux (Wayland + X11)
**Current Platform:** Windows 10/11 (.NET 10)

---

## Executive Summary

SpotlightDimmer is a Windows-native application that creates semi-transparent overlays to dim inactive displays, providing a "spotlight" effect on the active window. This report analyzes the feasibility of porting the application to Linux, supporting both Wayland and X11 display servers.

### Key Findings

| Aspect | Status | Feasibility |
|--------|--------|-------------|
| **Core Logic** | ✅ Platform-agnostic | 100% - No changes needed |
| **Semi-Transparent Overlays** | ⚠️ Requires rewrite | 85% - Technically feasible with caveats |
| **Event-Driven Focus Tracking** | ⚠️ Challenging | 70% - X11: Good, Wayland: Limited |
| **Multi-Monitor Support** | ✅ Straightforward | 90% - Well-supported on both |
| **File-Based Configuration** | ✅ Works out of box | 100% - FileSystemWatcher works on Linux |
| **Config GUI** | ❌ Complete rewrite | 90% - GTK/Qt alternative needed |
| **System Tray Integration** | ⚠️ Varies by DE | 75% - No universal standard |
| **Auto-Start Registration** | ✅ Standard mechanism | 95% - .desktop files well-supported |

### Overall Assessment

**Porting is FEASIBLE but requires significant effort.** Approximately **70-80% of the Windows-specific code** needs to be rewritten. The core overlay calculation logic (which is platform-agnostic) represents only ~5% of the codebase.

**Estimated Effort:** 3-6 months for a single experienced developer to achieve feature parity across both X11 and Wayland.

---

## 1. Core Features Feasibility Analysis

### 1.1 Semi-Transparent Overlays ⭐ CRITICAL FEATURE

**Windows Implementation:**
- Uses `WS_EX_LAYERED` + `WS_EX_TRANSPARENT` window styles
- `SetLayeredWindowAttributes()` sets opacity (0-255)
- GDI solid brushes for rendering solid colors
- `DeferWindowPos()` for atomic batch updates (flicker-free)

**X11 Equivalent:**

✅ **FEASIBLE** - Multiple approaches available

| Approach | Pros | Cons | Feasibility |
|----------|------|------|-------------|
| **XComposite + RGBA Visual** | Standard, well-supported | Requires compositor | 95% |
| **Cairo Rendering** | Modern, clean API | Slightly more complex | 90% |
| **XRender Extension** | Low-level control | More code | 85% |

**Implementation Strategy (X11):**
```c
1. Create window with RGBA visual (32-bit depth)
2. Set _NET_WM_WINDOW_TYPE to _NET_WM_WINDOW_TYPE_DOCK or _UTILITY
3. Set _NET_WM_STATE_SKIP_TASKBAR and _NET_WM_STATE_SKIP_PAGER
4. Use XComposite extension for compositing
5. Set input region to None for click-through (XShape extension)
6. Render with Cairo (solid color fills)
7. Use XFlush() for batching updates
```

**Code Example (X11 with Cairo):**
```csharp
// P/Invoke for X11
[DllImport("libX11.so.6")]
static extern IntPtr XCreateWindow(IntPtr display, IntPtr parent,
    int x, int y, uint width, uint height, ...);

[DllImport("libcairo.so.2")]
static extern IntPtr cairo_create(IntPtr surface);

// Create RGBA window
var visual = XMatchVisualInfo(display, screen, 32, TrueColor);
var window = XCreateWindow(display, root, x, y, width, height,
    0, 32, InputOutput, visual, CWBackPixel | CWBorderPixel | CWColormap);

// Set transparency via Cairo
var surface = cairo_xlib_surface_create(display, window, visual, width, height);
var cr = cairo_create(surface);
cairo_set_source_rgba(cr, r, g, b, opacity); // RGBA values
cairo_paint(cr);
```

**Wayland Equivalent:**

⚠️ **CHALLENGING** - Protocol limitations exist

| Approach | Pros | Cons | Feasibility |
|----------|------|------|-------------|
| **wl_surface + RGBA buffer** | Official protocol | No input region guarantee | 80% |
| **Layer Shell Protocol** | Designed for overlays | Not universally supported | 70% |
| **Subsurfaces** | Flexible positioning | Complex lifecycle | 75% |

**Implementation Strategy (Wayland):**
```c
1. Create wl_surface via wl_compositor
2. Attach RGBA buffer (WL_SHM_FORMAT_ARGB8888)
3. Use wl_region for input transparency
4. Set window type via xdg_shell or layer_shell
5. Render with Cairo or direct pixel manipulation
6. Use frame callbacks for vsync-aligned updates
```

**Critical Challenges:**

1. **Input Transparency Behavior Varies:**
   - X11: `XShapeCombineRectangles()` with `ShapeInput` reliably makes windows click-through
   - Wayland: `wl_surface.set_input_region(NULL)` _should_ work but compositor implementations vary
   - **Risk:** Some Wayland compositors may not honor input regions correctly

2. **No Atomic Batch Updates:**
   - Windows: `DeferWindowPos()` updates all windows atomically (zero flicker)
   - X11: `XFlush()` batches requests but not truly atomic
   - Wayland: Frame callbacks provide some atomicity but requires careful coordination
   - **Impact:** May see slight flicker during rapid window movements

3. **Topmost/Always-On-Top Behavior:**
   - Windows: `WS_EX_TOPMOST` style guarantees overlays stay on top
   - X11: `_NET_WM_STATE_ABOVE` hint (compositor may ignore)
   - Wayland: Layer shell protocol (zwlr_layer_shell_v1) provides layers but not universal
   - **Risk:** Overlays might appear below full-screen apps on some compositors

**Verdict:** ✅ **FEASIBLE** - 85% confidence
- X11: Excellent support, proven approach
- Wayland: Requires testing across compositors (GNOME, KDE, Sway, wlroots-based)
- Fallback: Disable on unsupported Wayland compositors

---

### 1.2 Event-Driven Focus Tracking ⭐ CRITICAL FEATURE

**Windows Implementation:**
- `SetWinEventHook()` with `EVENT_SYSTEM_FOREGROUND` for instant focus changes (0ms latency)
- `SetWinEventHook()` with `EVENT_OBJECT_LOCATIONCHANGE` for window movements
- 100% event-driven, zero polling (except 100ms UWP workaround)

**X11 Equivalent:**

✅ **GOOD** - Event-driven approach available

**Implementation Strategy:**
```c
1. XSelectInput() on root window with PropertyChangeMask
2. Monitor _NET_ACTIVE_WINDOW property changes (focus events)
3. XSelectInput() on focused window with StructureNotifyMask
4. Listen for ConfigureNotify events (window moves/resizes)
5. Use XGetWindowProperty() to read window bounds
```

**Code Example:**
```csharp
// Select events on root window
XSelectInput(display, root, PropertyChangeMask);

// Event loop
while (true) {
    XNextEvent(display, &event);

    if (event.type == PropertyNotify &&
        event.xproperty.atom == _NET_ACTIVE_WINDOW) {
        // Focus changed - get new active window
        var activeWindow = GetActiveWindow(display);
        UpdateFocusedDisplay(activeWindow);
    }
    else if (event.type == ConfigureNotify) {
        // Window moved/resized
        UpdateWindowPosition(event.xconfigure.window);
    }
}
```

**Latency Comparison:**
- Windows `EVENT_SYSTEM_FOREGROUND`: 0-5ms
- X11 `_NET_ACTIVE_WINDOW` property change: 10-50ms
- **Impact:** Slightly slower response but still imperceptible to users

**Wayland Equivalent:**

❌ **MAJOR LIMITATION** - No universal focus tracking protocol

| Approach | Pros | Cons | Feasibility |
|----------|------|------|-------------|
| **Compositor-Specific D-Bus** | Detailed info | Not portable | 40% |
| **Foreign Toplevel Protocol** | Standard protocol | Limited adoption | 60% |
| **Polling Fallback** | Works everywhere | CPU overhead | 80% |

**Wayland Challenges:**

1. **No _NET_ACTIVE_WINDOW Equivalent:**
   - Wayland security model prevents global window queries
   - Each compositor exposes focus differently (or not at all)

2. **Foreign Toplevel Management (ext_foreign_toplevel_list_v1):**
   - Provides window list and focus state
   - Supported: Sway, wlroots compositors
   - Not supported: GNOME 43+, older KDE versions
   - **Coverage:** ~60% of Wayland users

3. **Compositor-Specific D-Bus APIs:**
   - GNOME: `org.gnome.Shell` interface (deprecated, removed in GNOME 40+)
   - KDE: `org.kde.KWin` interface
   - Sway: No D-Bus, use `swaymsg` IPC
   - **Coverage:** Fragmented, requires per-compositor code

4. **Polling Fallback:**
   - Use `xdg_shell.configure` events for position updates
   - Poll active window via compositor IPC every 100-500ms
   - **Impact:** Higher CPU usage, slower response time

**Verdict:**
- X11: ✅ **FEASIBLE** - 90% confidence, excellent event-driven support
- Wayland: ⚠️ **LIMITED** - 60% confidence, requires compositor-specific implementations or polling
- **Recommended:** Support X11 fully, Wayland with degraded experience (polling mode)

---

### 1.3 Multi-Monitor Support ⭐ CRITICAL FEATURE

**Windows Implementation:**
- `EnumDisplayMonitors()` callback to enumerate all monitors
- `GetMonitorInfo()` retrieves bounds and work area
- `MonitorFromWindow()` finds which monitor contains a window
- Maintains HMONITOR handles for tracking

**X11 Equivalent:**

✅ **EXCELLENT** - XRandR extension provides robust multi-monitor support

**Implementation Strategy:**
```c
1. Use XRRGetScreenResources() to get output list
2. Query XRRGetOutputInfo() for each output
3. Get CRTC bounds via XRRGetCrtcInfo()
4. Monitor configuration changes via RRScreenChangeNotify events
5. Map window position to monitor by bounds intersection
```

**Code Example:**
```csharp
// Enumerate monitors
var resources = XRRGetScreenResources(display, root);
for (int i = 0; i < resources->noutput; i++) {
    var output = XRRGetOutputInfo(display, resources, resources->outputs[i]);
    if (output->connection == RR_Connected && output->crtc != None) {
        var crtc = XRRGetCrtcInfo(display, resources, output->crtc);
        // Monitor found: crtc->x, crtc->y, crtc->width, crtc->height
        monitors.Add(new Monitor(crtc->x, crtc->y, crtc->width, crtc->height));
    }
}

// Detect monitor changes
XRRSelectInput(display, root, RRScreenChangeNotifyMask);
```

**Wayland Equivalent:**

✅ **GOOD** - Well-defined protocol

**Implementation Strategy:**
```c
1. Listen for wl_registry.global events
2. Bind to wl_output interfaces as they appear
3. Subscribe to wl_output.geometry and wl_output.mode events
4. Use xdg_output extension for logical position/size (DPI scaling)
5. Handle wl_output removal for hot-plug support
```

**Code Example:**
```csharp
// Registry listener
registry_listener.global = (data, registry, name, interface, version) => {
    if (strcmp(interface, "wl_output") == 0) {
        var output = wl_registry_bind(registry, name, &wl_output_interface, 2);
        wl_output_add_listener(output, &output_listener, monitor_data);
    }
};

// Output listener
output_listener.geometry = (data, output, x, y, physical_width, physical_height, ...) => {
    // Store monitor position and size
};

output_listener.mode = (data, output, flags, width, height, refresh) => {
    if (flags & WL_OUTPUT_MODE_CURRENT) {
        // Current resolution
    }
};
```

**Challenges:**

1. **No Persistent Handles:**
   - Windows: HMONITOR stays valid until display removed
   - X11: Output IDs can change on configuration updates
   - Wayland: wl_output destroyed/recreated on changes
   - **Solution:** Track monitors by position+size+name, rebuild mapping on changes

2. **DPI Scaling:**
   - Windows: Monitor-aware DPI handled automatically
   - X11: Manual scaling needed (detect scale factor via `_NET_WM_SCALE`)
   - Wayland: Use xdg_output for logical coordinates
   - **Solution:** Use logical coordinates for overlay positioning

**Verdict:** ✅ **FEASIBLE** - 90% confidence
- Both X11 and Wayland have solid multi-monitor APIs
- Slightly more complex lifecycle management than Windows
- No major technical blockers

---

### 1.4 Configuration Hot-Reload 🔧 UTILITY FEATURE

**Windows Implementation:**
- `FileSystemWatcher` monitors `%AppData%\SpotlightDimmer\config.json`
- Fires `Changed` event when file modified
- ConfigurationManager deserializes JSON and triggers updates

**Linux Equivalent:**

✅ **WORKS OUT OF BOX** - FileSystemWatcher is cross-platform!

**Required Changes:**

1. **Config Path Update:**
   ```csharp
   // Windows
   var path = Path.Combine(
       Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
       "SpotlightDimmer", "config.json");

   // Linux (XDG Base Directory Specification)
   var configHome = Environment.GetEnvironmentVariable("XDG_CONFIG_HOME")
       ?? Path.Combine(Environment.GetEnvironmentVariable("HOME"), ".config");
   var path = Path.Combine(configHome, "SpotlightDimmer", "config.json");
   ```

2. **JSON Schema Location:**
   - Windows: Same folder as executable
   - Linux: System-wide: `/usr/share/SpotlightDimmer/config.schema.json`
   - User override: `~/.config/SpotlightDimmer/config.schema.json`

**FileSystemWatcher on Linux:**
- Uses inotify (Linux kernel feature) for efficient file monitoring
- Zero polling, event-driven
- Works identically to Windows implementation
- No code changes needed in ConfigurationManager.cs!

**Verdict:** ✅ **TRIVIAL** - 100% confidence
- Minimal changes required (just path handling)
- Core functionality works identically
- No technical challenges

---

### 1.5 Configuration GUI 🖥️ USER INTERFACE

**Windows Implementation:**
- Windows Forms (System.Windows.Forms)
- Designer-generated UI with ComboBox, NumericUpDown, TrackBar, etc.
- Two-way binding with JSON config
- ColorDialog for color pickers
- Profile management UI

**Linux Equivalent:**

❌ **COMPLETE REWRITE REQUIRED** - Windows Forms not available on Linux

**Option 1: GTK# (Recommended)**

✅ **BEST CHOICE** - Native Linux feel, well-integrated

| Aspect | Details |
|--------|---------|
| **Framework** | GTK 3/4 via GtkSharp bindings |
| **UI Toolkit** | Native widgets (GtkComboBox, GtkScale, GtkColorButton) |
| **Designer** | Glade UI designer (XML-based, like WinForms designer) |
| **Effort** | 2-3 weeks for feature parity |
| **Pros** | Native look, good documentation, active community |
| **Cons** | Different paradigm from WinForms (signal/callback based) |

**Example Migration:**
```csharp
// Windows Forms
var modeComboBox = new ComboBox();
modeComboBox.Items.AddRange(new[] { "FullScreen", "Partial", "PartialWithActive" });
modeComboBox.SelectedIndexChanged += OnModeChanged;

// GTK#
var modeComboBox = new ComboBoxText();
modeComboBox.AppendText("FullScreen");
modeComboBox.AppendText("Partial");
modeComboBox.AppendText("PartialWithActive");
modeComboBox.Changed += OnModeChanged;
```

**Option 2: Avalonia UI**

✅ **CROSS-PLATFORM** - Single codebase for Windows + Linux

| Aspect | Details |
|--------|---------|
| **Framework** | .NET XAML-based UI (like WPF) |
| **UI Toolkit** | Custom-rendered controls (consistent across platforms) |
| **Designer** | Visual Studio preview available |
| **Effort** | 3-4 weeks (learning curve if unfamiliar with XAML) |
| **Pros** | True cross-platform (Windows, Linux, macOS), modern |
| **Cons** | Not native-looking, larger binary size |

**Option 3: Qt via QtSharp**

⚠️ **LESS MATURE** - Bindings not well-maintained

| Aspect | Details |
|--------|---------|
| **Framework** | Qt 5/6 via QtSharp (community project) |
| **UI Toolkit** | Native widgets |
| **Effort** | 3-4 weeks |
| **Pros** | Excellent Qt Designer integration |
| **Cons** | QtSharp bindings are unmaintained, may have stability issues |

**Recommendation:**

For Linux-only: **GTK# (GTK 3)**
- Native integration with GNOME/Ubuntu
- Mature, stable bindings
- Good Glade UI designer

For Cross-Platform: **Avalonia UI**
- Maintain single GUI codebase
- Replace Windows Forms entirely
- Modern development experience

**Verdict:** ✅ **FEASIBLE** - 90% confidence
- Complete rewrite required but well-defined path
- Multiple viable options available
- Estimated effort: 2-4 weeks depending on toolkit choice

---

### 1.6 System Tray Integration 🔔 NICE-TO-HAVE

**Windows Implementation:**
- `Shell_NotifyIcon()` for adding tray icon
- Context menus via `CreatePopupMenu()` + `TrackPopupMenu()`
- Icon states: normal (active) and paused (dimmed)
- Menu items: Pause/Resume, Profiles, Settings, Quit

**Linux Equivalent:**

⚠️ **FRAGMENTED** - No universal standard, varies by desktop environment

**Option 1: StatusNotifier (freedesktop.org standard)**

✅ **MOST COMPATIBLE** - Supported by GNOME, KDE, XFCE

| Aspect | Details |
|--------|---------|
| **Protocol** | D-Bus-based StatusNotifierItem specification |
| **Libraries** | libappindicator (deprecated), ayatana-appindicator (modern) |
| **Coverage** | ~85% of Linux desktops |
| **Implementation** | P/Invoke to libayatana-appindicator or D-Bus direct |

**Code Example:**
```csharp
// Using libayatana-appindicator
[DllImport("libayatana-appindicator3.so.1")]
static extern IntPtr app_indicator_new(string id, string icon_name, int category);

[DllImport("libayatana-appindicator3.so.1")]
static extern void app_indicator_set_status(IntPtr indicator, int status);

// Create indicator
var indicator = app_indicator_new("spotlight-dimmer", "icon-active",
    APP_INDICATOR_CATEGORY_APPLICATION_STATUS);
var menu = gtk_menu_new();
gtk_menu_append(menu, CreateMenuItem("Pause", OnPauseClicked));
gtk_menu_append(menu, CreateMenuItem("Quit", OnQuitClicked));
app_indicator_set_menu(indicator, menu);
app_indicator_set_status(indicator, APP_INDICATOR_STATUS_ACTIVE);
```

**Option 2: Legacy System Tray (X11 only)**

⚠️ **DEPRECATED** - Still works but being phased out

| Aspect | Details |
|--------|---------|
| **Protocol** | XEmbed-based system tray protocol |
| **Coverage** | X11 only, not Wayland |
| **Status** | Deprecated in favor of StatusNotifier |

**Option 3: Desktop Environment-Specific**

❌ **NOT RECOMMENDED** - Too fragmented

- GNOME: Extensions API (requires extension installation)
- KDE: KStatusNotifierItem
- MATE/Cinnamon: Legacy tray support

**Wayland Challenges:**

- No built-in system tray in Wayland protocol
- Compositors use StatusNotifier over D-Bus
- No fallback mechanism - either supported or not

**Verdict:** ⚠️ **FEASIBLE WITH CAVEATS** - 75% confidence
- StatusNotifier works on most desktops
- May not work on minimal window managers (i3, dwm, etc.)
- Fallback: Command-line controls or GUI-only operation
- Estimated effort: 1-2 weeks

---

### 1.7 Auto-Start Registration ⚙️ CONVENIENCE FEATURE

**Windows Implementation:**
- Registry key: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- Value: "SpotlightDimmer" = quoted executable path

**Linux Equivalent:**

✅ **STANDARD MECHANISM** - .desktop files widely supported

**Implementation Strategy:**

1. **Create .desktop file at `~/.config/autostart/spotlight-dimmer.desktop`:**
   ```ini
   [Desktop Entry]
   Type=Application
   Name=SpotlightDimmer
   Exec=/usr/bin/spotlight-dimmer
   Icon=spotlight-dimmer
   Hidden=false
   NoDisplay=false
   X-GNOME-Autostart-enabled=true
   ```

2. **Code Implementation:**
   ```csharp
   public static void EnableAutoStart(string executablePath)
   {
       var autostartDir = Path.Combine(
           Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
           "autostart");
       Directory.CreateDirectory(autostartDir);

       var desktopFile = Path.Combine(autostartDir, "spotlight-dimmer.desktop");
       var content = $"""
           [Desktop Entry]
           Type=Application
           Name=SpotlightDimmer
           Exec={executablePath}
           Icon=spotlight-dimmer
           Hidden=false
           X-GNOME-Autostart-enabled=true
           """;
       File.WriteAllText(desktopFile, content);
   }

   public static void DisableAutoStart()
   {
       var desktopFile = Path.Combine(
           Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
           "autostart", "spotlight-dimmer.desktop");
       File.Delete(desktopFile);
   }
   ```

**Coverage:**
- GNOME: ✅ Full support
- KDE: ✅ Full support
- XFCE: ✅ Full support
- MATE: ✅ Full support
- Cinnamon: ✅ Full support
- Minimal WMs: ⚠️ May require manual setup

**Alternative: systemd User Units**

For advanced users or distribution packages:
```ini
# ~/.config/systemd/user/spotlight-dimmer.service
[Unit]
Description=SpotlightDimmer - Display dimming utility

[Service]
ExecStart=/usr/bin/spotlight-dimmer
Restart=on-failure

[Install]
WantedBy=default.target
```

Enable: `systemctl --user enable spotlight-dimmer.service`

**Verdict:** ✅ **TRIVIAL** - 95% confidence
- Well-established standard (.desktop files)
- Simple implementation
- Estimated effort: 1-2 days

---

## 2. Technical Deep Dive: Platform Abstraction

### 2.1 Recommended Architecture

The current architecture already separates Core (platform-agnostic) from WindowsBindings. Extend this pattern:

```
SpotlightDimmer.Core/
├─ AppState.cs                  ✅ No changes (platform-agnostic)
├─ OverlayDefinition.cs         ✅ No changes
├─ AppConfig.cs                 ⚠️ Minor path changes
└─ ConfigurationManager.cs      ⚠️ Minor path changes

SpotlightDimmer.Platform/       🆕 NEW - Platform abstraction interfaces
├─ IMonitorManager.cs
├─ IFocusTracker.cs
├─ IOverlayRenderer.cs
├─ ISystemTrayManager.cs
└─ IAutoStartManager.cs

SpotlightDimmer.WindowsClient/  ✅ Keep existing (Windows)
└─ WindowsBindings/
    ├─ WinApi.cs
    ├─ MonitorManager.cs
    ├─ FocusTracker.cs
    └─ LayeredWindowRenderer.cs

SpotlightDimmer.LinuxClient/    🆕 NEW - Linux implementation
├─ X11Bindings/
│   ├─ X11Api.cs                (XLib, XRandR, XComposite P/Invoke)
│   ├─ X11MonitorManager.cs     (XRandR-based monitor enumeration)
│   ├─ X11FocusTracker.cs       (_NET_ACTIVE_WINDOW property monitoring)
│   ├─ X11OverlayRenderer.cs    (Cairo + XComposite overlays)
│   └─ X11EventLoop.cs          (XNextEvent message loop)
│
├─ WaylandBindings/
│   ├─ WaylandApi.cs            (wayland-client protocol bindings)
│   ├─ WaylandMonitorManager.cs (wl_output-based enumeration)
│   ├─ WaylandFocusTracker.cs   (Foreign toplevel or polling)
│   ├─ WaylandOverlayRenderer.cs(wl_surface + Cairo overlays)
│   └─ WaylandEventLoop.cs      (wl_display_dispatch event loop)
│
├─ LinuxAutoStartManager.cs     (.desktop file management)
├─ LinuxSystemTrayManager.cs    (StatusNotifier implementation)
└─ Program.cs                   (Platform detection + initialization)

SpotlightDimmer.GtkConfig/      🆕 NEW - GTK-based GUI
├─ ConfigWindow.cs              (GTK# main window)
├─ ConfigWindow.glade           (Glade UI definition)
└─ Program.cs                   (GTK application entry point)
```

### 2.2 Interface Definitions

```csharp
// IMonitorManager.cs
public interface IMonitorManager
{
    DisplayInfo[] GetDisplays();
    int GetDisplayForWindow(nint windowHandle);
    event Action? DisplayConfigurationChanged;
}

// IFocusTracker.cs
public interface IFocusTracker : IDisposable
{
    event Action<int, Rectangle>? FocusedDisplayChanged;
    event Action<int, Rectangle>? WindowPositionChanged;

    int CurrentFocusedDisplayIndex { get; }
    Rectangle? CurrentWindowRect { get; }
    bool HasFocus { get; }

    void Start();
}

// IOverlayRenderer.cs
public interface IOverlayRenderer : IDisposable
{
    void CreateOverlays(DisplayInfo[] displays, OverlayCalculationConfig config);
    void UpdateOverlays(DisplayOverlayState[] states);
    void UpdateBrushColors(OverlayCalculationConfig config);
    void HideAllOverlays();
}

// ISystemTrayManager.cs
public interface ISystemTrayManager : IDisposable
{
    event Action? PauseResumeClicked;
    event Action? QuitClicked;
    event Action<string>? ProfileSelected;

    void SetPausedState(bool paused);
    void UpdateProfileList(string[] profiles, string currentProfile);
}

// IAutoStartManager.cs
public interface IAutoStartManager
{
    bool IsAutoStartEnabled();
    void SetAutoStart(bool enabled);
}
```

### 2.3 Platform Detection

```csharp
// Program.cs - Platform detection
public static class PlatformFactory
{
    public static IPlatformBindings Create()
    {
        if (OperatingSystem.IsWindows())
            return new WindowsBindings();

        if (OperatingSystem.IsLinux())
        {
            // Detect display server
            var waylandDisplay = Environment.GetEnvironmentVariable("WAYLAND_DISPLAY");
            var x11Display = Environment.GetEnvironmentVariable("DISPLAY");

            if (!string.IsNullOrEmpty(waylandDisplay))
                return new WaylandBindings();
            else if (!string.IsNullOrEmpty(x11Display))
                return new X11Bindings();
            else
                throw new PlatformNotSupportedException("No display server detected");
        }

        throw new PlatformNotSupportedException($"Unsupported OS: {Environment.OSVersion}");
    }
}

public interface IPlatformBindings
{
    IMonitorManager CreateMonitorManager();
    IFocusTracker CreateFocusTracker(IMonitorManager monitors);
    IOverlayRenderer CreateOverlayRenderer();
    ISystemTrayManager CreateSystemTrayManager();
    IAutoStartManager CreateAutoStartManager();
}
```

---

## 3. Implementation Roadmap

### Phase 1: Core Porting Foundation (Weeks 1-2)

**Goals:**
- Set up platform abstraction layer
- Implement X11 monitor enumeration
- Basic overlay rendering (solid color, no transparency yet)

**Deliverables:**
- [x] Platform interface definitions
- [x] X11MonitorManager (XRandR-based)
- [x] X11OverlayRenderer (basic XCreateWindow, no composition)
- [x] Simple test: Display red rectangles on all monitors

**Risk Level:** Low
**Blockers:** None

### Phase 2: X11 Transparency & Focus (Weeks 3-5)

**Goals:**
- Implement semi-transparent overlays with XComposite
- Event-driven focus tracking via _NET_ACTIVE_WINDOW
- Match Windows feature parity on X11

**Deliverables:**
- [x] X11OverlayRenderer with RGBA visuals + Cairo rendering
- [x] X11FocusTracker with PropertyNotify events
- [x] Overlay calculation integration
- [x] Configuration hot-reload
- [x] Test: Full feature parity on X11

**Risk Level:** Medium
**Blockers:** Compositor availability (need running compositor for transparency)

### Phase 3: Wayland Support (Weeks 6-9)

**Goals:**
- Implement Wayland overlay rendering
- Focus tracking with foreign toplevel protocol
- Fallback to polling if protocol unsupported

**Deliverables:**
- [x] WaylandOverlayRenderer with wl_surface + Cairo
- [x] WaylandFocusTracker with foreign toplevel protocol
- [x] Polling fallback for unsupported compositors
- [x] Test on GNOME, KDE, Sway

**Risk Level:** High
**Blockers:**
- Foreign toplevel protocol not universally supported
- Input region behavior varies by compositor

### Phase 4: GUI Rewrite (Weeks 10-12)

**Goals:**
- Replace Windows Forms with GTK# or Avalonia
- Feature parity with Windows config app
- Cross-platform consistency

**Deliverables:**
- [x] GTK# ConfigWindow with Glade UI
- [x] Profile management UI
- [x] Color pickers, opacity sliders
- [x] Real-time preview
- [x] Test on multiple desktop environments

**Risk Level:** Medium
**Blockers:** Learning curve for GTK# if unfamiliar

### Phase 5: System Integration (Weeks 13-14)

**Goals:**
- System tray integration
- Auto-start mechanism
- Logging and diagnostics

**Deliverables:**
- [x] StatusNotifier system tray implementation
- [x] .desktop file auto-start manager
- [x] Linux-specific logging paths
- [x] Installation scripts / packages

**Risk Level:** Low
**Blockers:** None

### Phase 6: Testing & Polish (Weeks 15-16)

**Goals:**
- Multi-distribution testing
- Performance optimization
- Bug fixes and edge cases

**Deliverables:**
- [x] Tested on Ubuntu, Fedora, Arch, openSUSE
- [x] Tested on GNOME, KDE, XFCE, i3
- [x] Memory leak testing (GDI object equivalent for X11)
- [x] Documentation updates
- [x] Package creation (deb, rpm, flatpak)

**Risk Level:** Medium
**Blockers:** Access to testing environments

---

## 4. Risk Assessment

### 4.1 High-Risk Items

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Wayland input transparency not honored** | Users can't click through overlays | 40% | Detect compositor, disable on unsupported systems |
| **Foreign toplevel protocol unavailable** | No focus tracking on Wayland | 30% | Fallback to polling (degraded experience) |
| **Compositor crashes with overlays** | App unusable | 15% | Extensive testing, error recovery |
| **Performance degradation** | Higher CPU/memory usage than Windows | 25% | Profile hot paths, optimize rendering |

### 4.2 Medium-Risk Items

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **System tray not available** | Reduced usability | 20% | CLI controls, desktop notifications |
| **DPI scaling issues** | Overlays misaligned | 30% | Use logical coordinates, test on HiDPI |
| **X11 vs Wayland feature parity gap** | Inconsistent experience | 50% | Document limitations, recommend X11 |

### 4.3 Low-Risk Items

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Config file incompatibility** | One-time migration needed | 10% | Auto-migration script |
| **Package distribution challenges** | Installation friction | 20% | Provide multiple formats |
| **GTK version conflicts** | Build issues | 15% | Target GTK 3 (universal) |

---

## 5. Effort Estimation

### 5.1 Development Time Breakdown

| Component | Complexity | Estimated Hours | Weeks (40h) |
|-----------|------------|-----------------|-------------|
| **Platform Abstraction** | Medium | 40 | 1.0 |
| **X11 Monitor Manager** | Low | 16 | 0.4 |
| **X11 Overlay Renderer** | High | 80 | 2.0 |
| **X11 Focus Tracker** | Medium | 40 | 1.0 |
| **Wayland Monitor Manager** | Medium | 24 | 0.6 |
| **Wayland Overlay Renderer** | Very High | 120 | 3.0 |
| **Wayland Focus Tracker** | Very High | 100 | 2.5 |
| **GTK# Config GUI** | Medium | 60 | 1.5 |
| **System Tray (StatusNotifier)** | Medium | 40 | 1.0 |
| **Auto-Start Manager** | Low | 8 | 0.2 |
| **Testing & Bug Fixes** | High | 80 | 2.0 |
| **Documentation** | Low | 16 | 0.4 |
| **Packaging** | Medium | 24 | 0.6 |
| **TOTAL** | | **648 hours** | **16.2 weeks** |

### 5.2 Team Size Scenarios

**1 Developer (Experienced with Linux graphics):**
- Duration: 16-20 weeks (4-5 months)
- Requires: C#, X11/Wayland knowledge, GTK#

**2 Developers (Split X11/Wayland + GUI):**
- Duration: 10-12 weeks (2.5-3 months)
- Developer 1: X11/Wayland bindings
- Developer 2: GUI, system integration

**3 Developers (Specialized):**
- Duration: 7-9 weeks (2 months)
- Developer 1: X11 bindings
- Developer 2: Wayland bindings
- Developer 3: GUI, testing, packaging

### 5.3 Skill Requirements

**Essential:**
- ✅ Proficient in C# and .NET
- ✅ Understanding of Linux graphics stack (X11/Wayland basics)
- ✅ Experience with P/Invoke or native interop
- ✅ Debugging skills for native code crashes

**Highly Beneficial:**
- ⭐ Prior X11 or Wayland development
- ⭐ Cairo rendering experience
- ⭐ GTK# or Avalonia UI experience
- ⭐ Linux packaging knowledge

**Learning Curve Items:**
- XLib API (2-3 weeks to proficiency)
- Wayland protocols (3-4 weeks to proficiency)
- GTK# (1-2 weeks if familiar with UI frameworks)

---

## 6. Alternative Approaches

### 6.1 Electron-Based GUI

**Pros:**
- Cross-platform GUI (single codebase)
- Modern web technologies (React, Vue, etc.)
- Rich UI capabilities

**Cons:**
- Large binary size (~100-200 MB)
- Higher memory usage (~80-150 MB RAM)
- Doesn't match native look-and-feel
- Requires JavaScript knowledge

**Verdict:** ❌ Not recommended for lightweight system utility

### 6.2 Wayland-First Implementation (UPDATED RECOMMENDATION)

> **Updated Dec 2025:** Given the accelerating deprecation of X11 (GNOME 50 drops it entirely), Wayland-first is now the recommended approach.

**Pros:**
- Future-proof (X11 being deprecated across major DEs)
- Modern architecture with better security model
- Focus on growing user base (~60% and increasing)

**Cons:**
- Requires compositor-specific geometry providers (Hyprland IPC, Sway IPC, KWin D-Bus)
- GNOME Wayland requires separate GNOME Shell Extension (JavaScript, not C#)

**Implementation Strategy:**
1. **wlr-layer-shell compositors** (Sway, Hyprland, River): Full native C# implementation
2. **KDE Plasma**: Native C# with KWin D-Bus for geometry
3. **GNOME Wayland**: Separate GNOME Shell Extension (JavaScript)
4. **X11 fallback**: Secondary priority for legacy systems

**Verdict:** ✅ **Now recommended** - Focus on Wayland with GNOME Shell Extension for GNOME users

See `docs/GNOME_SHELL_EXTENSION.md` for detailed GNOME extension architecture.

### 6.3 Rust Rewrite

**Pros:**
- Better performance
- Safer native interop
- Smaller binaries

**Cons:**
- Complete rewrite (6+ months)
- Lose existing Core logic
- Different ecosystem

**Verdict:** ⚠️ Consider only if C# bindings prove problematic

### 6.4 Use Existing Linux Dimming Tools

**Existing Tools:**
- **redshift**: Color temperature adjustment (not dimming)
- **xrandr**: Brightness control (affects hardware, not overlays)
- **compton/picom**: Compositor-level dimming (requires forking)

**Verdict:** ❌ None provide SpotlightDimmer's spotlight effect functionality

---

## 7. Recommendations

### 7.1 Immediate Next Steps

1. **Prototype Phase (2-3 weeks):**
   - Build minimal X11 overlay renderer
   - Test transparency + click-through on multiple compositors
   - Validate focus tracking latency
   - **Goal:** Prove core concept works acceptably

2. **Decision Gate:**
   - If prototype successful → Proceed with full implementation
   - If major blockers → Re-evaluate approach or scope

### 7.2 Recommended Technology Stack

| Component | Recommendation | Alternative |
|-----------|---------------|-------------|
| **Display Server** | X11 (primary), Wayland (secondary) | - |
| **Overlay Rendering** | Cairo + XComposite (X11), Cairo + wl_surface (Wayland) | XRender (X11 only) |
| **GUI Framework** | GTK# 3.x | Avalonia UI |
| **System Tray** | libayatana-appindicator | D-Bus StatusNotifier direct |
| **Build System** | .NET 10 SDK | - |
| **Packaging** | deb (Ubuntu/Debian), rpm (Fedora), flatpak (universal) | AppImage, snap |

### 7.3 Success Criteria

**Must Have (MVP):**
- ✅ Semi-transparent overlays with 60% opacity
- ✅ Click-through functionality
- ✅ Multi-monitor support (2-3 monitors)
- ✅ Focus tracking with <100ms latency
- ✅ Configuration file hot-reload
- ✅ Basic GTK config GUI
- ✅ Works on Ubuntu 22.04+ and Fedora 38+

**Should Have:**
- ✅ Wayland support (even if degraded)
- ✅ System tray integration
- ✅ Auto-start mechanism
- ✅ Performance parity with Windows (CPU < 1%, RAM < 50MB)
- ✅ Works on GNOME, KDE, XFCE

**Nice to Have:**
- ⭐ Flatpak distribution
- ⭐ i3/Sway support
- ⭐ Screen capture exclusion
- ⭐ Animated transitions

### 7.4 Go/No-Go Decision Factors

**Proceed if:**
- ✅ Prototype achieves <50ms focus tracking latency on X11
- ✅ Transparency + click-through works on GNOME and KDE
- ✅ Team has or can acquire X11/Wayland expertise
- ✅ 3-6 month timeline is acceptable

**Do NOT proceed if:**
- ❌ Input transparency unreliable across compositors
- ❌ Focus tracking latency >200ms
- ❌ No team members with Linux graphics experience and <2 months timeline
- ❌ Showstopper bugs in .NET Linux graphics interop

---

## 8. Conclusion

### 8.1 Final Verdict

**Porting SpotlightDimmer to Linux/Wayland is FEASIBLE** with the following caveats:

1. **X11 Support: Excellent** - All features can achieve near-parity with Windows
2. **Wayland Support: Challenging** - Some features require compositor-specific implementations or degradation
3. **Effort Required: Significant** - 3-6 months for experienced developer(s)
4. **Technical Risk: Medium** - Core rendering is proven, but Wayland fragmentation poses challenges

### 8.2 Key Takeaways

✅ **Strengths:**
- Core logic (20% of codebase) requires zero changes
- X11 has mature, well-documented APIs for all required features
- FileSystemWatcher works identically on Linux
- Auto-start and packaging are straightforward

⚠️ **Challenges:**
- 70-80% of code needs platform-specific rewrite
- Wayland lacks universal focus tracking protocol
- GUI requires complete rewrite (Windows Forms → GTK#/Avalonia)
- System tray support varies by desktop environment
- Testing across distributions and desktop environments is time-consuming

❌ **Blockers:**
- None identified - all features have viable (if imperfect) Linux equivalents

### 8.3 Recommendation

**Proceed with phased implementation:**

1. **Phase 1:** Build X11-only version with full feature parity (3-4 months)
2. **Phase 2:** Release and gather user feedback on X11
3. **Phase 3:** Add Wayland support with documented limitations (1-2 months)
4. **Phase 4:** Iterate based on real-world usage

This approach minimizes risk by proving the concept on X11 (which has the largest current user base) before tackling Wayland's fragmentation challenges.

**Expected Result:** A high-quality Linux port that works excellently on X11 and adequately on Wayland, providing 80-90% feature parity with the Windows version for most users.

---

## Appendix A: Linux Graphics Stack Primer

### X11 Architecture
```
┌─────────────────────────────────────────────┐
│ Application (SpotlightDimmer)               │
│ - Uses XLib/XCB for window management      │
│ - Uses Cairo for rendering                 │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│ X11 Display Server (Xorg)                   │
│ - Manages windows, input, displays          │
│ - Extensions: XRandR, XComposite, XRender   │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│ Compositor (Picom, Compton, or built-in)    │
│ - Handles transparency and effects          │
│ - Composites windows into final image       │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│ GPU Driver                                   │
└──────────────────────────────────────────────┘
```

**Key Concepts:**
- **XLib:** C library for X11 protocol communication
- **XComposite:** Extension allowing windows to render off-screen for compositing
- **XRandR:** Extension for display configuration and hot-plug detection
- **EWMH (_NET hints):** Window manager conventions for focus tracking, window types, etc.

### Wayland Architecture
```
┌─────────────────────────────────────────────┐
│ Application (SpotlightDimmer)               │
│ - Directly renders to buffers               │
│ - Uses EGL/Vulkan or Cairo                 │
└─────────────────┬───────────────────────────┘
                  │ wl_surface, wl_buffer
┌─────────────────▼───────────────────────────┐
│ Wayland Compositor (GNOME, KDE, Sway, etc.) │
│ - Combines display server + compositor      │
│ - Handles windows, input, displays          │
│ - Composites surfaces directly              │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│ GPU Driver (KMS/DRM)                         │
└──────────────────────────────────────────────┘
```

**Key Concepts:**
- **wl_compositor:** Core protocol for creating surfaces
- **xdg_shell:** Protocol for desktop window management
- **wl_output:** Monitor enumeration protocol
- **Layer Shell:** Extension for overlay surfaces (wlroots-based compositors)
- **Foreign Toplevel:** Extension for window list and focus tracking (not universal)

### Comparison

| Aspect | X11 | Wayland |
|--------|-----|---------|
| **Architecture** | Client-server | Direct rendering |
| **Transparency** | XComposite extension | Built-in (RGBA buffers) |
| **Focus Tracking** | EWMH (_NET_ACTIVE_WINDOW) | Compositor-specific |
| **Input Transparency** | XShape extension | wl_surface.set_input_region |
| **Multi-Monitor** | XRandR extension | wl_output + xdg_output |
| **Security Model** | Permissive (global access) | Restrictive (isolated apps) |
| **Performance** | Good (optimized over decades) | Better (direct rendering) |
| **Adoption** | ~40% of desktop Linux (declining) | ~60% and growing |

> **⚠️ Update (Dec 2025):** X11 adoption is declining faster than originally estimated. GNOME 49 defaults to Wayland-only, and GNOME 50 removes X11 support entirely. KDE Plasma 7 (approx. 5 years away) will also drop X11. For future-proofing, Wayland-first development is now recommended.

---

## Appendix B: C# Wayland Protocol Bindings (NEW)

Several .NET libraries are now available for Wayland development:

| Library | NuGet Version | Description | Status |
|---------|---------------|-------------|--------|
| [WaylandSharp](https://www.nuget.org/packages/WaylandSharp) | 0.2.1 | Source generator from protocol XML files | Active |
| [Wayland.SourceGenerator](https://www.nuget.org/packages/Wayland.SourceGenerator) | 0.0.2 | Alternative libwayland bindings generator | Active |

### WaylandSharp Usage

```xml
<!-- .csproj configuration -->
<PropertyGroup>
  <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
</PropertyGroup>

<ItemGroup>
  <PackageReference Include="WaylandSharp" Version="0.2.1" />
  <AdditionalFiles Include="protocols/wayland.xml" WaylandProtocol="client" />
  <AdditionalFiles Include="protocols/wlr-layer-shell-unstable-v1.xml" WaylandProtocol="client" />
</ItemGroup>
```

Both libraries use C# source generators to create type-safe bindings from Wayland protocol XML files, enabling native Wayland development in .NET.

---

## Appendix C: GTK4/.NET Bindings (NEW)

For the configuration GUI, modern GTK4 bindings are available:

| Library | Version | Features |
|---------|---------|----------|
| [GirCore.Gtk-4.0](https://www.nuget.org/packages/GirCore.Gtk-4.0/) | 0.6.3 | GTK 4.18, GNOME 48, .NET 9 |
| [GirCore.Adw-1](https://www.nuget.org/profiles/GirCore) | 0.6.3 | libadwaita 1.7 for modern GNOME styling |
| [Gtk4DotNet](https://www.nuget.org/packages/Gtk4DotNet/) | 7.0.4-beta | Alternative GTK4 bindings |

**GirCore** provides full GObject introspection bindings for .NET, including support for:
- GTK4 widgets
- libadwaita (modern GNOME UI components)
- GLib (file monitoring, main loop)
- Cairo (2D rendering)

### Avalonia UI Status Update

> **⚠️ Note (Dec 2025):** Avalonia UI's native Wayland support is still in preview and not production-ready. Current Avalonia apps run on Wayland via XWayland compatibility layer. For native Wayland apps, GTK4 via GirCore is recommended.

---

## Appendix D: Required Native Libraries

### X11 Dependencies

| Library | Purpose | Debian Package | Fedora Package |
|---------|---------|----------------|----------------|
| libX11 | Core X11 protocol | libx11-dev | libX11-devel |
| libXrandr | Monitor enumeration | libxrandr-dev | libXrandr-devel |
| libXcomposite | Transparency/composition | libxcomposite-dev | libXcomposite-devel |
| libXext | X11 extensions | libxext-dev | libXext-devel |
| libXfixes | Input regions | libxfixes-dev | libXfixes-devel |
| libcairo | 2D rendering | libcairo2-dev | cairo-devel |

### Wayland Dependencies

| Library | Purpose | Debian Package | Fedora Package |
|---------|---------|----------------|----------------|
| libwayland-client | Core Wayland protocol | libwayland-dev | wayland-devel |
| libwayland-egl | EGL integration | libwayland-dev | wayland-devel |
| libcairo | 2D rendering | libcairo2-dev | cairo-devel |
| wayland-protocols | Protocol definitions | wayland-protocols | wayland-protocols-devel |

### GUI Dependencies (GTK#)

| Library | Purpose | Debian Package | Fedora Package |
|---------|---------|----------------|----------------|
| GTK 3 | GUI toolkit | libgtk-3-dev | gtk3-devel |
| gtk-sharp | .NET bindings | gtk-sharp3 | gtk-sharp3 |

### System Tray Dependencies

| Library | Purpose | Debian Package | Fedora Package |
|---------|---------|----------------|----------------|
| libayatana-appindicator | StatusNotifier | libayatana-appindicator3-dev | libayatana-appindicator-gtk3-devel |

---

## Appendix E: Code Size Estimation

### Lines of Code Breakdown

| Component | Windows (Current) | Linux (Estimated) | Reuse % |
|-----------|-------------------|-------------------|---------|
| Core Logic | ~1,500 LOC | ~1,500 LOC | 100% |
| Platform Bindings | ~2,800 LOC | ~4,500 LOC | 0% |
| GUI | ~1,200 LOC | ~1,500 LOC | 0% |
| System Integration | ~400 LOC | ~600 LOC | 0% |
| Tests | ~300 LOC | ~500 LOC | 30% |
| **TOTAL** | **~6,200 LOC** | **~8,600 LOC** | **~22%** |

**Notes:**
- Linux bindings larger due to supporting both X11 and Wayland
- GUI slightly larger due to GTK# verbosity vs Windows Forms designer
- Only Core logic can be directly reused (22% of total codebase)

---

**Report End** | Questions? Contact the development team.
