using SpotlightDimmer.Core;

namespace SpotlightDimmer.Tests;

/// <summary>
/// Tests for the v1 pane report wire protocol parser.
/// </summary>
public class PaneReportParserTests
{
    [Fact]
    public void TryParse_FullUpdateLine_ParsesAllFields()
    {
        var line = "v1|update|tty=/dev/pts/3|cells=81,0,80,45|grid=162,46|status=1,bottom|wt=abc-123|wz=7|sid=$1";

        Assert.True(PaneReport.TryParse(line, out var report));
        Assert.Equal(PaneReportKind.Update, report.Kind);
        Assert.Equal("/dev/pts/3", report.Tty);
        Assert.Equal(81, report.PaneLeft);
        Assert.Equal(0, report.PaneTop);
        Assert.Equal(80, report.PaneWidth);
        Assert.Equal(45, report.PaneHeight);
        Assert.Equal(162, report.GridCols);
        Assert.Equal(46, report.GridRows);
        Assert.Equal(1, report.StatusRows);
        Assert.False(report.StatusAtTop);
        Assert.Equal("abc-123", report.WtSession);
        Assert.Equal("7", report.WeztermPane);
        Assert.Equal("$1", report.SessionId);
    }

    [Fact]
    public void TryParse_MinimalUpdateLine_ParsesWithDefaults()
    {
        var line = "v1|update|tty=/dev/pts/0|cells=0,0,120,30|grid=120,30";

        Assert.True(PaneReport.TryParse(line, out var report));
        Assert.Equal(0, report.StatusRows);
        Assert.False(report.StatusAtTop);
        Assert.Null(report.WtSession);
        Assert.Null(report.WeztermPane);
        Assert.Null(report.SessionId);
    }

    [Fact]
    public void TryParse_StatusAtTop_IsParsed()
    {
        var line = "v1|update|tty=/dev/pts/0|cells=0,1,120,29|grid=120,30|status=1,top";

        Assert.True(PaneReport.TryParse(line, out var report));
        Assert.Equal(1, report.StatusRows);
        Assert.True(report.StatusAtTop);
    }

    [Fact]
    public void TryParse_ClearLine_ParsesKindAndTty()
    {
        Assert.True(PaneReport.TryParse("v1|clear|tty=/dev/pts/3", out var report));
        Assert.Equal(PaneReportKind.Clear, report.Kind);
        Assert.Equal("/dev/pts/3", report.Tty);
    }

    [Theory]
    [InlineData("")]
    [InlineData("v2|update|tty=/dev/pts/0|cells=0,0,1,1|grid=1,1")] // unknown version
    [InlineData("v1|reset|tty=/dev/pts/0")] // unknown kind
    [InlineData("v1|update|cells=0,0,1,1|grid=1,1")] // missing tty
    [InlineData("v1|update|tty=/dev/pts/0|grid=1,1")] // missing cells
    [InlineData("v1|update|tty=/dev/pts/0|cells=0,0,1,1")] // missing grid
    [InlineData("v1|update|tty=/dev/pts/0|cells=0,0,0,1|grid=1,1")] // zero pane width
    [InlineData("v1|update|tty=/dev/pts/0|cells=0,0,1,1|grid=0,1")] // zero grid cols
    [InlineData("v1|update|tty=/dev/pts/0|cells=-1,0,1,1|grid=1,1")] // negative pane position
    [InlineData("v1|update|tty=/dev/pts/0|cells=a,b,c,d|grid=1,1")] // non-numeric cells
    [InlineData("v1|clear")] // clear without tty
    [InlineData("garbage")]
    public void TryParse_InvalidLines_AreRejected(string line)
    {
        Assert.False(PaneReport.TryParse(line, out _));
    }

    [Fact]
    public void TryParse_UnknownKeysAreIgnored()
    {
        var line = "v1|update|tty=/dev/pts/0|cells=0,0,10,10|grid=10,10|future=stuff|x=1";

        Assert.True(PaneReport.TryParse(line, out var report));
        Assert.Equal("/dev/pts/0", report.Tty);
    }

    [Fact]
    public void TryParse_OverlongLineIsRejected()
    {
        var line = "v1|update|tty=/dev/pts/0|cells=0,0,10,10|grid=10,10|pad=" + new string('x', 1100);

        Assert.False(PaneReport.TryParse(line, out _));
    }

    [Fact]
    public void TryParse_TrimsSurroundingWhitespace()
    {
        Assert.True(PaneReport.TryParse("  v1|clear|tty=/dev/pts/9\n", out var report));
        Assert.Equal("/dev/pts/9", report.Tty);
    }
}
