using System.ComponentModel;
using System.Windows;
using System.Windows.Forms;
using System.Collections.Generic;
using System.Windows.Media;
using SpotlightDimmer.Models;
using System;

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

            BuildTheViewModel();

            SetTheIconAndMinimizeToTrayOptions();

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

        private void SetTheIconAndMinimizeToTrayOptions()
        {
            // Create the NotifyIcon object
            _notifyIcon = new NotifyIcon();
            _notifyIcon.Icon = new System.Drawing.Icon("icon.ico");
            _notifyIcon.Text = "Spotlight Dimmer";

            _notifyIcon.Visible = true;

            // Handle the StateChanged event of the Window
            this.StateChanged += MainWindow_StateChanged;

            // Handle the DoubleClick event of the NotifyIcon
            _notifyIcon.DoubleClick += NotifyIcon_DoubleClick;
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
            if (WindowState == WindowState.Minimized)
            {
                // Hide the window and show the NotifyIcon
                Hide();
            }
        }

        private void NotifyIcon_DoubleClick(object? sender, System.EventArgs e)
        {
            // Show the window again
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

            // Dispose the NotifyIcon object
            _notifyIcon.Dispose();
        }
    }
}
