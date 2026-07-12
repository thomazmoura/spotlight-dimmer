using SpotlightDimmer.Core;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Tests for the pane geometry math. The PaneRect cases are ports of the Linux
/// daemon's core/src/pane.rs tests; the CellPaneRect cases cover the Windows
/// cell-grid derivation (ConPTY reports no pixel sizes, so pixels come from
/// dividing the content rect by the tmux client grid).
/// </summary>
public class PaneGeometryTests
{
    private static readonly Rectangle Frame = new(100, 200, 1200, 800);

    [Fact]
    public void PaneRect_JoinsOffsetsAndPaneRect()
    {
        // Content offset (10, 40), terminal pane at origin, tmux pane at (300, 0) sized 500x760
        var result = PaneGeometry.PaneRect(Frame, 10, 40, 0, 0, new Rectangle(300, 0, 500, 760));

        Assert.Equal(new Rectangle(410, 240, 500, 760), result);
    }

    [Fact]
    public void PaneRect_TerminalSplitOffsetShiftsOrigin()
    {
        var result = PaneGeometry.PaneRect(Frame, 0, 0, 600, 0, new Rectangle(0, 0, 400, 300));

        Assert.Equal(new Rectangle(700, 200, 400, 300), result);
    }

    [Fact]
    public void PaneRect_ResultIsClampedToFrame()
    {
        // Pane extends past the right/bottom of the frame
        var result = PaneGeometry.PaneRect(Frame, 0, 0, 0, 0, new Rectangle(1000, 700, 500, 500));

        Assert.Equal(new Rectangle(1100, 900, 200, 100), result);
    }

    [Fact]
    public void PaneRect_EmptyAfterClampingReturnsNull()
    {
        // Pane entirely outside the frame
        Assert.Null(PaneGeometry.PaneRect(Frame, 0, 0, 0, 0, new Rectangle(2000, 0, 100, 100)));

        // Zero-size pane
        Assert.Null(PaneGeometry.PaneRect(Frame, 0, 0, 0, 0, new Rectangle(0, 0, 0, 100)));
    }

    private static PaneReport MakeReport(
        int paneLeft, int paneTop, int paneWidth, int paneHeight,
        int gridCols, int gridRows,
        int statusRows = 0, bool statusAtTop = false)
    {
        return new PaneReport
        {
            Kind = PaneReportKind.Update,
            Tty = "/dev/pts/1",
            PaneLeft = paneLeft,
            PaneTop = paneTop,
            PaneWidth = paneWidth,
            PaneHeight = paneHeight,
            GridCols = gridCols,
            GridRows = gridRows,
            StatusRows = statusRows,
            StatusAtTop = statusAtTop
        };
    }

    [Fact]
    public void CellPaneRect_MapsCellsOverContentRect()
    {
        // Content rect 1600x900 at (100, 200), grid 160x45 -> cell size 10x20.
        // Right half of a horizontal split: pane at column 80, width 80, full height.
        var contentRect = new Rectangle(100, 200, 1600, 900);
        var frame = new Rectangle(100, 200, 1600, 900);
        var report = MakeReport(80, 0, 80, 45, 160, 45);

        var result = PaneGeometry.CellPaneRect(frame, contentRect, report);

        Assert.Equal(new Rectangle(900, 200, 800, 900), result);
    }

    [Fact]
    public void CellPaneRect_StatusBarAtBottomDoesNotShiftPanes()
    {
        // Grid 160x46 with 1 status row at the bottom: panes cover rows 0..44.
        var contentRect = new Rectangle(0, 0, 1600, 920);
        var frame = new Rectangle(0, 0, 1600, 920);
        var report = MakeReport(0, 0, 160, 45, 160, 46, statusRows: 1, statusAtTop: false);

        var result = PaneGeometry.CellPaneRect(frame, contentRect, report);

        // 45 rows of 20px each = 900px starting at y=0
        Assert.Equal(new Rectangle(0, 0, 1600, 900), result);
    }

    [Fact]
    public void CellPaneRect_StatusBarAtTopShiftsPanesDown()
    {
        var contentRect = new Rectangle(0, 0, 1600, 920);
        var frame = new Rectangle(0, 0, 1600, 920);
        var report = MakeReport(0, 0, 160, 45, 160, 46, statusRows: 1, statusAtTop: true);

        var result = PaneGeometry.CellPaneRect(frame, contentRect, report);

        // Pane rows start below the 20px status row
        Assert.Equal(new Rectangle(0, 20, 1600, 900), result);
    }

    [Fact]
    public void CellPaneRect_FractionalCellSizesRoundPerEdge()
    {
        // 1043px over 46 rows = 22.673...px per row. Edge-based rounding must keep
        // adjacent panes contiguous: bottom of row 23 == top of row 23.
        var contentRect = new Rectangle(0, 0, 1000, 1043);
        var frame = new Rectangle(0, 0, 1000, 1043);
        var top = MakeReport(0, 0, 100, 23, 100, 46);
        var bottom = MakeReport(0, 23, 100, 23, 100, 46);

        var topRect = PaneGeometry.CellPaneRect(frame, contentRect, top);
        var bottomRect = PaneGeometry.CellPaneRect(frame, contentRect, bottom);

        Assert.NotNull(topRect);
        Assert.NotNull(bottomRect);
        Assert.Equal(topRect.Value.Bottom, bottomRect.Value.Top);
    }

    [Fact]
    public void CellPaneRect_ZeroGridReturnsNull()
    {
        var contentRect = new Rectangle(0, 0, 1600, 900);
        var frame = new Rectangle(0, 0, 1600, 900);
        var report = MakeReport(0, 0, 80, 45, 0, 0);

        Assert.Null(PaneGeometry.CellPaneRect(frame, contentRect, report));
    }

    [Fact]
    public void CellPaneRect_EmptyContentRectReturnsNull()
    {
        var frame = new Rectangle(0, 0, 1600, 900);
        var report = MakeReport(0, 0, 80, 45, 160, 45);

        Assert.Null(PaneGeometry.CellPaneRect(frame, new Rectangle(0, 0, 0, 0), report));
    }

    [Fact]
    public void CellPaneRect_ContentRectOutsideFrameIsClamped()
    {
        // Misconfigured offsets push the content rect past the frame; the result
        // must never highlight outside the window.
        var frame = new Rectangle(0, 0, 1000, 800);
        var contentRect = new Rectangle(900, 700, 1600, 900);
        var report = MakeReport(0, 0, 160, 45, 160, 45);

        var result = PaneGeometry.CellPaneRect(frame, contentRect, report);

        Assert.NotNull(result);
        Assert.True(result.Value.Right <= frame.Right);
        Assert.True(result.Value.Bottom <= frame.Bottom);
    }
}
