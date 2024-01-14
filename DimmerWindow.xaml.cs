using SpotlightDimmer.Models;
using System;
using System.Linq;
using System.Windows.Forms;
using System.Windows.Interop;

namespace SpotlightDimmer
{
    public partial class DimmerWindow : Window
    {
        private readonly string _screenDeviceName;
        private readonly MainWindow _mainWindow;
        private readonly DimmerState _state;
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

        public DimmerWindow(Screen screen, DimmerState state, MainWindow mainWindow)
        {
            InitializeComponent();

            _mainWindow = mainWindow;
            _screenDeviceName = screen.DeviceName;
            _state = state;
            DataContext = _state;

            Left = screen.Bounds.Left;
            Top = screen.Bounds.Top;
            Width = screen.Bounds.Width;
            Height = screen.Bounds.Height;

            UpdateVisibilityOnFocusedScreenChange();

            Show();
        }

        private void UpdateVisibilityOnFocusedScreenChange()
        {
            _state.PropertyChanged += (object? sender, PropertyChangedEventArgs e) =>
            {
                if (e.PropertyName == nameof(_state.FocusedScreen))
                {
                    SetVisibilityRelatedToFocus();
                }
            };
        }

        public void SetVisibilityRelatedToFocus()
        {
            if (_state.FocusedScreen.DeviceName == _screenDeviceName)
                Visibility = Visibility.Hidden;
            else if (!Screen.AllScreens.Any(screen => screen.DeviceName == _screenDeviceName))
                Visibility = Visibility.Hidden;
            else
                Visibility = Visibility.Visible;
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

        private void ScreenDimmerWindow_Activated(object sender, EventArgs e)
        {
            _mainWindow.Activate();
        }
    }
}
