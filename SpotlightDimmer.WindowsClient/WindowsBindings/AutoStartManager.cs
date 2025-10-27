using Microsoft.Extensions.Logging;
using Microsoft.Win32;
using System.Reflection;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Manages auto-start at login functionality via Windows Registry.
/// Adds/removes registry entry in HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
/// </summary>
internal class AutoStartManager
{
    private readonly ILogger<AutoStartManager> _logger;
    private const string RUN_KEY_PATH = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string APP_NAME = "SpotlightDimmer";

    public AutoStartManager(ILogger<AutoStartManager> logger)
    {
        _logger = logger;
    }

    /// <summary>
    /// Enables auto-start at login by adding a registry entry.
    /// </summary>
    /// <returns>True if successful, false otherwise.</returns>
    public bool Enable()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RUN_KEY_PATH, writable: true);
            if (key == null)
            {
                _logger.LogError("Failed to open registry key for writing");
                return false;
            }

            // Get the path to the current executable
            var exePath = Environment.ProcessPath;
            if (string.IsNullOrEmpty(exePath))
            {
                _logger.LogError("Failed to get executable path");
                return false;
            }

            // Add registry entry with quoted path to handle spaces
            key.SetValue(APP_NAME, $"\"{exePath}\"", RegistryValueKind.String);
            _logger.LogInformation("Auto-start enabled - Executable: {ExePath}", exePath);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to enable auto-start");
            return false;
        }
    }

    /// <summary>
    /// Disables auto-start at login by removing the registry entry.
    /// </summary>
    /// <returns>True if successful, false otherwise.</returns>
    public bool Disable()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RUN_KEY_PATH, writable: true);
            if (key == null)
            {
                _logger.LogError("Failed to open registry key for writing");
                return false;
            }

            // Check if the entry exists before trying to delete
            if (key.GetValue(APP_NAME) != null)
            {
                key.DeleteValue(APP_NAME, throwOnMissingValue: false);
                _logger.LogInformation("Auto-start disabled");
                return true;
            }
            else
            {
                // Entry doesn't exist, consider it already disabled
                _logger.LogWarning("Auto-start already disabled");
                return true;
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to disable auto-start");
            return false;
        }
    }

    /// <summary>
    /// Checks if auto-start is currently enabled.
    /// </summary>
    /// <returns>True if enabled, false otherwise.</returns>
    public bool IsEnabled()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RUN_KEY_PATH, writable: false);
            if (key == null)
            {
                return false;
            }

            var value = key.GetValue(APP_NAME) as string;
            return !string.IsNullOrEmpty(value);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to check auto-start status");
            return false;
        }
    }
}
