using System.Text.Json;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Round-trip serialization tests for the AppIntegrations config section through
/// the source-generated AppConfigJsonContext (the same path AOT builds use).
/// </summary>
public class AppIntegrationsSerializationTests
{
    [Fact]
    public void AppConfig_RoundTripsAppIntegrations()
    {
        var config = AppConfig.Default;
        config.AppIntegrations.Add(new AppIntegration
        {
            ProcessName = "WindowsTerminal.exe",
            Provider = "windows-terminal",
            ContentOffsetX = 8,
            ContentOffsetY = 8
        });
        config.AppIntegrations.Add(new AppIntegration
        {
            ProcessName = "wezterm-gui.exe",
            Provider = "wezterm"
        });

        var json = JsonSerializer.Serialize(config, AppConfigJsonContext.Default.Options);
        var restored = JsonSerializer.Deserialize<AppConfig>(json, AppConfigJsonContext.Default.Options);

        Assert.NotNull(restored);
        Assert.Equal(2, restored.AppIntegrations.Count);
        Assert.Equal("WindowsTerminal.exe", restored.AppIntegrations[0].ProcessName);
        Assert.Equal("windows-terminal", restored.AppIntegrations[0].Provider);
        Assert.Equal(8, restored.AppIntegrations[0].ContentOffsetX);
        Assert.Equal(8, restored.AppIntegrations[0].ContentOffsetY);
        Assert.Equal("wezterm-gui.exe", restored.AppIntegrations[1].ProcessName);
        Assert.Equal("wezterm", restored.AppIntegrations[1].Provider);
        Assert.Equal(0, restored.AppIntegrations[1].ContentOffsetX);
    }

    [Fact]
    public void AppConfig_MissingAppIntegrationsDeserializesToEmptyList()
    {
        // Older config files have no AppIntegrations section
        var json = "{ \"Overlay\": { \"Mode\": \"Partial\" } }";

        var restored = JsonSerializer.Deserialize<AppConfig>(json, AppConfigJsonContext.Default.Options);

        Assert.NotNull(restored);
        Assert.NotNull(restored.AppIntegrations);
        Assert.Empty(restored.AppIntegrations);
    }

    [Fact]
    public void AppConfig_AppIntegrationDefaultsApplyWhenFieldsOmitted()
    {
        var json = "{ \"AppIntegrations\": [ { \"ProcessName\": \"alacritty.exe\" } ] }";

        var restored = JsonSerializer.Deserialize<AppConfig>(json, AppConfigJsonContext.Default.Options);

        Assert.NotNull(restored);
        var integration = Assert.Single(restored.AppIntegrations);
        Assert.Equal("alacritty.exe", integration.ProcessName);
        Assert.Equal("tmux", integration.Provider);
        Assert.Equal(0, integration.ContentOffsetX);
        Assert.Equal(0, integration.ContentOffsetY);
    }
}
