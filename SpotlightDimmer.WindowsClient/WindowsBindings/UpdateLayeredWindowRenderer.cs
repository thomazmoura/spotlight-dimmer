using System.Runtime.InteropServices;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Renderer using UpdateLayeredWindow API for atomic position+size+content updates.
/// More efficient than Legacy renderer - updates position, size, and bitmap in a single operation.
/// Uses WS_EX_LAYERED windows with DIB (Device Independent Bitmap) rendering.
/// Should reduce resize lag compared to SetWindowPos approach.
/// </summary>
internal class UpdateLayeredWindowRenderer : IOverlayRenderer
{
    private const string WINDOW_CLASS_NAME = "SpotlightDimmerOverlayULW";
    private static bool _classRegistered = false;
    private static readonly WinApi.WndProc _wndProcDelegate = WndProc;

    // Pool of overlay windows keyed by (displayIndex, region)
    private readonly Dictionary<(int displayIndex, OverlayRegion region), LayeredOverlay> _overlayPool = new();

    public UpdateLayeredWindowRenderer()
    {
        EnsureWindowClassRegistered();
    }

    public void CreateOverlays(Core.DisplayInfo[] displays, OverlayCalculationConfig config)
    {
        foreach (var display in displays)
        {
            // Create one window for each region (6 total per display)
            for (int i = 0; i < 6; i++)
            {
                var region = (OverlayRegion)i;
                var key = (display.Index, region);

                // Create window with pre-allocated bitmap
                var overlay = new LayeredOverlay(region, display.Bounds, config);
                _overlayPool[key] = overlay;
            }
        }
    }

    public void UpdateBrushColors(OverlayCalculationConfig config)
    {
        foreach (var overlay in _overlayPool.Values)
        {
            overlay.UpdateColors(config);
        }
    }

    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        foreach (var state in states)
        {
            foreach (var sourceOverlay in state.Overlays)
            {
                var key = (state.DisplayIndex, sourceOverlay.Region);

                if (_overlayPool.TryGetValue(key, out var overlay))
                {
                    overlay.Update(sourceOverlay);
                }
            }
        }
    }

    public int UpdateScreenCaptureExclusion(bool exclude)
    {
        uint affinity = exclude ? WinApi.WDA_EXCLUDEFROMCAPTURE : WinApi.WDA_NONE;
        int successCount = 0;

        foreach (var overlay in _overlayPool.Values)
        {
            if (overlay.SetDisplayAffinity(affinity))
            {
                successCount++;
            }
        }

        return successCount;
    }

    public void HideAllOverlays()
    {
        foreach (var overlay in _overlayPool.Values)
        {
            overlay.Hide();
        }
    }

    public void CleanupOverlays()
    {
        foreach (var overlay in _overlayPool.Values)
        {
            overlay.Dispose();
        }
        _overlayPool.Clear();
    }

    public void Dispose()
    {
        CleanupOverlays();
    }

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
            hCursor = WinApi.LoadCursor(IntPtr.Zero, new IntPtr(32512)) // IDC_ARROW
        };

        var atom = WinApi.RegisterClassEx(wndClass);
        if (atom == 0)
        {
            throw new InvalidOperationException($"Failed to register window class. Error: {Marshal.GetLastWin32Error()}");
        }

        _classRegistered = true;
    }

    private static IntPtr WndProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        switch (msg)
        {
            case WinApi.WM_ERASEBKGND:
                return new IntPtr(1); // Handled

            case WinApi.WM_DESTROY:
            case WinApi.WM_CLOSE:
                return IntPtr.Zero;

            default:
                return WinApi.DefWindowProc(hWnd, msg, wParam, lParam);
        }
    }

    /// <summary>
    /// Represents a single layered overlay using UpdateLayeredWindow API.
    /// Owns a DIB (Device Independent Bitmap) that is reused across updates.
    /// </summary>
    private class LayeredOverlay : IDisposable
    {
        private IntPtr _hwnd = IntPtr.Zero;
        private IntPtr _memoryDc = IntPtr.Zero;
        private IntPtr _bitmap = IntPtr.Zero;
        private IntPtr _bitmapBits = IntPtr.Zero;
        private int _bitmapWidth = 0;
        private int _bitmapHeight = 0;

        private OverlayDefinition _localState;
        private Core.Color _activeColor;
        private Core.Color _inactiveColor;

        public LayeredOverlay(OverlayRegion region, Core.Rectangle displayBounds, OverlayCalculationConfig config)
        {
            _localState = new OverlayDefinition(region);
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            CreateWindow(region, displayBounds);
            CreateBitmap(1, 1); // Start with minimal bitmap
        }

        private void CreateWindow(OverlayRegion region, Core.Rectangle displayBounds)
        {
            _hwnd = WinApi.CreateWindowEx(
                WinApi.WS_EX_TOPMOST | WinApi.WS_EX_LAYERED | WinApi.WS_EX_TRANSPARENT | WinApi.WS_EX_TOOLWINDOW | WinApi.WS_EX_NOACTIVATE,
                WINDOW_CLASS_NAME,
                $"SpotlightDimmer Overlay ULW - {region}",
                WinApi.WS_POPUP,
                displayBounds.X,
                displayBounds.Y,
                1,
                1,
                IntPtr.Zero,
                IntPtr.Zero,
                WinApi.GetModuleHandle(null),
                IntPtr.Zero);

            if (_hwnd == IntPtr.Zero)
            {
                throw new InvalidOperationException($"Failed to create overlay window. Error: {Marshal.GetLastWin32Error()}");
            }
        }

        private void CreateBitmap(int width, int height)
        {
            // Clean up existing bitmap if any
            if (_bitmap != IntPtr.Zero)
            {
                WinApi.DeleteObject(_bitmap);
                _bitmap = IntPtr.Zero;
            }

            if (_memoryDc != IntPtr.Zero)
            {
                WinApi.DeleteDC(_memoryDc);
            }

            // Create memory DC
            var screenDc = WinApi.GetDC(IntPtr.Zero);
            _memoryDc = WinApi.CreateCompatibleDC(screenDc);
            WinApi.ReleaseDC(IntPtr.Zero, screenDc);

            // Create DIB section for 32-bit ARGB bitmap
            var bmi = new WinApi.BITMAPINFO
            {
                bmiHeader = new WinApi.BITMAPINFOHEADER
                {
                    biSize = (uint)Marshal.SizeOf<WinApi.BITMAPINFOHEADER>(),
                    biWidth = width,
                    biHeight = -height, // Negative for top-down DIB
                    biPlanes = 1,
                    biBitCount = 32, // 32-bit ARGB
                    biCompression = WinApi.BI_RGB,
                    biSizeImage = 0
                }
            };

            _bitmap = WinApi.CreateDIBSection(_memoryDc, ref bmi, WinApi.DIB_RGB_COLORS, out _bitmapBits, IntPtr.Zero, 0);
            if (_bitmap == IntPtr.Zero)
            {
                throw new InvalidOperationException($"Failed to create DIB section. Error: {Marshal.GetLastWin32Error()}");
            }

            WinApi.SelectObject(_memoryDc, _bitmap);
            _bitmapWidth = width;
            _bitmapHeight = height;
        }

        public void UpdateColors(OverlayCalculationConfig config)
        {
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            // Trigger refresh if visible
            if (_localState.IsVisible)
            {
                Update(_localState); // Re-render with new colors
            }
        }

        public void Update(OverlayDefinition source)
        {
            if (_hwnd == IntPtr.Zero)
                return;

            // Check if we need to resize bitmap
            if (source.Bounds.Width != _bitmapWidth || source.Bounds.Height != _bitmapHeight)
            {
                if (source.Bounds.Width > 0 && source.Bounds.Height > 0)
                {
                    CreateBitmap(source.Bounds.Width, source.Bounds.Height);
                }
            }

            // Fill bitmap with color if visible
            if (source.IsVisible && _bitmapBits != IntPtr.Zero)
            {
                FillBitmap(source.Color, source.Opacity);
            }

            // Update window using UpdateLayeredWindow
            var screenDc = WinApi.GetDC(IntPtr.Zero);
            try
            {
                var position = new WinApi.POINT { X = source.Bounds.X, Y = source.Bounds.Y };
                var size = new WinApi.SIZE(source.Bounds.Width, source.Bounds.Height);
                var sourcePoint = new WinApi.POINT { X = 0, Y = 0 };
                var blend = new WinApi.BLENDFUNCTION
                {
                    BlendOp = WinApi.AC_SRC_OVER,
                    BlendFlags = 0,
                    SourceConstantAlpha = source.Opacity,
                    AlphaFormat = 0 // No per-pixel alpha (solid color)
                };

                WinApi.UpdateLayeredWindow(
                    _hwnd,
                    screenDc,
                    ref position,
                    ref size,
                    _memoryDc,
                    ref sourcePoint,
                    0,
                    ref blend,
                    WinApi.ULW_ALPHA);

                // Show/hide window
                WinApi.ShowWindow(_hwnd, source.IsVisible ? 5 : 0); // SW_SHOW : SW_HIDE
            }
            finally
            {
                WinApi.ReleaseDC(IntPtr.Zero, screenDc);
            }

            // Copy source to local state
            _localState.CopyFrom(source);
        }

        private unsafe void FillBitmap(Core.Color color, byte opacity)
        {
            if (_bitmapBits == IntPtr.Zero || _bitmapWidth <= 0 || _bitmapHeight <= 0)
                return;

            // Convert color to premultiplied alpha format (BGRA)
            // Since we use SourceConstantAlpha in BLENDFUNCTION, we don't need to premultiply here
            uint pixel = (uint)((color.B << 0) | (color.G << 8) | (color.R << 16) | (0xFF << 24));

            int pixelCount = _bitmapWidth * _bitmapHeight;
            uint* pixels = (uint*)_bitmapBits;

            // Fill bitmap with solid color
            for (int i = 0; i < pixelCount; i++)
            {
                pixels[i] = pixel;
            }
        }

        public void Hide()
        {
            if (_hwnd != IntPtr.Zero)
            {
                WinApi.ShowWindow(_hwnd, 0); // SW_HIDE
                _localState.IsVisible = false;
            }
        }

        public bool SetDisplayAffinity(uint affinity)
        {
            if (_hwnd == IntPtr.Zero)
                return false;

            return WinApi.SetWindowDisplayAffinity(_hwnd, affinity);
        }

        public void Dispose()
        {
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
                _hwnd = IntPtr.Zero;
            }
        }
    }
}
