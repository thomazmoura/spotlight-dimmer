using SpotlightDimmer.Models;
using System;
using System.ComponentModel;
using System.Configuration;
using System.Windows;
using System.Windows.Media;

namespace SpotlightDimmer
{
    public class DimmerSettings: INotifyPropertyChanged
    {
        private readonly Configuration _configuration;
        private readonly DimmerState _state;

        public DimmerSettings(DimmerState state)
        {
            _state = state;
            _configuration = ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None);
            _state.SelectedColor = GetColorFromSettings();
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        protected void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        public Color GetColorFromSettings()
        {
            var fallbackColor = Color.FromArgb(128, 128, 128, 128);
            try
            {
                string? _backgroundHexSettings = _configuration.AppSettings?.Settings["BackgroundHex"]?.Value;
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
                _state.DebugInfo = ex.ToString();
                return fallbackColor;
            }
        }

        public string CurrentSavedColor => _configuration.AppSettings.Settings["BackgroundHex"] != null?
            $"#{_configuration.AppSettings.Settings["BackgroundHex"].Value}":
            "No saved configuration found";

        public void SaveSettings()
        {
            _state.DebugInfo = "Saving settings";
            try
            {
                if (_configuration.AppSettings.Settings["BackgroundHex"] == null)
                    _configuration.AppSettings.Settings.Add("BackgroundHex", _state.SelectedColor.ToString().Replace("#", String.Empty));
                else
                    _configuration.AppSettings.Settings["BackgroundHex"].Value = _state.SelectedColor.ToString().Replace("#", String.Empty);
                _configuration.Save(ConfigurationSaveMode.Full);
                ConfigurationManager.RefreshSection("appSettings");
                OnPropertyChanged(nameof(CurrentSavedColor));
                _state.DebugInfo = $"Settings saved successfuly.\r\nSaved color: {_state.SelectedColor}";
            }
            catch (Exception ex)
            {
                _state.DebugInfo = ex.ToString();
            }
        }
    }
}
