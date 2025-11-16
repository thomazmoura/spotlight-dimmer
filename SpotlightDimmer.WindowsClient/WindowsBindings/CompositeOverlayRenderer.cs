using System.Runtime.InteropServices;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Renderer that uses only 2 windows per display (fullscreen + partial) instead of 6.
/// Leverages per-pixel alpha compositing to draw multiple overlay regions into a single bitmap.
///
/// Advantages:
/// - Fewer GDI handles (2 windows per display vs 6)
/// - No window resize/reposition operations (windows stay fullscreen)
/// - Simplified window management
///
/// Architecture:
/// - Fullscreen window: Used in FullScreen mode, contains single solid rectangle
/// - Partial window: Used in Partial/PartialWithActive modes, contains up to 5 drawn regions
/// </summary>
public sealed class CompositeOverlayRenderer : IOverlayRenderer
{
    private DisplayOverlays[]? _displayOverlays;
    private bool _disposed;

    public void CreateOverlays(Core.DisplayInfo[] displays, OverlayCalculationConfig config)
    {
        if (_disposed) throw new ObjectDisposedException(nameof(CompositeOverlayRenderer));

        // Clean up any existing overlays
        CleanupOverlays();

        _displayOverlays = new DisplayOverlays[displays.Length];

        for (int i = 0; i < displays.Length; i++)
        {
            var display = displays[i];
            _displayOverlays[i] = new DisplayOverlays(display, config);
        }
    }

    public void UpdateBrushColors(OverlayCalculationConfig config)
    {
        if (_disposed) throw new ObjectDisposedException(nameof(CompositeOverlayRenderer));
        if (_displayOverlays == null) return;

        foreach (var displayOverlay in _displayOverlays)
        {
            displayOverlay.FullscreenOverlay.UpdateColors(config);
            displayOverlay.PartialOverlay.UpdateColors(config);
        }
    }

    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        if (_disposed) throw new ObjectDisposedException(nameof(CompositeOverlayRenderer));
        if (_displayOverlays == null) return;

        for (int i = 0; i < states.Length; i++)
        {
            _displayOverlays[i].Update(states[i]);
        }
    }

    public int UpdateScreenCaptureExclusion(bool exclude)
    {
        if (_disposed) throw new ObjectDisposedException(nameof(CompositeOverlayRenderer));
        if (_displayOverlays == null) return 0;

        int successCount = 0;
        foreach (var displayOverlay in _displayOverlays)
        {
            if (displayOverlay.FullscreenOverlay.UpdateScreenCaptureExclusion(exclude))
                successCount++;
            if (displayOverlay.PartialOverlay.UpdateScreenCaptureExclusion(exclude))
                successCount++;
        }
        return successCount;
    }

    public void HideAllOverlays()
    {
        if (_disposed) throw new ObjectDisposedException(nameof(CompositeOverlayRenderer));
        if (_displayOverlays == null) return;

        foreach (var displayOverlay in _displayOverlays)
        {
            displayOverlay.FullscreenOverlay.Hide();
            displayOverlay.PartialOverlay.Hide();
        }
    }

    public void CleanupOverlays()
    {
        if (_displayOverlays == null) return;

        foreach (var displayOverlay in _displayOverlays)
        {
            displayOverlay.FullscreenOverlay.Dispose();
            displayOverlay.PartialOverlay.Dispose();
        }

        _displayOverlays = null;
    }

    public void Dispose()
    {
        if (_disposed) return;

        CleanupOverlays();
        _disposed = true;
    }

    /// <summary>
    /// Contains the 2 overlay windows for a single display
    /// </summary>
    private struct DisplayOverlays
    {
        public CompositeOverlay FullscreenOverlay;
        public CompositeOverlay PartialOverlay;

        public DisplayOverlays(Core.DisplayInfo display, OverlayCalculationConfig config)
        {
            FullscreenOverlay = new CompositeOverlay(display, isPartialMode: false, config);
            PartialOverlay = new CompositeOverlay(display, isPartialMode: true, config);
        }

        public void Update(DisplayOverlayState state)
        {
            // Determine which overlay mode is active based on visible regions
            bool hasFullscreen = state.Overlays[(int)OverlayRegion.FullScreen].IsVisible;
            bool hasPartialRegions = state.Overlays[(int)OverlayRegion.Top].IsVisible ||
                                     state.Overlays[(int)OverlayRegion.Bottom].IsVisible ||
                                     state.Overlays[(int)OverlayRegion.Left].IsVisible ||
                                     state.Overlays[(int)OverlayRegion.Right].IsVisible ||
                                     state.Overlays[(int)OverlayRegion.Center].IsVisible;

            if (hasFullscreen)
            {
                // FullScreen mode: show fullscreen overlay, hide partial
                FullscreenOverlay.Update(state.Overlays[(int)OverlayRegion.FullScreen]);
                PartialOverlay.Hide();
            }
            else if (hasPartialRegions)
            {
                // Partial/PartialWithActive mode: hide fullscreen, show partial with multiple regions
                FullscreenOverlay.Hide();
                PartialOverlay.UpdateMultipleRegions(state.Overlays);
            }
            else
            {
                // No overlays visible: hide both windows
                FullscreenOverlay.Hide();
                PartialOverlay.Hide();
            }
        }
    }

    /// <summary>
    /// A single fullscreen overlay window that can contain one or more drawn regions.
    /// Uses UpdateLayeredWindow with a DIB bitmap for per-pixel alpha compositing.
    /// </summary>
    private sealed class CompositeOverlay : IDisposable
    {
        private readonly IntPtr _hwnd;
        private readonly Core.DisplayInfo _display;
        private readonly bool _isPartialMode; // True for partial window, false for fullscreen window

        private IntPtr _memoryDc;
        private IntPtr _bitmap;
        private IntPtr _bitmapBits;
        private int _screenWidth;
        private int _screenHeight;

        // Color cache to avoid repeated brush creation
        private Core.Color _activeColor;
        private Core.Color _inactiveColor;

        // Track current state to detect changes
        private bool _isVisible;

        public CompositeOverlay(Core.DisplayInfo display, bool isPartialMode, OverlayCalculationConfig config)
        {
            _display = display;
            _isPartialMode = isPartialMode;
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            // Create overlay window (always fullscreen-sized)
            _hwnd = CreateOverlayWindow(display);
            _memoryDc = WinApi.CreateCompatibleDC(IntPtr.Zero);

            // Create bitmap matching display dimensions
            _screenWidth = display.Bounds.Width;
            _screenHeight = display.Bounds.Height;
            CreateBitmap(_screenWidth, _screenHeight);
        }

        private IntPtr CreateOverlayWindow(Core.DisplayInfo display)
        {
            const int WS_EX_LAYERED = 0x00080000;
            const int WS_EX_TRANSPARENT = 0x00000020;
            const int WS_EX_TOPMOST = 0x00000008;
            const int WS_EX_NOACTIVATE = 0x08000000;
            const int WS_POPUP = unchecked((int)0x80000000);
            const int WS_VISIBLE = 0x10000000;

            // IMPORTANT: Create window with WS_VISIBLE to avoid needing ShowWindow() later.
            // Calling ShowWindow() breaks WDA_EXCLUDEFROMCAPTURE (see Hide() method comments).
            IntPtr hwnd = WinApi.CreateWindowEx(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                "Static",
                "SpotlightDimmer Composite Overlay",
                unchecked((uint)(WS_POPUP | WS_VISIBLE)),
                display.Bounds.X,
                display.Bounds.Y,
                display.Bounds.Width,
                display.Bounds.Height,
                IntPtr.Zero,
                IntPtr.Zero,
                IntPtr.Zero,
                IntPtr.Zero
            );

            if (hwnd == IntPtr.Zero)
            {
                throw new InvalidOperationException($"Failed to create overlay window. Error: {Marshal.GetLastWin32Error()}");
            }

            return hwnd;
        }

        private void CreateBitmap(int width, int height)
        {
            // Clean up old bitmap if it exists
            if (_bitmap != IntPtr.Zero)
            {
                WinApi.DeleteObject(_bitmap);
                _bitmap = IntPtr.Zero;
                _bitmapBits = IntPtr.Zero;
            }

            // Create 32-bit ARGB DIB section
            WinApi.BITMAPINFO bmi = new()
            {
                bmiHeader = new WinApi.BITMAPINFOHEADER
                {
                    biSize = (uint)Marshal.SizeOf<WinApi.BITMAPINFOHEADER>(),
                    biWidth = width,
                    biHeight = -height, // Negative = top-down DIB
                    biPlanes = 1,
                    biBitCount = 32,
                    biCompression = 0, // BI_RGB
                    biSizeImage = 0,
                    biXPelsPerMeter = 0,
                    biYPelsPerMeter = 0,
                    biClrUsed = 0,
                    biClrImportant = 0
                }
            };

            _bitmap = WinApi.CreateDIBSection(
                _memoryDc,
                ref bmi,
                0, // DIB_RGB_COLORS
                out _bitmapBits,
                IntPtr.Zero,
                0
            );

            if (_bitmap == IntPtr.Zero)
            {
                throw new InvalidOperationException($"Failed to create DIB section. Error: {Marshal.GetLastWin32Error()}");
            }

            WinApi.SelectObject(_memoryDc, _bitmap);
        }

        public void UpdateColors(OverlayCalculationConfig config)
        {
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            // Force redraw if visible
            if (_isVisible)
            {
                // This is called during config changes, actual redraw will happen
                // on next UpdateOverlays call from Program.cs
            }
        }

        /// <summary>
        /// Update for fullscreen mode - single solid rectangle
        /// </summary>
        public void Update(OverlayDefinition overlay)
        {
            if (!overlay.IsVisible)
            {
                Hide();
                return;
            }

            // For fullscreen mode, fill entire bitmap with single color
            ClearBitmap();
            FillRectangle(
                0, 0, _screenWidth, _screenHeight,
                overlay.Color, overlay.Opacity
            );

            UpdateWindow();
            _isVisible = true;
        }

        /// <summary>
        /// Update for partial mode - multiple drawn regions
        /// </summary>
        public void UpdateMultipleRegions(ReadOnlySpan<OverlayDefinition> overlays)
        {
            // Clear bitmap to transparent
            ClearBitmap();

            bool anyVisible = false;

            // Draw each visible region
            for (int i = 0; i < overlays.Length; i++)
            {
                var overlay = overlays[i];
                if (!overlay.IsVisible) continue;

                // Convert from display-absolute coordinates to window-relative (since window is at display origin)
                int relativeX = overlay.Bounds.X - _display.Bounds.X;
                int relativeY = overlay.Bounds.Y - _display.Bounds.Y;

                FillRectangle(
                    relativeX,
                    relativeY,
                    overlay.Bounds.Width,
                    overlay.Bounds.Height,
                    overlay.Color,
                    overlay.Opacity
                );

                anyVisible = true;
            }

            if (anyVisible)
            {
                UpdateWindow();
                _isVisible = true;
            }
            else
            {
                Hide();
            }
        }

        /// <summary>
        /// Clears the bitmap to fully transparent (ARGB: 0x00000000)
        /// </summary>
        private unsafe void ClearBitmap()
        {
            if (_bitmapBits == IntPtr.Zero) return;

            uint* pixels = (uint*)_bitmapBits;
            int pixelCount = _screenWidth * _screenHeight;

            // Fast clear to transparent
            for (int i = 0; i < pixelCount; i++)
            {
                pixels[i] = 0x00000000;
            }
        }

        /// <summary>
        /// Fills a rectangle in the bitmap with the specified color and opacity.
        /// Uses unsafe pointer manipulation for performance.
        /// </summary>
        private unsafe void FillRectangle(int x, int y, int width, int height, Core.Color color, byte opacity)
        {
            if (_bitmapBits == IntPtr.Zero) return;

            // Clamp rectangle to bitmap bounds
            if (x < 0) { width += x; x = 0; }
            if (y < 0) { height += y; y = 0; }
            if (x + width > _screenWidth) width = _screenWidth - x;
            if (y + height > _screenHeight) height = _screenHeight - y;

            if (width <= 0 || height <= 0) return;

            // Pre-multiply alpha for correct blending
            // Format: 0xAARRGGBB (big-endian in memory due to little-endian architecture)
            uint argb = ((uint)opacity << 24) |
                        ((uint)((color.R * opacity) / 255) << 16) |
                        ((uint)((color.G * opacity) / 255) << 8) |
                        ((uint)((color.B * opacity) / 255));

            uint* pixels = (uint*)_bitmapBits;

            for (int row = 0; row < height; row++)
            {
                int rowStart = (y + row) * _screenWidth + x;
                for (int col = 0; col < width; col++)
                {
                    pixels[rowStart + col] = argb;
                }
            }
        }

        /// <summary>
        /// Updates the window with the current bitmap contents
        /// </summary>
        private void UpdateWindow()
        {
            IntPtr screenDc = WinApi.GetDC(IntPtr.Zero);
            try
            {
                WinApi.POINT windowPos = new() { X = _display.Bounds.X, Y = _display.Bounds.Y };
                WinApi.SIZE windowSize = new() { cx = _screenWidth, cy = _screenHeight };
                WinApi.POINT sourcePos = new() { X = 0, Y = 0 };
                WinApi.BLENDFUNCTION blend = new()
                {
                    BlendOp = 0, // AC_SRC_OVER
                    BlendFlags = 0,
                    SourceConstantAlpha = 255, // Use per-pixel alpha from bitmap
                    AlphaFormat = 1 // AC_SRC_ALPHA
                };

                const uint ULW_ALPHA = 0x00000002;

                bool success = WinApi.UpdateLayeredWindow(
                    _hwnd,
                    screenDc,
                    ref windowPos,
                    ref windowSize,
                    _memoryDc,
                    ref sourcePos,
                    0,
                    ref blend,
                    ULW_ALPHA
                );

                if (!success)
                {
                    int error = Marshal.GetLastWin32Error();
                    Console.WriteLine($"UpdateLayeredWindow failed. Error: {error}");
                }
                // Note: Window is created with WS_VISIBLE, so no ShowWindow() call needed.
                // UpdateLayeredWindow positions the window via windowPos parameter (line 428).
                // Using ShowWindow() would break WDA_EXCLUDEFROMCAPTURE (see Hide() method comments).
            }
            finally
            {
                WinApi.ReleaseDC(IntPtr.Zero, screenDc);
            }
        }

        public void Hide()
        {
            if (!_isVisible) return;

            // IMPORTANT: Do NOT use ShowWindow(SW_HIDE) when WDA_EXCLUDEFROMCAPTURE is enabled!
            // After hide/show, Windows breaks WDA_EXCLUDEFROMCAPTURE and falls back to WDA_MONITOR (black screen).
            // Workaround: Move window off-screen instead of hiding it.
            // See: Electron issue #29085 and fix in PR #31340
            const uint SWP_NOACTIVATE = 0x0010;
            const uint SWP_NOZORDER = 0x0004;
            WinApi.SetWindowPos(_hwnd, IntPtr.Zero, -32000, -32000, 0, 0,
                SWP_NOACTIVATE | SWP_NOZORDER);
            _isVisible = false;
        }

        /// <summary>
        /// Sets the display affinity for this window to control screen capture behavior.
        /// Returns true if successful, false otherwise (e.g., due to Windows API limitations).
        /// </summary>
        public bool UpdateScreenCaptureExclusion(bool exclude)
        {
            if (_hwnd == IntPtr.Zero)
                return false;

            uint affinity = exclude ? WinApi.WDA_EXCLUDEFROMCAPTURE : WinApi.WDA_NONE;
            bool result = WinApi.SetWindowDisplayAffinity(_hwnd, affinity);

            // Note: We don't log here to avoid polluting logs with repeated messages
            // The caller (CompositeOverlayRenderer) will aggregate results and log appropriately
            return result;
        }

        public void Dispose()
        {
            // Clean up in reverse order of creation
            if (_bitmap != IntPtr.Zero)
            {
                WinApi.DeleteObject(_bitmap);
                _bitmap = IntPtr.Zero;
            }

            if (_memoryDc != IntPtr.Zero)
            {
                WinApi.DeleteDC(_memoryDc);
                _memoryDc = IntPtr.Zero;
            }

            if (_hwnd != IntPtr.Zero)
            {
                WinApi.DestroyWindow(_hwnd);
            }
        }
    }
}
