# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SpotlightDimmer is a Windows utility that creates semi-transparent overlays to dim inactive displays or regions, creating a "spotlight" effect on the active window. This is a .NET 10 implementation that replaces the previous Rust version, leveraging native Windows event hooks for zero-polling, event-driven architecture.

**Key Features:**
- Multi-monitor support with event-driven focus tracking
- Three dimming modes: FullScreen, Partial, and PartialWithActive
- Hot-reloadable JSON configuration at `%AppData%\SpotlightDimmer\config.json`
- Native AOT compilation support for fast startup and small binaries
- Zero-allocation hot path design for minimal GC pressure

## Build & Run Commands

### Development
```bash
# Build the project
dotnet build

# Run with standard logging
dotnet run

# Run with verbose GDI object leak detection
dotnet run -- --verbose

# Run tests (test programs are in Test*.cs files)
# Note: Tests are standalone classes with Run() methods, not NUnit/xUnit tests
# Uncomment the test class's Run() call in Program.cs to execute
```

### AOT Compilation
```bash
# Publish with Native AOT (requires Visual Studio C++ tools)
dotnet publish -c Release -r win-x64

# Output location
# bin/Release/net10.0-windows/win-x64/publish/
```

### Testing
The project uses standalone test programs rather than a traditional test framework:
- `TestOverlayCalculator.cs` - Tests AppState in-place update logic
- `TestWindowMovement.cs` - Tests window movement detection
- To run tests: Uncomment the test's `Run()` method call in `Program.cs:7-15`

## Architecture

The codebase is organized into two main layers to separate platform-agnostic logic from Windows-specific implementation:

### Core Layer (`Core/`)
Pure C# calculation logic with zero Windows dependencies. All types are platform-agnostic.

**Key Files:**
- `AppState.cs` - Central state manager with pre-allocated overlay definitions. Uses in-place updates to achieve zero allocations in the hot path.
- `OverlayDefinition.cs` - Represents a single overlay's bounds, color, opacity, and visibility. Uses `CopyFrom()` for zero-allocation updates.
- `DisplayOverlayState.cs` - Holds 6 pre-allocated OverlayDefinitions per display (one for each OverlayRegion).
- `ConfigurationManager.cs` - Loads config from JSON and watches for file changes using FileSystemWatcher.
- `AppConfig.cs` - Deserialized configuration with mode, colors, and opacity settings.
- `DimmingMode.cs` - Enum for FullScreen, Partial, and PartialWithActive modes.
- `Primitives.cs` - Basic structs (Rectangle, Color, Point) used throughout Core.

### WindowsBindings Layer (`WindowsBindings/`)
Platform-specific Windows API integration using P/Invoke.

**Key Files:**
- `WinApi.cs` - All P/Invoke declarations for Win32 APIs (CreateWindowEx, SetWinEventHook, DeferWindowPos, etc.).
- `MonitorManager.cs` - Enumerates displays using EnumDisplayMonitors.
- `FocusTracker.cs` - Event-driven focus tracking using SetWinEventHook (EVENT_SYSTEM_FOREGROUND and EVENT_OBJECT_LOCATIONCHANGE). No polling!
- `OverlayRenderer.cs` - Manages a pool of overlay windows (6 per display), uses DeferWindowPos for atomic batch updates.

### Main Program (`Program.cs`)
Wires Core and WindowsBindings together. Creates instances, connects event handlers, runs Windows message loop.

## Critical Performance Patterns

### Zero-Allocation Hot Path
The codebase is designed to eliminate allocations during window movement/focus changes:

1. **Pre-allocation**: All overlays are created at startup (6 per display).
2. **In-place updates**: `OverlayDefinition.CopyFrom()` updates existing objects instead of creating new ones.
3. **Cached config**: `Program.cs:73-76` caches `displays` and `config` to avoid re-allocating on every event.
4. **Batch updates**: `OverlayRenderer.BatchUpdateWindows()` uses DeferWindowPos to update all windows atomically.

**IMPORTANT**: When modifying update logic, ensure you:
- Never allocate in the hot path (UpdateOverlays, Calculate, event handlers)
- Use `CopyFrom()` instead of creating new OverlayDefinition instances
- Reuse pre-allocated arrays and lists (see `_updateBatch` in OverlayRenderer.cs:23)

### Memory Leak Prevention
The codebase has specific patterns to prevent GDI/handle leaks:

**DeferWindowPos Handle Management** (`OverlayRenderer.cs:105-153`):
- Always call `EndDeferWindowPos()` even on failure (line 132)
- Track `lastValidHdwp` to ensure we clean up the correct handle
- See comment at line 128-131 for rationale

**GDI Object Monitoring** (`Program.cs:136-179`):
- Verbose mode logs GDI object counts every 5 seconds
- Helps detect brush/window handle leaks during development
- Use `--verbose` flag to enable

### Event-Driven Architecture
Unlike typical polling-based solutions, SpotlightDimmer is 100% event-driven:

- `EVENT_SYSTEM_FOREGROUND` - Fires when switching applications (0ms latency)
- `EVENT_OBJECT_LOCATIONCHANGE` - Fires on window movement (drag, Win+Arrow keyboard shortcuts)
- `FileSystemWatcher` - Detects config file changes for hot-reload

**CRITICAL**: Never add polling loops. Use Windows event hooks instead.

## Configuration System

Configuration is loaded from `%AppData%\SpotlightDimmer\config.json` with hot-reload support.

**File Structure:**
```json
{
  "Mode": "FullScreen",           // FullScreen, Partial, or PartialWithActive
  "InactiveColor": "#000000",     // Hex color for inactive overlays
  "InactiveOpacity": 153,         // 0-255 (153 = 60% opacity)
  "ActiveColor": "#000000",       // Hex color for active overlay (PartialWithActive only)
  "ActiveOpacity": 102            // 0-255 (102 = 40% opacity)
}
```

**Hot-Reload Behavior:**
- `ConfigurationManager` watches for file changes using `FileSystemWatcher`
- Fires `ConfigurationChanged` event when file is modified
- `Program.cs:86-101` handles the event by updating brushes and recalculating overlays
- Changes apply instantly without restart

See `CONFIGURATION.md` for detailed configuration options and examples.

## Overlay Calculation Logic

Each display can have up to 6 overlays (one per `OverlayRegion` enum value):
- `FullScreen` - Entire display
- `Top`, `Bottom`, `Left`, `Right` - Four sides around active window
- `Center` - Active window itself (used in PartialWithActive mode)

**Mode Behaviors:**
- **FullScreen**: Inactive displays show FullScreen overlay. Focused display has no overlays.
- **Partial**: Focused display shows Top/Bottom/Left/Right overlays around active window. Inactive displays show FullScreen overlay.
- **PartialWithActive**: Like Partial, but adds Center overlay on active window for enhanced spotlight effect.

**Calculation Flow** (`AppState.cs:41-79`):
1. `AppState.Calculate()` is called when focus/position changes
2. For each display, determine if it has the focused window
3. Based on mode and focus state, call appropriate update method:
   - `UpdateFullScreenOverlay()` - Single overlay covering entire display
   - `UpdatePartialOverlays()` - Four overlays around window
   - `UpdatePartialWithActiveOverlays()` - Five overlays (four sides + center)
4. Each method calls `OverlayDefinition.Update()` to modify existing objects
5. Hidden overlays are reset via `HideAllOverlays()` before calculation

## Common Development Scenarios

### Adding a New Dimming Mode
1. Add enum value to `Core/DimmingMode.cs`
2. Add case to `AppState.Calculate()` switch statement (line 60-71)
3. Implement update logic method (follow pattern of `UpdatePartialOverlays()`)
4. Update `CONFIGURATION.md` with new mode documentation

### Modifying Overlay Rendering
1. Core logic changes go in `Core/AppState.cs`
2. Windows-specific changes go in `WindowsBindings/OverlayRenderer.cs`
3. Maintain separation: Core has no Windows dependencies
4. Ensure zero allocations in update paths

### Adding Configuration Options
1. Add property to `Core/AppConfig.cs`
2. Update `ToOverlayConfig()` method to map to `OverlayCalculationConfig`
3. Handle in `Program.cs` ConfigurationChanged event if needed
4. Document in `CONFIGURATION.md`

### Debugging Focus Tracking Issues
1. Run with `--verbose` flag to see all focus events
2. Check `FocusTracker.cs` event filtering (OBJID_WINDOW check at line 115)
3. Monitor GDI object counts to detect handle leaks
4. Use Spy++ to inspect window messages and hooks

## Platform-Specific Notes

### Windows-Only Compilation
- Project targets `net10.0-windows` (see `SpotlightDimmer.WindowsClient.csproj:5`)
- Requires `AllowUnsafeBlocks` for P/Invoke (line 8)
- Core layer is platform-agnostic but WindowsClient requires Windows

### Native AOT Constraints
- Code must be AOT-compatible (see csproj lines 12-20)
- Avoid reflection and dynamic code generation
- All P/Invoke signatures must be AOT-safe
- `InvariantGlobalization` is enabled (line 14)

### Visual Studio C++ Tools
Required for Native AOT compilation. Without them:
- Regular build/run works fine
- Publishing with AOT will fail with linker errors

## Key Win32 APIs Used

Understanding these APIs is essential for working with the WindowsBindings layer:

- **SetWinEventHook** - Registers callbacks for window events (focus, movement)
- **CreateWindowEx** - Creates overlay windows with WS_EX_LAYERED and WS_EX_TRANSPARENT
- **SetLayeredWindowAttributes** - Sets overlay opacity (60% for inactive, 40% for active)
- **DeferWindowPos/BeginDeferWindowPos/EndDeferWindowPos** - Batch window updates atomically
- **EnumDisplayMonitors** - Enumerates all connected displays
- **GetMessage/DispatchMessage** - Windows message loop in `Program.cs:154-180`

## Code Style & Patterns

- Use `ReadOnlySpan<T>` for passing arrays without allocations
- Prefer structs over classes for hot-path data (Color, Rectangle, Point)
- Use `CopyFrom()` pattern for in-place updates instead of creating new instances
- Always dispose IDisposable resources in finally blocks
- Use XML doc comments (`///`) for all public APIs
- Keep Core layer pure - no Windows dependencies

## Documentation Guidelines

**IMPORTANT**: All documentation files (*.md files other than README.md and CONFIGURATION.md in the root) should be created in the `docs/` folder unless explicitly instructed otherwise.

- ✅ Place new documentation in `docs/` directory
- ✅ Root-level README.md and CONFIGURATION.md are exceptions
- ❌ Do NOT create new .md files in the root directory
- ❌ Do NOT create documentation in `.github/` directory unless it's GitHub-specific (like PULL_REQUEST_TEMPLATE.md)

Examples:
- Setup guides → `docs/WINGET_SETUP.md`
- Architecture docs → `docs/ARCHITECTURE.md`
- Troubleshooting → `docs/TROUBLESHOOTING.md`

## Git Commit Messages

**IMPORTANT**: Do NOT include any references to Claude, Claude Code, AI tools, or co-authorship attributions in git commit messages.

- Write clear, descriptive commit messages focused on the changes made
- Use conventional commit format when appropriate (e.g., "Fix:", "Add:", "Update:")
- Keep commit messages professional and technical
- No footer lines like "Generated with Claude Code" or "Co-Authored-By: Claude"
