namespace SpotlightDimmer.Core;

/// <summary>
/// Defines the complete state of a single overlay window.
/// This is a mutable class with settable properties to allow in-place updates.
/// Objects are pre-allocated and reused to eliminate per-frame allocations.
/// </summary>
public class OverlayDefinition
{
    /// <summary>
    /// The region this overlay covers (FullScreen, Top, Bottom, Left, Right, or Center).
    /// </summary>
    public OverlayRegion Region { get; set; }

    /// <summary>
    /// The rectangular bounds of this overlay in screen coordinates.
    /// </summary>
    public Rectangle Bounds { get; set; }

    /// <summary>
    /// The RGB color of this overlay.
    /// </summary>
    public Color Color { get; set; }

    /// <summary>
    /// The opacity of this overlay (0 = fully transparent, 255 = fully opaque).
    /// Common values: 153 (~60% opaque), 128 (50% opaque), 102 (~40% opaque).
    /// </summary>
    public byte Opacity { get; set; }

    /// <summary>
    /// Whether this overlay should be visible.
    /// When false, the overlay window should be hidden.
    /// </summary>
    public bool IsVisible { get; set; }

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
    public OverlayDefinition(OverlayRegion region)
    {
        Region = region;
        Bounds = default; // More efficient than new Rectangle(0, 0, 0, 0)
        Color = Color.Black;
        Opacity = 0;
        IsVisible = false;
    }

    /// <summary>
    /// Sets this overlay to be hidden (zero-allocation reset).
    /// </summary>
    public void Hide()
    {
        IsVisible = false;
        Bounds = default; // More efficient than new Rectangle(0, 0, 0, 0)
        Opacity = 0;
    }

    /// <summary>
    /// Updates all properties of this overlay in place.
    /// </summary>
    public void Update(Rectangle bounds, Color color, byte opacity, bool isVisible)
    {
        Bounds = bounds;
        Color = color;
        Opacity = opacity;
        IsVisible = isVisible;
    }

    /// <summary>
    /// Copies all values from another OverlayDefinition to this one.
    /// Used for zero-allocation updates where we need to transfer state
    /// without creating new objects or storing references.
    /// </summary>
    public void CopyFrom(OverlayDefinition source)
    {
        Region = source.Region;
        Bounds = source.Bounds;
        Color = source.Color;
        Opacity = source.Opacity;
        IsVisible = source.IsVisible;
    }
}
