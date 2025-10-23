using System.Runtime.InteropServices;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Manages detection and tracking of all connected monitors.
/// Maintains mapping between display indices (Core) and HMONITOR handles (Windows).
/// </summary>
internal class MonitorManager
{
    private readonly List<MonitorHandle> _monitors = new();

    /// <summary>
    /// Gets information about all connected monitors as Core DisplayInfo array.
    /// </summary>
    public DisplayInfo[] GetDisplayInfo()
    {
        return _monitors
            .Select((m, index) => new DisplayInfo(index, m.Bounds))
            .ToArray();
    }

    /// <summary>
    /// Gets the internal monitor handle information (for Windows-specific operations).
    /// </summary>
    internal IReadOnlyList<MonitorHandle> Monitors => _monitors;

    public MonitorManager()
    {
        RefreshMonitors();
    }

    /// <summary>
    /// Refreshes the list of connected monitors.
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
            Console.WriteLine($"  Monitor {i}: {mon.Bounds.Width}x{mon.Bounds.Height} at ({mon.Bounds.X}, {mon.Bounds.Y})");
        }
    }

    /// <summary>
    /// Gets the display index for the monitor that contains the specified window.
    /// </summary>
    /// <returns>Display index, or -1 if not found.</returns>
    public int GetDisplayIndexForWindow(IntPtr hwnd)
    {
        if (hwnd == IntPtr.Zero)
            return -1;

        var hMonitor = WinApi.MonitorFromWindow(hwnd, WinApi.MONITOR_DEFAULTTONEAREST);
        if (hMonitor == IntPtr.Zero)
            return -1;

        for (int i = 0; i < _monitors.Count; i++)
        {
            if (_monitors[i].Handle == hMonitor)
                return i;
        }

        return -1;
    }

    /// <summary>
    /// Gets the monitor handle for the specified display index.
    /// </summary>
    public IntPtr GetMonitorHandle(int displayIndex)
    {
        if (displayIndex < 0 || displayIndex >= _monitors.Count)
            return IntPtr.Zero;

        return _monitors[displayIndex].Handle;
    }

    /// <summary>
    /// Callback invoked for each monitor during enumeration.
    /// </summary>
    private bool MonitorEnumCallback(IntPtr hMonitor, IntPtr hdcMonitor, ref WinApi.RECT lprcMonitor, IntPtr dwData)
    {
        var info = new WinApi.MONITORINFO();
        info.cbSize = Marshal.SizeOf(info);

        if (WinApi.GetMonitorInfo(hMonitor, ref info))
        {
            var bounds = WinApi.ToRectangle(info.rcMonitor);
            _monitors.Add(new MonitorHandle(hMonitor, bounds));
        }

        return true; // Continue enumeration
    }
}

/// <summary>
/// Internal structure mapping a Windows monitor handle to Core display bounds.
/// </summary>
internal readonly record struct MonitorHandle(IntPtr Handle, Core.Rectangle Bounds);
