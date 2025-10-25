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

    // Pre-allocated list for batching updates (reused every frame to avoid allocations)
    private readonly List<(OverlayWindow window, OverlayDefinition definition)> _updateBatch = new();

    public OverlayRenderer()
    {
        EnsureWindowClassRegistered();
    }

    /// <summary>
    /// Pre-creates all overlay windows for the given displays.
    /// Creates 6 windows per display (one for each OverlayRegion), all initially hidden.
    /// Pre-allocates brushes for the configured colors to eliminate any allocations during updates.
    /// Call this once at startup after getting display information.
    /// </summary>
    public void CreateOverlays(Core.DisplayInfo[] displays, OverlayCalculationConfig config)
    {
        foreach (var display in displays)
        {
            // Create one window for each region (6 total per display)
            for (int i = 0; i < 6; i++)
            {
                var region = (OverlayRegion)i;
                var key = (display.Index, region);

                // Create window with pre-allocated brushes for both active and inactive colors
                var window = new OverlayWindow(region, display.Bounds, config);
                _overlayPool[key] = window;
            }
        }
    }

    /// <summary>
    /// Updates all brush colors when configuration changes.
    /// Recreates brushes with new colors to reflect updated config.
    /// </summary>
    public void UpdateBrushColors(OverlayCalculationConfig config)
    {
        foreach (var window in _overlayPool.Values)
        {
            window.UpdateBrushColors(config);
        }
    }

    /// <summary>
    /// Updates all overlay windows based on the calculated overlay states.
    /// All windows are pre-created, so this only updates their state.
    /// Uses deferred window positioning for atomic, flicker-free batch updates.
    /// ZERO allocations - reuses pre-allocated batch list.
    /// </summary>
    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        // Clear the pre-allocated batch list (no allocation)
        _updateBatch.Clear();

        // Collect all windows that need updates
        // Process ALL overlays (visible and invisible) to properly update state
        foreach (var state in states)
        {
            foreach (var sourceOverlay in state.Overlays)
            {
                var key = (state.DisplayIndex, sourceOverlay.Region);

                if (_overlayPool.TryGetValue(key, out var window))
                {
                    // Add to batch - window will copy values from sourceOverlay
                    _updateBatch.Add((window, sourceOverlay));
                }
                // Note: If window doesn't exist, it means CreateOverlays wasn't called
                // or display configuration changed. We silently skip it.
            }
        }

        // Batch update all windows using DeferWindowPos
        if (_updateBatch.Count > 0)
        {
            BatchUpdateWindows(_updateBatch);
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

        // Track the last valid handle to prevent leaks on failure
        var lastValidHdwp = hdwp;

        // Queue all position/size updates
        foreach (var (window, definition) in updates)
        {
            hdwp = window.DeferUpdate(hdwp, definition);
            if (hdwp == IntPtr.Zero)
            {
                // CRITICAL: Clean up the last valid handle to prevent leak
                // While Microsoft docs suggest not calling EndDeferWindowPos on failure,
                // failing to do so causes a handle leak. This is a well-documented issue:
                // "Simply returning without calling EndDeferWindowPos will leak a handle"
                WinApi.EndDeferWindowPos(lastValidHdwp);

                // Fall back to remaining individual updates
                foreach (var (w, d) in updates)
                {
                    w.Update(d);
                }
                return;
            }
            // Update last valid handle for next iteration
            lastValidHdwp = hdwp;
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
            hCursor = WinApi.LoadCursor(IntPtr.Zero, new IntPtr(32512)) // IDC_ARROW
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

    /// <summary>
    /// Hides all overlay windows (for pause functionality).
    /// </summary>
    public void HideAllOverlays()
    {
        foreach (var window in _overlayPool.Values)
        {
            window.Hide();
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
    /// Each window owns its local OverlayDefinition to prevent reference aliasing bugs.
    /// </summary>
    private class OverlayWindow : IDisposable
    {
        private IntPtr _hwnd = IntPtr.Zero;
        private OverlayDefinition _localState; // Our own copy - never shared with source
        private IntPtr _activeBrush = IntPtr.Zero;   // Pre-allocated brush for active color
        private IntPtr _inactiveBrush = IntPtr.Zero; // Pre-allocated brush for inactive color
        private Core.Color _activeColor;   // Track active color for brush selection
        private Core.Color _inactiveColor; // Track inactive color for brush selection

        /// <summary>
        /// Creates a new overlay window with its own local state.
        /// Pre-allocates brushes for both active and inactive colors.
        /// The window is created hidden with minimal initial bounds.
        /// </summary>
        public OverlayWindow(OverlayRegion region, Core.Rectangle displayBounds, OverlayCalculationConfig config)
        {
            // Create our own local OverlayDefinition (initially hidden)
            _localState = new OverlayDefinition(region);

            // Store colors for brush selection
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            // Pre-create brushes for both active and inactive colors
            _activeBrush = WinApi.CreateSolidBrush(WinApi.ToWindowsRgb(config.ActiveColor));
            _inactiveBrush = WinApi.CreateSolidBrush(WinApi.ToWindowsRgb(config.InactiveColor));

            // Create the window HWND (initially hidden with minimal size)
            CreateWindow(region, displayBounds);
            _windowMap[_hwnd] = this;
        }

        /// <summary>
        /// Updates brushes when configuration colors change.
        /// Deletes old brushes and creates new ones.
        /// </summary>
        public void UpdateBrushColors(OverlayCalculationConfig config)
        {
            // Delete old brushes
            if (_activeBrush != IntPtr.Zero)
            {
                WinApi.DeleteObject(_activeBrush);
            }
            if (_inactiveBrush != IntPtr.Zero)
            {
                WinApi.DeleteObject(_inactiveBrush);
            }

            // Update stored colors
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            // Create new brushes with updated colors
            _activeBrush = WinApi.CreateSolidBrush(WinApi.ToWindowsRgb(config.ActiveColor));
            _inactiveBrush = WinApi.CreateSolidBrush(WinApi.ToWindowsRgb(config.InactiveColor));

            // Trigger repaint if window is visible to show new colors
            if (_localState.IsVisible)
            {
                var hdc = WinApi.GetDC(_hwnd);
                if (hdc != IntPtr.Zero)
                {
                    PaintWindow(hdc, _localState.Color, _localState.Bounds);
                    WinApi.ReleaseDC(_hwnd, hdc);
                }
            }
        }

        /// <summary>
        /// Updates this window's state from a source OverlayDefinition.
        /// Copies values from source to local state without creating new objects.
        /// </summary>
        public void Update(OverlayDefinition source)
        {
            if (_hwnd == IntPtr.Zero)
            {
                // Window should already exist - this shouldn't happen
                return;
            }

            // Check what changed by comparing source with our local state
            bool boundsChanged = source.Bounds != _localState.Bounds;
            bool colorOrOpacityChanged =
                source.Color != _localState.Color ||
                source.Opacity != _localState.Opacity;
            bool visibilityChanged = source.IsVisible != _localState.IsVisible;

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
                    source.Bounds.X,
                    source.Bounds.Y,
                    source.Bounds.Width,
                    source.Bounds.Height,
                    WinApi.SWP_NOACTIVATE | WinApi.SWP_NOZORDER
                );
            }

            // Update color/opacity if needed
            if (colorOrOpacityChanged)
            {
                // Update opacity
                if (source.Opacity != _localState.Opacity)
                {
                    WinApi.SetLayeredWindowAttributes(
                        _hwnd,
                        0,
                        source.Opacity,
                        WinApi.LWA_ALPHA
                    );
                }

                // Trigger repaint for color change (only if visible)
                if (source.Color != _localState.Color && source.IsVisible)
                {
                    var hdc = WinApi.GetDC(_hwnd);
                    if (hdc != IntPtr.Zero)
                    {
                        PaintWindow(hdc, source.Color, source.Bounds);
                        WinApi.ReleaseDC(_hwnd, hdc);
                    }
                }
            }

            // Update visibility if needed
            if (visibilityChanged)
            {
                if (source.IsVisible)
                {
                    WinApi.ShowWindow(_hwnd, 5); // SW_SHOW
                }
                else
                {
                    WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
                }
            }

            // Copy source values to our local state (NO allocation)
            _localState.CopyFrom(source);
        }

        /// <summary>
        /// Queues a deferred window position update (for batching).
        /// Only handles position, size, and visibility. Returns updated HDWP handle.
        /// Does NOT update local state yet - call UpdateNonPositionProperties after batching.
        /// </summary>
        public IntPtr DeferUpdate(IntPtr hdwp, OverlayDefinition source)
        {
            if (_hwnd == IntPtr.Zero)
                return hdwp;

            // Check what changed by comparing source with our local state
            bool boundsChanged = source.Bounds != _localState.Bounds;
            bool visibilityChanged = source.IsVisible != _localState.IsVisible;

            // Only defer position/size/visibility changes
            if (!boundsChanged && !visibilityChanged)
                return hdwp;

            uint flags = WinApi.SWP_NOACTIVATE | WinApi.SWP_NOZORDER;

            // Handle visibility changes
            if (visibilityChanged)
            {
                flags |= source.IsVisible ? WinApi.SWP_SHOWWINDOW : WinApi.SWP_HIDEWINDOW;
            }

            // Queue the deferred position update
            return WinApi.DeferWindowPos(
                hdwp,
                _hwnd,
                IntPtr.Zero,
                source.Bounds.X,
                source.Bounds.Y,
                source.Bounds.Width,
                source.Bounds.Height,
                flags
            );
        }

        /// <summary>
        /// Updates non-position properties (color, opacity) that can't be deferred.
        /// Call this after EndDeferWindowPos to handle color/opacity changes.
        /// Copies source values to local state after applying changes.
        /// </summary>
        public void UpdateNonPositionProperties(OverlayDefinition source)
        {
            if (_hwnd == IntPtr.Zero)
                return;

            bool colorOrOpacityChanged =
                source.Color != _localState.Color ||
                source.Opacity != _localState.Opacity;

            if (colorOrOpacityChanged)
            {
                // Update opacity
                if (source.Opacity != _localState.Opacity)
                {
                    WinApi.SetLayeredWindowAttributes(
                        _hwnd,
                        0,
                        source.Opacity,
                        WinApi.LWA_ALPHA
                    );
                }

                // Trigger repaint for color change (only if visible)
                if (source.Color != _localState.Color && source.IsVisible)
                {
                    var hdc = WinApi.GetDC(_hwnd);
                    if (hdc != IntPtr.Zero)
                    {
                        PaintWindow(hdc, source.Color, source.Bounds);
                        WinApi.ReleaseDC(_hwnd, hdc);
                    }
                }
            }

            // Copy source values to our local state (NO allocation)
            _localState.CopyFrom(source);
        }

        /// <summary>
        /// Hides this overlay window by updating both the HWND and local state.
        /// Safe to call - only mutates our local state, not the source.
        /// </summary>
        public void Hide()
        {
            if (_hwnd != IntPtr.Zero)
            {
                WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
                _localState.IsVisible = false; // Update our local state
            }
        }

        /// <summary>
        /// Creates the actual overlay window HWND.
        /// Window is created hidden with minimal initial size.
        /// </summary>
        private void CreateWindow(OverlayRegion region, Core.Rectangle displayBounds)
        {
            // Create window hidden at display origin with minimal size
            // It will be properly positioned/sized on first Update call
            _hwnd = WinApi.CreateWindowEx(
                WinApi.WS_EX_TOPMOST | WinApi.WS_EX_LAYERED | WinApi.WS_EX_TRANSPARENT | WinApi.WS_EX_TOOLWINDOW | WinApi.WS_EX_NOACTIVATE,
                WINDOW_CLASS_NAME,
                $"SpotlightDimmer Overlay - {region}",
                WinApi.WS_POPUP, // Initially hidden
                displayBounds.X,
                displayBounds.Y,
                1, // Minimal initial size
                1,
                IntPtr.Zero,
                IntPtr.Zero,
                WinApi.GetModuleHandle(null),
                IntPtr.Zero);

            if (_hwnd == IntPtr.Zero)
            {
                throw new InvalidOperationException($"Failed to create overlay window. Error: {Marshal.GetLastWin32Error()}");
            }

            // Set initial opacity to 0 (will be updated on first Update call)
            WinApi.SetLayeredWindowAttributes(_hwnd, 0, 0, WinApi.LWA_ALPHA);

            // Keep window hidden initially
            WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
        }

        /// <summary>
        /// Handles WM_PAINT message to paint the window with the overlay color.
        /// </summary>
        public void Paint()
        {
            var hdc = WinApi.BeginPaint(_hwnd, out var ps);
            if (hdc != IntPtr.Zero)
            {
                PaintWindow(hdc, _localState.Color, _localState.Bounds);
                WinApi.EndPaint(_hwnd, ref ps);
            }
        }

        /// <summary>
        /// Paints the entire window with the specified color.
        /// Uses pre-allocated brushes - zero allocations.
        /// </summary>
        private void PaintWindow(IntPtr hdc, Core.Color color, Core.Rectangle bounds)
        {
            var rect = new WinApi.RECT
            {
                Left = 0,
                Top = 0,
                Right = bounds.Width,
                Bottom = bounds.Height
            };

            // Use pre-allocated brush based on color
            // This is just a fast struct comparison (3 bytes), no allocations
            IntPtr brush = GetBrushForColor(color);

            WinApi.FillRect(hdc, ref rect, brush);
        }

        /// <summary>
        /// Gets the pre-allocated brush for the specified color.
        /// Fast struct comparison (3 bytes) - zero allocations.
        /// </summary>
        private IntPtr GetBrushForColor(Core.Color color)
        {
            // Fast equality check - Color is a readonly record struct
            // This compiles to a simple 3-byte comparison
            if (color == _activeColor)
            {
                return _activeBrush;
            }

            // Default to inactive brush (most overlays use this)
            return _inactiveBrush;
        }

        public void Dispose()
        {
            // Clean up pre-allocated brushes
            if (_activeBrush != IntPtr.Zero)
            {
                WinApi.DeleteObject(_activeBrush);
                _activeBrush = IntPtr.Zero;
            }
            if (_inactiveBrush != IntPtr.Zero)
            {
                WinApi.DeleteObject(_inactiveBrush);
                _inactiveBrush = IntPtr.Zero;
            }

            // Clean up window
            if (_hwnd != IntPtr.Zero)
            {
                _windowMap.Remove(_hwnd);
                WinApi.DestroyWindow(_hwnd);
                _hwnd = IntPtr.Zero;
            }
        }
    }
}
