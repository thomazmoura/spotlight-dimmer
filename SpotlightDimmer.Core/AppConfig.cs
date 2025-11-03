namespace SpotlightDimmer.Core;

/// <summary>
/// Profile representing a saved overlay configuration preset.
/// </summary>
public class Profile
{
    /// <summary>
    /// The name of the profile.
    /// </summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// The dimming mode: "FullScreen", "Partial", or "PartialWithActive".
    /// </summary>
    public string Mode { get; set; } = "FullScreen";

    /// <summary>
    /// Inactive overlay color in hex format (e.g., "#000000" for black).
    /// </summary>
    public string InactiveColor { get; set; } = "#000000";

    /// <summary>
    /// Inactive overlay opacity (0-255).
    /// </summary>
    public int InactiveOpacity { get; set; } = 153;

    /// <summary>
    /// Active overlay color in hex format (e.g., "#000000" for black).
    /// </summary>
    public string ActiveColor { get; set; } = "#000000";

    /// <summary>
    /// Active overlay opacity (0-255).
    /// </summary>
    public int ActiveOpacity { get; set; } = 102;
}

/// <summary>
/// System configuration settings.
/// </summary>
public class SystemConfig
{
    /// <summary>
    /// Enable file-based logging.
    /// Default: true
    /// </summary>
    public bool EnableLogging { get; set; } = true;

    /// <summary>
    /// Log level for file output: "Error", "Warning", "Information", or "Debug".
    /// Default: "Information"
    /// </summary>
    public string LogLevel { get; set; } = "Information";

    /// <summary>
    /// Number of days to retain log files. Older logs are automatically deleted.
    /// Default: 7
    /// </summary>
    public int LogRetentionDays { get; set; } = 7;
}

/// <summary>
/// Overlay configuration settings for dimming behavior.
/// </summary>
public class OverlayConfig
{
    /// <summary>
    /// The dimming mode: "FullScreen", "Partial", or "PartialWithActive".
    /// Default: "FullScreen"
    /// </summary>
    public string Mode { get; set; } = "FullScreen";

    /// <summary>
    /// Inactive overlay color in hex format (e.g., "#000000" for black).
    /// Default: "#000000"
    /// </summary>
    public string InactiveColor { get; set; } = "#000000";

    /// <summary>
    /// Inactive overlay opacity (0-255).
    /// Default: 153 (~60% opacity)
    /// </summary>
    public int InactiveOpacity { get; set; } = 153;

    /// <summary>
    /// Active overlay color in hex format (e.g., "#000000" for black).
    /// Default: "#000000"
    /// </summary>
    public string ActiveColor { get; set; } = "#000000";

    /// <summary>
    /// Active overlay opacity (0-255).
    /// Default: 102 (~40% opacity)
    /// </summary>
    public int ActiveOpacity { get; set; } = 102;

    /// <summary>
    /// Exclude overlay windows from screen captures (screenshots).
    /// Uses Windows SetWindowDisplayAffinity API with WDA_EXCLUDEFROMCAPTURE flag.
    /// Default: false (overlays appear in screenshots)
    ///
    /// EXPERIMENTAL: This feature may not work on all systems due to Windows API limitations
    /// with layered windows. Success rate varies by Windows version and system configuration.
    /// When enabled but not supported, overlays will still function normally but may appear in screenshots.
    /// </summary>
    public bool ExcludeFromScreenCapture { get; set; } = false;

    /// <summary>
    /// Enable vertical sync (VSync) for overlay updates.
    /// When enabled, overlay updates are synchronized to the display's refresh rate using DwmFlush(),
    /// which can reduce visual tearing during window movement but adds 8-16ms latency per update.
    /// Default: false (no explicit VSync, DWM handles composition)
    ///
    /// Performance Impact:
    /// - CPU: +5-10% during window movement/focus changes
    /// - Latency: +8-16ms per update (1-2 frames at 60Hz)
    /// - Benefits: Smoother visual updates, reduced tearing
    ///
    /// Note: Only works when Desktop Window Manager composition is enabled (default on Windows Vista+).
    /// </summary>
    public bool EnableVSync { get; set; } = false;
}

/// <summary>
/// Application configuration that can be serialized to/from JSON.
/// This represents the user-facing configuration structure.
/// </summary>
public class AppConfig
{
    /// <summary>
    /// Overlay configuration settings.
    /// </summary>
    public OverlayConfig Overlay { get; set; } = new();

    /// <summary>
    /// System configuration settings.
    /// </summary>
    public SystemConfig System { get; set; } = new();

    /// <summary>
    /// List of saved profiles for quick overlay configuration switching.
    /// </summary>
    public List<Profile> Profiles { get; set; } = new()
    {
        new Profile
        {
            Name = "Light Mode",
            Mode = "Partial",
            InactiveColor = "#000000",
            InactiveOpacity = 128,
            ActiveColor = "#000000",
            ActiveOpacity = 102
        },
        new Profile
        {
            Name = "Dark Mode",
            Mode = "PartialWithActive",
            InactiveColor = "#000000",
            InactiveOpacity = 204,
            ActiveColor = "#000000",
            ActiveOpacity = 128
        }
    };

    /// <summary>
    /// The name of the currently active profile, or null if none/custom.
    /// </summary>
    public string? CurrentProfile { get; set; } = null;

    /// <summary>
    /// Converts this AppConfig to an OverlayCalculationConfig.
    /// </summary>
    public OverlayCalculationConfig ToOverlayConfig()
    {
        return new OverlayCalculationConfig(
            Mode: ParseMode(Overlay.Mode),
            InactiveColor: ParseColor(Overlay.InactiveColor),
            InactiveOpacity: ClampOpacity(Overlay.InactiveOpacity),
            ActiveColor: ParseColor(Overlay.ActiveColor),
            ActiveOpacity: ClampOpacity(Overlay.ActiveOpacity)
        );
    }

    /// <summary>
    /// Creates an AppConfig from an OverlayCalculationConfig.
    /// </summary>
    public static AppConfig FromOverlayConfig(OverlayCalculationConfig config)
    {
        return new AppConfig
        {
            Overlay = new OverlayConfig
            {
                Mode = config.Mode.ToString(),
                InactiveColor = ColorToHex(config.InactiveColor),
                InactiveOpacity = config.InactiveOpacity,
                ActiveColor = ColorToHex(config.ActiveColor),
                ActiveOpacity = config.ActiveOpacity
            },
            System = new SystemConfig()
        };
    }

    /// <summary>
    /// Creates a default configuration.
    /// </summary>
    public static AppConfig Default => new();

    /// <summary>
    /// Applies a profile to the current overlay configuration.
    /// </summary>
    /// <param name="profileName">The name of the profile to apply.</param>
    /// <returns>True if the profile was found and applied, false otherwise.</returns>
    public bool ApplyProfile(string profileName)
    {
        var profile = Profiles.FirstOrDefault(p => p.Name == profileName);
        if (profile == null)
            return false;

        Overlay.Mode = profile.Mode;
        Overlay.InactiveColor = profile.InactiveColor;
        Overlay.InactiveOpacity = profile.InactiveOpacity;
        Overlay.ActiveColor = profile.ActiveColor;
        Overlay.ActiveOpacity = profile.ActiveOpacity;
        CurrentProfile = profileName;

        return true;
    }

    /// <summary>
    /// Checks if the current overlay configuration matches a specific profile.
    /// </summary>
    /// <param name="profileName">The name of the profile to check against.</param>
    /// <returns>True if the overlay matches the profile exactly, false otherwise.</returns>
    public bool DoesOverlayMatchProfile(string profileName)
    {
        var profile = Profiles.FirstOrDefault(p => p.Name == profileName);
        if (profile == null)
            return false;

        return Overlay.Mode == profile.Mode &&
               Overlay.InactiveColor == profile.InactiveColor &&
               Overlay.InactiveOpacity == profile.InactiveOpacity &&
               Overlay.ActiveColor == profile.ActiveColor &&
               Overlay.ActiveOpacity == profile.ActiveOpacity;
    }

    private static DimmingMode ParseMode(string mode)
    {
        return mode?.ToLowerInvariant() switch
        {
            "fullscreen" => DimmingMode.FullScreen,
            "partial" => DimmingMode.Partial,
            "partialwithactive" => DimmingMode.PartialWithActive,
            _ => DimmingMode.FullScreen
        };
    }

    private static Color ParseColor(string hex)
    {
        if (string.IsNullOrWhiteSpace(hex))
            return Color.Black;

        hex = hex.TrimStart('#');

        if (hex.Length != 6)
            return Color.Black;

        try
        {
            var r = Convert.ToByte(hex.Substring(0, 2), 16);
            var g = Convert.ToByte(hex.Substring(2, 2), 16);
            var b = Convert.ToByte(hex.Substring(4, 2), 16);
            return new Color(r, g, b);
        }
        catch
        {
            return Color.Black;
        }
    }

    private static string ColorToHex(Color color)
    {
        return $"#{color.R:X2}{color.G:X2}{color.B:X2}";
    }

    private static byte ClampOpacity(int opacity)
    {
        return (byte)Math.Clamp(opacity, 0, 255);
    }
}
