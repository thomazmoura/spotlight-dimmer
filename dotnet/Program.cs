using SpotlightDimmer.Core;
using SpotlightDimmer.WindowsBindings;

Console.WriteLine("SpotlightDimmer .NET - Refactored Architecture");
Console.WriteLine("===============================================");
Console.WriteLine("Core: Pure overlay calculation logic");
Console.WriteLine("WindowsBindings: Windows-specific rendering");
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

var monitorManager = new MonitorManager();

if (monitorManager.Monitors.Count == 0)
{
    Console.WriteLine("No monitors detected. Exiting.");
    return 1;
}

var focusTracker = new FocusTracker(monitorManager);
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
// This eliminates window creation overhead during updates
renderer.CreateOverlays(displays);

Console.WriteLine($"\nOverlay Configuration:");
Console.WriteLine($"  Config file: {ConfigurationManager.GetDefaultConfigPath()}");
Console.WriteLine($"  Mode: {config.Mode}");
Console.WriteLine($"  Inactive: {config.InactiveColor.R},{config.InactiveColor.G},{config.InactiveColor.B} @ {config.InactiveOpacity}/255 opacity");
Console.WriteLine($"  Active: {config.ActiveColor.R},{config.ActiveColor.G},{config.ActiveColor.B} @ {config.ActiveOpacity}/255 opacity");

// ========================================================================
// Wire Core and WindowsBindings together
// ========================================================================

// Helper function to update overlays
void UpdateOverlays(int displayIndex, Rectangle windowBounds)
{
    var displays = monitorManager.GetDisplayInfo();
    var currentConfig = configManager.Current.ToOverlayConfig();
    appState.Calculate(displays, windowBounds, displayIndex, currentConfig);
    renderer.UpdateOverlays(appState.DisplayStates);
}

// Handle configuration changes - recalculate and render overlays
configManager.ConfigurationChanged += (newAppConfig) =>
{
    var newConfig = newAppConfig.ToOverlayConfig();

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
    Console.WriteLine($"Display {displayIndex} gained focus");
    UpdateOverlays(displayIndex, windowBounds);
};

// Handle window position/size changes - recalculate and render overlays
focusTracker.WindowPositionChanged += (displayIndex, windowBounds) =>
{
    // This event fires frequently during window movement
    // In FullScreen mode, we don't need to update (only display changes matter)
    // In Partial/PartialWithActive modes, we need to update on every movement
    var currentConfig = configManager.Current.ToOverlayConfig();
    if (currentConfig.Mode != DimmingMode.FullScreen)
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
Console.WriteLine($"\nConfiguration updates will be automatically applied.");
Console.WriteLine($"Edit the config file to change settings in real-time.\n");

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
    focusTracker.Dispose();
    renderer.Dispose();
    configManager.Dispose();
}

Console.WriteLine("Goodbye!");
return 0;
