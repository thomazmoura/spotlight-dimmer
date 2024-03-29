﻿using System.Configuration;
using Color = System.Windows.Media.Color;

namespace SpotlightDimmer.Models;

public class DimmerSettings: INotifyPropertyChanged
{
    private readonly Configuration _configuration;
    private readonly DimmerState _state;

    /// <summary>
    /// This class is responsible for both persisting and retriving the program local settings.
    /// </summary>
    /// <param name="state">The current system state so it can update it's state based on the settings. </param>
    public DimmerSettings(DimmerState state)
    {
        _state = state;
        _configuration = ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None);
        _state.SelectedColor = GetColorFromSettings();
        _state.Topmost = GetTopmostFromSettings();
        _state.MinimizeToTray = GetMinimizeToTrayFromSettings();
        _state.DebugInfo = $"Saved Settings: \r\n{GetSavedSettings()}";
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

    private string GetSavedSettings()
    {
        var savedSettings = _configuration.AppSettings.Settings.AllKeys.Select(key => $"({key}: {_configuration.AppSettings.Settings[key]})");
        return String.Join(", ", savedSettings);
    }

    public bool GetTopmostFromSettings()
    {
        var fallbackValue = false;
        try
        {
            string? topMostSettings = _configuration.AppSettings?.Settings["Topmost"]?.Value;
            topMostSettings ??= fallbackValue.ToString();

            return bool.Parse(topMostSettings);
        }
        catch (Exception ex)
        {
            _state.DebugInfo = ex.ToString();
            return fallbackValue;
        }
    }

    public bool GetMinimizeToTrayFromSettings()
    {
        var fallbackValue = true;
        try
        {
            string? minimizeToTray = _configuration.AppSettings?.Settings["MinimizeToTray"]?.Value;
            minimizeToTray ??= fallbackValue.ToString();

            return bool.Parse(minimizeToTray);
        }
        catch (Exception ex)
        {
            _state.DebugInfo = ex.ToString();
            return fallbackValue;
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

            if (_configuration.AppSettings.Settings["Topmost"] == null)
                _configuration.AppSettings.Settings.Add("Topmost", _state.Topmost.ToString());
            else
                _configuration.AppSettings.Settings["Topmost"].Value = _state.Topmost.ToString();

            if (_configuration.AppSettings.Settings["MinimizeToTray"] == null)
                _configuration.AppSettings.Settings.Add("MinimizeToTray", _state.MinimizeToTray.ToString());
            else
                _configuration.AppSettings.Settings["MinimizeToTray"].Value = _state.MinimizeToTray.ToString();

            _configuration.Save(ConfigurationSaveMode.Full);
            ConfigurationManager.RefreshSection("appSettings");

            _state.DebugInfo = $"Settings saved successfuly.\r\nSaved color: {_state.SelectedColor}\r\nTopmost: {_state.Topmost}\r\nMinimizeToTray: {_state.MinimizeToTray}";
        }
        catch (Exception ex)
        {
            _state.DebugInfo = ex.ToString();
        }
    }
}
