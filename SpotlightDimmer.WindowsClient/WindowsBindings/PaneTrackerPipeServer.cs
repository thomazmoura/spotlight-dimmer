using System.Collections.Concurrent;
using System.IO.Pipes;
using System.Text;
using Microsoft.Extensions.Logging;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Named-pipe server receiving pane geometry reports from
/// SpotlightDimmer.PaneReport.exe (invoked by tmux hooks inside WSL through
/// Windows interop). Windows analog of the Linux daemon's D-Bus PaneTracker
/// service.
///
/// Protocol: one UTF-8 line (max 1024 bytes) per connection, then the client
/// disconnects. Parsed reports are queued and a WM_APP_PANE_REPORT message is
/// posted to the tracker's message window so the report is consumed on the
/// main thread (same marshalling pattern as FocusTracker's WM_FOCUS_UPDATE).
/// All I/O and parse failures are logged at Debug and swallowed - a broken
/// client must never affect the overlay pipeline.
/// </summary>
internal class PaneTrackerPipeServer : IDisposable
{
    /// <summary>The pipe name; full path is \\.\pipe\SpotlightDimmer.PaneTracker.</summary>
    public const string PipeName = "SpotlightDimmer.PaneTracker";

    private const int MaxMessageBytes = 1024;

    private readonly ILogger<PaneTrackerPipeServer> _logger;
    private readonly ConcurrentQueue<PaneReport> _queue = new();
    private readonly CancellationTokenSource _cts = new();
    private IntPtr _notifyWindow;
    private Task? _serverTask;

    public PaneTrackerPipeServer(ILogger<PaneTrackerPipeServer> logger)
    {
        _logger = logger;
    }

    /// <summary>
    /// Starts the pipe server. Parsed reports trigger a WM_APP_PANE_REPORT
    /// message to the given window.
    /// </summary>
    /// <param name="notifyWindow">Message window that consumes queued reports on the main thread.</param>
    public void Start(IntPtr notifyWindow)
    {
        if (_serverTask != null)
            return;

        _notifyWindow = notifyWindow;
        _serverTask = Task.Run(() => ServerLoopAsync(_cts.Token));
        _logger.LogDebug("[PANE] Pipe server started on \\\\.\\pipe\\{PipeName}", PipeName);
    }

    /// <summary>
    /// Dequeues one pending report. Called from the main thread while handling
    /// WM_APP_PANE_REPORT.
    /// </summary>
    public bool TryDequeue(out PaneReport report) => _queue.TryDequeue(out report);

    private async Task ServerLoopAsync(CancellationToken ct)
    {
        var buffer = new byte[MaxMessageBytes];

        while (!ct.IsCancellationRequested)
        {
            try
            {
                using var server = new NamedPipeServerStream(
                    PipeName,
                    PipeDirection.In,
                    NamedPipeServerStream.MaxAllowedServerInstances,
                    PipeTransmissionMode.Byte,
                    PipeOptions.Asynchronous);

                await server.WaitForConnectionAsync(ct).ConfigureAwait(false);

                var total = 0;
                while (total < buffer.Length)
                {
                    var read = await server.ReadAsync(buffer.AsMemory(total, buffer.Length - total), ct).ConfigureAwait(false);
                    if (read <= 0)
                        break;
                    total += read;
                    if (buffer.AsSpan(total - read, read).IndexOf((byte)'\n') >= 0)
                        break;
                }

                if (total > 0)
                {
                    var line = Encoding.UTF8.GetString(buffer, 0, total);
                    if (PaneReport.TryParse(line, out var report))
                    {
                        _queue.Enqueue(report);
                        WinApi.PostMessage(_notifyWindow, WinApi.WM_APP_PANE_REPORT, IntPtr.Zero, IntPtr.Zero);
                        _logger.LogDebug("[PANE] Report received: {Kind} tty={Tty} cells=({Left},{Top}) {Width}x{Height} grid={Cols}x{Rows}",
                            report.Kind, report.Tty, report.PaneLeft, report.PaneTop, report.PaneWidth, report.PaneHeight, report.GridCols, report.GridRows);
                    }
                    else
                    {
                        _logger.LogDebug("[PANE] Ignoring unparseable report ({Length} bytes)", total);
                    }
                }
            }
            catch (OperationCanceledException)
            {
                break;
            }
            catch (Exception ex)
            {
                _logger.LogDebug("[PANE] Pipe server error (retrying): {Message}", ex.Message);
                try
                {
                    await Task.Delay(100, ct).ConfigureAwait(false);
                }
                catch (OperationCanceledException)
                {
                    break;
                }
            }
        }
    }

    public void Dispose()
    {
        _cts.Cancel();
        try
        {
            _serverTask?.Wait(500);
        }
        catch
        {
            // Shutdown must never throw
        }
        _cts.Dispose();
        _logger.LogDebug("[PANE] Pipe server stopped");
    }
}
