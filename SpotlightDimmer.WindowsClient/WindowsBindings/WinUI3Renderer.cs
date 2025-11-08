using Microsoft.UI;
using Microsoft.UI.Composition;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using SpotlightDimmer.Core;
using Windows.Graphics;
using WinRT.Interop;
using System.Runtime.InteropServices;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// WinUI3-based renderer using modern Windows composition APIs.
/// This renderer uses the WinUI3 framework with XAML windows for overlay rendering.
///
/// PERFORMANCE CHARACTERISTICS:
/// - Memory: HIGH (~50-150MB due to WinUI3/XAML runtime overhead)
/// - Disk: HIGH (~50-100MB due to WindowsAppSDK dependencies)
/// - CPU: MODERATE-HIGH (DirectComposition + XAML overhead)
/// - NOT compatible with Native AOT compilation
///
/// This renderer is experimental and intended for performance comparison testing.
/// </summary>
public class WinUI3Renderer : IOverlayRenderer
{
    private readonly Dictionary<(int displayIndex, OverlayRegion region), OverlayWindow> _overlayPool = new();
    private bool _disposed = false;
    private static bool _winUIInitialized = false;

    public WinUI3Renderer()
    {
        // Initialize WinUI3 for unpackaged app (one-time initialization)
        if (!_winUIInitialized)
        {
            InitializeWinUI();
            _winUIInitialized = true;
        }
    }

    /// <summary>
    /// Initialize Windows App SDK for unpackaged applications.
    /// CRITICAL: Must call Bootstrap.Initialize() before any WinUI3 objects.
    /// </summary>
    private static void InitializeWinUI()
    {
        try
        {
            // STEP 1: Bootstrap the Windows App SDK runtime (CRITICAL for unpackaged apps)
            // This MUST happen before any WinUI3 COM objects are created
            Microsoft.Windows.ApplicationModel.DynamicDependency.Bootstrap.Initialize(
                0x00010007, // Version 1.7 (major version in high word, minor in low word)
                null);      // Use default architecture

            // Step 2: Create DispatcherQueueController for the current thread
            var options = new DispatcherQueueOptions
            {
                dwSize = Marshal.SizeOf<DispatcherQueueOptions>(),
                threadType = 2, // DQTYPE_THREAD_CURRENT
                apartmentType = 2  // DQTAT_COM_STA
            };

            IntPtr controller;
            int hr = CreateDispatcherQueueController(options, out controller);

            if (hr != 0 || controller == IntPtr.Zero)
            {
                throw new InvalidOperationException($"Failed to create DispatcherQueueController (HRESULT: 0x{hr:X8})");
            }

            // Step 3: Create a minimal WinUI3 Application instance
            // This is required for Window creation in unpackaged apps
            if (Application.Current == null)
            {
                // Create our minimal application instance
                new WinUI3RendererApplication();
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException(
                "Failed to initialize WinUI3.\n" +
                "Possible causes:\n" +
                "1. Microsoft.WindowsAppSDK runtime is not deployed alongside the exe\n" +
                "2. Windows App SDK version mismatch\n" +
                "3. Missing runtime DLLs in the output folder", ex);
        }
    }

    /// <summary>
    /// Minimal WinUI3 Application subclass for unpackaged apps.
    /// This provides the Application context needed for Window creation
    /// without taking over the message loop.
    /// </summary>
    private class WinUI3RendererApplication : Application
    {
        protected override void OnLaunched(LaunchActivatedEventArgs args)
        {
            // Don't create any windows here - we'll create them manually
            // This just provides the Application context
        }
    }

    // P/Invoke structures and functions for DispatcherQueue creation
    [StructLayout(LayoutKind.Sequential)]
    private struct DispatcherQueueOptions
    {
        public int dwSize;
        public int threadType;    // DISPATCHERQUEUE_THREAD_TYPE
        public int apartmentType; // DISPATCHERQUEUE_THREAD_APARTMENTTYPE
    }

    [DllImport("CoreMessaging.dll")]
    private static extern int CreateDispatcherQueueController(
        DispatcherQueueOptions options,
        out IntPtr dispatcherQueueController);

    public void CreateOverlays(DisplayInfo[] displays, OverlayCalculationConfig config)
    {
        foreach (var display in displays)
        {
            // Create one window for each region (6 total per display)
            for (int i = 0; i < 6; i++)
            {
                var region = (OverlayRegion)i;
                var key = (display.Index, region);

                // Create WinUI3 window for this overlay
                var window = new OverlayWindow(region, display.Bounds, config);
                _overlayPool[key] = window;
            }
        }
    }

    public void UpdateBrushColors(OverlayCalculationConfig config)
    {
        foreach (var window in _overlayPool.Values)
        {
            window.UpdateBrushColors(config);
        }
    }

    public int UpdateScreenCaptureExclusion(bool exclude)
    {
        int successCount = 0;

        foreach (var window in _overlayPool.Values)
        {
            if (window.SetDisplayAffinity(exclude))
            {
                successCount++;
            }
        }

        return successCount;
    }

    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        foreach (var state in states)
        {
            foreach (var sourceOverlay in state.Overlays)
            {
                var key = (state.DisplayIndex, sourceOverlay.Region);

                if (_overlayPool.TryGetValue(key, out var window))
                {
                    window.Update(sourceOverlay);
                }
            }
        }
    }

    public void HideAllOverlays()
    {
        foreach (var window in _overlayPool.Values)
        {
            window.Hide();
        }
    }

    public void CleanupOverlays()
    {
        foreach (var window in _overlayPool.Values)
        {
            window.Dispose();
        }
        _overlayPool.Clear();
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            CleanupOverlays();

            // Shutdown Windows App SDK bootstrap if this is the last renderer instance
            // Note: We don't actually shutdown here because other parts of the app might
            // still be using WinUI3. The runtime will be cleaned up when the process exits.

            _disposed = true;
        }
    }

    /// <summary>
    /// Represents a single WinUI3 overlay window.
    /// Uses XAML Border control for rendering the colored overlay.
    /// </summary>
    private class OverlayWindow : IDisposable
    {
        private Window? _window;
        private Border? _border;
        private OverlayDefinition _localState;
        private AppWindow? _appWindow;
        private Core.Color _activeColor;
        private Core.Color _inactiveColor;
        private bool _disposed = false;

        public OverlayWindow(OverlayRegion region, Core.Rectangle displayBounds, OverlayCalculationConfig config)
        {
            _localState = new OverlayDefinition(region);
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            // Create WinUI3 window on the UI thread
            CreateWindow(region, displayBounds, config);
        }

        private void CreateWindow(OverlayRegion region, Core.Rectangle displayBounds, OverlayCalculationConfig config)
        {
            // WinUI3 windows must be created on a UI thread with a dispatcher
            // Create the window
            _window = new Window
            {
                Title = $"SpotlightDimmer Overlay - {region}"
            };

            // Get the AppWindow for window management
            var hwnd = WindowNative.GetWindowHandle(_window);
            var windowId = Win32Interop.GetWindowIdFromWindow(hwnd);
            _appWindow = AppWindow.GetFromWindowId(windowId);

            // Configure window as a transparent overlay
            if (_appWindow != null)
            {
                // Set window to be topmost, transparent, and click-through
                _appWindow.IsShownInSwitchers = false;

                // Get presenter and configure as overlay
                var presenter = _appWindow.Presenter as OverlappedPresenter;
                if (presenter != null)
                {
                    presenter.IsAlwaysOnTop = true;
                    presenter.IsResizable = false;
                    presenter.IsMaximizable = false;
                    presenter.IsMinimizable = false;
                    presenter.SetBorderAndTitleBar(false, false);
                }

                // Set initial position (minimal size, will be updated later)
                _appWindow.MoveAndResize(new RectInt32(displayBounds.X, displayBounds.Y, 1, 1));
            }

            // Set window as click-through using Win32 API
            SetClickThrough(hwnd);

            // Create XAML content - a simple colored Border
            _border = new Border
            {
                Background = CreateBrush(config.InactiveColor, config.InactiveOpacity)
            };

            _window.Content = _border;

            // Keep window hidden initially
            // Note: WinUI3 windows are shown by default, so we need to hide them
            if (_appWindow != null)
            {
                _appWindow.Hide();
            }
        }

        private void SetClickThrough(IntPtr hwnd)
        {
            // Use Win32 API to make window click-through
            const int GWL_EXSTYLE = -20;
            const uint WS_EX_TRANSPARENT = 0x00000020;
            const uint WS_EX_LAYERED = 0x00080000;
            const uint WS_EX_TOOLWINDOW = 0x00000080;
            const uint WS_EX_NOACTIVATE = 0x08000000;

            var extendedStyle = GetWindowLong(hwnd, GWL_EXSTYLE);
            SetWindowLong(hwnd, GWL_EXSTYLE,
                extendedStyle | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE);
        }

        [System.Runtime.InteropServices.DllImport("user32.dll")]
        private static extern uint GetWindowLong(IntPtr hWnd, int nIndex);

        [System.Runtime.InteropServices.DllImport("user32.dll")]
        private static extern uint SetWindowLong(IntPtr hWnd, int nIndex, uint dwNewLong);

        [System.Runtime.InteropServices.DllImport("user32.dll")]
        private static extern bool SetWindowDisplayAffinity(IntPtr hWnd, uint dwAffinity);

        private SolidColorBrush CreateBrush(Core.Color color, byte opacity)
        {
            return new SolidColorBrush(Windows.UI.Color.FromArgb(
                opacity,
                color.R,
                color.G,
                color.B
            ));
        }

        public void UpdateBrushColors(OverlayCalculationConfig config)
        {
            _activeColor = config.ActiveColor;
            _inactiveColor = config.InactiveColor;

            // Update the brush if window is visible
            if (_border != null && _localState.IsVisible)
            {
                _border.DispatcherQueue.TryEnqueue(() =>
                {
                    _border.Background = CreateBrush(_localState.Color, _localState.Opacity);
                });
            }
        }

        public void Update(OverlayDefinition source)
        {
            if (_window == null || _appWindow == null || _border == null)
                return;

            // Check what changed
            bool boundsChanged = source.Bounds != _localState.Bounds;
            bool colorOrOpacityChanged =
                source.Color != _localState.Color ||
                source.Opacity != _localState.Opacity;
            bool visibilityChanged = source.IsVisible != _localState.IsVisible;

            // Update position/size if needed
            if (boundsChanged)
            {
                _appWindow.MoveAndResize(new RectInt32(
                    source.Bounds.X,
                    source.Bounds.Y,
                    source.Bounds.Width,
                    source.Bounds.Height
                ));
            }

            // Update color/opacity if needed
            if (colorOrOpacityChanged && source.IsVisible)
            {
                _border.DispatcherQueue.TryEnqueue(() =>
                {
                    _border.Background = CreateBrush(source.Color, source.Opacity);
                });
            }

            // Update visibility if needed
            if (visibilityChanged)
            {
                if (source.IsVisible)
                {
                    _appWindow.Show();
                }
                else
                {
                    _appWindow.Hide();
                }
            }

            // Copy source values to local state
            _localState.CopyFrom(source);
        }

        public void Hide()
        {
            if (_appWindow != null)
            {
                _appWindow.Hide();
                _localState.IsVisible = false;
            }
        }

        public bool SetDisplayAffinity(bool exclude)
        {
            if (_window == null)
                return false;

            var hwnd = WindowNative.GetWindowHandle(_window);
            const uint WDA_EXCLUDEFROMCAPTURE = 0x00000011;
            const uint WDA_NONE = 0x00000000;

            uint affinity = exclude ? WDA_EXCLUDEFROMCAPTURE : WDA_NONE;
            return SetWindowDisplayAffinity(hwnd, affinity);
        }

        public void Dispose()
        {
            if (!_disposed)
            {
                _window?.Close();
                _window = null;
                _border = null;
                _appWindow = null;
                _disposed = true;
            }
        }
    }
}
