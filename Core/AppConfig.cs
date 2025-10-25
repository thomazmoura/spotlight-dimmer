namespace SpotlightDimmer.Core;

/// <summary>
/// System configuration settings.
/// </summary>
public class SystemConfig
{
    /// <summary>
    /// Enable verbose logging for debugging purposes.
    /// Default: false
    /// </summary>
    public bool VerboseLoggingEnabled { get; set; } = false;
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
