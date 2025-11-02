namespace SpotlightDimmer.Core;

/// <summary>
/// Represents the result of processing a focus change.
/// </summary>
public enum FocusChangeResult
{
    /// <summary>
    /// The focus change was ignored (e.g., zero-dimension window).
    /// </summary>
    Ignored,

    /// <summary>
    /// The display containing the focused window changed.
    /// </summary>
    DisplayChanged,

    /// <summary>
    /// The window position or size changed (but not the display).
    /// </summary>
    PositionChanged,

    /// <summary>
    /// No significant change occurred.
    /// </summary>
    NoChange
}

/// <summary>
/// Handles focus change logic and decides when to update overlays.
/// This class contains the platform-agnostic decision logic for tracking window focus changes.
/// </summary>
public class FocusChangeHandler
{
    private readonly IOverlayUpdateService _overlayUpdateService;
    private int _lastFocusedDisplayIndex = -1;
    private Rectangle? _lastWindowRect;

    /// <summary>
    /// Gets the current focused display index.
    /// </summary>
    public int CurrentFocusedDisplayIndex => _lastFocusedDisplayIndex;

    /// <summary>
    /// Gets the current window rectangle.
    /// </summary>
    public Rectangle? CurrentWindowRect => _lastWindowRect;

    /// <summary>
    /// Gets whether there is a focused window being tracked.
    /// </summary>
    public bool HasFocus => _lastFocusedDisplayIndex >= 0 && _lastWindowRect.HasValue;

    /// <summary>
    /// Creates a new FocusChangeHandler with the specified overlay update service.
    /// </summary>
    /// <param name="overlayUpdateService">The service to call when overlays need updating.</param>
    public FocusChangeHandler(IOverlayUpdateService overlayUpdateService)
    {
        _overlayUpdateService = overlayUpdateService ?? throw new ArgumentNullException(nameof(overlayUpdateService));
    }

    /// <summary>
    /// Processes a focus change event and determines if overlay updates are needed.
    /// </summary>
    /// <param name="displayIndex">The index of the display containing the focused window.</param>
    /// <param name="windowBounds">The bounds of the focused window. Null if no valid bounds.</param>
    /// <returns>A FocusChangeResult indicating what action was taken.</returns>
    public FocusChangeResult ProcessFocusChange(int displayIndex, Rectangle? windowBounds)
    {
        // Handle windows with zero dimensions (e.g., popups during initialization, minimized windows)
        // Track display changes but don't update overlays until we get valid dimensions
        if (windowBounds.HasValue && (windowBounds.Value.Width == 0 || windowBounds.Value.Height == 0))
        {
            // Check if display changed - track it but wait for valid dimensions before updating overlays
            if (displayIndex != _lastFocusedDisplayIndex)
            {
                _lastFocusedDisplayIndex = displayIndex;
                _lastWindowRect = null; // Clear last rect to ensure next valid bounds trigger an update
            }

            return FocusChangeResult.Ignored;
        }

        // Ignore if we don't have valid window bounds
        if (!windowBounds.HasValue)
        {
            return FocusChangeResult.Ignored;
        }

        bool displayChanged = displayIndex != _lastFocusedDisplayIndex;
        bool rectChanged = _lastWindowRect != windowBounds;

        // Handle display change
        if (displayChanged)
        {
            _lastFocusedDisplayIndex = displayIndex;
            _lastWindowRect = windowBounds;

            // Update overlays for the new display
            _overlayUpdateService.UpdateOverlays(displayIndex, windowBounds.Value);

            return FocusChangeResult.DisplayChanged;
        }

        // Handle position/size change (same display)
        if (rectChanged)
        {
            _lastWindowRect = windowBounds;

            // Update overlays for the position change
            _overlayUpdateService.UpdateOverlays(displayIndex, windowBounds.Value);

            return FocusChangeResult.PositionChanged;
        }

        return FocusChangeResult.NoChange;
    }

    /// <summary>
    /// Resets the focus state (useful for testing or when display configuration changes).
    /// </summary>
    public void ResetState()
    {
        _lastFocusedDisplayIndex = -1;
        _lastWindowRect = null;
    }
}
