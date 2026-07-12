using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.Extensions.Logging;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// The focused WezTerm pane, resolved from the `wezterm cli`.
/// Coordinates are pixels relative to the window's content origin.
/// </summary>
/// <param name="PaneId">The wezterm pane id (join key against reported WEZTERM_PANE hints).</param>
/// <param name="RelX">Pane origin X relative to the window content, in pixels.</param>
/// <param name="RelY">Pane origin Y relative to the window content, in pixels.</param>
/// <param name="Width">Pane width in pixels (0 when wezterm reports no pixel sizes).</param>
/// <param name="Height">Pane height in pixels (0 when wezterm reports no pixel sizes).</param>
internal readonly record struct WeztermActivePane(string PaneId, int RelX, int RelY, int Width, int Height);

/// <summary>
/// Async wrapper around `wezterm cli list-clients` and `wezterm cli list`.
/// Port of the Linux daemon's integrations/wezterm.rs query chain, minus the
/// `tmux list-clients` liveness check (replaced on Windows by explicit clear
/// reports and the deterministic WEZTERM_PANE join). Runs off the main thread;
/// results are marshalled back by the caller. Every failure path returns null
/// so the caller falls back to the whole window.
/// </summary>
internal class WeztermCliClient
{
    private const int CliTimeoutMs = 2000;

    private readonly ILogger<WeztermCliClient> _logger;

    public WeztermCliClient(ILogger<WeztermCliClient> logger)
    {
        _logger = logger;
    }

    /// <summary>
    /// Resolves the focused wezterm pane: `list-clients` yields the focused
    /// pane id, `list` yields that pane's cell origin and pixel size, from
    /// which the pixel origin is derived (cell size = pane pixel size / pane
    /// cells, so no font metrics are needed).
    /// </summary>
    public async Task<WeztermActivePane?> QueryActivePaneAsync()
    {
        try
        {
            var clientsJson = await RunCliAsync("cli list-clients --format json").ConfigureAwait(false);
            if (clientsJson is null)
                return null;

            var clients = JsonSerializer.Deserialize(clientsJson, WeztermCliJsonContext.Default.ListWeztermClientDto);
            var focusedPaneId = clients?.FirstOrDefault(c => c.FocusedPaneId >= 0)?.FocusedPaneId;
            if (focusedPaneId is null or < 0)
                return null;

            var panesJson = await RunCliAsync("cli list --format json").ConfigureAwait(false);
            if (panesJson is null)
                return null;

            var panes = JsonSerializer.Deserialize(panesJson, WeztermCliJsonContext.Default.ListWeztermPaneDto);
            var pane = panes?.FirstOrDefault(p => p.PaneId == focusedPaneId);
            if (pane is null)
                return null;

            var relX = 0;
            var relY = 0;
            var width = 0;
            var height = 0;

            if (pane.Size is { } size && size.Cols > 0 && size.Rows > 0 &&
                size.PixelWidth > 0 && size.PixelHeight > 0)
            {
                // Cell size derived from the pane's own pixel dimensions
                var cellWidth = size.PixelWidth / (double)size.Cols;
                var cellHeight = size.PixelHeight / (double)size.Rows;
                relX = (int)Math.Round(pane.LeftCol * cellWidth);
                relY = (int)Math.Round(pane.TopRow * cellHeight);
                width = size.PixelWidth;
                height = size.PixelHeight;
            }

            return new WeztermActivePane(
                focusedPaneId.Value.ToString(System.Globalization.CultureInfo.InvariantCulture),
                relX, relY, width, height);
        }
        catch (Exception ex)
        {
            _logger.LogDebug("[WEZTERM] Query failed: {Message}", ex.Message);
            return null;
        }
    }

    private async Task<string?> RunCliAsync(string arguments)
    {
        using var process = new Process();
        process.StartInfo = new ProcessStartInfo
        {
            FileName = "wezterm.exe",
            Arguments = arguments,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true
        };

        try
        {
            if (!process.Start())
                return null;

            var outputTask = process.StandardOutput.ReadToEndAsync();

            using var cts = new CancellationTokenSource(CliTimeoutMs);
            await process.WaitForExitAsync(cts.Token).ConfigureAwait(false);

            if (process.ExitCode != 0)
                return null;

            return await outputTask.ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            _logger.LogDebug("[WEZTERM] CLI timed out: wezterm {Arguments}", arguments);
            try { process.Kill(entireProcessTree: true); } catch { /* already gone */ }
            return null;
        }
        catch (Exception ex)
        {
            // wezterm not installed / not on PATH is the common case
            _logger.LogDebug("[WEZTERM] CLI unavailable: {Message}", ex.Message);
            return null;
        }
    }
}

internal sealed class WeztermClientDto
{
    [JsonPropertyName("focused_pane_id")]
    public int FocusedPaneId { get; set; } = -1;
}

internal sealed class WeztermPaneDto
{
    [JsonPropertyName("pane_id")]
    public int PaneId { get; set; } = -1;

    [JsonPropertyName("left_col")]
    public int LeftCol { get; set; }

    [JsonPropertyName("top_row")]
    public int TopRow { get; set; }

    [JsonPropertyName("size")]
    public WeztermPaneSizeDto? Size { get; set; }
}

internal sealed class WeztermPaneSizeDto
{
    [JsonPropertyName("cols")]
    public int Cols { get; set; }

    [JsonPropertyName("rows")]
    public int Rows { get; set; }

    [JsonPropertyName("pixel_width")]
    public int PixelWidth { get; set; }

    [JsonPropertyName("pixel_height")]
    public int PixelHeight { get; set; }
}

/// <summary>
/// Source-generated JSON context for the wezterm CLI DTOs (AOT requirement).
/// </summary>
[JsonSourceGenerationOptions(PropertyNameCaseInsensitive = false)]
[JsonSerializable(typeof(List<WeztermClientDto>))]
[JsonSerializable(typeof(List<WeztermPaneDto>))]
internal partial class WeztermCliJsonContext : JsonSerializerContext
{
}
