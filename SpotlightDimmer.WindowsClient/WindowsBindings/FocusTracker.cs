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
    private IntPtr _messageWindow = IntPtr.Zero;

    // Polling timer to catch foreground changes that don't fire events (UWP app launches)
    private System.Threading.Timer? _pollingTimer;
    private const int POLLING_INTERVAL_MS = 100; // Poll every 100ms to catch missed events

    // Must keep references to prevent garbage collection
    private readonly WinApi.WinEventDelegate _hookDelegate;
    private readonly WinApi.WndProc _wndProcDelegate;

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
        _wndProcDelegate = MessageWindowProc;
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

        _logger.LogDebug("[FOCUS] Focus tracking started:");
        _logger.LogDebug("[FOCUS]   - EVENT_SYSTEM_FOREGROUND: Instant app switching");
        _logger.LogDebug("[FOCUS]   - EVENT_OBJECT_LOCATIONCHANGE: Window movement detection");
        _logger.LogDebug("[FOCUS]   - Polling (100ms): Catches UWP app launches that don't fire events");

        // Create message-only window for marshalling polling updates to UI thread
        // This ensures all SetWindowPos calls happen on the thread that owns the overlay windows
        CreateMessageWindow();

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
    /// Creates a message-only window for handling focus update messages from the polling thread.
    /// </summary>
    private void CreateMessageWindow()
    {
        // Register window class
        var wc = new WinApi.WNDCLASSEX
        {
            cbSize = System.Runtime.InteropServices.Marshal.SizeOf<WinApi.WNDCLASSEX>(),
            lpfnWndProc = System.Runtime.InteropServices.Marshal.GetFunctionPointerForDelegate(_wndProcDelegate),
            hInstance = WinApi.GetModuleHandle(null),
            lpszClassName = "SpotlightDimmer_FocusTracker_MessageWindow"
        };

        ushort classAtom = WinApi.RegisterClassEx(wc);
        if (classAtom == 0)
        {
            throw new InvalidOperationException("Failed to register message window class");
        }

        // Create message-only window (HWND_MESSAGE parent = message-only)
        _messageWindow = WinApi.CreateWindowEx(
            0,
            wc.lpszClassName,
            "SpotlightDimmer Focus Message Window",
            0,
            0, 0, 0, 0,
            WinApi.HWND_MESSAGE,
            IntPtr.Zero,
            wc.hInstance,
            IntPtr.Zero);

        if (_messageWindow == IntPtr.Zero)
        {
            throw new InvalidOperationException("Failed to create message window");
        }

        _logger.LogDebug("[FOCUS] Created message window for thread marshalling: {Handle:X}", _messageWindow);
    }

    /// <summary>
    /// Window procedure for the message-only window.
    /// Handles WM_FOCUS_UPDATE by calling UpdateFocusedDisplay on the UI thread.
    /// </summary>
    private IntPtr MessageWindowProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        if (msg == WinApi.WM_FOCUS_UPDATE)
        {
            // This now runs on the UI thread (the thread that owns the overlay windows)
            UpdateFocusedDisplay("Polling detected");
            return IntPtr.Zero;
        }

        return WinApi.DefWindowProc(hWnd, msg, wParam, lParam);
    }

    /// <summary>
    /// Polling callback that checks if the foreground window changed without an event.
    /// This catches UWP app launches where ApplicationFrameHost becomes foreground
    /// without firing EVENT_SYSTEM_FOREGROUND.
    /// Posts a message to the UI thread instead of calling UpdateFocusedDisplay directly.
    /// </summary>
    private void PollingCallback(object? state)
    {
        try
        {
            var currentForegroundWindow = WinApi.GetForegroundWindow();

            // Check if foreground window changed
            if (currentForegroundWindow != _lastForegroundWindow && currentForegroundWindow != IntPtr.Zero)
            {
                _logger.LogDebug("[FOCUS] [Polling] Detected foreground change from {Old:X} to {New:X}",
                    _lastForegroundWindow, currentForegroundWindow);

                // Post message to UI thread instead of calling UpdateFocusedDisplay directly
                // This ensures SetWindowPos calls happen on the thread that owns the overlay windows
                if (_messageWindow != IntPtr.Zero)
                {
                    WinApi.PostMessage(_messageWindow, WinApi.WM_FOCUS_UPDATE, IntPtr.Zero, IntPtr.Zero);
                }
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
        {
            _logger.LogDebug(
                "There is no foreground window, skipping {MethodName}",
                nameof(UpdateFocusedDisplay)
            );
            return;
        }

        // Track foreground window for polling detection
        _lastForegroundWindow = foregroundWindow;

        var focusedDisplayIndex = _monitorManager.GetDisplayIndexForWindow(foregroundWindow);

        string? processName = WinApi.GetProcessName(foregroundWindow) ?? "unknown";

        // CRITICAL: For UWP apps (ApplicationFrameHost), get the actual content window
        // The foreground window is just the frame - the content is in a child window
        var currentRect = WinApi.GetUwpContentBounds(foregroundWindow, msg => _logger.LogDebug(msg), processName);

        // Process the focus change through the Core handler
        var result = _focusChangeHandler.ProcessFocusChange(focusedDisplayIndex, currentRect);

        // Handle the result and fire appropriate events
        switch (result)
        {
            case FocusChangeResult.Ignored:
                _logger.LogDebug("[FOCUS] Focus change ignored (likely 0x0 window or invalid bounds) - {Process}", processName);
                break;

            case FocusChangeResult.DisplayChanged:
                if (focusedDisplayIndex >= 0)
                {
                    var reasonText = reason != null ? $" ({reason})" : "";
                    _logger.LogDebug("[FOCUS] Display {DisplayIndex} is now active ({ReasonText}), ({X},{Y}) {Width}x{Height} - {Process}", focusedDisplayIndex, reasonText, currentRect.X, currentRect.Y, currentRect.Width, currentRect.Height, processName);
                    FocusedDisplayChanged?.Invoke(focusedDisplayIndex, currentRect);
                }
                break;

            case FocusChangeResult.PositionChanged:
                if (reason != null)
                {
                    _logger.LogDebug("[FOCUS] Window position/size changed: ({X},{Y}) {Width}x{Height} #{Index} ({Reason}), - {Process}",
                        currentRect.X, currentRect.Y, currentRect.Width, currentRect.Height, focusedDisplayIndex, reason, processName);
                }
                WindowPositionChanged?.Invoke(focusedDisplayIndex, currentRect);
                break;

            case FocusChangeResult.NoChange:
                _logger.LogDebug("[FOCUS] Focus changed ({reason}) ignored because no change is needed - {Process}", reason, processName);
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

        // Destroy message window
        if (_messageWindow != IntPtr.Zero)
        {
            WinApi.DestroyWindow(_messageWindow);
            _messageWindow = IntPtr.Zero;
        }

        _logger.LogDebug("[FOCUS] Focus tracking stopped");
    }
}
