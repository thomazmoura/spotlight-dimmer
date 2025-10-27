using Microsoft.Extensions.Logging;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Tracks window focus and movement changes using Windows event hooks (no polling!)
/// </summary>
internal class FocusTracker : IDisposable
{
    private readonly MonitorManager _monitorManager;
    private readonly ILogger<FocusTracker> _logger;
    private IntPtr _foregroundHook = IntPtr.Zero;
    private IntPtr _locationHook = IntPtr.Zero;
    private int _lastFocusedDisplayIndex = -1;
    private Core.Rectangle? _lastWindowRect;

    // Must keep a reference to prevent garbage collection
    private readonly WinApi.WinEventDelegate _hookDelegate;

    /// <summary>
    /// Fired when the focused display changes (window moved to different monitor).
    /// Provides: (displayIndex, windowRect)
    /// </summary>
    public event Action<int, Core.Rectangle>? FocusedDisplayChanged;

    /// <summary>
    /// Fired when the focused window's position/size changes (even on same display).
    /// Provides: (displayIndex, windowRect)
    /// </summary>
    public event Action<int, Core.Rectangle>? WindowPositionChanged;

    public int CurrentFocusedDisplayIndex => _lastFocusedDisplayIndex;
    public Core.Rectangle? CurrentWindowRect => _lastWindowRect;
    public bool HasFocus => _lastFocusedDisplayIndex >= 0 && _lastWindowRect.HasValue;

    public FocusTracker(MonitorManager monitorManager, ILogger<FocusTracker> logger)
    {
        _monitorManager = monitorManager;
        _logger = logger;
        _hookDelegate = OnWinEvent;
    }

    /// <summary>
    /// Starts tracking focus and window movement changes.
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

        _logger.LogDebug("Focus tracking started:");
        _logger.LogDebug("  - EVENT_SYSTEM_FOREGROUND: Instant app switching");
        _logger.LogDebug("  - EVENT_OBJECT_LOCATIONCHANGE: Window movement detection");
        _logger.LogDebug("  - Fully event-driven, no polling!");

        // Get the initial focused display
        UpdateFocusedDisplay();
    }

    /// <summary>
    /// Callback invoked when a window gains focus or moves.
    /// </summary>
    private void OnWinEvent(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime)
    {
        if (eventType == WinApi.EVENT_SYSTEM_FOREGROUND)
        {
            // Process foreground changes for windows (not child objects)
            if (idObject == WinApi.OBJID_WINDOW)
            {
                UpdateFocusedDisplay("Focus change");
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
                    UpdateFocusedDisplay("Window moved");
                }
            }
        }
    }

    /// <summary>
    /// Updates which display currently has the focused window and tracks position/size changes.
    /// </summary>
    private void UpdateFocusedDisplay(string? reason = null)
    {
        var foregroundWindow = WinApi.GetForegroundWindow();
        if (foregroundWindow == IntPtr.Zero)
            return;

        var focusedDisplayIndex = _monitorManager.GetDisplayIndexForWindow(foregroundWindow);

        // Get window rectangle (excluding invisible borders) and convert to Core.Rectangle
        Core.Rectangle? currentRect = null;
        if (WinApi.GetExtendedWindowRect(foregroundWindow, out var winRect))
        {
            currentRect = WinApi.ToRectangle(winRect);
        }

        bool displayChanged = focusedDisplayIndex != _lastFocusedDisplayIndex;
        bool rectChanged = currentRect.HasValue && _lastWindowRect != currentRect;

        // Fire display change event if display actually changed
        if (displayChanged && currentRect.HasValue)
        {
            _lastFocusedDisplayIndex = focusedDisplayIndex;

            if (focusedDisplayIndex >= 0)
            {
                var reasonText = reason != null ? $" ({reason})" : "";
                _logger.LogDebug("Display {DisplayIndex} is now active{ReasonText}", focusedDisplayIndex, reasonText);
            }

            FocusedDisplayChanged?.Invoke(focusedDisplayIndex, currentRect.Value);
        }

        // Fire position change event if window moved/resized (even on same display)
        if (rectChanged && currentRect.HasValue)
        {
            _lastWindowRect = currentRect;

            if (!displayChanged && reason != null)
            {
                // Only log if display didn't change (to avoid double logging)
                _logger.LogDebug("Window position/size changed: ({X},{Y}) {Width}x{Height} ({Reason})",
                    currentRect.Value.X, currentRect.Value.Y, currentRect.Value.Width, currentRect.Value.Height, reason);
            }

            WindowPositionChanged?.Invoke(focusedDisplayIndex, currentRect.Value);
        }
        else if (displayChanged && currentRect.HasValue)
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

        _logger.LogDebug("Focus tracking stopped");
    }
}
