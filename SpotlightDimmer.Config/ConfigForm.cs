using System.Reflection;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using SpotlightDimmer.Core;
using SpotlightDimmer.WindowsBindings;

namespace SpotlightDimmer.Config;

public partial class ConfigForm : Form
{
    private readonly ConfigurationManager _configManager;
    private readonly ILogger<ConfigForm> _logger;
    private bool _isLoading = false;

    public ConfigForm()
    {
        InitializeComponent();

        // Initialize file-based logging with default settings
        // This ensures we capture config operations, parse errors, etc.
        var loggerFactory = LoggingConfiguration.Initialize(AppConfig.Default);
        _logger = loggerFactory.CreateLogger<ConfigForm>();

        // Get application version for schema URL generation
        var appVersion = Assembly.GetExecutingAssembly()
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?
            .InformationalVersion ?? "0.0.0";

        _logger.LogInformation("SpotlightDimmer Config App v{Version} starting", appVersion);

        // Create ConfigurationManager with proper logging
        _configManager = new ConfigurationManager(LoggingConfiguration.GetLogger<ConfigurationManager>(), appVersion);

        // Reconfigure logging based on loaded settings
        LoggingConfiguration.Reconfigure(_configManager.Current);
        _logger.LogInformation("Config app initialized with LogLevel={LogLevel}", _configManager.Current.System.LogLevel);

        // Load and set the paused icon for the config app
        var iconPath = Path.Combine(AppContext.BaseDirectory, "spotlight-dimmer-icon-paused.ico");
        if (File.Exists(iconPath))
        {
            Icon = new Icon(iconPath);
        }

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

            // Populate profile list
            PopulateProfileList();

            // Set selected profile
            if (!string.IsNullOrEmpty(config.CurrentProfile))
            {
                profileComboBox.SelectedItem = config.CurrentProfile;
            }
            else
            {
                profileComboBox.SelectedIndex = -1; // No profile selected
            }

            // Update delete button state
            deleteProfileButton.Enabled = profileComboBox.SelectedIndex != -1;

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

            // Set renderer backend
            rendererBackendComboBox.SelectedItem = config.System.RendererBackend;

            // Set logging configuration
            enableLoggingCheckBox.Checked = config.System.EnableLogging;
            logLevelComboBox.SelectedItem = config.System.LogLevel;
            logRetentionDaysNumericUpDown.Value = config.System.LogRetentionDays;

            // Set experimental features
            excludeFromScreenCaptureCheckBox.Checked = config.Overlay.ExcludeFromScreenCapture;
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
            // Save changes to OverlayConfig immediately
            // Profile selection remains unchanged - user must explicitly save to update profile
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

            // Save changes to OverlayConfig immediately
            // Profile selection remains unchanged - user must explicitly save to update profile
            SaveConfiguration();
        }
    }

    private void OnProfileSelected(object? sender, EventArgs e)
    {
        if (_isLoading)
            return;

        // Handle deselection (when dropdown is cleared)
        if (profileComboBox.SelectedItem == null)
        {
            deleteProfileButton.Enabled = false;
            return;
        }

        var profileName = profileComboBox.SelectedItem.ToString();
        if (string.IsNullOrEmpty(profileName))
        {
            deleteProfileButton.Enabled = false;
            return;
        }

        // Apply the selected profile to OverlayConfig
        _logger.LogInformation("Applying profile: {ProfileName}", profileName);
        var config = _configManager.Current;
        if (config.ApplyProfile(profileName))
        {
            // Save the config directly (bypass SaveConfiguration which reads from UI controls)
            _configManager.SaveConfiguration(config);

            // Now reload the UI to reflect the profile's values
            LoadConfiguration();
        }
        else
        {
            _logger.LogWarning("Failed to apply profile: {ProfileName}", profileName);
        }

        deleteProfileButton.Enabled = true;
    }

    private void OnSaveProfile(object? sender, EventArgs e)
    {
        var config = _configManager.Current;

        // Pre-fill with current profile name if one is selected
        var defaultName = profileComboBox.SelectedItem?.ToString() ?? "";

        // Show input dialog
        var profileName = ShowInputDialog("Save Profile", "Profile name:", defaultName);
        if (string.IsNullOrWhiteSpace(profileName))
            return;

        // Check if profile already exists
        var existingProfile = config.Profiles.FirstOrDefault(p => p.Name == profileName);
        if (existingProfile != null)
        {
            _logger.LogInformation("Updating existing profile: {ProfileName}", profileName);
            // Update existing profile
            existingProfile.Mode = config.Overlay.Mode;
            existingProfile.InactiveColor = config.Overlay.InactiveColor;
            existingProfile.InactiveOpacity = config.Overlay.InactiveOpacity;
            existingProfile.ActiveColor = config.Overlay.ActiveColor;
            existingProfile.ActiveOpacity = config.Overlay.ActiveOpacity;
        }
        else
        {
            _logger.LogInformation("Creating new profile: {ProfileName}", profileName);
            // Create new profile
            config.Profiles.Add(new SpotlightDimmer.Core.Profile
            {
                Name = profileName,
                Mode = config.Overlay.Mode,
                InactiveColor = config.Overlay.InactiveColor,
                InactiveOpacity = config.Overlay.InactiveOpacity,
                ActiveColor = config.Overlay.ActiveColor,
                ActiveOpacity = config.Overlay.ActiveOpacity
            });
        }

        config.CurrentProfile = profileName;
        SaveConfiguration();
        LoadConfiguration(); // Refresh profile list and selection
    }

    private void OnDeleteProfile(object? sender, EventArgs e)
    {
        if (profileComboBox.SelectedItem == null)
            return;

        var profileName = profileComboBox.SelectedItem.ToString();
        if (string.IsNullOrEmpty(profileName))
            return;

        var result = MessageBox.Show(
            $"Are you sure you want to delete the profile '{profileName}'?",
            "Delete Profile",
            MessageBoxButtons.YesNo,
            MessageBoxIcon.Question
        );

        if (result != DialogResult.Yes)
            return;

        var config = _configManager.Current;
        var profile = config.Profiles.FirstOrDefault(p => p.Name == profileName);
        if (profile != null)
        {
            _logger.LogInformation("Deleting profile: {ProfileName}", profileName);
            config.Profiles.Remove(profile);

            // Clear current profile if it was the deleted one
            if (config.CurrentProfile == profileName)
            {
                config.CurrentProfile = null;
            }

            SaveConfiguration();
            LoadConfiguration(); // Refresh profile list
        }
        else
        {
            _logger.LogWarning("Attempted to delete non-existent profile: {ProfileName}", profileName);
        }
    }

    private void OnEnableLoggingChanged(object? sender, EventArgs e)
    {
        if (!_isLoading)
        {
            var config = _configManager.Current;
            config.System.EnableLogging = enableLoggingCheckBox.Checked;
            SaveConfiguration();
        }
    }

    private void OnLogLevelChanged(object? sender, EventArgs e)
    {
        if (!_isLoading)
        {
            var config = _configManager.Current;
            config.System.LogLevel = logLevelComboBox.SelectedItem?.ToString() ?? "Information";
            SaveConfiguration();
        }
    }

    private void OnLogRetentionDaysChanged(object? sender, EventArgs e)
    {
        if (!_isLoading)
        {
            var config = _configManager.Current;
            config.System.LogRetentionDays = (int)logRetentionDaysNumericUpDown.Value;
            SaveConfiguration();
        }
    }

    private void OnRendererBackendChanged(object? sender, EventArgs e)
    {
        if (!_isLoading)
        {
            var config = _configManager.Current;
            config.System.RendererBackend = rendererBackendComboBox.SelectedItem?.ToString() ?? "Legacy";
            SaveConfiguration();
        }
    }

    private void OnExcludeFromScreenCaptureChanged(object? sender, EventArgs e)
    {
        if (!_isLoading)
        {
            var config = _configManager.Current;
            config.Overlay.ExcludeFromScreenCapture = excludeFromScreenCaptureCheckBox.Checked;
            SaveConfiguration();
        }
    }

    private void OnOpenLogsFolderClicked(object? sender, EventArgs e)
    {
        try
        {
            var appDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
            var logsDir = Path.Combine(appDataPath, "SpotlightDimmer", "logs");

            // Create directory if it doesn't exist
            if (!Directory.Exists(logsDir))
            {
                Directory.CreateDirectory(logsDir);
            }

            var processStartInfo = new System.Diagnostics.ProcessStartInfo
            {
                FileName = logsDir,
                UseShellExecute = true
            };

            System.Diagnostics.Process.Start(processStartInfo);
        }
        catch (Exception ex)
        {
            MessageBox.Show($"Failed to open logs folder: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }

    private void PopulateProfileList()
    {
        var config = _configManager.Current;
        profileComboBox.Items.Clear();

        foreach (var profile in config.Profiles)
        {
            profileComboBox.Items.Add(profile.Name);
        }
    }

    private static string? ShowInputDialog(string title, string prompt, string defaultValue = "")
    {
        var inputForm = new Form
        {
            Text = title,
            Width = 400,
            Height = 150,
            FormBorderStyle = FormBorderStyle.FixedDialog,
            StartPosition = FormStartPosition.CenterParent,
            MaximizeBox = false,
            MinimizeBox = false
        };

        var label = new Label
        {
            Text = prompt,
            Left = 20,
            Top = 20,
            Width = 350
        };

        var textBox = new TextBox
        {
            Left = 20,
            Top = 50,
            Width = 340,
            Text = defaultValue
        };

        var okButton = new Button
        {
            Text = "OK",
            Left = 200,
            Top = 80,
            Width = 75,
            DialogResult = DialogResult.OK
        };

        var cancelButton = new Button
        {
            Text = "Cancel",
            Left = 285,
            Top = 80,
            Width = 75,
            DialogResult = DialogResult.Cancel
        };

        inputForm.Controls.Add(label);
        inputForm.Controls.Add(textBox);
        inputForm.Controls.Add(okButton);
        inputForm.Controls.Add(cancelButton);
        inputForm.AcceptButton = okButton;
        inputForm.CancelButton = cancelButton;

        return inputForm.ShowDialog() == DialogResult.OK ? textBox.Text : null;
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
            config.Overlay.ExcludeFromScreenCapture = excludeFromScreenCaptureCheckBox.Checked;

            _logger.LogDebug("Saving configuration: Mode={Mode}, InactiveOpacity={InactiveOpacity}, ActiveOpacity={ActiveOpacity}, ExcludeFromScreenCapture={ExcludeFromScreenCapture}",
                config.Overlay.Mode, config.Overlay.InactiveOpacity, config.Overlay.ActiveOpacity, config.Overlay.ExcludeFromScreenCapture);

            _configManager.SaveConfiguration(config);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error saving configuration");
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
