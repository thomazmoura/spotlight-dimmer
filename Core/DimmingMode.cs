namespace SpotlightDimmer.Core;

/// <summary>
/// Defines the different dimming modes for overlay calculation.
/// </summary>
public enum DimmingMode
{
    /// <summary>
    /// Full-screen dimming mode.
    /// Each non-focused display shows a single full-screen overlay.
    /// Simple and performant - only one overlay per dimmed display.
    /// </summary>
    FullScreen,

    /// <summary>
    /// Partial dimming mode with 4-sided overlays.
    /// Creates overlays on the top, bottom, left, and right edges of the focused window,
    /// leaving a cutout in the middle where the active window is visible.
    /// Used when the focused window doesn't fill the entire display.
    /// </summary>
    Partial,

    /// <summary>
    /// Partial dimming mode with active window highlighting.
    /// Creates 4-sided overlays (top, bottom, left, right) with inactive color,
    /// plus a center overlay with active color to highlight the focused window.
    /// Provides visual distinction between the focused window area and dimmed areas.
    /// </summary>
    PartialWithActive
}
