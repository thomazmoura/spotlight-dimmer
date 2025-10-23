using SpotlightDimmer.Core;
using SpotlightDimmer.WindowsBindings;

Console.WriteLine("SpotlightDimmer .NET - Refactored Architecture");
Console.WriteLine("===============================================");
Console.WriteLine("Core: Pure overlay calculation logic");
Console.WriteLine("WindowsBindings: Windows-specific rendering");
Console.WriteLine("\nPress Ctrl+C to exit\n");

// Set up graceful shutdown
using var cts = new CancellationTokenSource();
Console.CancelKeyPress += (sender, e) =>
{
    e.Cancel = true;
    cts.Cancel();
    Console.WriteLine("\nShutting down...");

    // Post a quit message to immediately unblock the message loop
    WinApi.PostQuitMessage(0);
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

var calculator = new OverlayCalculator();

// Configuration: Start with FullScreen mode, can be changed at runtime
var config = new OverlayCalculationConfig(
    Mode: DimmingMode.PartialWithActive,  // Try: FullScreen, Partial, PartialWithActive
    InactiveColor: Color.Black,
    InactiveOpacity: 153,  // ~60% opacity
    ActiveColor: Color.Black,
    ActiveOpacity: 102  // ~40% opacity
);

Console.WriteLine($"\nOverlay Configuration:");
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
    var states = calculator.Calculate(displays, windowBounds, displayIndex, config);
    renderer.UpdateOverlays(states);
}

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
    if (config.Mode != DimmingMode.FullScreen)
    {
        UpdateOverlays(displayIndex, windowBounds);
    }
};

// Start tracking
focusTracker.Start();

Console.WriteLine("\nSpotlightDimmer is running.");
Console.WriteLine($"Current mode: {config.Mode}");
Console.WriteLine("The focused display will remain bright.");
Console.WriteLine("Move windows between monitors to see the effect.\n");

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
}

Console.WriteLine("Goodbye!");
return 0;
