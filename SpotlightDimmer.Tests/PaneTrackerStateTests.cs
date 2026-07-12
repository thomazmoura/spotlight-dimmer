using SpotlightDimmer.Core;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Tests for pane report storage and the focused-terminal disambiguation ladder
/// (deterministic hint match, then single live report, then freshest report),
/// plus the AppIntegrations config matching (mirrors the Linux daemon's config
/// tests in core/src/config.rs).
/// </summary>
public class PaneTrackerStateTests
{
    private static PaneReport Update(string tty, string? weztermPane = null)
    {
        return new PaneReport
        {
            Kind = PaneReportKind.Update,
            Tty = tty,
            PaneLeft = 0,
            PaneTop = 0,
            PaneWidth = 80,
            PaneHeight = 24,
            GridCols = 80,
            GridRows = 24,
            WeztermPane = weztermPane
        };
    }

    private static PaneReport Clear(string tty) =>
        new() { Kind = PaneReportKind.Clear, Tty = tty };

    [Fact]
    public void SingleReport_IsSelectedWithoutHint()
    {
        var state = new PaneTrackerState();
        state.Apply(Update("/dev/pts/1"), nowTicks: 100);

        Assert.True(state.TrySelectReport(null, out var report));
        Assert.Equal("/dev/pts/1", report.Tty);
    }

    [Fact]
    public void HintMatch_WinsOverFreshness()
    {
        var state = new PaneTrackerState();
        state.Apply(Update("/dev/pts/1", weztermPane: "7"), nowTicks: 100);
        state.Apply(Update("/dev/pts/2", weztermPane: "9"), nowTicks: 200);

        Assert.True(state.TrySelectReport("7", out var report));
        Assert.Equal("/dev/pts/1", report.Tty);
    }

    [Fact]
    public void NoHintMatch_FallsBackToFreshest()
    {
        var state = new PaneTrackerState();
        state.Apply(Update("/dev/pts/1"), nowTicks: 100);
        state.Apply(Update("/dev/pts/2"), nowTicks: 200);
        state.Apply(Update("/dev/pts/1"), nowTicks: 300);

        Assert.True(state.TrySelectReport("no-such-pane", out var report));
        Assert.Equal("/dev/pts/1", report.Tty);
    }

    [Fact]
    public void EmptyState_SelectsNothing()
    {
        var state = new PaneTrackerState();

        Assert.False(state.TrySelectReport(null, out _));
    }

    [Fact]
    public void ClearReport_RemovesEntry()
    {
        var state = new PaneTrackerState();
        state.Apply(Update("/dev/pts/1"), nowTicks: 100);
        state.Apply(Clear("/dev/pts/1"), nowTicks: 200);

        Assert.Equal(0, state.Count);
        Assert.False(state.TrySelectReport(null, out _));
    }

    [Fact]
    public void ClearUnknownTty_ReportsNoChange()
    {
        var state = new PaneTrackerState();

        Assert.False(state.Apply(Clear("/dev/pts/9"), nowTicks: 100));
    }

    [Fact]
    public void ClearAll_EmptiesState()
    {
        var state = new PaneTrackerState();
        state.Apply(Update("/dev/pts/1"), nowTicks: 100);
        state.Apply(Update("/dev/pts/2"), nowTicks: 200);

        state.ClearAll();

        Assert.Equal(0, state.Count);
    }

    [Fact]
    public void UpdateSameTty_ReplacesGeometry()
    {
        var state = new PaneTrackerState();
        state.Apply(Update("/dev/pts/1"), nowTicks: 100);

        var updated = Update("/dev/pts/1") with { PaneLeft = 40, PaneWidth = 40 };
        state.Apply(updated, nowTicks: 200);

        Assert.Equal(1, state.Count);
        Assert.True(state.TrySelectReport(null, out var report));
        Assert.Equal(40, report.PaneLeft);
    }

    // --- AppIntegrations config matching (mirrors Linux core/src/config.rs tests) ---

    [Fact]
    public void MatchIntegration_MatchesProcessNameCaseInsensitively()
    {
        var config = new AppConfig
        {
            AppIntegrations =
            {
                new AppIntegration { ProcessName = "WindowsTerminal.exe", Provider = "windows-terminal" }
            }
        };

        var match = config.MatchIntegration("windowsterminal.exe");

        Assert.NotNull(match);
        Assert.Equal("windows-terminal", match.Provider);
    }

    [Fact]
    public void MatchIntegration_ReturnsNullForUnknownProcess()
    {
        var config = new AppConfig
        {
            AppIntegrations =
            {
                new AppIntegration { ProcessName = "WindowsTerminal.exe" }
            }
        };

        Assert.Null(config.MatchIntegration("notepad.exe"));
        Assert.Null(config.MatchIntegration(null));
        Assert.Null(config.MatchIntegration(""));
    }

    [Fact]
    public void MatchIntegration_SkipsEntriesWithEmptyProcessName()
    {
        var config = new AppConfig
        {
            AppIntegrations =
            {
                new AppIntegration { ProcessName = "" },
                new AppIntegration { ProcessName = "wezterm-gui.exe", Provider = "wezterm" }
            }
        };

        var match = config.MatchIntegration("wezterm-gui.exe");

        Assert.NotNull(match);
        Assert.Equal("wezterm", match.Provider);
    }

    [Fact]
    public void AppIntegration_DefaultsProviderToTmux()
    {
        var integration = new AppIntegration();

        Assert.Equal("tmux", integration.Provider);
        Assert.Equal(0, integration.ContentOffsetX);
        Assert.Equal(0, integration.ContentOffsetY);
    }
}
