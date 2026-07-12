namespace SpotlightDimmer.Core;

/// <summary>
/// Stores the latest pane report per tmux client tty and selects which report
/// applies to the focused terminal. Windows analog of the Linux daemon's
/// IntegrationState (integrations/mod.rs), with one key difference: Windows
/// cannot discover a WSL pts from the terminal side, so instead of a direct
/// tty join the selection uses a fallback ladder:
///
/// 1. Deterministic hint match (WEZTERM_PANE reported by the hook script vs.
///    the focused wezterm pane id) when a hint is provided.
/// 2. Single live report (the dominant single-tmux-client case).
/// 3. Most recently updated report (every pane switch fires a tmux hook, so
///    the freshest report almost always belongs to the pane the user just
///    interacted with; best-effort for multi-client setups).
/// </summary>
public class PaneTrackerState
{
    private readonly struct StoredReport(PaneReport report, long updateTicks)
    {
        public PaneReport Report { get; } = report;
        public long UpdateTicks { get; } = updateTicks;
    }

    private readonly Dictionary<string, StoredReport> _reportsByTty = new(StringComparer.Ordinal);

    /// <summary>Number of live (non-cleared) reports.</summary>
    public int Count => _reportsByTty.Count;

    /// <summary>
    /// Applies a report: updates store geometry, clears remove it.
    /// </summary>
    /// <param name="report">The parsed report.</param>
    /// <param name="nowTicks">A monotonic timestamp for freshness ordering (e.g. Environment.TickCount64).</param>
    /// <returns>True when the stored state changed.</returns>
    public bool Apply(in PaneReport report, long nowTicks)
    {
        if (report.Tty is null)
            return false;

        if (report.Kind == PaneReportKind.Clear)
            return _reportsByTty.Remove(report.Tty);

        _reportsByTty[report.Tty] = new StoredReport(report, nowTicks);
        return true;
    }

    /// <summary>
    /// Removes the stored report for a tty.
    /// </summary>
    public bool Clear(string tty) => _reportsByTty.Remove(tty);

    /// <summary>
    /// Removes all stored reports (config reload, integration teardown).
    /// </summary>
    public void ClearAll() => _reportsByTty.Clear();

    /// <summary>
    /// Selects the report that applies to the focused terminal via the
    /// disambiguation ladder (hint match, then single entry, then freshest).
    /// </summary>
    /// <param name="weztermPaneHint">The focused wezterm pane id to match against reported WEZTERM_PANE hints, or null.</param>
    /// <param name="report">The selected report.</param>
    /// <returns>True when a report was selected.</returns>
    public bool TrySelectReport(string? weztermPaneHint, out PaneReport report)
    {
        report = default;

        if (_reportsByTty.Count == 0)
            return false;

        // 1. Deterministic hint match
        if (!string.IsNullOrEmpty(weztermPaneHint))
        {
            foreach (var stored in _reportsByTty.Values)
            {
                if (string.Equals(stored.Report.WeztermPane, weztermPaneHint, StringComparison.Ordinal))
                {
                    report = stored.Report;
                    return true;
                }
            }
        }

        // 2. Single live report
        if (_reportsByTty.Count == 1)
        {
            foreach (var stored in _reportsByTty.Values)
            {
                report = stored.Report;
                return true;
            }
        }

        // 3. Most recently updated report
        var bestTicks = long.MinValue;
        var found = false;
        foreach (var stored in _reportsByTty.Values)
        {
            if (stored.UpdateTicks > bestTicks)
            {
                bestTicks = stored.UpdateTicks;
                report = stored.Report;
                found = true;
            }
        }

        return found;
    }
}
