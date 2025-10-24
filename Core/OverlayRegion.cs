namespace SpotlightDimmer.Core;

/// <summary>
/// Defines the six possible overlay regions for each display.
/// Each display can have up to 6 overlays active at once.
/// </summary>
public enum OverlayRegion
{
    /// <summary>
    /// Full-screen overlay covering the entire display.
    /// Used in FullScreen dimming mode for non-focused displays.
    /// Mutually exclusive with other overlay regions.
    /// </summary>
    FullScreen,

    /// <summary>
    /// Top edge overlay.
    /// Covers the area from the top of the display to the top of the focused window.
    /// Used in Partial and PartialWithActive dimming modes.
    /// </summary>
    Top,

    /// <summary>
    /// Bottom edge overlay.
    /// Covers the area from the bottom of the focused window to the bottom of the display.
    /// Used in Partial and PartialWithActive dimming modes.
    /// </summary>
    Bottom,

    /// <summary>
    /// Left edge overlay.
    /// Covers the area from the left of the display to the left of the focused window.
    /// Used in Partial and PartialWithActive dimming modes.
    /// </summary>
    Left,

    /// <summary>
    /// Right edge overlay.
    /// Covers the area from the right of the focused window to the right of the display.
    /// Used in Partial and PartialWithActive dimming modes.
    /// </summary>
    Right,

    /// <summary>
    /// Center overlay covering the focused window area.
    /// Used in PartialWithActive dimming mode to highlight the active window.
    /// Rendered with a different color/opacity than the edge overlays.
    /// </summary>
    Center
}
