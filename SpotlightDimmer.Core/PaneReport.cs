namespace SpotlightDimmer.Core;

/// <summary>
/// The kind of pane report message.
/// </summary>
public enum PaneReportKind
{
    /// <summary>Updates the focused pane geometry for a tmux client.</summary>
    Update,

    /// <summary>Removes the stored geometry for a tmux client (detach/exit).</summary>
    Clear
}

/// <summary>
/// A parsed pane report message (wire protocol v1), as sent by the tmux hook
/// script through SpotlightDimmer.PaneReport.exe over the named pipe.
///
/// Wire grammar (UTF-8, one line per pipe connection, max 1024 bytes):
/// <code>
/// v1|update|tty=/dev/pts/3|cells=81,0,80,45|grid=162,46|status=1,bottom|wt=&lt;GUID&gt;|wz=7|sid=$1
/// v1|clear|tty=/dev/pts/3
/// </code>
///
/// Coordinates are tmux CELLS, not pixels: ConPTY reports zero pixel sizes to
/// tmux inside WSL, so the Windows side derives pixels from the terminal's
/// content rect divided by <c>grid</c>. Unknown versions are rejected; unknown
/// keys are ignored for forward compatibility.
/// </summary>
public readonly record struct PaneReport
{
    /// <summary>The message kind (update or clear).</summary>
    public PaneReportKind Kind { get; init; }

    /// <summary>The reporting tmux client's tty (e.g. "/dev/pts/3"). Storage key on the receiving side.</summary>
    public string Tty { get; init; }

    /// <summary>Focused pane's left edge in cells, relative to the tmux window area.</summary>
    public int PaneLeft { get; init; }

    /// <summary>Focused pane's top edge in cells, relative to the tmux window area.</summary>
    public int PaneTop { get; init; }

    /// <summary>Focused pane's width in cells.</summary>
    public int PaneWidth { get; init; }

    /// <summary>Focused pane's height in cells.</summary>
    public int PaneHeight { get; init; }

    /// <summary>Total client grid width in cells (tmux #{client_width}).</summary>
    public int GridCols { get; init; }

    /// <summary>Total client grid height in cells (tmux #{client_height}), including status rows.</summary>
    public int GridRows { get; init; }

    /// <summary>Rows occupied by the tmux status bar (0 when off).</summary>
    public int StatusRows { get; init; }

    /// <summary>True when the status bar is at the top (pane rows are shifted down).</summary>
    public bool StatusAtTop { get; init; }

    /// <summary>WT_SESSION join hint from the reporting environment, or null.</summary>
    public string? WtSession { get; init; }

    /// <summary>WEZTERM_PANE join hint from the reporting environment, or null.</summary>
    public string? WeztermPane { get; init; }

    /// <summary>tmux session id (diagnostics only), or null.</summary>
    public string? SessionId { get; init; }

    /// <summary>
    /// Parses a v1 wire line into a <see cref="PaneReport"/>.
    /// Returns false for unknown versions, malformed messages, or missing
    /// required fields (update requires tty, cells and grid; clear requires tty).
    /// </summary>
    public static bool TryParse(ReadOnlySpan<char> line, out PaneReport report)
    {
        report = default;

        line = line.Trim();
        if (line.IsEmpty || line.Length > 1024)
            return false;

        // Version token
        var next = NextToken(ref line);
        if (!next.SequenceEqual("v1"))
            return false;

        // Kind token
        var kindToken = NextToken(ref line);
        PaneReportKind kind;
        if (kindToken.SequenceEqual("update"))
            kind = PaneReportKind.Update;
        else if (kindToken.SequenceEqual("clear"))
            kind = PaneReportKind.Clear;
        else
            return false;

        string? tty = null;
        string? wtSession = null;
        string? weztermPane = null;
        string? sessionId = null;
        int paneLeft = 0, paneTop = 0, paneWidth = 0, paneHeight = 0;
        int gridCols = 0, gridRows = 0;
        int statusRows = 0;
        bool statusAtTop = false;
        bool hasCells = false, hasGrid = false;

        while (!line.IsEmpty)
        {
            var token = NextToken(ref line);
            var eq = token.IndexOf('=');
            if (eq <= 0)
                continue; // tolerate malformed/unknown tokens

            var key = token[..eq];
            var value = token[(eq + 1)..];

            if (key.SequenceEqual("tty"))
            {
                if (!value.IsEmpty)
                    tty = value.ToString();
            }
            else if (key.SequenceEqual("cells"))
            {
                hasCells = TryParseIntQuad(value, out paneLeft, out paneTop, out paneWidth, out paneHeight);
            }
            else if (key.SequenceEqual("grid"))
            {
                hasGrid = TryParseIntPair(value, out gridCols, out gridRows);
            }
            else if (key.SequenceEqual("status"))
            {
                // "rows,top" or "rows,bottom"; malformed status degrades to 0 rows
                var comma = value.IndexOf(',');
                if (comma > 0 && int.TryParse(value[..comma], out var rows) && rows >= 0)
                {
                    statusRows = rows;
                    statusAtTop = value[(comma + 1)..].SequenceEqual("top");
                }
            }
            else if (key.SequenceEqual("wt"))
            {
                if (!value.IsEmpty)
                    wtSession = value.ToString();
            }
            else if (key.SequenceEqual("wz"))
            {
                if (!value.IsEmpty)
                    weztermPane = value.ToString();
            }
            else if (key.SequenceEqual("sid"))
            {
                if (!value.IsEmpty)
                    sessionId = value.ToString();
            }
            // Unknown keys are ignored (forward compatibility)
        }

        if (tty is null)
            return false;

        if (kind == PaneReportKind.Update)
        {
            if (!hasCells || !hasGrid)
                return false;
            if (paneWidth <= 0 || paneHeight <= 0 || gridCols <= 0 || gridRows <= 0)
                return false;
            if (paneLeft < 0 || paneTop < 0)
                return false;
        }

        report = new PaneReport
        {
            Kind = kind,
            Tty = tty,
            PaneLeft = paneLeft,
            PaneTop = paneTop,
            PaneWidth = paneWidth,
            PaneHeight = paneHeight,
            GridCols = gridCols,
            GridRows = gridRows,
            StatusRows = statusRows,
            StatusAtTop = statusAtTop,
            WtSession = wtSession,
            WeztermPane = weztermPane,
            SessionId = sessionId
        };
        return true;
    }

    private static ReadOnlySpan<char> NextToken(ref ReadOnlySpan<char> line)
    {
        var sep = line.IndexOf('|');
        ReadOnlySpan<char> token;
        if (sep < 0)
        {
            token = line;
            line = default;
        }
        else
        {
            token = line[..sep];
            line = line[(sep + 1)..];
        }
        return token;
    }

    private static bool TryParseIntPair(ReadOnlySpan<char> value, out int a, out int b)
    {
        a = b = 0;
        var comma = value.IndexOf(',');
        if (comma <= 0)
            return false;
        return int.TryParse(value[..comma], out a) && int.TryParse(value[(comma + 1)..], out b);
    }

    private static bool TryParseIntQuad(ReadOnlySpan<char> value, out int a, out int b, out int c, out int d)
    {
        a = b = c = d = 0;
        Span<int> parts = stackalloc int[4];
        var count = 0;
        while (count < 4)
        {
            var comma = value.IndexOf(',');
            var piece = comma < 0 ? value : value[..comma];
            if (!int.TryParse(piece, out parts[count]))
                return false;
            count++;
            if (comma < 0)
                break;
            value = value[(comma + 1)..];
        }

        if (count != 4)
            return false;

        a = parts[0];
        b = parts[1];
        c = parts[2];
        d = parts[3];
        return true;
    }
}
