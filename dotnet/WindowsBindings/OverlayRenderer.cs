using System.Runtime.InteropServices;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Manages a pool of Windows overlay windows and renders DisplayOverlayState.
/// Reuses windows instead of creating/destroying them for better performance.
/// </summary>
internal class OverlayRenderer : IDisposable
{
    private const string WINDOW_CLASS_NAME = "SpotlightDimmerOverlay";
    private static bool _classRegistered = false;
    private static readonly WinApi.WndProc _wndProcDelegate = WndProc;

    // Pool of overlay windows keyed by (displayIndex, region)
    private readonly Dictionary<(int displayIndex, OverlayRegion region), OverlayWindow> _overlayPool = new();

    public OverlayRenderer()
    {
        EnsureWindowClassRegistered();
    }

    /// <summary>
    /// Updates all overlay windows based on the calculated overlay states.
    /// Creates, updates, or hides windows as needed.
    /// </summary>
    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        // Track which overlays are currently visible
        var activeKeys = new HashSet<(int, OverlayRegion)>();

        // Update or create overlays for each state
        foreach (var state in states)
        {
            foreach (var overlayDef in state.Overlays)
            {
                // Skip overlays that aren't visible to avoid unnecessary window operations
                if (!overlayDef.IsVisible)
                    continue;

                var key = (state.DisplayIndex, overlayDef.Region);
                activeKeys.Add(key);

                if (_overlayPool.TryGetValue(key, out var window))
                {
                    // Update existing window
                    window.Update(overlayDef);
                }
                else
                {
                    // Create new window
                    window = new OverlayWindow(overlayDef);
                    _overlayPool[key] = window;
                }
            }
        }

        // Hide any overlays that aren't in the current active set
        foreach (var kvp in _overlayPool)
        {
            if (!activeKeys.Contains(kvp.Key))
            {
                kvp.Value.Hide();
            }
        }
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
    /// Window procedure for handling window messages.
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
        foreach (var window in _overlayPool.Values)
        {
            window.Dispose();
        }
        _overlayPool.Clear();
    }

    /// <summary>
    /// Represents a single overlay window (wrapper around Windows HWND).
    /// </summary>
    private class OverlayWindow : IDisposable
    {
        private IntPtr _hwnd = IntPtr.Zero;
        private OverlayDefinition _currentState;

        public OverlayWindow(OverlayDefinition definition)
        {
            CreateWindow(definition);
            _currentState = definition;
        }

        /// <summary>
        /// Updates this window's state (position, color, opacity, visibility).
        /// </summary>
        public void Update(OverlayDefinition definition)
        {
            if (_hwnd == IntPtr.Zero)
            {
                CreateWindow(definition);
                _currentState = definition;
                return;
            }

            // Check what changed to minimize Windows API calls
            bool boundsChanged = definition.Bounds != _currentState.Bounds;
            bool colorOrOpacityChanged =
                definition.Color != _currentState.Color ||
                definition.Opacity != _currentState.Opacity;
            bool visibilityChanged = definition.IsVisible != _currentState.IsVisible;

            // Update position/size if needed
            if (boundsChanged)
            {
                WinApi.SetWindowLongPtr(
                    _hwnd,
                    WinApi.GWL_STYLE,
                    new IntPtr((long)(WinApi.WS_POPUP | WinApi.WS_VISIBLE))
                );

                // Use SetWindowPos for moving/resizing (more efficient than recreating)
                const int SWP_NOACTIVATE = 0x0010;
                const int SWP_NOZORDER = 0x0004;
                SetWindowPos(
                    _hwnd,
                    IntPtr.Zero,
                    definition.Bounds.X,
                    definition.Bounds.Y,
                    definition.Bounds.Width,
                    definition.Bounds.Height,
                    SWP_NOACTIVATE | SWP_NOZORDER
                );
            }

            // Update color/opacity if needed
            if (colorOrOpacityChanged)
            {
                WinApi.SetLayeredWindowAttributes(
                    _hwnd,
                    0,
                    definition.Opacity,
                    WinApi.LWA_ALPHA
                );
            }

            // Update visibility if needed
            if (visibilityChanged)
            {
                if (definition.IsVisible)
                {
                    WinApi.ShowWindow(_hwnd, 5); // SW_SHOW
                }
                else
                {
                    WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
                }
            }

            _currentState = definition;
        }

        /// <summary>
        /// Hides this overlay window.
        /// </summary>
        public void Hide()
        {
            if (_hwnd != IntPtr.Zero)
            {
                WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
                _currentState = _currentState with { IsVisible = false };
            }
        }

        /// <summary>
        /// Creates the actual overlay window.
        /// </summary>
        private void CreateWindow(OverlayDefinition definition)
        {
            _hwnd = WinApi.CreateWindowEx(
                WinApi.WS_EX_TOPMOST | WinApi.WS_EX_LAYERED | WinApi.WS_EX_TRANSPARENT | WinApi.WS_EX_TOOLWINDOW | WinApi.WS_EX_NOACTIVATE,
                WINDOW_CLASS_NAME,
                $"SpotlightDimmer Overlay - {definition.Region}",
                WinApi.WS_POPUP | WinApi.WS_VISIBLE,
                definition.Bounds.X,
                definition.Bounds.Y,
                definition.Bounds.Width,
                definition.Bounds.Height,
                IntPtr.Zero,
                IntPtr.Zero,
                WinApi.GetModuleHandle(null),
                IntPtr.Zero);

            if (_hwnd == IntPtr.Zero)
            {
                throw new InvalidOperationException($"Failed to create overlay window. Error: {Marshal.GetLastWin32Error()}");
            }

            // Set the overlay color and opacity
            WinApi.SetLayeredWindowAttributes(_hwnd, 0, definition.Opacity, WinApi.LWA_ALPHA);

            // Show or hide based on initial state
            if (definition.IsVisible)
            {
                WinApi.ShowWindow(_hwnd, 5); // SW_SHOW
            }
            else
            {
                WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
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

        // SetWindowPos P/Invoke (not in WinApi.cs yet)
        [DllImport("user32.dll", SetLastError = true)]
        private static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
    }
}
