using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows;
using System.Windows.Forms;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Interop;
using System.Linq;
using System.Collections.Generic;
using System.Windows.Media;
using SpotlightDimmer.Settings;

namespace SpotlightDimmer
{
    public partial class DimmerWindow : Window
    {
        private Screen _screen;
        private DimmerSettings _dimmerSettings;
        private const int WS_EX_TRANSPARENT = 0x00000020;
        private const int GWL_EXSTYLE = (-20);
        private const uint EVENT_SYSTEM_FOREGROUND = 0x0003;

        private readonly string _isDebugEnabledEnvironmentVariableName = "SpotlightDimmer__IsDebugEnabled";
        private readonly string _backGroundHexEnvironmentVariableName = "SpotlightDimmer__BackgroundHex";
        private readonly string[] _ignoredWindows;

        private readonly IntPtr _windowsFocusHook;
        private readonly IntPtr _windowsResizedHook;
        private readonly WinEventDelegate _winEventDelegate;

        // Methods to make the window transparent to clicks
        [DllImport("user32.dll")]
        static extern int GetWindowLong(IntPtr hwnd, int index);
        [DllImport("user32.dll")]
        static extern int SetWindowLong(IntPtr hwnd, int index, int newStyle);

        // Methods to get focus events
        private delegate void WinEventDelegate(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime);

        [DllImport("user32.dll")]
        private static extern IntPtr SetWinEventHook(uint eventMin, uint eventMax, IntPtr hmodWinEventProc, WinEventDelegate lpfnWinEventProc, uint idProcess, uint idThread, uint dwFlags);
        [DllImport("user32.dll")]
        private static extern bool UnhookWinEvent(IntPtr hWinEventHook);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        private static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetWindowRect(IntPtr hWnd, ref RECT lpRect);

        // Methods to get resize events
        private const uint WINEVENT_OUTOFCONTEXT = 0x0000; // Events are ASYNC
        private const uint EVENT_OBJECT_LOCATIONCHANGE = 0x800B;
        [DllImport("user32.dll")]
        private static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll")]
        private static extern IntPtr GetFocus();

        public struct RECT
        {
            public int left;
            public int top;
            public int right;
            public int bottom;
        }

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

            _winEventDelegate = new WinEventDelegate(WinEventProc);
            //_windowsFocusHook = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, _winEventDelegate, 0, 0, 0);
            //_windowsResizedHook = SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE, IntPtr.Zero, _winEventDelegate, 0, 0, WINEVENT_OUTOFCONTEXT);

            SetOverlayConfiguration();

            _ignoredWindows = new[] { MainDimmerWindow.Title, "Task Switching", "Iniciar", "Pesquisar" };

            // Show the window
            Show();
        }

        private void SetOverlayConfiguration()
        {
            SetOverlayDebugText();
            SetOverlayBackground();
        }

        private void SetOverlayDebugText()
        {
            string? isDebugEnabledEnvironmentVariable = System.Environment.GetEnvironmentVariable(_isDebugEnabledEnvironmentVariableName);
            bool isDebugEnabled = !String.IsNullOrWhiteSpace(isDebugEnabledEnvironmentVariable) && bool.Parse(isDebugEnabledEnvironmentVariable);
        }

        private void SetOverlayBackground()
        {
            //string? _backgroundHexEnvironmentVariable = System.Environment.GetEnvironmentVariable(_backGroundHexEnvironmentVariableName);
            //int backgroundColorIntValue;
            //if (String.IsNullOrWhiteSpace(_backgroundHexEnvironmentVariable))
            //    backgroundColorIntValue = int.Parse("88888888", System.Globalization.NumberStyles.HexNumber);
            //else
            //    backgroundColorIntValue = int.Parse(_backgroundHexEnvironmentVariable, System.Globalization.NumberStyles.HexNumber);

            //var backgroundColor = Color.FromArgb(
            //    (byte)((backgroundColorIntValue >> 24) & 0xff),
            //    (byte)((backgroundColorIntValue >> 16) & 0xff),
            //    (byte)((backgroundColorIntValue >> 8) & 0xff),
            //    (byte)(backgroundColorIntValue & 0xff)
            //);

            //MainGrid.Background = new SolidColorBrush(backgroundColor);
            //MainGrid.Background = _dimmerSettings.SelectedBrush;
        }

        public static void SetWindowExTransparent(IntPtr hwnd)
        {
            var extendedStyle = GetWindowLong(hwnd, GWL_EXSTYLE);
            _ = SetWindowLong(hwnd, GWL_EXSTYLE, extendedStyle | WS_EX_TRANSPARENT);
        }

        protected override void OnSourceInitialized(EventArgs e)
        {
            base.OnSourceInitialized(e);
            var hwnd = new WindowInteropHelper(this).Handle;
            SetWindowExTransparent(hwnd);
        }

        protected override void OnClosing(CancelEventArgs e)
        {
            base.OnClosing(e);

            UnhookWinEvent(_windowsFocusHook);
        }

        private void WinEventProc(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime)
        {
            const int nChars = 256;
            var stringBuilder = new StringBuilder(nChars);

            // get the title of the window
            if (GetWindowText(hwnd, stringBuilder, nChars) > 0)
            {
                var title = stringBuilder.ToString();

                if (_ignoredWindows.Contains(title) || !HasTextFocus(hwnd))
                    return;

                var rect = new RECT();
                GetWindowRect(hwnd, ref rect);
                int x = rect.left;
                int y = rect.top;
                var position = $"({x},{y})";

                var inactiveScreens = GetNonIntersectingScreens(rect, -20);
                var inactiveScreensName = String.Join(", ", inactiveScreens.Select(screen => screen.DeviceName));
                var currentInactiveScreen = inactiveScreens.FirstOrDefault();

                if (currentInactiveScreen != null)
                {
                    Left = currentInactiveScreen.Bounds.Left;
                    Top = currentInactiveScreen.Bounds.Top;
                    Width = currentInactiveScreen.Bounds.Right - currentInactiveScreen.Bounds.Left;
                    Height = currentInactiveScreen.Bounds.Bottom - currentInactiveScreen.Bounds.Top;
                }

//                Info.Text = @$"Debug details:
                
//Window Title={title}
//Position={position}
//IdObject={idObject}
//IdChild={idChild}
//winEventHook={hWinEventHook}
//hwnd={hwnd}
//inactiveScreens={inactiveScreensName}

//Screens:
//{ScreenDebug()}";
            }

        }

        private static string ScreenDebug()
        {
            var stringBuilder = new StringBuilder();
            foreach (var screen in Screen.AllScreens)
                stringBuilder.Append($"Screen: {screen.DeviceName}, Top: {screen.Bounds.Top}, Bottom: {screen.Bounds.Bottom}, Left: {screen.Bounds.Left}, Right: {screen.Bounds.Right}\r\n");
            return stringBuilder.ToString();
        }

        public static List<Screen> GetNonIntersectingScreens(RECT rect, int sensitivity)
        {
            var nonIntersectingScreens = new List<Screen>();

            foreach (var screen in Screen.AllScreens)
            {
                System.Drawing.Rectangle screenBounds = screen.Bounds;

                // Expand the screen bounds by the sensitivity amount
                screenBounds.Inflate(sensitivity, sensitivity);

                // Check if the expanded screen intersects with the given rectangle
                if (screenBounds.IntersectsWith(System.Drawing.Rectangle.FromLTRB(rect.left, rect.top, rect.right, rect.bottom)))
                {
                    // Screen intersects with the given rectangle
                    continue;
                }

                // Screen does not intersect with the given rectangle
                nonIntersectingScreens.Add(screen);
            }

            return nonIntersectingScreens;
        }

        private bool HasTextFocus(IntPtr windowHandle)
        {
            // Check if the window is in the foreground
            if (windowHandle == GetForegroundWindow())
            {
                return true;
            }

            return false;
        }

    }
}
