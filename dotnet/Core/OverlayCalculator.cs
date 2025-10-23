namespace SpotlightDimmer.Core;

/// <summary>
/// Calculates overlay positions and states based on display configuration and focused window.
/// Uses instance-based design with collection reuse to minimize GC pressure.
/// </summary>
public class OverlayCalculator
{
    // Reusable collections to avoid allocations on every calculation
    private readonly List<DisplayOverlayState> _resultCache = new();
    private readonly List<OverlayDefinition> _overlayCache = new();

    /// <summary>
    /// Calculates the overlay states for all displays based on current focus and configuration.
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
        _resultCache.Clear();

        for (int i = 0; i < displays.Length; i++)
        {
            var display = displays[i];
            bool isFocusedDisplay = (i == focusedDisplayIndex);

            OverlayDefinition[] overlays;

            if (isFocusedDisplay && focusedWindowBounds.HasValue)
            {
                // This display has the focused window
                overlays = config.Mode switch
                {
                    DimmingMode.FullScreen => CreateNoOverlays(),
                    DimmingMode.Partial => CreatePartialOverlays(display, focusedWindowBounds.Value, config),
                    DimmingMode.PartialWithActive => CreatePartialWithActiveOverlays(display, focusedWindowBounds.Value, config),
                    _ => CreateNoOverlays()
                };
            }
            else
            {
                // This display does not have the focused window - dim the entire display
                overlays = CreateFullScreenOverlay(display, config);
            }

            _resultCache.Add(new DisplayOverlayState(display.Index, display.Bounds, overlays));
        }

        return _resultCache.ToArray();
    }

    /// <summary>
    /// Creates an empty array (no overlays visible).
    /// </summary>
    private OverlayDefinition[] CreateNoOverlays()
    {
        return Array.Empty<OverlayDefinition>();
    }

    /// <summary>
    /// Creates a single full-screen overlay for the entire display.
    /// Used in FullScreen mode for non-focused displays.
    /// </summary>
    private OverlayDefinition[] CreateFullScreenOverlay(DisplayInfo display, OverlayCalculationConfig config)
    {
        return new[]
        {
            new OverlayDefinition
            {
                Region = OverlayRegion.FullScreen,
                Bounds = display.Bounds,
                Color = config.InactiveColor,
                Opacity = config.InactiveOpacity,
                IsVisible = true
            }
        };
    }

    /// <summary>
    /// Creates 4-sided overlays around the focused window (Partial mode).
    /// Top, Bottom, Left, Right overlays with inactive color.
    /// </summary>
    private OverlayDefinition[] CreatePartialOverlays(
        DisplayInfo display,
        Rectangle windowBounds,
        OverlayCalculationConfig config)
    {
        _overlayCache.Clear();

        // Clamp window bounds to display bounds (in case window extends beyond display)
        var clampedWindow = ClampToDisplay(windowBounds, display.Bounds);

        // Top overlay: full width, from display top to window top
        if (clampedWindow.Top > display.Bounds.Top)
        {
            _overlayCache.Add(new OverlayDefinition
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
            });
        }

        // Bottom overlay: full width, from window bottom to display bottom
        if (clampedWindow.Bottom < display.Bounds.Bottom)
        {
            _overlayCache.Add(new OverlayDefinition
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
            });
        }

        // Left overlay: from window top to window bottom, from display left to window left
        if (clampedWindow.Left > display.Bounds.Left)
        {
            _overlayCache.Add(new OverlayDefinition
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
            });
        }

        // Right overlay: from window top to window bottom, from window right to display right
        if (clampedWindow.Right < display.Bounds.Right)
        {
            _overlayCache.Add(new OverlayDefinition
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
            });
        }

        return _overlayCache.ToArray();
    }

    /// <summary>
    /// Creates 4-sided overlays plus center highlight (PartialWithActive mode).
    /// Top, Bottom, Left, Right overlays with inactive color.
    /// Center overlay with active color to highlight the focused window.
    /// </summary>
    private OverlayDefinition[] CreatePartialWithActiveOverlays(
        DisplayInfo display,
        Rectangle windowBounds,
        OverlayCalculationConfig config)
    {
        // Start with the 4-sided overlays
        var partialOverlays = CreatePartialOverlays(display, windowBounds, config);
        _overlayCache.Clear();
        _overlayCache.AddRange(partialOverlays);

        // Add center overlay with active color
        var clampedWindow = ClampToDisplay(windowBounds, display.Bounds);
        _overlayCache.Add(new OverlayDefinition
        {
            Region = OverlayRegion.Center,
            Bounds = clampedWindow,
            Color = config.ActiveColor,
            Opacity = config.ActiveOpacity,
            IsVisible = true
        });

        return _overlayCache.ToArray();
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
