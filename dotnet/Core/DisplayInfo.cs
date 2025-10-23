namespace SpotlightDimmer.Core;

/// <summary>
/// Contains information about a single display.
/// Uses an index-based identifier instead of platform-specific handles.
/// </summary>
public readonly record struct DisplayInfo(
    /// <summary>
    /// The zero-based index of this display.
    /// The WindowsBindings layer maintains the mapping from index to HMONITOR.
    /// </summary>
    int Index,

    /// <summary>
    /// The rectangular bounds of this display in screen coordinates.
    /// </summary>
    Rectangle Bounds
);
