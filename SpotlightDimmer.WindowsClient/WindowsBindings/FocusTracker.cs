using Microsoft.Extensions.Logging;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Tracks window focus and movement changes using Windows event hooks + lightweight polling.
/// Event hooks handle 99% of cases; polling (100ms) catches UWP app launches that don't fire events.
/// </summary>
internal class FocusTracker : IDisposable
{
    private readonly MonitorManager _monitorManager;
    private readonly ILogger<FocusTracker> _logger;
    private readonly FocusChangeHandler _focusChangeHandler;
    private IntPtr _foregroundHook = IntPtr.Zero;
    private IntPtr _locationHook = IntPtr.Zero;
    private IntPtr _lastForegroundWindow = IntPtr.Zero;

    // Polling timer to catch foreground changes that don't fire events (UWP app launches)
    private System.Threading.Timer? _pollingTimer;
    private const int POLLING_INTERVAL_MS = 100; // Poll every 100ms to catch missed events

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

    public int CurrentFocusedDisplayIndex => _focusChangeHandler.CurrentFocusedDisplayIndex;
    public Core.Rectangle? CurrentWindowRect => _focusChangeHandler.CurrentWindowRect;
    public bool HasFocus => _focusChangeHandler.HasFocus;

    public FocusTracker(MonitorManager monitorManager, FocusChangeHandler focusChangeHandler, ILogger<FocusTracker> logger)
    {
        _monitorManager = monitorManager;
        _focusChangeHandler = focusChangeHandler ?? throw new ArgumentNullException(nameof(focusChangeHandler));
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
        _logger.LogDebug("  - Polling (100ms): Catches UWP app launches that don't fire events");

        // Get the initial focused display
        UpdateFocusedDisplay();

        // Start polling timer to catch foreground changes that don't fire events
        // This is needed for UWP app launches where ApplicationFrameHost becomes foreground
        // without firing EVENT_SYSTEM_FOREGROUND
        _pollingTimer = new System.Threading.Timer(
            PollingCallback,
            null,
            POLLING_INTERVAL_MS,
            POLLING_INTERVAL_MS);
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
    /// Polling callback that checks if the foreground window changed without an event.
    /// This catches UWP app launches where ApplicationFrameHost becomes foreground
    /// without firing EVENT_SYSTEM_FOREGROUND.
    /// </summary>
    private void PollingCallback(object? state)
    {
        try
        {
            var currentForegroundWindow = WinApi.GetForegroundWindow();

            // Check if foreground window changed
            if (currentForegroundWindow != _lastForegroundWindow && currentForegroundWindow != IntPtr.Zero)
            {
                _logger.LogDebug("[Polling] Detected foreground change from {Old:X} to {New:X}",
                    _lastForegroundWindow, currentForegroundWindow);
                UpdateFocusedDisplay("Polling detected");
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in polling callback");
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

        // Track foreground window for polling detection
        _lastForegroundWindow = foregroundWindow;

        // CRITICAL: For UWP apps (ApplicationFrameHost), get the actual content window
        // The foreground window is just the frame - the content is in a child window
        var contentWindow = WinApi.GetUwpContentWindow(foregroundWindow, msg => _logger.LogDebug(msg));

        var focusedDisplayIndex = _monitorManager.GetDisplayIndexForWindow(contentWindow);

        // Get window rectangle (excluding invisible borders) and convert to Core.Rectangle
        Core.Rectangle? currentRect = null;
        if (WinApi.GetExtendedWindowRect(contentWindow, out var winRect))
        {
            currentRect = WinApi.ToRectangle(winRect);
        }

        // Process the focus change through the Core handler
        var result = _focusChangeHandler.ProcessFocusChange(focusedDisplayIndex, currentRect);

        // Handle the result and fire appropriate events
        switch (result)
        {
            case FocusChangeResult.Ignored:
                _logger.LogDebug("Focus change ignored (likely 0x0 window or invalid bounds)");
                break;

            case FocusChangeResult.DisplayChanged:
                if (focusedDisplayIndex >= 0 && currentRect.HasValue)
                {
                    var reasonText = reason != null ? $" ({reason})" : "";
                    _logger.LogDebug("Display {DisplayIndex} is now active{ReasonText}", focusedDisplayIndex, reasonText);
                    FocusedDisplayChanged?.Invoke(focusedDisplayIndex, currentRect.Value);
                }
                break;

            case FocusChangeResult.PositionChanged:
                if (currentRect.HasValue && reason != null)
                {
                    _logger.LogDebug("Window position/size changed: ({X},{Y}) {Width}x{Height} #{Index} ({Reason})",
                        currentRect.Value.X, currentRect.Value.Y, currentRect.Value.Width, currentRect.Value.Height, focusedDisplayIndex, reason);
                }
                if (currentRect.HasValue)
                {
                    WindowPositionChanged?.Invoke(focusedDisplayIndex, currentRect.Value);
                }
                break;

            case FocusChangeResult.NoChange:
                // No action needed
                break;
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

        // Stop polling timer
        _pollingTimer?.Dispose();
        _pollingTimer = null;

        _logger.LogDebug("Focus tracking stopped");
    }
}
