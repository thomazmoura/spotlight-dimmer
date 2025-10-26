using Microsoft.Win32;
using System.Reflection;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Manages auto-start at login functionality via Windows Registry.
/// Adds/removes registry entry in HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
/// </summary>
internal static class AutoStartManager
{
    private const string RUN_KEY_PATH = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string APP_NAME = "SpotlightDimmer";

    /// <summary>
    /// Enables auto-start at login by adding a registry entry.
    /// </summary>
    /// <returns>True if successful, false otherwise.</returns>
    public static bool Enable()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RUN_KEY_PATH, writable: true);
            if (key == null)
            {
                Console.WriteLine("[AutoStart] Failed to open registry key for writing");
                return false;
            }

            // Get the path to the current executable
            var exePath = Environment.ProcessPath;
            if (string.IsNullOrEmpty(exePath))
            {
                Console.WriteLine("[AutoStart] Failed to get executable path");
                return false;
            }

            // Add registry entry with quoted path to handle spaces
            key.SetValue(APP_NAME, $"\"{exePath}\"", RegistryValueKind.String);
            Console.WriteLine($"[AutoStart] Enabled - Executable: {exePath}");
            return true;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[AutoStart] Failed to enable: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Disables auto-start at login by removing the registry entry.
    /// </summary>
    /// <returns>True if successful, false otherwise.</returns>
    public static bool Disable()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RUN_KEY_PATH, writable: true);
            if (key == null)
            {
                Console.WriteLine("[AutoStart] Failed to open registry key for writing");
                return false;
            }

            // Check if the entry exists before trying to delete
            if (key.GetValue(APP_NAME) != null)
            {
                key.DeleteValue(APP_NAME, throwOnMissingValue: false);
                Console.WriteLine("[AutoStart] Disabled");
                return true;
            }
            else
            {
                // Entry doesn't exist, consider it already disabled
                Console.WriteLine("[AutoStart] Already disabled");
                return true;
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[AutoStart] Failed to disable: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Checks if auto-start is currently enabled.
    /// </summary>
    /// <returns>True if enabled, false otherwise.</returns>
    public static bool IsEnabled()
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
            Console.WriteLine($"[AutoStart] Failed to check status: {ex.Message}");
            return false;
        }
    }
}
