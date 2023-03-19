﻿using System.ComponentModel;
using System.Windows;
using System.Windows.Forms;
using System.Collections.Generic;
using System.Windows.Media;
using SpotlightDimmer.Models;

namespace SpotlightDimmer
{
    public partial class MainWindow : Window
    {
        public DimmerSettings _dimmerSettings;
        public WindowsEventsManager _dimmerStateManager;
        public DimmerState _state;
        private List<Window> _dimmerWindows;

        public MainWindow()
        {
            InitializeComponent();

            BuildTheViewModel();

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
                var dimmerWindow = new DimmerWindow(screen, _state);
                dimmerWindow.Show();
                _dimmerWindows.Add(dimmerWindow);
            }
        }

        private void saveSettingsButton_Click(object sender, RoutedEventArgs e)
        {
            _dimmerSettings.SaveSettings();
        }

        private void OnClosing(object sender, CancelEventArgs e)
        {
            foreach (var childWindow in _dimmerWindows)
                childWindow.Close();
            _dimmerStateManager.Dispose();
        }
    }
}
