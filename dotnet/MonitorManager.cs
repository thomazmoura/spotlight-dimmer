using System.Runtime.InteropServices;

namespace SpotlightDimmer;

/// <summary>
/// Manages detection and tracking of all connected monitors
/// </summary>
internal class MonitorManager
{
    private readonly List<MonitorInfo> _monitors = new();

    public IReadOnlyList<MonitorInfo> Monitors => _monitors;

    public MonitorManager()
    {
        RefreshMonitors();
    }

    /// <summary>
    /// Refreshes the list of connected monitors
    /// </summary>
    public void RefreshMonitors()
    {
        _monitors.Clear();

        // Enumerate all monitors - the callback will be called for each one
        WinApi.EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, MonitorEnumCallback, IntPtr.Zero);

        Console.WriteLine($"Detected {_monitors.Count} monitor(s)");
        for (int i = 0; i < _monitors.Count; i++)
        {
            var mon = _monitors[i];
            Console.WriteLine($"  Monitor {i + 1}: {mon.Bounds.Width}x{mon.Bounds.Height} at ({mon.Bounds.Left}, {mon.Bounds.Top})");
        }
    }

    /// <summary>
    /// Gets the monitor that contains the specified window
    /// </summary>
    public MonitorInfo? GetMonitorForWindow(IntPtr hwnd)
    {
        if (hwnd == IntPtr.Zero)
            return null;

        var hMonitor = WinApi.MonitorFromWindow(hwnd, WinApi.MONITOR_DEFAULTTONEAREST);
        if (hMonitor == IntPtr.Zero)
            return null;

        return _monitors.FirstOrDefault(m => m.Handle == hMonitor);
    }

    /// <summary>
    /// Callback invoked for each monitor during enumeration
    /// </summary>
    private bool MonitorEnumCallback(IntPtr hMonitor, IntPtr hdcMonitor, ref WinApi.RECT lprcMonitor, IntPtr dwData)
    {
        var info = new WinApi.MONITORINFO();
        info.cbSize = Marshal.SizeOf(info);

        if (WinApi.GetMonitorInfo(hMonitor, ref info))
        {
            _monitors.Add(new MonitorInfo(hMonitor, info.rcMonitor));
        }

        return true; // Continue enumeration
    }
}

/// <summary>
/// Information about a single monitor
/// </summary>
internal class MonitorInfo
{
    public IntPtr Handle { get; }
    public WinApi.RECT Bounds { get; }

    public MonitorInfo(IntPtr handle, WinApi.RECT bounds)
    {
        Handle = handle;
        Bounds = bounds;
    }

    public override bool Equals(object? obj)
    {
        return obj is MonitorInfo other && Handle == other.Handle;
    }

    public override int GetHashCode()
    {
        return Handle.GetHashCode();
    }
}
