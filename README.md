# SpotlightDimmer

A Windows utility that creates semi-transparent overlays to dim inactive displays or regions, creating a "spotlight" effect on the active window. Built with .NET 10 and native Windows APIs for optimal performance.

## Features

✅ **Multi-monitor support** - Automatically detects all connected monitors
✅ **Configurable overlays** - Customize colors and opacity for both inactive and active regions independently
✅ **Multiple rendering backends** - Choose between:
  - **LayeredWindow**: Extremely lightweight (< 10MB RAM usage)
  - **CompositeOverlay**: Better visual quality during window dragging (~50MB RAM for dual monitor setup with partial overlays)
✅ **Small footprint** - Less than 50MB installed, installer under 10MB
✅ **Click-through overlays** - Overlays don't capture mouse input (WS_EX_TRANSPARENT)
✅ **100% event-driven** - Uses Windows event hooks instead of polling for zero CPU usage when idle
✅ **No admin privileges** - Runs as a regular user process
✅ **Hot-reloadable configuration** - Changes apply instantly without restart
✅ **Native AOT compilation** - Fast startup and minimal runtime dependencies

## How It Works

The application uses a **dual event hook system** for comprehensive window tracking:

### Event Hooks (100% Event-Driven - No Polling!)
- **EVENT_SYSTEM_FOREGROUND** - Instant detection when switching between applications
- **EVENT_OBJECT_LOCATIONCHANGE** - Real-time detection of window movement:
  - Detects windows being dragged between monitors with the mouse
  - Detects Win+Arrow and Win+Shift+Arrow keyboard shortcuts
  - Filters out cursor/caret events using `OBJID_WINDOW` check

### Window Management APIs
- **EnumDisplayMonitors** - Detects all connected monitors
- **CreateWindowEx** - Creates overlay windows with layered and transparent styles
- **SetLayeredWindowAttributes** - Sets the semi-transparent appearance with configurable opacity

## Building

### Regular Build
```bash
dotnet build
```

### Run
```bash
dotnet run
```

### AOT Build (requires Visual Studio C++ tools)
```bash
dotnet publish -c Release -r win-x64
```

## Architecture

- **WinApi.cs** - P/Invoke declarations for Windows APIs
- **MonitorManager.cs** - Multi-monitor detection and management
- **OverlayWindow.cs** - Semi-transparent, click-through overlay windows
- **FocusTracker.cs** - Event-driven focus tracking using Windows hooks
- **Program.cs** - Main application logic and message loop

## Performance Notes

- **100% event-driven** - No polling whatsoever!
- **Zero CPU usage when idle** - Only activates on actual window changes
- **Instant response** - Event hooks provide immediate notification of window changes
- **Efficient movement detection** - Tracks window position and focus changes in real-time
- Native Windows API calls for maximum performance
- AOT compilation eliminates JIT overhead and reduces startup time

## Configuration

SpotlightDimmer can be configured via JSON file located at `%AppData%\SpotlightDimmer\config.json`. See [CONFIGURATION.md](CONFIGURATION.md) for detailed configuration options.
