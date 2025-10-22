using System.Runtime.InteropServices;

namespace SpotlightDimmer;

/// <summary>
/// Represents a semi-transparent, click-through overlay window that covers a monitor
/// </summary>
internal class OverlayWindow : IDisposable
{
    private const string WINDOW_CLASS_NAME = "SpotlightDimmerOverlay";
    private static bool _classRegistered = false;
    private static readonly WinApi.WndProc _wndProcDelegate = WndProc;

    private readonly MonitorInfo _monitor;
    private IntPtr _hwnd = IntPtr.Zero;
    private bool _isVisible = false;

    public MonitorInfo Monitor => _monitor;
    public bool IsVisible => _isVisible;

    public OverlayWindow(MonitorInfo monitor)
    {
        _monitor = monitor;
        EnsureWindowClassRegistered();
        CreateOverlayWindow();
    }

    /// <summary>
    /// Shows the overlay on the monitor
    /// </summary>
    public void Show()
    {
        if (!_isVisible && _hwnd != IntPtr.Zero)
        {
            WinApi.ShowWindow(_hwnd, 5); // SW_SHOW = 5
            _isVisible = true;
        }
    }

    /// <summary>
    /// Hides the overlay
    /// </summary>
    public void Hide()
    {
        if (_isVisible && _hwnd != IntPtr.Zero)
        {
            WinApi.ShowWindow(_hwnd, 0); // SW_HIDE = 0
            _isVisible = false;
        }
    }

    /// <summary>
    /// Ensures the window class is registered (only needs to happen once)
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
            hbrBackground = WinApi.CreateSolidBrush(WinApi.RGB(0, 0, 0)),
            hCursor = WinApi.LoadCursor(IntPtr.Zero, 32512) // IDC_ARROW
        };

        var atom = WinApi.RegisterClassEx(wndClass);
        if (atom == 0)
        {
            throw new InvalidOperationException($"Failed to register window class. Error: {Marshal.GetLastWin32Error()}");
        }

        _classRegistered = true;
    }

    /// <summary>
    /// Creates the actual overlay window
    /// </summary>
    private void CreateOverlayWindow()
    {
        var bounds = _monitor.Bounds;

        _hwnd = WinApi.CreateWindowEx(
            WinApi.WS_EX_TOPMOST | WinApi.WS_EX_LAYERED | WinApi.WS_EX_TRANSPARENT | WinApi.WS_EX_TOOLWINDOW | WinApi.WS_EX_NOACTIVATE,
            WINDOW_CLASS_NAME,
            "SpotlightDimmer Overlay",
            WinApi.WS_POPUP | WinApi.WS_VISIBLE,
            bounds.Left,
            bounds.Top,
            bounds.Width,
            bounds.Height,
            IntPtr.Zero,
            IntPtr.Zero,
            WinApi.GetModuleHandle(null),
            IntPtr.Zero);

        if (_hwnd == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to create window. Error: {Marshal.GetLastWin32Error()}");
        }

        // Set the overlay to be semi-transparent black (60% opacity = alpha 153)
        // You can adjust the alpha value (0-255) to change opacity
        // 153 = ~60% opacity, 128 = 50%, 102 = ~40%
        WinApi.SetLayeredWindowAttributes(_hwnd, 0, 153, WinApi.LWA_ALPHA);

        // Start hidden - the main app will show overlays as needed
        Hide();
    }

    /// <summary>
    /// Window procedure for handling window messages
    /// </summary>
    private static IntPtr WndProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        switch (msg)
        {
            case WinApi.WM_DESTROY:
            case WinApi.WM_CLOSE:
                return IntPtr.Zero;

            default:
                return WinApi.DefWindowProc(hWnd, msg, wParam, lParam);
        }
    }

    public void Dispose()
    {
        if (_hwnd != IntPtr.Zero)
        {
            WinApi.DestroyWindow(_hwnd);
            _hwnd = IntPtr.Zero;
        }
    }
}
