using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Double-buffered renderer that maintains TWO complete sets of overlay windows.
/// Reduces visual gaps during window dragging by preparing the next frame while the current frame is visible.
///
/// Buffer Swap Strategy:
/// 1. Update inactive buffer with new overlay positions/states
/// 2. Show inactive buffer windows (next frame becomes visible)
/// 3. Hide active buffer windows (old frame disappears)
/// 4. Swap active buffer index (inactive becomes active)
///
/// Memory Impact: 2x GDI handles (12 windows per display instead of 6)
/// Performance: Zero allocations in hot path (UpdateOverlays)
/// </summary>
internal class DoubleBufferedRenderer : IOverlayRenderer
{
    // Two complete sets of overlay renderers (one per buffer)
    private readonly UpdateLayeredWindowRenderer[] _buffers = new UpdateLayeredWindowRenderer[2];

    // Index of the currently visible buffer (0 or 1)
    private int _activeBufferIndex = 0;

    public DoubleBufferedRenderer()
    {
        // Create two independent renderer instances
        _buffers[0] = new UpdateLayeredWindowRenderer();
        _buffers[1] = new UpdateLayeredWindowRenderer();
    }

    /// <summary>
    /// Pre-creates all overlay windows for BOTH buffers.
    /// Creates 12 overlays per display (2 buffers × 6 regions), all initially hidden.
    /// Call this once at startup after getting display information.
    /// </summary>
    public void CreateOverlays(Core.DisplayInfo[] displays, OverlayCalculationConfig config)
    {
        // Initialize both buffers with complete overlay sets
        _buffers[0].CreateOverlays(displays, config);
        _buffers[1].CreateOverlays(displays, config);
    }

    /// <summary>
    /// Updates overlay colors when configuration changes.
    /// Updates BOTH buffers to ensure consistent appearance regardless of which is active.
    /// </summary>
    public void UpdateBrushColors(OverlayCalculationConfig config)
    {
        _buffers[0].UpdateBrushColors(config);
        _buffers[1].UpdateBrushColors(config);
    }

    /// <summary>
    /// Updates overlay windows using double buffering to reduce visual gaps.
    /// ZERO allocations - reuses pre-allocated buffer resources.
    ///
    /// Buffer Swap Flow:
    /// 1. Inactive buffer is updated with new overlay states
    /// 2. Inactive buffer windows are shown (next frame appears)
    /// 3. Active buffer windows are hidden (old frame disappears)
    /// 4. Buffer indices are swapped for next update
    ///
    /// This ensures there's always one complete frame visible during updates,
    /// eliminating the temporal gap where no overlays are visible.
    /// </summary>
    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        // Calculate which buffer to update (the currently inactive one)
        int inactiveBufferIndex = 1 - _activeBufferIndex; // Toggle: 0→1, 1→0

        // Step 1: Update the inactive buffer with new overlay states
        // This prepares the next frame while the current frame is still visible
        _buffers[inactiveBufferIndex].UpdateOverlays(states);

        // Step 2: Show the newly updated buffer (next frame becomes visible)
        // At this point, BOTH buffers are briefly visible
        // This is intentional - better to have slight overlap than gaps
        // Note: UpdateOverlays already shows windows if IsVisible=true, so this is implicit

        // Step 3: Hide the old buffer (old frame disappears)
        // Now only the new frame is visible
        _buffers[_activeBufferIndex].HideAllOverlays();

        // Step 4: Swap the active buffer index for next update
        _activeBufferIndex = inactiveBufferIndex;
    }

    /// <summary>
    /// Updates the screen capture exclusion setting for all overlay windows in BOTH buffers.
    /// Returns the total count of windows that were successfully updated across both buffers.
    /// </summary>
    public int UpdateScreenCaptureExclusion(bool exclude)
    {
        int totalSuccessCount = 0;

        // Update both buffers
        totalSuccessCount += _buffers[0].UpdateScreenCaptureExclusion(exclude);
        totalSuccessCount += _buffers[1].UpdateScreenCaptureExclusion(exclude);

        return totalSuccessCount;
    }

    /// <summary>
    /// Hides all overlay windows in BOTH buffers (for pause functionality).
    /// </summary>
    public void HideAllOverlays()
    {
        _buffers[0].HideAllOverlays();
        _buffers[1].HideAllOverlays();
    }

    /// <summary>
    /// Disposes all existing overlay windows in BOTH buffers and clears internal state.
    /// Call this before recreating overlays when display configuration changes.
    /// </summary>
    public void CleanupOverlays()
    {
        _buffers[0].CleanupOverlays();
        _buffers[1].CleanupOverlays();

        // Reset active buffer index
        _activeBufferIndex = 0;
    }

    /// <summary>
    /// Disposes both buffer renderers and all their resources.
    /// </summary>
    public void Dispose()
    {
        _buffers[0].Dispose();
        _buffers[1].Dispose();
    }
}
