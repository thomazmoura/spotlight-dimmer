using System.Windows.Forms;
using System.Collections.Generic;
using SpotlightDimmer.Models;
using System;
using System.Windows.Interop;
using System.Windows.Media.Imaging;

namespace SpotlightDimmer
{
    public partial class MainWindow : Window
    {
        public DimmerSettings _dimmerSettings;
        public WindowsEventsManager _dimmerStateManager;
        public DimmerState _state;
        private List<Window> _dimmerWindows;
        private NotifyIcon _notifyIcon;

        public MainWindow()
        {
            InitializeComponent();

            SetApplicationIcon();
            BuildTheViewModel();
            SetMinimizeToTrayOptions();
            CreateTheDimmerWindows();
            Closing += OnClosing;
        }

        private void BuildTheViewModel()
        {
            _state = new DimmerState();
            _dimmerStateManager = new WindowsEventsManager(_state);
            _dimmerSettings = new DimmerSettings(_state);
            DataContext = _state;
        }

        protected void CreateTheDimmerWindows()
        {
            _dimmerWindows = new List<Window>();
            foreach(var screen in Screen.AllScreens)
            {
                var dimmerWindow = new DimmerWindow(screen, _state, this);
                dimmerWindow.Show();
                _dimmerWindows.Add(dimmerWindow);
            }
        }

        private void SetApplicationIcon()
        {
            var icon = GetSpotlightDimmerIcon();
            var handle = icon.Handle;
            this.Icon = Imaging.CreateBitmapSourceFromHIcon(handle, Int32Rect.Empty, BitmapSizeOptions.FromEmptyOptions());
        }

        private void SetMinimizeToTrayOptions()
        {
            // Create the NotifyIcon object
            _notifyIcon = new NotifyIcon();
            _notifyIcon.Icon = GetSpotlightDimmerIcon();
            _notifyIcon.Text = "Spotlight Dimmer";

            _notifyIcon.BalloonTipText = "The application is still running on the system tray. Click here to open it again";
            _notifyIcon.BalloonTipTitle = "Spotlight Dimmer";
            _notifyIcon.BalloonTipIcon = ToolTipIcon.Info;

            _notifyIcon.Visible = true;

            // Handle the StateChanged event of the Window
            this.StateChanged += MainWindow_StateChanged;

            // Handle the DoubleClick event of the NotifyIcon
            _notifyIcon.Click += NotifyIcon_Click;
        }

        private void saveSettingsButton_Click(object? sender, RoutedEventArgs e)
        {
            _dimmerSettings.SaveSettings();
        }

        private void Window_Activated(object? sender, EventArgs e)
        {
            this.Activate();
            this.Focus();
        }


        private void MainWindow_StateChanged(object? sender, System.EventArgs e)
        {
            if (WindowState == WindowState.Minimized && _state.MinimizeToTray)
            {
                Hide();
                _notifyIcon.ShowBalloonTip((int)TimeSpan.FromSeconds(5).TotalMilliseconds);
            }
        }

        private void NotifyIcon_Click(object? sender, System.EventArgs e)
        {
            Show();
            this.Activate();
            this.Focus();
            WindowState = WindowState.Normal;
        }

        private void DebugInfoTextBox_TextChanged(object? sender, System.Windows.Controls.TextChangedEventArgs e)
        {
            DebugInfoTextBox.ScrollToEnd();
        }

        private void OnClosing(object? sender, CancelEventArgs e)
        {
            foreach (var childWindow in _dimmerWindows)
                childWindow.Close();
            _dimmerStateManager.Dispose();

            _notifyIcon.Dispose();
        }

        private System.Drawing.Icon GetSpotlightDimmerIcon()
        {
            using var stream = typeof(MainWindow).Assembly.GetManifestResourceStream("SpotlightDimmer.ico");
            return new System.Drawing.Icon(stream);
        }
    }
}
