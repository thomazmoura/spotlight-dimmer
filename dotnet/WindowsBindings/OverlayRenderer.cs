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

    // Map window handles to their overlay windows for WM_PAINT handling
    private static readonly Dictionary<IntPtr, OverlayWindow> _windowMap = new();

    public OverlayRenderer()
    {
        EnsureWindowClassRegistered();
    }

    /// <summary>
    /// Updates all overlay windows based on the calculated overlay states.
    /// Creates, updates, or hides windows as needed.
    /// Uses deferred window positioning for atomic, flicker-free batch updates.
    /// </summary>
    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        // Track which overlays are currently visible
        var activeKeys = new HashSet<(int, OverlayRegion)>();

        // Lists to collect updates for batching
        var windowsToUpdate = new List<(OverlayWindow window, OverlayDefinition definition)>();
        var windowsToCreate = new List<(int displayIndex, OverlayRegion region, OverlayDefinition definition)>();

        // First pass: collect all updates
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
                    // Queue existing window for update
                    windowsToUpdate.Add((window, overlayDef));
                }
                else
                {
                    // Queue window for creation
                    windowsToCreate.Add((state.DisplayIndex, overlayDef.Region, overlayDef));
                }
            }
        }

        // Create new windows (these need to be created before batching)
        foreach (var (displayIndex, region, definition) in windowsToCreate)
        {
            var window = new OverlayWindow(definition);
            _overlayPool[(displayIndex, region)] = window;
            // Add to update list so it gets included in the batch positioning
            windowsToUpdate.Add((window, definition));
        }

        // Batch update all windows using DeferWindowPos
        if (windowsToUpdate.Count > 0)
        {
            BatchUpdateWindows(windowsToUpdate);
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
    /// Applies updates to multiple windows atomically using DeferWindowPos.
    /// This reduces flicker by applying all position/size/visibility changes in a single operation.
    /// </summary>
    private void BatchUpdateWindows(List<(OverlayWindow window, OverlayDefinition definition)> updates)
    {
        // Begin deferred window positioning for all windows
        var hdwp = WinApi.BeginDeferWindowPos(updates.Count);
        if (hdwp == IntPtr.Zero)
        {
            // Fallback to individual updates if batch fails
            foreach (var (window, definition) in updates)
            {
                window.Update(definition);
            }
            return;
        }

        // Queue all position/size updates
        foreach (var (window, definition) in updates)
        {
            hdwp = window.DeferUpdate(hdwp, definition);
            if (hdwp == IntPtr.Zero)
            {
                // If defer fails, fall back to remaining individual updates
                foreach (var (w, d) in updates)
                {
                    w.Update(d);
                }
                return;
            }
        }

        // Apply all updates atomically
        WinApi.EndDeferWindowPos(hdwp);

        // Handle non-position updates (color, opacity) that can't be deferred
        foreach (var (window, definition) in updates)
        {
            window.UpdateNonPositionProperties(definition);
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
            hbrBackground = IntPtr.Zero, // No background brush - we'll paint manually
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
            case WinApi.WM_ERASEBKGND:
                // Return 1 to indicate we handled background erasing (prevents flicker)
                return new IntPtr(1);

            case WinApi.WM_PAINT:
                // Paint the window with the overlay's color
                if (_windowMap.TryGetValue(hWnd, out var window))
                {
                    window.Paint();
                }
                return IntPtr.Zero;

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
            _windowMap[_hwnd] = this;
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
                WinApi.SetWindowPos(
                    _hwnd,
                    IntPtr.Zero,
                    definition.Bounds.X,
                    definition.Bounds.Y,
                    definition.Bounds.Width,
                    definition.Bounds.Height,
                    WinApi.SWP_NOACTIVATE | WinApi.SWP_NOZORDER
                );
            }

            // Update color/opacity if needed
            if (colorOrOpacityChanged)
            {
                // Update opacity
                WinApi.SetLayeredWindowAttributes(
                    _hwnd,
                    0,
                    definition.Opacity,
                    WinApi.LWA_ALPHA
                );

                // Trigger repaint for color change
                if (definition.Color != _currentState.Color)
                {
                    var hdc = WinApi.GetDC(_hwnd);
                    if (hdc != IntPtr.Zero)
                    {
                        PaintWindow(hdc, definition.Color);
                        WinApi.ReleaseDC(_hwnd, hdc);
                    }
                }
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
        /// Queues a deferred window position update (for batching).
        /// Only handles position, size, and visibility. Returns updated HDWP handle.
        /// </summary>
        public IntPtr DeferUpdate(IntPtr hdwp, OverlayDefinition definition)
        {
            if (_hwnd == IntPtr.Zero)
                return hdwp;

            // Check what changed
            bool boundsChanged = definition.Bounds != _currentState.Bounds;
            bool visibilityChanged = definition.IsVisible != _currentState.IsVisible;

            // Only defer position/size/visibility changes
            if (!boundsChanged && !visibilityChanged)
                return hdwp;

            uint flags = WinApi.SWP_NOACTIVATE | WinApi.SWP_NOZORDER;

            // Handle visibility changes
            if (visibilityChanged)
            {
                flags |= definition.IsVisible ? WinApi.SWP_SHOWWINDOW : WinApi.SWP_HIDEWINDOW;
            }

            // Queue the deferred position update
            return WinApi.DeferWindowPos(
                hdwp,
                _hwnd,
                IntPtr.Zero,
                definition.Bounds.X,
                definition.Bounds.Y,
                definition.Bounds.Width,
                definition.Bounds.Height,
                flags
            );
        }

        /// <summary>
        /// Updates non-position properties (color, opacity) that can't be deferred.
        /// Call this after EndDeferWindowPos to handle color/opacity changes.
        /// </summary>
        public void UpdateNonPositionProperties(OverlayDefinition definition)
        {
            if (_hwnd == IntPtr.Zero)
                return;

            bool colorOrOpacityChanged =
                definition.Color != _currentState.Color ||
                definition.Opacity != _currentState.Opacity;

            if (colorOrOpacityChanged)
            {
                // Update opacity
                if (definition.Opacity != _currentState.Opacity)
                {
                    WinApi.SetLayeredWindowAttributes(
                        _hwnd,
                        0,
                        definition.Opacity,
                        WinApi.LWA_ALPHA
                    );
                }

                // Trigger repaint for color change
                if (definition.Color != _currentState.Color)
                {
                    var hdc = WinApi.GetDC(_hwnd);
                    if (hdc != IntPtr.Zero)
                    {
                        PaintWindow(hdc, definition.Color);
                        WinApi.ReleaseDC(_hwnd, hdc);
                    }
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

            // Set the overlay opacity
            WinApi.SetLayeredWindowAttributes(_hwnd, 0, definition.Opacity, WinApi.LWA_ALPHA);

            // Paint the window with the overlay color
            var hdc = WinApi.GetDC(_hwnd);
            if (hdc != IntPtr.Zero)
            {
                PaintWindow(hdc, definition.Color);
                WinApi.ReleaseDC(_hwnd, hdc);
            }

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

        /// <summary>
        /// Handles WM_PAINT message to paint the window with the overlay color.
        /// </summary>
        public void Paint()
        {
            var hdc = WinApi.BeginPaint(_hwnd, out var ps);
            if (hdc != IntPtr.Zero)
            {
                PaintWindow(hdc, _currentState.Color);
                WinApi.EndPaint(_hwnd, ref ps);
            }
        }

        /// <summary>
        /// Paints the entire window with the specified color.
        /// </summary>
        private void PaintWindow(IntPtr hdc, Core.Color color)
        {
            var rect = new WinApi.RECT
            {
                Left = 0,
                Top = 0,
                Right = _currentState.Bounds.Width,
                Bottom = _currentState.Bounds.Height
            };

            var brush = WinApi.CreateSolidBrush(WinApi.ToWindowsRgb(color));
            WinApi.FillRect(hdc, ref rect, brush);
            WinApi.DeleteObject(brush);
        }

        public void Dispose()
        {
            if (_hwnd != IntPtr.Zero)
            {
                _windowMap.Remove(_hwnd);
                WinApi.DestroyWindow(_hwnd);
                _hwnd = IntPtr.Zero;
            }
        }
    }
}
