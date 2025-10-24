namespace SpotlightDimmer.Core;

/// <summary>
/// Configuration for overlay calculation.
/// Passed to OverlayCalculator to control dimming behavior.
/// </summary>
public readonly record struct OverlayCalculationConfig(
    /// <summary>
    /// The dimming mode to use.
    /// </summary>
    DimmingMode Mode,

    /// <summary>
    /// The color to use for inactive/dimmed overlays.
    /// Typically black, but could be any color.
    /// </summary>
    Color InactiveColor,

    /// <summary>
    /// The opacity for inactive/dimmed overlays (0-255).
    /// Common value: 153 (~60% opaque) for good dimming effect.
    /// </summary>
    byte InactiveOpacity,

    /// <summary>
    /// The color to use for the active window highlight overlay (PartialWithActive mode only).
    /// Typically black or a subtle tint.
    /// </summary>
    Color ActiveColor,

    /// <summary>
    /// The opacity for the active window highlight overlay (0-255).
    /// Should be less than InactiveOpacity to create a "spotlight" effect.
    /// Common value: 102 (~40% opaque) for subtle highlighting.
    /// </summary>
    byte ActiveOpacity
)
{
    /// <summary>
    /// Creates a default configuration with black dimming at 60% opacity.
    /// </summary>
    public static OverlayCalculationConfig Default => new(
        Mode: DimmingMode.FullScreen,
        InactiveColor: Color.Black,
        InactiveOpacity: 153,
        ActiveColor: Color.Black,
        ActiveOpacity: 102
    );
}
