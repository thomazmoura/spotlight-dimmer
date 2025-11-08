using SpotlightDimmer.Core;

namespace SpotlightDimmer.WinUI3Renderer;

/// <summary>
/// Abstraction for rendering overlay windows using different Windows APIs.
/// Implementations can use different rendering technologies (layered windows, DirectComposition, etc.)
/// while consuming the same Core layer data structures.
/// </summary>
internal interface IOverlayRenderer : IDisposable
{
    /// <summary>
    /// Pre-creates all overlay windows for the given displays.
    /// Creates 6 overlays per display (one for each OverlayRegion), all initially hidden.
    /// Call this once at startup after getting display information.
    /// </summary>
    void CreateOverlays(Core.DisplayInfo[] displays, OverlayCalculationConfig config);

    /// <summary>
    /// Updates overlay colors when configuration changes.
    /// Recreates rendering resources with new colors to reflect updated config.
    /// </summary>
    void UpdateBrushColors(OverlayCalculationConfig config);

    /// <summary>
    /// Updates all overlay windows based on the calculated overlay states.
    /// ZERO allocations - should reuse pre-allocated resources where possible.
    /// </summary>
    void UpdateOverlays(DisplayOverlayState[] states);

    /// <summary>
    /// Updates the screen capture exclusion setting for all overlay windows.
    /// Uses SetWindowDisplayAffinity with WDA_EXCLUDEFROMCAPTURE to hide windows from screenshots.
    /// Returns the count of windows that were successfully updated.
    /// EXPERIMENTAL: May not work on all systems due to Windows API limitations.
    /// </summary>
    int UpdateScreenCaptureExclusion(bool exclude);

    /// <summary>
    /// Hides all overlay windows (for pause functionality).
    /// </summary>
    void HideAllOverlays();

    /// <summary>
    /// Disposes all existing overlay windows and clears internal state.
    /// Call this before recreating overlays when display configuration changes.
    /// </summary>
    void CleanupOverlays();
}
