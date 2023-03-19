
using System;
using System.ComponentModel;
using System.Configuration;
using System.Windows;
using System.Windows.Media;

namespace SpotlightDimmer.Settings
{
    public class DimmerSettings: INotifyPropertyChanged
    {
        private bool _isDebugInfoVisible;
        private Color? _selectedColor;
        private string _currentSavedColor;
        private readonly Configuration _configuration;
        public DimmerSettings()
        {
            _configuration = ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None);
            IsDebugInfoVisible = true;
        }

        public Color SelectedColor
        {
            get {
                if (_selectedColor == null)
                    _selectedColor = GetColorFromSettings();
                return _selectedColor.Value;
            }
            set
            {

                if (value.A > 225)
                    value.A = 225;
                _selectedColor = value;
                OnPropertyChanged(nameof(SelectedColor));
                OnPropertyChanged(nameof(SelectedBrush));
            }
        }
        public Brush SelectedBrush
        {
            get { return new SolidColorBrush(SelectedColor); }
        }
        public bool IsDebugInfoVisible
        {
            get { return _isDebugInfoVisible; }
            set
            {
                _isDebugInfoVisible = value;
                OnPropertyChanged(nameof(DebugInfoVisibility));
            }
        }

        private string _debugInfo = "This is some debug information.";

        public string DebugInfo
        {
            get { return _debugInfo; }
            set
            {
                _debugInfo = value;
                OnPropertyChanged(nameof(DebugInfo));
            }
        }
        public Visibility DebugInfoVisibility => IsDebugInfoVisible ? Visibility.Visible : Visibility.Collapsed;

        public event PropertyChangedEventHandler PropertyChanged;

        protected void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        public Color GetColorFromSettings()
        {
            var fallbackColor = Color.FromArgb(128, 128, 128, 128);
            try
            {
                string? _backgroundHexSettings = _configuration.AppSettings.Settings["BackgroundHex"]?.Value;
                _backgroundHexSettings ??= fallbackColor.ToString().Replace("#", String.Empty);

                var backgroundColorIntValue = int.Parse(_backgroundHexSettings, System.Globalization.NumberStyles.HexNumber);

                var backgroundColor = Color.FromArgb(
                    (byte)((backgroundColorIntValue >> 24) & 0xff),
                    (byte)((backgroundColorIntValue >> 16) & 0xff),
                    (byte)((backgroundColorIntValue >> 8) & 0xff),
                    (byte)(backgroundColorIntValue & 0xff)
                );

                return backgroundColor;
            }
            catch (Exception ex)
            {
                DebugInfo = ex.ToString();
                return fallbackColor;
            }
        }

        public string CurrentSavedColor => _configuration.AppSettings.Settings["BackgroundHex"] != null?
            $"#{_configuration.AppSettings.Settings["BackgroundHex"].Value}":
            "No saved configuration found";

        public void SaveSettings()
        {
            if (_selectedColor == null)
            {
                DebugInfo = "Attempt at saving settings with null selected color";
                return;
            }

            try
            {
                if (_configuration.AppSettings.Settings["BackgroundHex"] == null)
                    _configuration.AppSettings.Settings.Add("BackgroundHex", _selectedColor.ToString().Replace("#", String.Empty));
                else
                    _configuration.AppSettings.Settings["BackgroundHex"].Value = _selectedColor.ToString().Replace("#", String.Empty);
                _configuration.Save(ConfigurationSaveMode.Full);
                ConfigurationManager.RefreshSection("appSettings");
                OnPropertyChanged(nameof(CurrentSavedColor));
                DebugInfo = $"Settings saved successfuly.\r\nSaved color: {_selectedColor}";
            }
            catch (Exception ex)
            {
                DebugInfo = ex.ToString();
            }
        }
    }
}
