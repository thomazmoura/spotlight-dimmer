using SpotlightDimmer.Core.ExternalCoordinates;

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
    NoChange,

    /// <summary>
    /// External coordinates were applied (from file-based provider).
    /// </summary>
    ExternalCoordinatesApplied
}

/// <summary>
/// Handles focus change logic and decides when to update overlays.
/// This class contains the platform-agnostic decision logic for tracking window focus changes.
/// </summary>
public class FocusChangeHandler
{
    private readonly IOverlayUpdateService _overlayUpdateService;
    private ExternalCoordinatesService? _externalCoordinatesService;
    private int _lastFocusedDisplayIndex = -1;
    private Rectangle? _lastWindowRect;        // Effective bounds (may be external)
    private Rectangle? _lastOriginalWindowRect; // Original window bounds (for ForceUpdate)
    private string? _lastWindowTitle;
    private bool _lastWasExternalOverride;

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
    /// Gets whether the last focus change used external coordinates.
    /// </summary>
    public bool LastWasExternalOverride => _lastWasExternalOverride;

    /// <summary>
    /// Gets the last effective bounds used (may be external coordinates or window bounds).
    /// </summary>
    public Rectangle? LastEffectiveBounds => _lastWindowRect;

    /// <summary>
    /// Gets the last window title used for external coordinate matching.
    /// </summary>
    public string? LastWindowTitle => _lastWindowTitle;

    /// <summary>
    /// Gets the last original window bounds (before external coordinate override).
    /// </summary>
    public Rectangle? LastOriginalWindowBounds => _lastOriginalWindowRect;

    /// <summary>
    /// Creates a new FocusChangeHandler with the specified overlay update service.
    /// </summary>
    /// <param name="overlayUpdateService">The service to call when overlays need updating.</param>
    public FocusChangeHandler(IOverlayUpdateService overlayUpdateService)
    {
        _overlayUpdateService = overlayUpdateService ?? throw new ArgumentNullException(nameof(overlayUpdateService));
    }

    /// <summary>
    /// Sets the external coordinates service for overriding window bounds.
    /// </summary>
    /// <param name="service">The external coordinates service, or null to disable.</param>
    public void SetExternalCoordinatesService(ExternalCoordinatesService? service)
    {
        _externalCoordinatesService = service;
    }

    /// <summary>
    /// Processes a focus change event and determines if overlay updates are needed.
    /// </summary>
    /// <param name="displayIndex">The index of the display containing the focused window.</param>
    /// <param name="windowBounds">The bounds of the focused window. Null if no valid bounds.</param>
    /// <returns>A FocusChangeResult indicating what action was taken.</returns>
    public FocusChangeResult ProcessFocusChange(int displayIndex, Rectangle? windowBounds)
    {
        return ProcessFocusChange(displayIndex, windowBounds, null);
    }

    /// <summary>
    /// Processes a focus change event with window title for external coordinate matching.
    /// </summary>
    /// <param name="displayIndex">The index of the display containing the focused window.</param>
    /// <param name="windowBounds">The bounds of the focused window. Null if no valid bounds.</param>
    /// <param name="windowTitle">The title of the focused window for external coordinate matching.</param>
    /// <returns>A FocusChangeResult indicating what action was taken.</returns>
    public FocusChangeResult ProcessFocusChange(int displayIndex, Rectangle? windowBounds, string? windowTitle)
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
                _lastWindowTitle = windowTitle;
            }

            return FocusChangeResult.Ignored;
        }

        // Ignore if we don't have valid window bounds
        if (!windowBounds.HasValue)
        {
            return FocusChangeResult.Ignored;
        }

        // Try to get external bounds if service is available and window title matches
        Rectangle effectiveBounds = windowBounds.Value;
        bool isExternalOverride = false;

        if (_externalCoordinatesService != null && !string.IsNullOrEmpty(windowTitle))
        {
            if (_externalCoordinatesService.TryGetExternalBounds(windowTitle, windowBounds.Value, out var externalBounds))
            {
                effectiveBounds = externalBounds;
                isExternalOverride = true;
            }
        }

        bool displayChanged = displayIndex != _lastFocusedDisplayIndex;
        bool rectChanged = _lastWindowRect != effectiveBounds;
        bool externalStateChanged = _lastWasExternalOverride != isExternalOverride;

        // Handle display change
        if (displayChanged || externalStateChanged)
        {
            _lastFocusedDisplayIndex = displayIndex;
            _lastWindowRect = effectiveBounds;
            _lastOriginalWindowRect = windowBounds.Value; // Store original for ForceUpdate
            _lastWindowTitle = windowTitle;
            _lastWasExternalOverride = isExternalOverride;

            // Update overlays for the new display
            _overlayUpdateService.UpdateOverlays(displayIndex, effectiveBounds);

            if (isExternalOverride)
                return FocusChangeResult.ExternalCoordinatesApplied;

            return FocusChangeResult.DisplayChanged;
        }

        // Handle position/size change (same display)
        if (rectChanged)
        {
            _lastWindowRect = effectiveBounds;
            _lastOriginalWindowRect = windowBounds.Value; // Store original for ForceUpdate
            _lastWindowTitle = windowTitle;
            _lastWasExternalOverride = isExternalOverride;

            // Update overlays for the position change
            _overlayUpdateService.UpdateOverlays(displayIndex, effectiveBounds);

            if (isExternalOverride)
                return FocusChangeResult.ExternalCoordinatesApplied;

            return FocusChangeResult.PositionChanged;
        }

        return FocusChangeResult.NoChange;
    }

    /// <summary>
    /// Forces an update using current state. Useful when external coordinates change.
    /// </summary>
    public void ForceUpdate()
    {
        if (HasFocus && _lastOriginalWindowRect.HasValue)
        {
            // Re-process with potentially new external coordinates
            // Use original window bounds to allow external service to recalculate
            var savedOriginalRect = _lastOriginalWindowRect;
            var savedTitle = _lastWindowTitle;
            _lastWindowRect = null; // Force change detection

            ProcessFocusChange(_lastFocusedDisplayIndex, savedOriginalRect, savedTitle);
        }
    }

    /// <summary>
    /// Resets the focus state (useful for testing or when display configuration changes).
    /// </summary>
    public void ResetState()
    {
        _lastFocusedDisplayIndex = -1;
        _lastWindowRect = null;
        _lastOriginalWindowRect = null;
        _lastWindowTitle = null;
        _lastWasExternalOverride = false;
    }
}
