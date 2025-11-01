namespace SpotlightDimmer.Core;

/// <summary>
/// Interface for updating overlays when focus or position changes occur.
/// This abstraction allows testing of focus change logic without Windows dependencies.
/// </summary>
public interface IOverlayUpdateService
{
    /// <summary>
    /// Updates the overlays for the current focused window.
    /// </summary>
    /// <param name="displayIndex">The index of the display containing the focused window.</param>
    /// <param name="windowBounds">The bounds of the focused window.</param>
    void UpdateOverlays(int displayIndex, Rectangle windowBounds);
}
