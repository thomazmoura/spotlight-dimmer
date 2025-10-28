using SpotlightDimmer.Core;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Tests for AppState overlay calculation logic with different dimming modes and focus scenarios.
/// </summary>
public class AppStateTests
{
    /// <summary>
    /// Tests that when in FullScreen mode and focus remains on the same display,
    /// the overlay state remains unchanged (focused display has no overlays, others have fullscreen overlay).
    /// </summary>
    [Theory]
    [InlineData(0, 0, 1920, 1080, 1920, 0, 1920, 1080, 0)] // Dual monitors side-by-side, focus on first
    [InlineData(0, 0, 1920, 1080, 1920, 0, 1920, 1080, 1)] // Dual monitors side-by-side, focus on second
    [InlineData(0, 0, 3840, 2160, 0, 2160, 3840, 2160, 0)] // Dual monitors stacked, focus on first
    public void FullScreenMode_WhenTheFocusIsOnTheSameDisplay_KeepsTheFocusUnchanged(
        int display1X, int display1Y, int display1Width, int display1Height,
        int display2X, int display2Y, int display2Width, int display2Height,
        int focusedDisplayIndex)
    {
        // Arrange
        var displays = new[]
        {
            new DisplayInfo(0, new Rectangle(display1X, display1Y, display1Width, display1Height)),
            new DisplayInfo(1, new Rectangle(display2X, display2Y, display2Width, display2Height))
        };

        var appState = new AppState(displays);
        var config = new OverlayCalculationConfig(
            Mode: DimmingMode.FullScreen,
            InactiveColor: Color.Black,
            InactiveOpacity: 153,
            ActiveColor: Color.Black,
            ActiveOpacity: 102
        );

        // Position focused window in the center of the focused display
        var focusedDisplay = displays[focusedDisplayIndex];
        var focusedWindowBounds = new Rectangle(
            focusedDisplay.Bounds.X + 100,
            focusedDisplay.Bounds.Y + 100,
            800,
            600
        );

        // Calculate initial state
        appState.Calculate(displays, focusedWindowBounds, focusedDisplayIndex, config);

        // Capture the initial state
        var initialStates = CaptureOverlayStates(appState);

        // Act - Calculate again with the same focus (simulating focus staying on the same display)
        appState.Calculate(displays, focusedWindowBounds, focusedDisplayIndex, config);

        // Assert
        var currentStates = CaptureOverlayStates(appState);

        // Verify the states match exactly
        AssertStatesEqual(initialStates, currentStates);

        // Additionally verify the expected behavior:
        // - Focused display should have no visible overlays
        // - Non-focused display should have exactly one fullscreen overlay
        for (int i = 0; i < displays.Length; i++)
        {
            var displayState = appState.DisplayStates[i];

            if (i == focusedDisplayIndex)
            {
                // Focused display: no overlays should be visible
                Assert.Equal(0, displayState.VisibleOverlayCount);
            }
            else
            {
                // Non-focused display: should have exactly 1 fullscreen overlay
                Assert.Equal(1, displayState.VisibleOverlayCount);

                var fullscreenOverlay = displayState.Overlays[(int)OverlayRegion.FullScreen];
                Assert.True(fullscreenOverlay.IsVisible);
                Assert.Equal(displays[i].Bounds, fullscreenOverlay.Bounds);
                Assert.Equal(config.InactiveColor, fullscreenOverlay.Color);
                Assert.Equal(config.InactiveOpacity, fullscreenOverlay.Opacity);
            }
        }
    }

    /// <summary>
    /// Tests that when in FullScreen mode and focus changes to a different display,
    /// the overlay state updates correctly (previously focused display gains overlay,
    /// newly focused display loses overlay).
    /// </summary>
    [Theory]
    [InlineData(0, 0, 1920, 1080, 1920, 0, 1920, 1080, 0, 1)] // Switch from first to second
    [InlineData(0, 0, 1920, 1080, 1920, 0, 1920, 1080, 1, 0)] // Switch from second to first
    [InlineData(0, 0, 3840, 2160, 0, 2160, 3840, 2160, 0, 1)] // Stacked: switch from top to bottom
    public void FullScreenMode_WhenTheFocusChangesToADifferentDisplay_UpdatesTheFocusToTheOtherDisplay(
        int display1X, int display1Y, int display1Width, int display1Height,
        int display2X, int display2Y, int display2Width, int display2Height,
        int initialFocusedDisplayIndex,
        int newFocusedDisplayIndex)
    {
        // Arrange
        var displays = new[]
        {
            new DisplayInfo(0, new Rectangle(display1X, display1Y, display1Width, display1Height)),
            new DisplayInfo(1, new Rectangle(display2X, display2Y, display2Width, display2Height))
        };

        var appState = new AppState(displays);
        var config = new OverlayCalculationConfig(
            Mode: DimmingMode.FullScreen,
            InactiveColor: Color.Black,
            InactiveOpacity: 153,
            ActiveColor: Color.Black,
            ActiveOpacity: 102
        );

        // Set up initial state: focus on initialFocusedDisplayIndex
        var initialDisplay = displays[initialFocusedDisplayIndex];
        var initialWindowBounds = new Rectangle(
            initialDisplay.Bounds.X + 100,
            initialDisplay.Bounds.Y + 100,
            800,
            600
        );

        appState.Calculate(displays, initialWindowBounds, initialFocusedDisplayIndex, config);

        // Verify initial state before the switch
        var initiallyFocusedDisplay = appState.DisplayStates[initialFocusedDisplayIndex];
        var initiallyUnfocusedDisplay = appState.DisplayStates[newFocusedDisplayIndex];

        Assert.Equal(0, initiallyFocusedDisplay.VisibleOverlayCount); // No overlay on focused display
        Assert.Equal(1, initiallyUnfocusedDisplay.VisibleOverlayCount); // Fullscreen overlay on unfocused
        Assert.True(initiallyUnfocusedDisplay.Overlays[(int)OverlayRegion.FullScreen].IsVisible);

        // Act - Switch focus to the other display
        var newDisplay = displays[newFocusedDisplayIndex];
        var newWindowBounds = new Rectangle(
            newDisplay.Bounds.X + 200,
            newDisplay.Bounds.Y + 200,
            1000,
            700
        );

        appState.Calculate(displays, newWindowBounds, newFocusedDisplayIndex, config);

        // Assert
        var previouslyFocusedDisplay = appState.DisplayStates[initialFocusedDisplayIndex];
        var newlyFocusedDisplay = appState.DisplayStates[newFocusedDisplayIndex];

        // Previously focused display should now have a fullscreen overlay
        Assert.Equal(1, previouslyFocusedDisplay.VisibleOverlayCount);
        var previousOverlay = previouslyFocusedDisplay.Overlays[(int)OverlayRegion.FullScreen];
        Assert.True(previousOverlay.IsVisible);
        Assert.Equal(displays[initialFocusedDisplayIndex].Bounds, previousOverlay.Bounds);
        Assert.Equal(config.InactiveColor, previousOverlay.Color);
        Assert.Equal(config.InactiveOpacity, previousOverlay.Opacity);

        // Newly focused display should have no overlays
        Assert.Equal(0, newlyFocusedDisplay.VisibleOverlayCount);
        Assert.False(newlyFocusedDisplay.Overlays[(int)OverlayRegion.FullScreen].IsVisible);

        // Verify all other overlay regions are hidden on the newly focused display
        for (int i = 0; i < newlyFocusedDisplay.Overlays.Length; i++)
        {
            Assert.False(newlyFocusedDisplay.Overlays[i].IsVisible);
        }
    }

    /// <summary>
    /// Tests that when the focused window has 0x0 dimensions (invalid state),
    /// the overlay state remains unchanged from the previous valid state.
    /// This prevents flickering when Windows temporarily reports invalid window bounds.
    /// </summary>
    [Theory]
    [InlineData(DimmingMode.FullScreen)]
    [InlineData(DimmingMode.Partial)]
    [InlineData(DimmingMode.PartialWithActive)]
    public void Calculate_WhenFocusedWindowHasZeroDimensions_KeepsStateUnchanged(DimmingMode mode)
    {
        // Arrange
        var displays = new[]
        {
            new DisplayInfo(0, new Rectangle(0, 0, 1920, 1080)),
            new DisplayInfo(1, new Rectangle(1920, 0, 1920, 1080))
        };

        var appState = new AppState(displays);
        var config = new OverlayCalculationConfig(
            Mode: mode,
            InactiveColor: Color.Black,
            InactiveOpacity: 153,
            ActiveColor: Color.Black,
            ActiveOpacity: 102
        );

        // Set up initial valid state with a normal window
        var initialValidBounds = new Rectangle(100, 100, 800, 600);
        appState.Calculate(displays, initialValidBounds, 0, config);

        // Capture the valid state
        var validState = CaptureOverlayStates(appState);

        // Act - Call Calculate with 0x0 focused window bounds (invalid state)
        var zeroWidthBounds = new Rectangle(100, 100, 0, 600); // Width = 0
        appState.Calculate(displays, zeroWidthBounds, 0, config);
        var afterZeroWidthState = CaptureOverlayStates(appState);

        var zeroHeightBounds = new Rectangle(100, 100, 800, 0); // Height = 0
        appState.Calculate(displays, zeroHeightBounds, 0, config);
        var afterZeroHeightState = CaptureOverlayStates(appState);

        var zeroBothBounds = new Rectangle(100, 100, 0, 0); // Both 0
        appState.Calculate(displays, zeroBothBounds, 0, config);
        var afterZeroBothState = CaptureOverlayStates(appState);

        // Assert - All states should remain unchanged from the initial valid state
        AssertStatesEqual(validState, afterZeroWidthState);
        AssertStatesEqual(validState, afterZeroHeightState);
        AssertStatesEqual(validState, afterZeroBothState);
    }

    #region Helper Methods

    /// <summary>
    /// Captures a snapshot of all overlay states for comparison.
    /// </summary>
    private static List<CapturedOverlayState> CaptureOverlayStates(AppState appState)
    {
        var states = new List<CapturedOverlayState>();

        foreach (var displayState in appState.DisplayStates)
        {
            foreach (var overlay in displayState.Overlays)
            {
                states.Add(new CapturedOverlayState(
                    displayState.DisplayIndex,
                    overlay.Region,
                    overlay.Bounds,
                    overlay.Color,
                    overlay.Opacity,
                    overlay.IsVisible
                ));
            }
        }

        return states;
    }

    /// <summary>
    /// Asserts that two overlay state snapshots are equal.
    /// </summary>
    private static void AssertStatesEqual(
        List<CapturedOverlayState> expected,
        List<CapturedOverlayState> actual)
    {
        Assert.Equal(expected.Count, actual.Count);

        for (int i = 0; i < expected.Count; i++)
        {
            var exp = expected[i];
            var act = actual[i];

            Assert.Equal(exp.DisplayIndex, act.DisplayIndex);
            Assert.Equal(exp.Region, act.Region);
            Assert.Equal(exp.Bounds, act.Bounds);
            Assert.Equal(exp.Color, act.Color);
            Assert.Equal(exp.Opacity, act.Opacity);
            Assert.Equal(exp.IsVisible, act.IsVisible);
        }
    }

    /// <summary>
    /// Represents a captured snapshot of an overlay state for testing.
    /// </summary>
    private record CapturedOverlayState(
        int DisplayIndex,
        OverlayRegion Region,
        Rectangle Bounds,
        Color Color,
        byte Opacity,
        bool IsVisible
    );

    #endregion
}
