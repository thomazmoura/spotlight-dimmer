using System.Text.Json;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Tests for $schema property serialization in AppConfig.
/// </summary>
public class SchemaPropertyTests
{
    [Fact]
    public void AppConfig_WhenSerialized_ShouldIncludeSchemaProperty()
    {
        // Arrange
        var config = AppConfig.Default;
        config.UpdateVersion("0.8.6");

        // Act
        var json = JsonSerializer.Serialize(config, AppConfigJsonContext.Default.Options);

        // Assert
        Assert.Contains("\"$schema\"", json);
    }

    [Fact]
    public void AppConfig_WhenTheConfigIncludesAdditionalValuesAfterPlusSignal_ShouldIgnoreTheAdditionalInfo()
    {
        // Arrange
        var config = AppConfig.Default;
        config.UpdateVersion("0.9.5+build123");

        // Act
        var schemaUrl = config.Schema;

        // Assert
        Assert.Equal("https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v0.9.5/config.schema.json", schemaUrl);
    }

    [Fact]
    public void AppConfig_WhenSerialized_SchemaPropertyShouldBeFirst()
    {
        // Arrange
        var config = AppConfig.Default;
        config.UpdateVersion("0.8.6");

        // Act
        var json = JsonSerializer.Serialize(config, AppConfigJsonContext.Default.Options);
        var lines = json.Split(new[] { "\r\n", "\r", "\n" }, StringSplitOptions.None);

        // Assert
        Assert.True(lines.Length > 1, "JSON should have multiple lines");
        Assert.StartsWith("  \"$schema\"", lines[1]);
    }

    [Fact]
    public void AppConfig_WhenSerialized_SchemaUrlShouldBeCorrect()
    {
        // Arrange
        var config = AppConfig.Default;
        config.UpdateVersion("0.8.6");

        // Act
        var json = JsonSerializer.Serialize(config, AppConfigJsonContext.Default.Options);
        var expectedUrl = "https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v0.8.6/config.schema.json";

        // Assert
        Assert.Contains($"\"$schema\": \"{expectedUrl}\"", json);
    }

    [Fact]
    public void AppConfig_AfterDeserialization_ShouldPreserveSchemaProperty()
    {
        // Arrange
        var config = AppConfig.Default;
        config.UpdateVersion("0.8.6");
        var json = JsonSerializer.Serialize(config, AppConfigJsonContext.Default.Options);

        // Act
        var deserializedConfig = JsonSerializer.Deserialize<AppConfig>(json, AppConfigJsonContext.Default.Options);

        // Assert
        Assert.NotNull(deserializedConfig);
        Assert.NotNull(deserializedConfig.Schema);
        Assert.NotEmpty(deserializedConfig.Schema);
        Assert.Equal("https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v0.8.6/config.schema.json",
            deserializedConfig.Schema);
    }

    [Fact]
    public void AppConfig_UpdateVersion_ShouldSetBothVersionAndSchema()
    {
        // Arrange
        var config = AppConfig.Default;
        var version = "1.0.0";

        // Act
        config.UpdateVersion(version);

        // Assert
        Assert.Equal(version, config.ConfigVersion);
        Assert.Equal($"https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v{version}/config.schema.json",
            config.Schema);
    }
}
