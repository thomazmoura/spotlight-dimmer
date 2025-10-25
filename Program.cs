using SpotlightDimmer.Core;
using SpotlightDimmer.WindowsBindings;

// Parse command-line arguments
bool verboseLogging = args.Contains("--verbose");

Console.WriteLine("SpotlightDimmer .NET - Refactored Architecture");
Console.WriteLine("===============================================");
Console.WriteLine("Core: Pure overlay calculation logic");
Console.WriteLine("WindowsBindings: Windows-specific rendering");
if (verboseLogging)
{
    Console.WriteLine("Verbose logging: ENABLED");
}
Console.WriteLine("\nPress Ctrl+C to exit\n");

// Set up graceful shutdown
using var cts = new CancellationTokenSource();
var mainThreadId = WinApi.GetCurrentThreadId();

Console.CancelKeyPress += (sender, e) =>
{
    e.Cancel = true;
    cts.Cancel();
    Console.WriteLine("\nShutting down...");

    // Post a quit message to the main thread's message queue
    // This immediately unblocks GetMessage() which runs on the main thread
    WinApi.PostThreadMessage(mainThreadId, WinApi.WM_QUIT, IntPtr.Zero, IntPtr.Zero);
};

// ========================================================================
// WindowsBindings Layer - Platform-specific components
// ========================================================================

// System Tray
var systemTray = new SystemTrayManager(
    "spotlight-dimmer-icon.ico",
    "spotlight-dimmer-icon-paused.ico");

var monitorManager = new MonitorManager();

if (monitorManager.Monitors.Count == 0)
{
    Console.WriteLine("No monitors detected. Exiting.");
    return 1;
}

var focusTracker = new FocusTracker(monitorManager, verboseLogging);
var renderer = new OverlayRenderer();

// ========================================================================
// Core Layer - Pure calculation logic
// ========================================================================

// Configuration: Load from file with hot-reload support
var configManager = new ConfigurationManager();
var config = configManager.Current.ToOverlayConfig();

// Create app state with pre-allocated overlay definitions
var displays = monitorManager.GetDisplayInfo();
var appState = new AppState(displays);

// Pre-create all overlay windows (6 per display, all initially hidden)
// Pre-allocate brushes for configured colors - zero allocations during updates
renderer.CreateOverlays(displays, config);

Console.WriteLine($"\nOverlay Configuration:");
Console.WriteLine($"  Config file: {ConfigurationManager.GetDefaultConfigPath()}");
Console.WriteLine($"  Mode: {config.Mode}");
Console.WriteLine($"  Inactive: {config.InactiveColor.R},{config.InactiveColor.G},{config.InactiveColor.B} @ {config.InactiveOpacity}/255 opacity");
Console.WriteLine($"  Active: {config.ActiveColor.R},{config.ActiveColor.G},{config.ActiveColor.B} @ {config.ActiveOpacity}/255 opacity");

// ========================================================================
// Wire Core and WindowsBindings together
// ========================================================================

// Cache displays and config to avoid allocations in hot path
// These are only updated when monitors change or config changes
var cachedDisplays = monitorManager.GetDisplayInfo();
var cachedConfig = configManager.Current.ToOverlayConfig();

// Helper function to update overlays (zero allocations - uses cached values)
void UpdateOverlays(int displayIndex, Rectangle windowBounds)
{
    // Don't update overlays if paused
    if (systemTray.IsPaused)
        return;

    appState.Calculate(cachedDisplays, windowBounds, displayIndex, cachedConfig);
    renderer.UpdateOverlays(appState.DisplayStates);
}

// ========================================================================
// System Tray Event Handlers
// ========================================================================

// Handle pause/resume from system tray
systemTray.PauseStateChanged += (isPaused) =>
{
    if (isPaused)
    {
        Console.WriteLine("\n[Paused] Overlays hidden. Double-click tray icon or use context menu to resume.");
        renderer.HideAllOverlays();
    }
    else
    {
        Console.WriteLine("\n[Resumed] Overlays active.");
        // Trigger an update to show overlays again
        if (focusTracker.HasFocus && focusTracker.CurrentWindowRect.HasValue)
        {
            UpdateOverlays(focusTracker.CurrentFocusedDisplayIndex, focusTracker.CurrentWindowRect.Value);
        }
    }
};

// Handle quit from system tray
systemTray.QuitRequested += () =>
{
    Console.WriteLine("\n[System Tray] Quit requested. Shutting down...");
    cts.Cancel();
    WinApi.PostThreadMessage(mainThreadId, WinApi.WM_QUIT, IntPtr.Zero, IntPtr.Zero);
};

// ========================================================================
// Configuration and Focus Event Handlers
// ========================================================================

// Handle configuration changes - recalculate and render overlays
configManager.ConfigurationChanged += (newAppConfig) =>
{
    // Update cached config when configuration changes
    cachedConfig = newAppConfig.ToOverlayConfig();

    // Update brush colors for all windows
    renderer.UpdateBrushColors(cachedConfig);

    // If we have a focused window, trigger an update with the new config
    if (focusTracker.HasFocus && focusTracker.CurrentWindowRect.HasValue)
    {
        UpdateOverlays(focusTracker.CurrentFocusedDisplayIndex, focusTracker.CurrentWindowRect.Value);
    }

    Console.WriteLine("[Config] Overlays updated with new configuration\n");
};

// Handle display changes - recalculate and render overlays
focusTracker.FocusedDisplayChanged += (displayIndex, windowBounds) =>
{
    if (verboseLogging)
    {
        Console.WriteLine($"[VERBOSE] Display {displayIndex} gained focus");
    }
    UpdateOverlays(displayIndex, windowBounds);
};

// Handle window position/size changes - recalculate and render overlays
focusTracker.WindowPositionChanged += (displayIndex, windowBounds) =>
{
    // This event fires frequently during window movement
    // In FullScreen mode, we don't need to update (only display changes matter)
    // In Partial/PartialWithActive modes, we need to update on every movement
    // CRITICAL: Use cachedConfig to avoid allocating on every event!
    if (cachedConfig.Mode != DimmingMode.FullScreen)
    {
        UpdateOverlays(displayIndex, windowBounds);
    }
};

// Start tracking
focusTracker.Start();

Console.WriteLine("\nSpotlightDimmer is running.");
Console.WriteLine($"Current mode: {config.Mode}");
Console.WriteLine("The focused display will remain bright.");
Console.WriteLine("Move windows between monitors to see the effect.");
Console.WriteLine($"\nSystem Tray: Available - Double-click to pause/resume, right-click for menu");
Console.WriteLine($"Configuration updates will be automatically applied.");
Console.WriteLine($"Edit the config file to change settings in real-time.\n");

// GDI object monitoring for leak detection (verbose mode only)
var initialGdiCount = 0;
var lastGdiCount = 0;
var gdiCheckTimer = System.Diagnostics.Stopwatch.StartNew();
if (verboseLogging)
{
    var currentProcess = System.Diagnostics.Process.GetCurrentProcess();
    initialGdiCount = WinApi.GetGuiResources(currentProcess.Handle, WinApi.GR_GDIOBJECTS);
    lastGdiCount = initialGdiCount;
    Console.WriteLine($"[VERBOSE] Initial GDI objects: {initialGdiCount}");
}

// ========================================================================
// Windows Message Loop
// ========================================================================

try
{
    while (!cts.Token.IsCancellationRequested)
    {
        // Process Windows messages
        while (WinApi.GetMessage(out var msg, IntPtr.Zero, 0, 0))
        {
            WinApi.TranslateMessage(ref msg);
            WinApi.DispatchMessage(ref msg);

            // Check for cancellation periodically
            if (cts.Token.IsCancellationRequested)
                break;

            // Periodic GDI object monitoring (verbose mode only, every 5 seconds)
            if (verboseLogging && gdiCheckTimer.Elapsed.TotalSeconds >= 5)
            {
                var currentProcess = System.Diagnostics.Process.GetCurrentProcess();
                var currentGdiCount = WinApi.GetGuiResources(currentProcess.Handle, WinApi.GR_GDIOBJECTS);
                if (currentGdiCount != lastGdiCount)
                {
                    var delta = currentGdiCount - initialGdiCount;
                    var deltaSign = delta >= 0 ? "+" : "";
                    Console.WriteLine($"[VERBOSE] GDI objects: {currentGdiCount} ({deltaSign}{delta} from start)");
                    lastGdiCount = currentGdiCount;
                }
                gdiCheckTimer.Restart();
            }
        }

        // If we exit the message loop, wait a bit before checking again
        if (!cts.Token.IsCancellationRequested)
        {
            Thread.Sleep(100);
        }
    }
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
    Console.WriteLine($"Stack trace: {ex.StackTrace}");
    return 1;
}
finally
{
    // Clean up
    systemTray.Dispose();
    focusTracker.Dispose();
    renderer.Dispose();
    configManager.Dispose();
}

Console.WriteLine("Goodbye!");
return 0;
