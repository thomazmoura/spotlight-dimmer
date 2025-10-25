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
            inactiveOpacityNumeric.Value = config.Overlay.InactiveOpacity;

            // Set active color and opacity
            var activeColor = ParseHexColor(config.Overlay.ActiveColor);
            activeColorPanel.BackColor = activeColor;
            activeOpacityNumeric.Value = config.Overlay.ActiveOpacity;
        }
        finally
        {
            _isLoading = false;
        }
    }

    private void SelectColor(Panel panel, NumericUpDown opacityControl)
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

    private void SaveConfiguration()
    {
        try
        {
            var config = _configManager.Current;

            // Update overlay config
            config.Overlay.Mode = modeComboBox.SelectedItem?.ToString() ?? "FullScreen";
            config.Overlay.InactiveColor = ColorToHex(inactiveColorPanel.BackColor);
            config.Overlay.InactiveOpacity = (int)inactiveOpacityNumeric.Value;
            config.Overlay.ActiveColor = ColorToHex(activeColorPanel.BackColor);
            config.Overlay.ActiveOpacity = (int)activeOpacityNumeric.Value;

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
            _configManager?.Dispose();
            components?.Dispose();
        }
        base.Dispose(disposing);
    }
}
