using System.Text.Json;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.Config;

public partial class ConfigForm : Form
{
    private readonly ConfigurationManager _configManager;
    private bool _isLoading = false;

    public ConfigForm()
    {
        InitializeComponent();
        _configManager = new ConfigurationManager();

        // Subscribe to configuration changes for two-way binding
        _configManager.ConfigurationChanged += OnConfigurationFileChanged;

        LoadConfiguration();
    }

    private void LoadConfiguration()
    {
        _isLoading = true;
        try
        {
            var config = _configManager.Current;

            // Set mode
            modeComboBox.SelectedItem = config.Overlay.Mode;

            // Set inactive color and opacity
            var inactiveColor = ParseHexColor(config.Overlay.InactiveColor);
            inactiveColorPanel.BackColor = inactiveColor;
            inactiveOpacityTrackBar.Value = config.Overlay.InactiveOpacity;
            inactiveOpacityValueLabel.Text = config.Overlay.InactiveOpacity.ToString();

            // Set active color and opacity
            var activeColor = ParseHexColor(config.Overlay.ActiveColor);
            activeColorPanel.BackColor = activeColor;
            activeOpacityTrackBar.Value = config.Overlay.ActiveOpacity;
            activeOpacityValueLabel.Text = config.Overlay.ActiveOpacity.ToString();
        }
        finally
        {
            _isLoading = false;
        }
    }

    private void SelectColor(Panel panel, TrackBar opacityControl)
    {
        using var colorDialog = new ColorDialog
        {
            Color = panel.BackColor,
            FullOpen = true
        };

        if (colorDialog.ShowDialog() == DialogResult.OK)
        {
            panel.BackColor = colorDialog.Color;
            SaveConfiguration();
        }
    }

    private void OnConfigChanged(object? sender, EventArgs e)
    {
        if (!_isLoading)
        {
            SaveConfiguration();
        }
    }

    private void OnOpacityChanged(object? sender, EventArgs e)
    {
        if (!_isLoading)
        {
            // Update the value labels
            inactiveOpacityValueLabel.Text = inactiveOpacityTrackBar.Value.ToString();
            activeOpacityValueLabel.Text = activeOpacityTrackBar.Value.ToString();

            SaveConfiguration();
        }
    }

    private void OnConfigurationFileChanged(AppConfig config)
    {
        // ConfigurationChanged is fired from FileSystemWatcher on a background thread
        // We need to invoke on the UI thread to update controls
        if (InvokeRequired)
        {
            Invoke(new Action<AppConfig>(OnConfigurationFileChanged), config);
            return;
        }

        // Reload the UI with the new configuration
        LoadConfiguration();
    }

    private void SaveConfiguration()
    {
        try
        {
            var config = _configManager.Current;

            // Update overlay config
            config.Overlay.Mode = modeComboBox.SelectedItem?.ToString() ?? "FullScreen";
            config.Overlay.InactiveColor = ColorToHex(inactiveColorPanel.BackColor);
            config.Overlay.InactiveOpacity = inactiveOpacityTrackBar.Value;
            config.Overlay.ActiveColor = ColorToHex(activeColorPanel.BackColor);
            config.Overlay.ActiveOpacity = activeOpacityTrackBar.Value;

            _configManager.SaveConfiguration(config);
        }
        catch (Exception ex)
        {
            MessageBox.Show(
                $"Error saving configuration: {ex.Message}",
                "Error",
                MessageBoxButtons.OK,
                MessageBoxIcon.Error
            );
        }
    }

    private static System.Drawing.Color ParseHexColor(string hex)
    {
        hex = hex.TrimStart('#');
        if (hex.Length == 6)
        {
            var r = Convert.ToByte(hex.Substring(0, 2), 16);
            var g = Convert.ToByte(hex.Substring(2, 2), 16);
            var b = Convert.ToByte(hex.Substring(4, 2), 16);
            return System.Drawing.Color.FromArgb(r, g, b);
        }
        return System.Drawing.Color.Black;
    }

    private static string ColorToHex(System.Drawing.Color color)
    {
        return $"#{color.R:X2}{color.G:X2}{color.B:X2}";
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
        {
            if (_configManager != null)
            {
                _configManager.ConfigurationChanged -= OnConfigurationFileChanged;
                _configManager.Dispose();
            }
            components?.Dispose();
        }
        base.Dispose(disposing);
    }
}
