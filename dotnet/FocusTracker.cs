namespace SpotlightDimmer;

/// <summary>
/// Tracks window focus and movement changes using Windows event hooks (no polling!)
/// </summary>
internal class FocusTracker : IDisposable
{
    private readonly MonitorManager _monitorManager;
    private IntPtr _foregroundHook = IntPtr.Zero;
    private IntPtr _locationHook = IntPtr.Zero;
    private MonitorInfo? _lastFocusedMonitor;
    private WinApi.RECT? _lastWindowRect;

    // Must keep a reference to prevent garbage collection
    private readonly WinApi.WinEventDelegate _hookDelegate;

    /// <summary>
    /// Fired when the focused monitor changes (window moved to different monitor)
    /// </summary>
    public event Action<MonitorInfo?>? FocusedMonitorChanged;

    /// <summary>
    /// Fired when the focused window's position/size changes (even on same monitor)
    /// Provides: (monitor, windowRect)
    /// </summary>
    public event Action<MonitorInfo?, WinApi.RECT>? WindowPositionChanged;

    public MonitorInfo? CurrentFocusedMonitor => _lastFocusedMonitor;
    public WinApi.RECT? CurrentWindowRect => _lastWindowRect;

    public FocusTracker(MonitorManager monitorManager)
    {
        _monitorManager = monitorManager;
        _hookDelegate = OnWinEvent;
    }

    /// <summary>
    /// Starts tracking focus and window movement changes
    /// </summary>
    public void Start()
    {
        if (_foregroundHook != IntPtr.Zero || _locationHook != IntPtr.Zero)
            return;

        // Hook 1: EVENT_SYSTEM_FOREGROUND for instant app switching detection
        _foregroundHook = WinApi.SetWinEventHook(
            WinApi.EVENT_SYSTEM_FOREGROUND,
            WinApi.EVENT_SYSTEM_FOREGROUND,
            IntPtr.Zero,
            _hookDelegate,
            0,
            0,
            WinApi.WINEVENT_OUTOFCONTEXT | WinApi.WINEVENT_SKIPOWNPROCESS);

        if (_foregroundHook == IntPtr.Zero)
        {
            throw new InvalidOperationException("Failed to set EVENT_SYSTEM_FOREGROUND hook");
        }

        // Hook 2: EVENT_OBJECT_LOCATIONCHANGE for window movement detection
        _locationHook = WinApi.SetWinEventHook(
            WinApi.EVENT_OBJECT_LOCATIONCHANGE,
            WinApi.EVENT_OBJECT_LOCATIONCHANGE,
            IntPtr.Zero,
            _hookDelegate,
            0,
            0,
            WinApi.WINEVENT_OUTOFCONTEXT | WinApi.WINEVENT_SKIPOWNPROCESS);

        if (_locationHook == IntPtr.Zero)
        {
            // Clean up foreground hook if location hook fails
            WinApi.UnhookWinEvent(_foregroundHook);
            _foregroundHook = IntPtr.Zero;
            throw new InvalidOperationException("Failed to set EVENT_OBJECT_LOCATIONCHANGE hook");
        }

        Console.WriteLine("Focus tracking started:");
        Console.WriteLine("  - EVENT_SYSTEM_FOREGROUND: Instant app switching");
        Console.WriteLine("  - EVENT_OBJECT_LOCATIONCHANGE: Window movement detection");
        Console.WriteLine("  - Fully event-driven, no polling!");

        // Get the initial focused monitor
        UpdateFocusedMonitor();
    }

    /// <summary>
    /// Callback invoked when a window gains focus or moves
    /// </summary>
    private void OnWinEvent(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime)
    {
        if (eventType == WinApi.EVENT_SYSTEM_FOREGROUND)
        {
            // Process foreground changes for windows (not child objects)
            if (idObject == WinApi.OBJID_WINDOW)
            {
                UpdateFocusedMonitor("Focus change");
            }
        }
        else if (eventType == WinApi.EVENT_OBJECT_LOCATIONCHANGE)
        {
            // CRITICAL: Filter out cursor movement events!
            // LOCATIONCHANGE fires for cursor, caret, and windows
            if (idObject == WinApi.OBJID_WINDOW)
            {
                // Only process if this is the foreground window moving
                var foregroundWindow = WinApi.GetForegroundWindow();
                if (hwnd == foregroundWindow)
                {
                    UpdateFocusedMonitor("Window moved");
                }
            }
        }
    }

    /// <summary>
    /// Updates which monitor currently has the focused window and tracks position/size changes
    /// </summary>
    private void UpdateFocusedMonitor(string? reason = null)
    {
        var foregroundWindow = WinApi.GetForegroundWindow();
        if (foregroundWindow == IntPtr.Zero)
            return;

        var focusedMonitor = _monitorManager.GetMonitorForWindow(foregroundWindow);

        // Get window rectangle
        WinApi.RECT? currentRect = null;
        if (WinApi.GetWindowRect(foregroundWindow, out var rect))
        {
            currentRect = rect;
        }

        bool monitorChanged = !Equals(focusedMonitor, _lastFocusedMonitor);
        bool rectChanged = currentRect.HasValue && _lastWindowRect != currentRect;

        // Fire monitor change event if monitor actually changed
        if (monitorChanged)
        {
            _lastFocusedMonitor = focusedMonitor;

            if (focusedMonitor != null)
            {
                var monitorIndex = Array.IndexOf(_monitorManager.Monitors.ToArray(), focusedMonitor) + 1;
                var reasonText = reason != null ? $" ({reason})" : "";
                Console.WriteLine($"Monitor {monitorIndex} is now active{reasonText}");
            }

            FocusedMonitorChanged?.Invoke(focusedMonitor);
        }

        // Fire position change event if window moved/resized (even on same monitor)
        if (rectChanged && currentRect.HasValue)
        {
            _lastWindowRect = currentRect;

            if (!monitorChanged && reason != null)
            {
                // Only log if monitor didn't change (to avoid double logging)
                Console.WriteLine($"Window position/size changed: {currentRect} ({reason})");
            }

            WindowPositionChanged?.Invoke(focusedMonitor, currentRect.Value);
        }
        else if (monitorChanged && currentRect.HasValue)
        {
            // Update rect even if it didn't trigger an event (for initial state)
            _lastWindowRect = currentRect;
        }
    }

    public void Dispose()
    {
        if (_foregroundHook != IntPtr.Zero)
        {
            WinApi.UnhookWinEvent(_foregroundHook);
            _foregroundHook = IntPtr.Zero;
        }

        if (_locationHook != IntPtr.Zero)
        {
            WinApi.UnhookWinEvent(_locationHook);
            _locationHook = IntPtr.Zero;
        }

        Console.WriteLine("Focus tracking stopped");
    }
}
