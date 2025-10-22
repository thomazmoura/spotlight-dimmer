using SpotlightDimmer;

Console.WriteLine("SpotlightDimmer .NET PoC");
Console.WriteLine("========================");
Console.WriteLine("Press Ctrl+C to exit\n");

// Set up graceful shutdown
using var cts = new CancellationTokenSource();
Console.CancelKeyPress += (sender, e) =>
{
    e.Cancel = true;
    cts.Cancel();
    Console.WriteLine("\nShutting down...");
};

// Initialize monitor management
var monitorManager = new MonitorManager();

if (monitorManager.Monitors.Count == 0)
{
    Console.WriteLine("No monitors detected. Exiting.");
    return 1;
}

// Create overlays for each monitor
var overlays = monitorManager.Monitors
    .Select(monitor => new OverlayWindow(monitor))
    .ToList();

Console.WriteLine($"\nCreated {overlays.Count} overlay window(s)");

// Set up focus tracking
using var focusTracker = new FocusTracker(monitorManager);

// Handle monitor changes - update which overlays are visible
focusTracker.FocusedMonitorChanged += (focusedMonitor) =>
{
    foreach (var overlay in overlays)
    {
        if (focusedMonitor != null && overlay.Monitor.Equals(focusedMonitor))
        {
            // Hide overlay on the focused monitor
            overlay.Hide();
        }
        else
        {
            // Show overlay on all other monitors
            overlay.Show();
        }
    }
};

// Handle window position/size changes (for partial dimming support)
focusTracker.WindowPositionChanged += (monitor, rect) =>
{
    // This event fires even when window moves/resizes on the same monitor
    // Perfect for implementing partial dimming in the future!
    // For now, we just log it to demonstrate detection works
};

// Start tracking
focusTracker.Start();

Console.WriteLine("\nSpotlightDimmer is running. The focused monitor will remain bright.");
Console.WriteLine("Move windows between monitors to see the effect.\n");

// Run the Windows message loop to process events
// This is necessary for the event hooks to work
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
    return 1;
}
finally
{
    // Clean up
    foreach (var overlay in overlays)
    {
        overlay.Dispose();
    }
}

Console.WriteLine("Goodbye!");
return 0;
