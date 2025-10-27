using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Monitors for display configuration changes (monitor added/removed, resolution changes, layout changes).
/// Creates a top-level window to receive WM_DISPLAYCHANGE notifications from Windows.
/// Uses immediate check + 2s safety timer because Windows sends WM_DISPLAYCHANGE before
/// displays are fully reconfigured.
/// </summary>
internal class DisplayChangeMonitor : IDisposable
{
    private const string WINDOW_CLASS_NAME = "SpotlightDimmerDisplayChangeMonitor";
    private static bool _classRegistered = false;
    private static readonly WinApi.WndProc _wndProcDelegate = WndProc;

    private readonly ILogger<DisplayChangeMonitor> _logger;
    private IntPtr _hwnd = IntPtr.Zero;
    private static DisplayChangeMonitor? _instance;

    // Timer ID for display change safety check
    private static readonly IntPtr TIMER_ID = new IntPtr(1);
    private const uint SAFETY_DELAY_MS = 2000; // 2 seconds

    /// <summary>
    /// Event fired when we should check and reset displays.
    /// Fires immediately on WM_DISPLAYCHANGE and again after 2s safety delay.
    /// </summary>
    public event Action? CheckDisplaysRequested;

    public DisplayChangeMonitor(ILogger<DisplayChangeMonitor> logger)
    {
        _logger = logger;
        _instance = this;
        EnsureWindowClassRegistered();
        CreateMessageWindow();
    }

    /// <summary>
    /// Ensures the window class is registered (only needs to happen once).
    /// </summary>
    private static void EnsureWindowClassRegistered()
    {
        if (_classRegistered)
            return;

        var wndClass = new WinApi.WNDCLASSEX
        {
            lpfnWndProc = Marshal.GetFunctionPointerForDelegate(_wndProcDelegate),
            hInstance = WinApi.GetModuleHandle(null),
            lpszClassName = WINDOW_CLASS_NAME,
            hbrBackground = IntPtr.Zero,
            hCursor = IntPtr.Zero
        };

        var atom = WinApi.RegisterClassEx(wndClass);
        if (atom == 0)
        {
            throw new InvalidOperationException($"Failed to register window class. Error: {Marshal.GetLastWin32Error()}");
        }

        _classRegistered = true;
    }

    /// <summary>
    /// Creates an invisible top-level window to receive WM_DISPLAYCHANGE broadcasts.
    /// Note: WM_DISPLAYCHANGE is only sent to top-level windows, not message-only windows.
    /// </summary>
    private void CreateMessageWindow()
    {
        // Create an invisible top-level window to receive WM_DISPLAYCHANGE
        // We need a real window (not HWND_MESSAGE) because broadcast messages
        // like WM_DISPLAYCHANGE are only sent to top-level windows
        _hwnd = WinApi.CreateWindowEx(
            WinApi.WS_EX_TOOLWINDOW, // Tool window - doesn't appear in taskbar
            WINDOW_CLASS_NAME,
            "SpotlightDimmer Display Change Monitor",
            WinApi.WS_POPUP, // Popup window (no decorations)
            0, 0, 1, 1, // Minimal size, positioned at origin
            IntPtr.Zero, // No parent - this is a top-level window
            IntPtr.Zero,
            WinApi.GetModuleHandle(null),
            IntPtr.Zero);

        if (_hwnd == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to create message window. Error: {Marshal.GetLastWin32Error()}");
        }

        // Keep window hidden - we only want it to receive messages
        WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
    }

    /// <summary>
    /// Window procedure for handling messages.
    /// </summary>
    private static IntPtr WndProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        switch (msg)
        {
            case WinApi.WM_DISPLAYCHANGE:
                // Display configuration changed (resolution, monitor added/removed, layout change)
                // Check immediately and set a 2s safety timer
                _instance?._logger.LogInformation("WM_DISPLAYCHANGE received - checking immediately and setting {SafetyDelayMs}ms safety timer", SAFETY_DELAY_MS);

                // Fire event immediately
                _instance?.CheckDisplaysRequested?.Invoke();

                // Set timer for safety check (will fire WM_TIMER in 2 seconds)
                if (_instance != null && _instance._hwnd != IntPtr.Zero)
                {
                    WinApi.SetTimer(_instance._hwnd, TIMER_ID, SAFETY_DELAY_MS, IntPtr.Zero);
                }
                return IntPtr.Zero;

            case WinApi.WM_TIMER:
                // Safety timer fired - check displays again
                if (wParam == TIMER_ID)
                {
                    _instance?._logger.LogDebug("Safety timer fired - rechecking displays");

                    // Fire event for safety check
                    _instance?.CheckDisplaysRequested?.Invoke();

                    // Kill the timer (one-shot)
                    if (_instance != null && _instance._hwnd != IntPtr.Zero)
                    {
                        WinApi.KillTimer(_instance._hwnd, TIMER_ID);
                    }
                }
                return IntPtr.Zero;

            case WinApi.WM_CREATE:
                _instance?._logger.LogDebug("Window created successfully");
                return IntPtr.Zero;

            default:
                return WinApi.DefWindowProc(hWnd, msg, wParam, lParam);
        }
    }

    public void Dispose()
    {
        if (_hwnd != IntPtr.Zero)
        {
            // Kill any pending timer
            WinApi.KillTimer(_hwnd, TIMER_ID);

            WinApi.DestroyWindow(_hwnd);
            _hwnd = IntPtr.Zero;
        }
        _instance = null;
    }
}
