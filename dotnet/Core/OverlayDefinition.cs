namespace SpotlightDimmer.Core;

/// <summary>
/// Defines the complete state of a single overlay window.
/// This is a readonly struct to avoid heap allocations in the hot path.
/// </summary>
public readonly struct OverlayDefinition
{
    /// <summary>
    /// The region this overlay covers (FullScreen, Top, Bottom, Left, Right, or Center).
    /// </summary>
    public OverlayRegion Region { get; init; }

    /// <summary>
    /// The rectangular bounds of this overlay in screen coordinates.
    /// </summary>
    public Rectangle Bounds { get; init; }

    /// <summary>
    /// The RGB color of this overlay.
    /// </summary>
    public Color Color { get; init; }

    /// <summary>
    /// The opacity of this overlay (0 = fully transparent, 255 = fully opaque).
    /// Common values: 153 (~60% opaque), 128 (50% opaque), 102 (~40% opaque).
    /// </summary>
    public byte Opacity { get; init; }

    /// <summary>
    /// Whether this overlay should be visible.
    /// When false, the overlay window should be hidden.
    /// </summary>
    public bool IsVisible { get; init; }

    /// <summary>
    /// Creates a new overlay definition with all properties.
    /// </summary>
    public OverlayDefinition(OverlayRegion region, Rectangle bounds, Color color, byte opacity, bool isVisible)
    {
        Region = region;
        Bounds = bounds;
        Color = color;
        Opacity = opacity;
        IsVisible = isVisible;
    }

    /// <summary>
    /// Creates a hidden overlay for the specified region.
    /// Useful for initializing overlays that aren't currently needed.
    /// </summary>
    public static OverlayDefinition CreateHidden(OverlayRegion region)
    {
        return new OverlayDefinition
        {
            Region = region,
            Bounds = new Rectangle(0, 0, 0, 0),
            Color = Color.Black,
            Opacity = 0,
            IsVisible = false
        };
    }
}
