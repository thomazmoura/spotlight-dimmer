namespace SpotlightDimmer.Core;

/// <summary>
/// Represents the complete overlay state for a single display.
/// Contains a fixed array of 6 overlays (one per region), with visibility flags.
/// This design minimizes allocations by reusing the same array structure.
/// </summary>
public class DisplayOverlayState
{
    /// <summary>
    /// The display index this state applies to.
    /// </summary>
    public int DisplayIndex { get; }

    /// <summary>
    /// The bounds of the display.
    /// Provided for convenience so the renderer doesn't need to look it up.
    /// </summary>
    public Rectangle DisplayBounds { get; }

    /// <summary>
    /// Fixed array of overlay definitions, one per region.
    /// Array is indexed by OverlayRegion enum value.
    /// Use IsVisible property to determine if overlay should be shown.
    /// </summary>
    public OverlayDefinition[] Overlays { get; }

    public DisplayOverlayState(int displayIndex, Rectangle displayBounds)
    {
        DisplayIndex = displayIndex;
        DisplayBounds = displayBounds;

        // Pre-allocate all 6 overlay slots (one per OverlayRegion)
        Overlays = new OverlayDefinition[6];

        // Initialize all as hidden
        for (int i = 0; i < Overlays.Length; i++)
        {
            Overlays[i] = new OverlayDefinition((OverlayRegion)i);
        }
    }

    /// <summary>
    /// Gets the number of visible overlays on this display.
    /// </summary>
    public int VisibleOverlayCount
    {
        get
        {
            int count = 0;
            for (int i = 0; i < Overlays.Length; i++)
            {
                if (Overlays[i].IsVisible)
                    count++;
            }
            return count;
        }
    }

    /// <summary>
    /// Gets all visible overlays (avoids allocation in most use cases).
    /// </summary>
    public IEnumerable<OverlayDefinition> VisibleOverlays
    {
        get
        {
            for (int i = 0; i < Overlays.Length; i++)
            {
                if (Overlays[i].IsVisible)
                    yield return Overlays[i];
            }
        }
    }
}
