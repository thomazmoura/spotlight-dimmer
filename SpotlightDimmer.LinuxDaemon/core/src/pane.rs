//! Screen-space join of an inner pane rect with the focused window geometry.
//!
//! Port of `getPaneRect()` from the GNOME extension's appIntegrations.js.
//! Coordinate model (docs/TMUX_INTEGRATION.md):
//! `screen_x = client.x + content_offset_x + wezterm_pane_offset_x + tmux_rel_x`
//! where `client` is the decoration-excluded client area when the adapter
//! reports one, else the window frame.

use crate::primitives::Rect;

/// Compute the screen-space rect of a focused inner region (tmux pane).
///
/// - `frame`: focused window frame rect (screen space, decorations included)
/// - `client`: client-area rect (decorations excluded), when the adapter
///   reports one. Using it as the base makes the join immune to the
///   decoration size changing between windowed and maximized states.
/// - `content_offset`: configured ContentOffsetX/Y (terminal-internal chrome:
///   tab bar, window padding)
/// - `pane_offset`: origin of the wezterm pane's cell grid within the window
///   (non-zero for wezterm-native splits)
/// - `pane`: tmux pane rect in pixels relative to the terminal content origin
///
/// The result is clamped to the base rect: a dimming overlay does not need
/// to be pixel-perfect, but it must never highlight outside the window (or
/// onto its title bar, when the client area is known).
/// Returns `None` when the clamped rect is empty, so callers fall back to the
/// whole window.
pub fn pane_rect(
    frame: &Rect,
    client: Option<&Rect>,
    content_offset: (i32, i32),
    pane_offset: (i32, i32),
    pane: &Rect,
) -> Option<Rect> {
    let base = client.unwrap_or(frame);
    let origin_x = base.x + content_offset.0 + pane_offset.0;
    let origin_y = base.y + content_offset.1 + pane_offset.1;

    let left = (origin_x + pane.x).max(base.x);
    let top = (origin_y + pane.y).max(base.y);
    let right = (origin_x + pane.x + pane.width).min(base.right());
    let bottom = (origin_y + pane.y + pane.height).min(base.bottom());

    if right - left <= 0 || bottom - top <= 0 {
        return None;
    }

    Some(Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

/// Compute the screen-space rect of a focused inner region reported in
/// terminal **cells** instead of pixels (the Herdr provider).
///
/// Herdr publishes its layout in cell coordinates and, unlike tmux, does not
/// report the terminal's cell pixel size. The grid is mapped proportionally
/// onto the window's content box instead, which needs no font metrics and
/// follows resize, font and DPI changes for free.
///
/// - `grid`: total size of the terminal cell grid (columns, rows), i.e. the
///   whole surface Herdr draws on, sidebar and tab bar included
/// - `cells`: the focused pane rect in that grid
/// - `content_offset`: the terminal's internal padding, applied on *both*
///   sides of each axis (the cell grid is inset by it)
///
/// Clamped to the client area and `None` when empty, exactly like
/// [`pane_rect`].
pub fn pane_rect_from_cells(
    frame: &Rect,
    client: Option<&Rect>,
    content_offset: (i32, i32),
    grid: (i32, i32),
    cells: &Rect,
) -> Option<Rect> {
    let base = client.unwrap_or(frame);
    let (cols, rows) = grid;
    if cols <= 0 || rows <= 0 {
        return None;
    }

    // The cell grid sits inside the padding on both sides of each axis.
    let inner_width = base.width - 2 * content_offset.0;
    let inner_height = base.height - 2 * content_offset.1;
    if inner_width <= 0 || inner_height <= 0 {
        return None;
    }

    let origin_x = base.x + content_offset.0;
    let origin_y = base.y + content_offset.1;

    // Map both edges (rather than origin + scaled size) so adjacent panes
    // stay flush: rounding cannot open a gap or an overlap between them.
    let left = origin_x + scale(cells.x, inner_width, cols);
    let right = origin_x + scale(cells.right(), inner_width, cols);
    let top = origin_y + scale(cells.y, inner_height, rows);
    let bottom = origin_y + scale(cells.bottom(), inner_height, rows);

    let rect = Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
    .clamp_to(base);

    (rect.width > 0 && rect.height > 0).then_some(rect)
}

/// `cell * span / count`, rounded to the nearest pixel.
fn scale(cell: i32, span: i32, count: i32) -> i32 {
    let scaled = cell as i64 * span as i64 * 2 + count as i64;
    (scaled / (count as i64 * 2)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Rect = Rect::new(100, 200, 1200, 800);

    #[test]
    fn joins_offsets_and_pane_rect() {
        // Content offset (10, 40), wezterm pane at origin, tmux pane at
        // (300, 0) sized 500x760
        let result =
            pane_rect(&FRAME, None, (10, 40), (0, 0), &Rect::new(300, 0, 500, 760)).unwrap();
        assert_eq!(result, Rect::new(410, 240, 500, 760));
    }

    #[test]
    fn wezterm_split_offset_shifts_origin() {
        let result = pane_rect(&FRAME, None, (0, 0), (600, 0), &Rect::new(0, 0, 400, 300)).unwrap();
        assert_eq!(result, Rect::new(700, 200, 400, 300));
    }

    #[test]
    fn result_is_clamped_to_frame() {
        // Pane extends past the right/bottom of the frame
        let result =
            pane_rect(&FRAME, None, (0, 0), (0, 0), &Rect::new(1000, 700, 500, 500)).unwrap();
        assert_eq!(result, Rect::new(1100, 900, 200, 100));
    }

    #[test]
    fn empty_after_clamping_returns_none() {
        // Pane entirely outside the frame
        assert!(pane_rect(&FRAME, None, (0, 0), (0, 0), &Rect::new(2000, 0, 100, 100)).is_none());
        // Zero-size pane
        assert!(pane_rect(&FRAME, None, (0, 0), (0, 0), &Rect::new(0, 0, 0, 100)).is_none());
    }

    #[test]
    fn client_rect_overrides_frame_as_origin() {
        // Windowed: 2px borders, 38px title bar above the client area
        let client = Rect::new(102, 240, 1196, 758);
        let result = pane_rect(
            &FRAME,
            Some(&client),
            (0, 0),
            (0, 0),
            &Rect::new(300, 0, 500, 700),
        )
        .unwrap();
        assert_eq!(result, Rect::new(402, 240, 500, 700));
    }

    #[test]
    fn clamps_to_client_rect_not_frame() {
        // Pane rect overhangs the client area on all sides; the highlight
        // must stay inside the client area (never on the title bar/borders)
        let client = Rect::new(102, 240, 1196, 758);
        let result = pane_rect(
            &FRAME,
            Some(&client),
            (0, 0),
            (0, 0),
            &Rect::new(-10, -10, 2000, 2000),
        )
        .unwrap();
        assert_eq!(result, client);
    }

    #[test]
    fn windowed_vs_maximized_same_offset() {
        // Regression test for the windowed-mode offset bug: the same
        // content_offset must align the pane in both states because the
        // decoration delta lives entirely in frame-vs-client, not the offset.
        let pane = Rect::new(300, 0, 500, 700);
        let content_offset = (4, 8); // wezterm padding, state-independent

        // Windowed: client is inset from the frame by the decorations
        let windowed_client = Rect::new(102, 240, 1196, 758);
        let windowed = pane_rect(&FRAME, Some(&windowed_client), content_offset, (0, 0), &pane)
            .unwrap();
        assert_eq!(windowed, Rect::new(406, 248, 500, 700));

        // Maximized: no decorations, client == frame
        let maximized = pane_rect(&FRAME, Some(&FRAME), content_offset, (0, 0), &pane).unwrap();
        assert_eq!(maximized, Rect::new(404, 208, 500, 700));

        // In both states the pane sits at client.origin + offset + pane.origin
        assert_eq!(
            (windowed.x - windowed_client.x, windowed.y - windowed_client.y),
            (maximized.x - FRAME.x, maximized.y - FRAME.y)
        );
    }
}

#[cfg(test)]
mod cell_tests {
    use super::*;

    // 200x100 client area, 2px padding on every side -> a 196x96 cell grid.
    const CLIENT: Rect = Rect::new(0, 0, 200, 100);
    const GRID: (i32, i32) = (98, 48); // 2px per column, 2px per row
    const OFFSET: (i32, i32) = (2, 2);

    #[test]
    fn maps_the_whole_grid_onto_the_content_box() {
        let all = Rect::new(0, 0, 98, 48);
        let result = pane_rect_from_cells(&CLIENT, Some(&CLIENT), OFFSET, GRID, &all).unwrap();
        assert_eq!(result, Rect::new(2, 2, 196, 96));
    }

    #[test]
    fn skips_the_sidebar_and_tab_bar() {
        // Herdr's pane area starts after the sidebar (x) and tab bar (y)
        let pane = Rect::new(26, 1, 72, 47);
        let result = pane_rect_from_cells(&CLIENT, Some(&CLIENT), OFFSET, GRID, &pane).unwrap();
        assert_eq!(result, Rect::new(2 + 52, 2 + 2, 144, 94));
    }

    #[test]
    fn adjacent_panes_stay_flush() {
        // 98 columns split 49/49 must not leave a seam or overlap
        let left = pane_rect_from_cells(
            &CLIENT,
            Some(&CLIENT),
            OFFSET,
            GRID,
            &Rect::new(0, 0, 49, 48),
        )
        .unwrap();
        let right = pane_rect_from_cells(
            &CLIENT,
            Some(&CLIENT),
            OFFSET,
            GRID,
            &Rect::new(49, 0, 49, 48),
        )
        .unwrap();
        assert_eq!(left.right(), right.x);
    }

    #[test]
    fn falls_back_to_the_frame_without_a_client_rect() {
        let frame = Rect::new(10, 20, 200, 100);
        let result =
            pane_rect_from_cells(&frame, None, (0, 0), (100, 50), &Rect::new(50, 25, 50, 25))
                .unwrap();
        assert_eq!(result, Rect::new(110, 70, 100, 50));
    }

    #[test]
    fn clamps_to_the_client_area() {
        // A stale grid (fewer columns than the pane claims) must not paint
        // outside the window
        let result = pane_rect_from_cells(
            &CLIENT,
            Some(&CLIENT),
            OFFSET,
            (10, 10),
            &Rect::new(0, 0, 40, 40),
        )
        .unwrap();
        // Left/top keep the padding origin; right/bottom stop at the window
        assert_eq!(result, Rect::new(2, 2, 198, 98));
    }

    #[test]
    fn degenerate_input_yields_nothing() {
        assert!(
            pane_rect_from_cells(&CLIENT, None, OFFSET, (0, 48), &Rect::new(0, 0, 1, 1)).is_none()
        );
        // Zero-width pane
        assert!(
            pane_rect_from_cells(&CLIENT, None, OFFSET, GRID, &Rect::new(0, 0, 0, 48)).is_none()
        );
        // Padding larger than the window
        assert!(
            pane_rect_from_cells(&CLIENT, None, (150, 2), GRID, &Rect::new(0, 0, 98, 48)).is_none()
        );
    }
}
