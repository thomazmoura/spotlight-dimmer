namespace SpotlightDimmer.Core;

/// <summary>
/// Calculates overlay positions and states based on display configuration and focused window.
/// Uses instance-based design with object reuse to minimize GC pressure.
/// Maintains a pool of DisplayOverlayState objects that are updated in place.
/// </summary>
public class OverlayCalculator
{
    // Pool of DisplayOverlayState objects indexed by display index
    // These are reused across calculations to avoid allocations
    private readonly Dictionary<int, DisplayOverlayState> _statePool = new();

    /// <summary>
    /// Calculates the overlay states for all displays based on current focus and configuration.
    /// Reuses existing DisplayOverlayState objects and updates them in place.
    /// </summary>
    /// <param name="displays">All connected displays.</param>
    /// <param name="focusedWindowBounds">Bounds of the currently focused window (null if no focus).</param>
    /// <param name="focusedDisplayIndex">Index of the display containing the focused window (-1 if none).</param>
    /// <param name="config">Overlay calculation configuration.</param>
    /// <returns>Array of overlay states, one per display.</returns>
    public DisplayOverlayState[] Calculate(
        ReadOnlySpan<DisplayInfo> displays,
        Rectangle? focusedWindowBounds,
        int focusedDisplayIndex,
        OverlayCalculationConfig config)
    {
        var results = new DisplayOverlayState[displays.Length];

        for (int i = 0; i < displays.Length; i++)
        {
            var display = displays[i];

            // Get or create the state object for this display
            if (!_statePool.TryGetValue(display.Index, out var state))
            {
                state = new DisplayOverlayState(display.Index, display.Bounds);
                _statePool[display.Index] = state;
            }

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

            results[i] = state;
        }

        return results;
    }

    /// <summary>
    /// Hides all overlays in the state by setting IsVisible to false.
    /// This is more efficient than recreating the array.
    /// </summary>
    private void HideAllOverlays(DisplayOverlayState state)
    {
        for (int i = 0; i < state.Overlays.Length; i++)
        {
            state.Overlays[i] = state.Overlays[i] with { IsVisible = false };
        }
    }

    /// <summary>
    /// Updates the state to show a single full-screen overlay for the entire display.
    /// Used in FullScreen mode for non-focused displays.
    /// </summary>
    private void UpdateFullScreenOverlay(DisplayOverlayState state, DisplayInfo display, OverlayCalculationConfig config)
    {
        int index = (int)OverlayRegion.FullScreen;
        state.Overlays[index] = new OverlayDefinition
        {
            Region = OverlayRegion.FullScreen,
            Bounds = display.Bounds,
            Color = config.InactiveColor,
            Opacity = config.InactiveOpacity,
            IsVisible = true
        };
    }

    /// <summary>
    /// Updates the state with 4-sided overlays around the focused window (Partial mode).
    /// Top, Bottom, Left, Right overlays with inactive color.
    /// Updates existing overlay definitions in place.
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
            state.Overlays[index] = new OverlayDefinition
            {
                Region = OverlayRegion.Top,
                Bounds = new Rectangle(
                    display.Bounds.Left,
                    display.Bounds.Top,
                    display.Bounds.Width,
                    clampedWindow.Top - display.Bounds.Top
                ),
                Color = config.InactiveColor,
                Opacity = config.InactiveOpacity,
                IsVisible = true
            };
        }

        // Bottom overlay: full width, from window bottom to display bottom
        if (clampedWindow.Bottom < display.Bounds.Bottom)
        {
            int index = (int)OverlayRegion.Bottom;
            state.Overlays[index] = new OverlayDefinition
            {
                Region = OverlayRegion.Bottom,
                Bounds = new Rectangle(
                    display.Bounds.Left,
                    clampedWindow.Bottom,
                    display.Bounds.Width,
                    display.Bounds.Bottom - clampedWindow.Bottom
                ),
                Color = config.InactiveColor,
                Opacity = config.InactiveOpacity,
                IsVisible = true
            };
        }

        // Left overlay: from window top to window bottom, from display left to window left
        if (clampedWindow.Left > display.Bounds.Left)
        {
            int index = (int)OverlayRegion.Left;
            state.Overlays[index] = new OverlayDefinition
            {
                Region = OverlayRegion.Left,
                Bounds = new Rectangle(
                    display.Bounds.Left,
                    clampedWindow.Top,
                    clampedWindow.Left - display.Bounds.Left,
                    clampedWindow.Height
                ),
                Color = config.InactiveColor,
                Opacity = config.InactiveOpacity,
                IsVisible = true
            };
        }

        // Right overlay: from window top to window bottom, from window right to display right
        if (clampedWindow.Right < display.Bounds.Right)
        {
            int index = (int)OverlayRegion.Right;
            state.Overlays[index] = new OverlayDefinition
            {
                Region = OverlayRegion.Right,
                Bounds = new Rectangle(
                    clampedWindow.Right,
                    clampedWindow.Top,
                    display.Bounds.Right - clampedWindow.Right,
                    clampedWindow.Height
                ),
                Color = config.InactiveColor,
                Opacity = config.InactiveOpacity,
                IsVisible = true
            };
        }
    }

    /// <summary>
    /// Updates the state with 4-sided overlays plus center highlight (PartialWithActive mode).
    /// Top, Bottom, Left, Right overlays with inactive color.
    /// Center overlay with active color to highlight the focused window.
    /// Updates existing overlay definitions in place.
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
        state.Overlays[index] = new OverlayDefinition
        {
            Region = OverlayRegion.Center,
            Bounds = clampedWindow,
            Color = config.ActiveColor,
            Opacity = config.ActiveOpacity,
            IsVisible = true
        };
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
