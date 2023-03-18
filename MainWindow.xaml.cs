using System.ComponentModel;
using System.Windows;
using System.Runtime.CompilerServices;
using System.Windows.Forms;
using SpotlightDimmer.Settings;
using System.Collections.Generic;

namespace SpotlightDimmer
{
    public partial class MainWindow : Window
    {
        public DimmerSettings _dimmerSettings;
        private List<Window> _dimmerWindows;

        public MainWindow()
        {
            InitializeComponent();

            SetTheDimmerSettingsAsDataContext();

            CreateTheDimmerWindows();
            Closing += OnClosing;
        }

        private void SetTheDimmerSettingsAsDataContext()
        {
            _dimmerSettings = new DimmerSettings();
            DataContext = _dimmerSettings;
        }

        protected void CreateTheDimmerWindows()
        {
            _dimmerWindows = new List<Window>();
            foreach(var screen in Screen.AllScreens)
            {
                var dimmerWindow = new DimmerWindow(screen, _dimmerSettings);
                dimmerWindow.Show();
                _dimmerWindows.Add(dimmerWindow);
            }
        }

        private void OnClosing(object sender, CancelEventArgs e)
        {
            foreach (var childWindow in _dimmerWindows)
                childWindow.Close();
        }
    }
}
