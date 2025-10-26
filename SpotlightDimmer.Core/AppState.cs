namespace SpotlightDimmer.Core;

/// <summary>
/// Represents the complete application state with pre-allocated overlay definitions.
/// This class holds all DisplayOverlayState objects and provides in-place update methods
/// to eliminate per-frame allocations. The Calculate method updates values without creating
/// new objects, achieving zero-allocation updates in the hot path.
/// </summary>
public class AppState
{
    /// <summary>
    /// Array of display overlay states, one per display.
    /// These are pre-allocated and reused across all calculations.
    /// </summary>
    public DisplayOverlayState[] DisplayStates { get; }

    /// <summary>
    /// Creates a new AppState with pre-allocated overlay states for each display.
    /// </summary>
    /// <param name="displays">All connected displays.</param>
    public AppState(ReadOnlySpan<DisplayInfo> displays)
    {
        DisplayStates = new DisplayOverlayState[displays.Length];

        // Pre-allocate DisplayOverlayState for each display
        for (int i = 0; i < displays.Length; i++)
        {
            var display = displays[i];
            DisplayStates[i] = new DisplayOverlayState(display.Index, display.Bounds);
        }
    }

    /// <summary>
    /// Updates all overlay states in place based on current focus and configuration.
    /// This method performs zero allocations - it only updates existing objects.
    /// </summary>
    /// <param name="displays">All connected displays.</param>
    /// <param name="focusedWindowBounds">Bounds of the currently focused window (null if no focus).</param>
    /// <param name="focusedDisplayIndex">Index of the display containing the focused window (-1 if none).</param>
    /// <param name="config">Overlay calculation configuration.</param>
    public void Calculate(
        ReadOnlySpan<DisplayInfo> displays,
        Rectangle? focusedWindowBounds,
        int focusedDisplayIndex,
        OverlayCalculationConfig config)
    {
        for (int i = 0; i < displays.Length; i++)
        {
            var display = displays[i];
            var state = DisplayStates[i];

            // Reset all overlays to hidden before calculating new state
            HideAllOverlays(state);

            bool isFocusedDisplay = (i == focusedDisplayIndex);

            if (isFocusedDisplay && focusedWindowBounds.HasValue)
            {
                // This display has the focused window
                switch (config.Mode)
                {
                    case DimmingMode.FullScreen:
                        // No overlays - keep all hidden
                        break;
                    case DimmingMode.Partial:
                        UpdatePartialOverlays(state, display, focusedWindowBounds.Value, config);
                        break;
                    case DimmingMode.PartialWithActive:
                        UpdatePartialWithActiveOverlays(state, display, focusedWindowBounds.Value, config);
                        break;
                }
            }
            else
            {
                // This display does not have the focused window - dim the entire display
                UpdateFullScreenOverlay(state, display, config);
            }
        }
    }

    /// <summary>
    /// Hides all overlays in the state by calling Hide() on each.
    /// Zero allocations - just updates existing objects.
    /// </summary>
    private void HideAllOverlays(DisplayOverlayState state)
    {
        for (int i = 0; i < state.Overlays.Length; i++)
        {
            state.Overlays[i].Hide();
        }
    }

    /// <summary>
    /// Updates the state to show a single full-screen overlay for the entire display.
    /// Used in FullScreen mode for non-focused displays.
    /// Updates existing overlay object in place - no allocations.
    /// </summary>
    private void UpdateFullScreenOverlay(DisplayOverlayState state, DisplayInfo display, OverlayCalculationConfig config)
    {
        int index = (int)OverlayRegion.FullScreen;
        state.Overlays[index].Update(
            display.Bounds,
            config.InactiveColor,
            config.InactiveOpacity,
            true
        );
    }

    /// <summary>
    /// Updates the state with 4-sided overlays around the focused window (Partial mode).
    /// Top, Bottom, Left, Right overlays with inactive color.
    /// Updates existing overlay objects in place - no allocations.
    /// </summary>
    private void UpdatePartialOverlays(
        DisplayOverlayState state,
        DisplayInfo display,
        Rectangle windowBounds,
        OverlayCalculationConfig config)
    {
        // Clamp window bounds to display bounds (in case window extends beyond display)
        var clampedWindow = ClampToDisplay(windowBounds, display.Bounds);

        // Top overlay: full width, from display top to window top
        if (clampedWindow.Top > display.Bounds.Top)
        {
            int index = (int)OverlayRegion.Top;
            state.Overlays[index].Update(
                new Rectangle(
                    display.Bounds.Left,
                    display.Bounds.Top,
                    display.Bounds.Width,
                    clampedWindow.Top - display.Bounds.Top
                ),
                config.InactiveColor,
                config.InactiveOpacity,
                true
            );
        }

        // Bottom overlay: full width, from window bottom to display bottom
        if (clampedWindow.Bottom < display.Bounds.Bottom)
        {
            int index = (int)OverlayRegion.Bottom;
            state.Overlays[index].Update(
                new Rectangle(
                    display.Bounds.Left,
                    clampedWindow.Bottom,
                    display.Bounds.Width,
                    display.Bounds.Bottom - clampedWindow.Bottom
                ),
                config.InactiveColor,
                config.InactiveOpacity,
                true
            );
        }

        // Left overlay: from window top to window bottom, from display left to window left
        if (clampedWindow.Left > display.Bounds.Left)
        {
            int index = (int)OverlayRegion.Left;
            state.Overlays[index].Update(
                new Rectangle(
                    display.Bounds.Left,
                    clampedWindow.Top,
                    clampedWindow.Left - display.Bounds.Left,
                    clampedWindow.Height
                ),
                config.InactiveColor,
                config.InactiveOpacity,
                true
            );
        }

        // Right overlay: from window top to window bottom, from window right to display right
        if (clampedWindow.Right < display.Bounds.Right)
        {
            int index = (int)OverlayRegion.Right;
            state.Overlays[index].Update(
                new Rectangle(
                    clampedWindow.Right,
                    clampedWindow.Top,
                    display.Bounds.Right - clampedWindow.Right,
                    clampedWindow.Height
                ),
                config.InactiveColor,
                config.InactiveOpacity,
                true
            );
        }
    }

    /// <summary>
    /// Updates the state with 4-sided overlays plus center highlight (PartialWithActive mode).
    /// Top, Bottom, Left, Right overlays with inactive color.
    /// Center overlay with active color to highlight the focused window.
    /// Updates existing overlay objects in place - no allocations.
    /// </summary>
    private void UpdatePartialWithActiveOverlays(
        DisplayOverlayState state,
        DisplayInfo display,
        Rectangle windowBounds,
        OverlayCalculationConfig config)
    {
        // Start with the 4-sided overlays
        UpdatePartialOverlays(state, display, windowBounds, config);

        // Add center overlay with active color
        var clampedWindow = ClampToDisplay(windowBounds, display.Bounds);
        int index = (int)OverlayRegion.Center;
        state.Overlays[index].Update(
            clampedWindow,
            config.ActiveColor,
            config.ActiveOpacity,
            true
        );
    }

    /// <summary>
    /// Clamps a rectangle to fit within display bounds.
    /// Ensures overlays don't extend beyond the display.
    /// </summary>
    private Rectangle ClampToDisplay(Rectangle rect, Rectangle displayBounds)
    {
        int left = Math.Max(rect.Left, displayBounds.Left);
        int top = Math.Max(rect.Top, displayBounds.Top);
        int right = Math.Min(rect.Right, displayBounds.Right);
        int bottom = Math.Min(rect.Bottom, displayBounds.Bottom);

        // Ensure width and height are non-negative
        int width = Math.Max(0, right - left);
        int height = Math.Max(0, bottom - top);

        return new Rectangle(left, top, width, height);
    }
}
