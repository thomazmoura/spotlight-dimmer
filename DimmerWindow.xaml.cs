using System;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Forms;
using System.Windows.Interop;
using SpotlightDimmer.Settings;

namespace SpotlightDimmer
{
    public partial class DimmerWindow : Window
    {
        private Screen _screen;
        private DimmerSettings _dimmerSettings;
        // Makes the window transparent and unclickable
        private const int WS_EX_TRANSPARENT = 0x00000020;
        // Makes the window not appear on alt+tab
        private const int WS_EX_TOOLWINDOW = 0x00000080;
        private const int GWL_EXSTYLE = (-20);


        // Methods to make the window transparent to clicks and not appear on alt+tab menus
        [DllImport("user32.dll")]
        static extern int GetWindowLong(IntPtr hwnd, int index);
        [DllImport("user32.dll")]
        static extern int SetWindowLong(IntPtr hwnd, int index, int newStyle);

        public DimmerWindow(Screen screen, DimmerSettings dimmerSettings)
        {
            InitializeComponent();

            _screen = screen;
            _dimmerSettings = dimmerSettings;
            DataContext = _dimmerSettings;

            Left = _screen.Bounds.Left;
            Top = _screen.Bounds.Top;
            Width = _screen.Bounds.Width;
            Height = _screen.Bounds.Height;

            // Show the window
            Show();
        }

        public static void SetWindowExTransparent(IntPtr hwnd)
        {
            var extendedStyle = GetWindowLong(hwnd, GWL_EXSTYLE);
            _ = SetWindowLong(hwnd, GWL_EXSTYLE, extendedStyle | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW);
        }

        protected override void OnSourceInitialized(EventArgs e)
        {
            base.OnSourceInitialized(e);
            var hwnd = new WindowInteropHelper(this).Handle;
            SetWindowExTransparent(hwnd);
        }
    }
}
