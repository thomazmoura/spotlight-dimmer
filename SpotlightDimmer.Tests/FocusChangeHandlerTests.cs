using NSubstitute;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Tests for FocusChangeHandler to ensure proper focus change logic.
/// These tests verify zero-dimension filtering, display change detection, and overlay update calls.
/// </summary>
public class FocusChangeHandlerTests
{
    [Fact]
    public void ProcessFocusChange_WithZeroDimensionWindow_ReturnsIgnored()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var zeroWindow = new Rectangle(100, 100, 0, 0);

        // Act
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: zeroWindow);

        // Assert
        Assert.Equal(FocusChangeResult.Ignored, result);
        mockUpdateService.DidNotReceive().UpdateOverlays(Arg.Any<int>(), Arg.Any<Rectangle>());
    }

    [Fact]
    public void ProcessFocusChange_WithZeroWidth_ReturnsIgnored()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var zeroWidthWindow = new Rectangle(100, 100, 0, 500);

        // Act
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: zeroWidthWindow);

        // Assert
        Assert.Equal(FocusChangeResult.Ignored, result);
        mockUpdateService.DidNotReceive().UpdateOverlays(Arg.Any<int>(), Arg.Any<Rectangle>());
    }

    [Fact]
    public void ProcessFocusChange_WithZeroHeight_ReturnsIgnored()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var zeroHeightWindow = new Rectangle(100, 100, 500, 0);

        // Act
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: zeroHeightWindow);

        // Assert
        Assert.Equal(FocusChangeResult.Ignored, result);
        mockUpdateService.DidNotReceive().UpdateOverlays(Arg.Any<int>(), Arg.Any<Rectangle>());
    }

    [Fact]
    public void ProcessFocusChange_WithNullBounds_ReturnsIgnored()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);

        // Act
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: null);

        // Assert
        Assert.Equal(FocusChangeResult.Ignored, result);
        mockUpdateService.DidNotReceive().UpdateOverlays(Arg.Any<int>(), Arg.Any<Rectangle>());
    }

    [Fact]
    public void ProcessFocusChange_FirstValidWindow_ReturnsDisplayChanged()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window = new Rectangle(100, 100, 800, 600);

        // Act
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);

        // Assert
        Assert.Equal(FocusChangeResult.DisplayChanged, result);
        mockUpdateService.Received(1).UpdateOverlays(0, window);
    }

    [Fact]
    public void ProcessFocusChange_SameDisplayDifferentBounds_ReturnsPositionChanged()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window1 = new Rectangle(100, 100, 800, 600);
        var window2 = new Rectangle(150, 150, 800, 600); // Same display, different position

        // Act
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window1);
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: window2);

        // Assert
        Assert.Equal(FocusChangeResult.PositionChanged, result);
        mockUpdateService.Received(1).UpdateOverlays(0, window1);
        mockUpdateService.Received(1).UpdateOverlays(0, window2);
    }

    [Fact]
    public void ProcessFocusChange_DifferentDisplay_ReturnsDisplayChanged()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window1 = new Rectangle(100, 100, 800, 600);
        var window2 = new Rectangle(1920, 100, 800, 600); // Different display

        // Act
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window1);
        var result = handler.ProcessFocusChange(displayIndex: 1, windowBounds: window2);

        // Assert
        Assert.Equal(FocusChangeResult.DisplayChanged, result);
        mockUpdateService.Received(1).UpdateOverlays(0, window1);
        mockUpdateService.Received(1).UpdateOverlays(1, window2);
    }

    [Fact]
    public void ProcessFocusChange_SameWindowTwice_ReturnsNoChange()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window = new Rectangle(100, 100, 800, 600);

        // Act
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);

        // Assert
        Assert.Equal(FocusChangeResult.NoChange, result);
        // UpdateOverlays should only be called once (for the first change)
        mockUpdateService.Received(1).UpdateOverlays(0, window);
    }

    [Fact]
    public void ProcessFocusChange_AfterZeroDimensionWindow_WorksCorrectly()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var zeroWindow = new Rectangle(100, 100, 0, 0);
        var validWindow = new Rectangle(100, 100, 800, 600);

        // Act
        var result1 = handler.ProcessFocusChange(displayIndex: 0, windowBounds: zeroWindow);
        var result2 = handler.ProcessFocusChange(displayIndex: 0, windowBounds: validWindow);

        // Assert
        Assert.Equal(FocusChangeResult.Ignored, result1);
        // After tracking display 0 from zero-dimension window, valid window on same display is PositionChanged
        Assert.Equal(FocusChangeResult.PositionChanged, result2);
        mockUpdateService.Received(1).UpdateOverlays(0, validWindow);
    }

    [Fact]
    public void ProcessFocusChange_ResizeWindow_ReturnsPositionChanged()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window1 = new Rectangle(100, 100, 800, 600);
        var window2 = new Rectangle(100, 100, 1024, 768); // Same position, different size

        // Act
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window1);
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: window2);

        // Assert
        Assert.Equal(FocusChangeResult.PositionChanged, result);
        mockUpdateService.Received(1).UpdateOverlays(0, window1);
        mockUpdateService.Received(1).UpdateOverlays(0, window2);
    }

    [Fact]
    public void HasFocus_InitiallyFalse()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);

        // Assert
        Assert.False(handler.HasFocus);
        Assert.Equal(-1, handler.CurrentFocusedDisplayIndex);
        Assert.Null(handler.CurrentWindowRect);
    }

    [Fact]
    public void HasFocus_TrueAfterValidFocusChange()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window = new Rectangle(100, 100, 800, 600);

        // Act
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);

        // Assert
        Assert.True(handler.HasFocus);
        Assert.Equal(0, handler.CurrentFocusedDisplayIndex);
        Assert.Equal(window, handler.CurrentWindowRect);
    }

    [Fact]
    public void ResetState_ClearsTrackedState()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window = new Rectangle(100, 100, 800, 600);
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);

        // Act
        handler.ResetState();

        // Assert
        Assert.False(handler.HasFocus);
        Assert.Equal(-1, handler.CurrentFocusedDisplayIndex);
        Assert.Null(handler.CurrentWindowRect);
    }

    [Fact]
    public void ProcessFocusChange_AfterReset_TreatsAsNewDisplay()
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window = new Rectangle(100, 100, 800, 600);
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);
        handler.ResetState();

        // Act
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);

        // Assert
        Assert.Equal(FocusChangeResult.DisplayChanged, result);
        mockUpdateService.Received(2).UpdateOverlays(0, window); // Once before reset, once after
    }

    [Theory]
    [InlineData(0, 0, 1920, 1080)] // Standard HD window
    [InlineData(100, 200, 800, 600)] // Offset window
    [InlineData(-100, -50, 500, 400)] // Negative coordinates (multi-monitor)
    [InlineData(0, 0, 3840, 2160)] // 4K window
    public void ProcessFocusChange_WithVariousValidDimensions_CallsUpdateOverlays(int x, int y, int width, int height)
    {
        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window = new Rectangle(x, y, width, height);

        // Act
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: window);

        // Assert
        Assert.Equal(FocusChangeResult.DisplayChanged, result);
        mockUpdateService.Received(1).UpdateOverlays(0, window);
    }

    [Fact]
    public void Constructor_WithNullService_ThrowsArgumentNullException()
    {
        // Act & Assert
        Assert.Throws<ArgumentNullException>(() => new FocusChangeHandler(null!));
    }

    [Fact]
    public void ProcessFocusChange_ZeroDimensionOnDisplayChange_TracksDisplayAndUpdatesOnValidBounds()
    {
        // This test simulates popup window behavior (e.g., system tray context menu):
        // 1. Popup appears with focus on display 1 but reports 0x0 dimensions during initialization
        // 2. Shortly after, another event arrives with valid dimensions
        // Expected: Display change is tracked, and overlays update when valid dimensions arrive

        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var window1 = new Rectangle(0, 0, 800, 600);
        var zeroWindow = new Rectangle(1920, 100, 0, 500); // Zero width popup on display 1
        var validPopupWindow = new Rectangle(1920, 100, 200, 500); // Valid popup dimensions

        // Start on display 0 with a valid window
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: window1);

        // Act - Popup appears on display 1 with zero width (common during popup initialization)
        var result1 = handler.ProcessFocusChange(displayIndex: 1, windowBounds: zeroWindow);

        // Popup reports valid dimensions shortly after
        var result2 = handler.ProcessFocusChange(displayIndex: 1, windowBounds: validPopupWindow);

        // Assert
        Assert.Equal(FocusChangeResult.Ignored, result1); // Zero-dimension ignored for overlay update
        Assert.Equal(FocusChangeResult.PositionChanged, result2); // Valid dimensions trigger update
        Assert.Equal(1, handler.CurrentFocusedDisplayIndex); // Display change was tracked
        Assert.Equal(validPopupWindow, handler.CurrentWindowRect); // Valid bounds stored

        // Verify overlay update was called only once (for valid dimensions, not zero-dimension)
        mockUpdateService.Received(1).UpdateOverlays(0, window1); // Initial window
        mockUpdateService.DidNotReceive().UpdateOverlays(1, zeroWindow); // Zero-dimension skipped
        mockUpdateService.Received(1).UpdateOverlays(1, validPopupWindow); // Valid popup
    }

    [Fact]
    public void ProcessFocusChange_ZeroDimensionOnSameDisplay_DoesNotUpdateOverlays()
    {
        // Verify that zero-dimension windows on the same display don't cause overlay updates
        // (prevents flickering during window transitions)

        // Arrange
        var mockUpdateService = Substitute.For<IOverlayUpdateService>();
        var handler = new FocusChangeHandler(mockUpdateService);
        var validWindow = new Rectangle(100, 100, 800, 600);
        var zeroWindow = new Rectangle(100, 100, 0, 0);

        // Act
        handler.ProcessFocusChange(displayIndex: 0, windowBounds: validWindow);
        var result = handler.ProcessFocusChange(displayIndex: 0, windowBounds: zeroWindow);

        // Assert
        Assert.Equal(FocusChangeResult.Ignored, result);
        Assert.Equal(0, handler.CurrentFocusedDisplayIndex); // Display stays same
        // Rect is preserved on same display to prevent flickering - overlays stay at last valid position
        Assert.Equal(validWindow, handler.CurrentWindowRect);

        // Verify overlay was updated only once (for the valid window, not the zero-dimension)
        mockUpdateService.Received(1).UpdateOverlays(0, validWindow);
        mockUpdateService.DidNotReceive().UpdateOverlays(0, zeroWindow);
    }
}
