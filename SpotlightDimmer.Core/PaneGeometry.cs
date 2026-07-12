namespace SpotlightDimmer.Core;

/// <summary>
/// Screen-space join of an inner pane rect with the focused window frame.
/// Port of the Linux daemon's <c>core/src/pane.rs</c>, extended with cell-grid
/// math because ConPTY does not propagate pixel cell sizes into WSL: tmux pane
/// reports carry cell coordinates, and pixels are derived by dividing the
/// terminal's known content rect by the total cell grid.
/// </summary>
public static class PaneGeometry
{
    /// <summary>
    /// Computes the screen-space rect of a focused inner region (pane) given in pixels.
    ///
    /// Coordinate model (docs/TMUX_INTEGRATION.md):
    /// <c>screen_x = frame.x + contentOffsetX + paneOffsetX + pane.X</c>
    ///
    /// The result is clamped to the window frame: a dimming overlay does not need to
    /// be pixel-perfect, but it must never highlight outside the window. Returns null
    /// when the clamped rect is empty, so callers fall back to the whole window.
    /// </summary>
    /// <param name="frame">Focused window frame rect (screen space).</param>
    /// <param name="contentOffsetX">Configured ContentOffsetX (window chrome).</param>
    /// <param name="contentOffsetY">Configured ContentOffsetY (window chrome).</param>
    /// <param name="paneOffsetX">Origin of the terminal pane's cell grid within the window (non-zero for terminal-native splits).</param>
    /// <param name="paneOffsetY">Origin of the terminal pane's cell grid within the window.</param>
    /// <param name="pane">Inner pane rect in pixels relative to the terminal content origin.</param>
    public static Rectangle? PaneRect(
        in Rectangle frame,
        int contentOffsetX,
        int contentOffsetY,
        int paneOffsetX,
        int paneOffsetY,
        in Rectangle pane)
    {
        var originX = frame.X + contentOffsetX + paneOffsetX;
        var originY = frame.Y + contentOffsetY + paneOffsetY;

        var left = Math.Max(originX + pane.X, frame.X);
        var top = Math.Max(originY + pane.Y, frame.Y);
        var right = Math.Min(originX + pane.X + pane.Width, frame.Right);
        var bottom = Math.Min(originY + pane.Y + pane.Height, frame.Bottom);

        if (right - left <= 0 || bottom - top <= 0)
            return null;

        return Rectangle.FromLTRB(left, top, right, bottom);
    }

    /// <summary>
    /// Converts a tmux pane's cell-grid geometry to a pixel rect relative to the
    /// terminal content origin, then joins it with the window frame via
    /// <see cref="PaneRect"/>.
    ///
    /// The cell size is derived from the content rect and the total client grid
    /// (<c>cellWidth = contentRect.Width / gridCols</c>), because ConPTY reports
    /// zero pixel sizes to tmux inside WSL. Edges are computed independently
    /// (<c>right = (paneLeft + paneWidth) * cellWidth</c>) so rounding never drifts
    /// across panes.
    ///
    /// tmux pane coordinates are relative to the window area, which sits below the
    /// status bar when the status bar is at the top; that offset is applied here.
    /// </summary>
    /// <param name="frame">Focused window frame rect (screen space).</param>
    /// <param name="contentRect">Terminal content rect in screen space (the cell grid's bounding box, e.g. the focused Windows Terminal pane control, already shifted by ContentOffsetX/Y).</param>
    /// <param name="report">The tmux pane report carrying cell geometry.</param>
    /// <returns>The pane rect clamped to the frame, or null to fall back to the whole window.</returns>
    public static Rectangle? CellPaneRect(
        in Rectangle frame,
        in Rectangle contentRect,
        in PaneReport report)
    {
        if (report.GridCols <= 0 || report.GridRows <= 0)
            return null;
        if (contentRect.Width <= 0 || contentRect.Height <= 0)
            return null;

        var cellWidth = contentRect.Width / (double)report.GridCols;
        var cellHeight = contentRect.Height / (double)report.GridRows;

        // Rows above the pane grid when the status bar is at the top.
        var statusTopRows = report.StatusAtTop ? report.StatusRows : 0;

        var left = (int)Math.Round(report.PaneLeft * cellWidth);
        var top = (int)Math.Round((statusTopRows + report.PaneTop) * cellHeight);
        var right = (int)Math.Round((report.PaneLeft + report.PaneWidth) * cellWidth);
        var bottom = (int)Math.Round((statusTopRows + report.PaneTop + report.PaneHeight) * cellHeight);

        if (right - left <= 0 || bottom - top <= 0)
            return null;

        // contentRect is already in screen space, so the content offset relative to
        // the frame is (contentRect.X - frame.X, contentRect.Y - frame.Y).
        var pane = Rectangle.FromLTRB(left, top, right, bottom);
        return PaneRect(
            frame,
            contentRect.X - frame.X,
            contentRect.Y - frame.Y,
            0,
            0,
            pane);
    }
}
