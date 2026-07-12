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
