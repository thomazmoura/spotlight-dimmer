namespace SpotlightDimmer.Core;

/// <summary>
/// Represents the complete overlay state for a single display.
/// Contains all overlays (0-6) that should be rendered on this display.
/// </summary>
public readonly record struct DisplayOverlayState(
    /// <summary>
    /// The display index this state applies to.
    /// </summary>
    int DisplayIndex,

    /// <summary>
    /// The bounds of the display.
    /// Provided for convenience so the renderer doesn't need to look it up.
    /// </summary>
    Rectangle DisplayBounds,

    /// <summary>
    /// The overlay definitions for this display.
    /// - 0 overlays: Display is focused and no partial dimming is active
    /// - 1 overlay: Full-screen dimming on non-focused display
    /// - 4 overlays: Partial dimming (Top, Bottom, Left, Right)
    /// - 5 overlays: Partial with active (Top, Bottom, Left, Right, Center)
    /// - 6 overlays: All regions (rare, mainly for testing)
    /// </summary>
    OverlayDefinition[] Overlays
)
{
    /// <summary>
    /// Gets the number of visible overlays on this display.
    /// </summary>
    public int VisibleOverlayCount => Overlays.Count(o => o.IsVisible);
}
