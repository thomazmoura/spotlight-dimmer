using SpotlightDimmer.Core;
using SpotlightDimmer.Core.ExternalCoordinates;
using Xunit;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Tests for the ExternalCoordinates feature.
/// </summary>
public class ExternalCoordinatesTests
{
    [Fact]
    public void ExternalPaneCoordinates_ToPixelBounds_CalculatesCorrectly()
    {
        // Arrange
        var coordinates = new ExternalPaneCoordinates
        {
            Pane = new CharacterCoordinates(Top: 1, Left: 0, Width: 80, Height: 24),
            Cell = new CellDimensions(Width: 10, Height: 20)
        };

        var windowBounds = new Rectangle(100, 100, 1000, 800);
        var padding = new TerminalPadding(Top: 32, Left: 8, Bottom: 0, Right: 8);

        // Act
        var result = coordinates.ToPixelBounds(windowBounds, padding);

        // Assert
        // Expected X: windowBounds.X + padding.Left + (pane.Left * cell.Width) = 100 + 8 + (0 * 10) = 108
        Assert.Equal(108, result.X);

        // Expected Y: windowBounds.Y + padding.Top + (pane.Top * cell.Height) = 100 + 32 + (1 * 20) = 152
        Assert.Equal(152, result.Y);

        // Expected Width: pane.Width * cell.Width = 80 * 10 = 800
        Assert.Equal(800, result.Width);

        // Expected Height: pane.Height * cell.Height = 24 * 20 = 480
        Assert.Equal(480, result.Height);
    }

    [Fact]
    public void ExternalPaneCoordinates_IsValid_ReturnsTrueForValidCoordinates()
    {
        // Arrange
        var coordinates = new ExternalPaneCoordinates
        {
            Pane = new CharacterCoordinates(0, 0, 80, 24),
            Cell = new CellDimensions(10, 20)
        };

        // Act & Assert
        Assert.True(coordinates.IsValid);
    }

    [Fact]
    public void ExternalPaneCoordinates_IsValid_ReturnsFalseForZeroCellWidth()
    {
        // Arrange
        var coordinates = new ExternalPaneCoordinates
        {
            Pane = new CharacterCoordinates(0, 0, 80, 24),
            Cell = new CellDimensions(0, 20)
        };

        // Act & Assert
        Assert.False(coordinates.IsValid);
    }

    [Fact]
    public void ExternalPaneCoordinates_IsValid_ReturnsFalseForZeroPaneHeight()
    {
        // Arrange
        var coordinates = new ExternalPaneCoordinates
        {
            Pane = new CharacterCoordinates(0, 0, 80, 0),
            Cell = new CellDimensions(10, 20)
        };

        // Act & Assert
        Assert.False(coordinates.IsValid);
    }

    [Fact]
    public void ExternalCoordinatesConfig_HasCorrectDefaults()
    {
        // Arrange & Act
        var config = new ExternalCoordinatesConfig();

        // Assert
        Assert.False(config.Enabled);
        Assert.NotNull(config.Providers);
        Assert.Empty(config.Providers);
    }

    [Fact]
    public void ExternalCoordinatesProviderConfig_HasCorrectDefaults()
    {
        // Arrange & Act
        var config = new ExternalCoordinatesProviderConfig();

        // Assert
        Assert.Equal("TMUX.*", config.WindowTitlePattern);
        Assert.Contains("external-pane.json", config.FilePath);
        Assert.NotNull(config.TerminalPadding);
    }

    [Fact]
    public void TerminalPaddingConfig_HasCorrectDefaults()
    {
        // Arrange & Act
        var config = new TerminalPaddingConfig();

        // Assert
        Assert.Equal(32, config.Top);
        Assert.Equal(8, config.Left);
        Assert.Equal(0, config.Bottom);
        Assert.Equal(8, config.Right);
    }

    [Fact]
    public void TerminalPaddingConfig_ToTerminalPadding_ConvertsCorrectly()
    {
        // Arrange
        var config = new TerminalPaddingConfig
        {
            Top = 40,
            Left = 10,
            Bottom = 5,
            Right = 10
        };

        // Act
        var result = config.ToTerminalPadding();

        // Assert
        Assert.Equal(40, result.Top);
        Assert.Equal(10, result.Left);
        Assert.Equal(5, result.Bottom);
        Assert.Equal(10, result.Right);
    }

    [Fact]
    public void ExternalCoordinatesProviderConfig_GetExpandedFilePath_ExpandsEnvironmentVariables()
    {
        // Arrange
        var config = new ExternalCoordinatesProviderConfig
        {
            FilePath = "%TEMP%\\test-file.json"
        };

        // Act
        var result = config.GetExpandedFilePath();

        // Assert
        var expectedTempPath = Environment.GetEnvironmentVariable("TEMP") ?? "";
        Assert.Contains(expectedTempPath, result);
        Assert.EndsWith("test-file.json", result);
    }

    [Fact]
    public void CharacterCoordinates_StoresValuesCorrectly()
    {
        // Arrange & Act
        var coords = new CharacterCoordinates(1, 2, 80, 24);

        // Assert
        Assert.Equal(1, coords.Top);
        Assert.Equal(2, coords.Left);
        Assert.Equal(80, coords.Width);
        Assert.Equal(24, coords.Height);
    }

    [Fact]
    public void CellDimensions_StoresValuesCorrectly()
    {
        // Arrange & Act
        var dims = new CellDimensions(10, 20);

        // Assert
        Assert.Equal(10, dims.Width);
        Assert.Equal(20, dims.Height);
    }

    [Fact]
    public void TerminalPadding_StoresValuesCorrectly()
    {
        // Arrange & Act
        var padding = new TerminalPadding(32, 8, 0, 8);

        // Assert
        Assert.Equal(32, padding.Top);
        Assert.Equal(8, padding.Left);
        Assert.Equal(0, padding.Bottom);
        Assert.Equal(8, padding.Right);
    }

    [Fact]
    public void ExternalCoordinatesService_HasProviders_ReturnsFalseWhenEmpty()
    {
        // Arrange & Act
        using var service = new ExternalCoordinatesService();

        // Assert
        Assert.False(service.HasProviders);
    }

    [Fact]
    public void ExternalCoordinatesService_TryGetExternalBounds_ReturnsFalseWithNoProviders()
    {
        // Arrange
        using var service = new ExternalCoordinatesService();
        var windowBounds = new Rectangle(100, 100, 800, 600);

        // Act
        var result = service.TryGetExternalBounds("TMUX - bash", windowBounds, out var externalBounds);

        // Assert
        Assert.False(result);
        Assert.Equal(default, externalBounds);
    }

    [Fact]
    public void ExternalCoordinatesService_TryGetExternalBounds_ReturnsFalseWithNullTitle()
    {
        // Arrange
        using var service = new ExternalCoordinatesService();
        var windowBounds = new Rectangle(100, 100, 800, 600);

        // Act
        var result = service.TryGetExternalBounds(null, windowBounds, out var externalBounds);

        // Assert
        Assert.False(result);
    }

    [Fact]
    public void ExternalCoordinatesService_TryGetExternalBounds_ReturnsFalseWithEmptyTitle()
    {
        // Arrange
        using var service = new ExternalCoordinatesService();
        var windowBounds = new Rectangle(100, 100, 800, 600);

        // Act
        var result = service.TryGetExternalBounds("", windowBounds, out var externalBounds);

        // Assert
        Assert.False(result);
    }
}
