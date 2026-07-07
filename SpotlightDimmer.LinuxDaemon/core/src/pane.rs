//! Screen-space join of an inner pane rect with the focused window frame.
//!
//! Port of `getPaneRect()` from the GNOME extension's appIntegrations.js.
//! Coordinate model (docs/TMUX_INTEGRATION.md):
//! `screen_x = frame.x + content_offset_x + wezterm_pane_offset_x + tmux_rel_x`

use crate::primitives::Rect;

/// Compute the screen-space rect of a focused inner region (tmux pane).
///
/// - `frame`: focused window frame rect (screen space)
/// - `content_offset`: configured ContentOffsetX/Y (window chrome)
/// - `pane_offset`: origin of the wezterm pane's cell grid within the window
///   (non-zero for wezterm-native splits)
/// - `pane`: tmux pane rect in pixels relative to the terminal content origin
///
/// The result is clamped to the window frame: a dimming overlay does not need
/// to be pixel-perfect, but it must never highlight outside the window.
/// Returns `None` when the clamped rect is empty, so callers fall back to the
/// whole window.
pub fn pane_rect(
    frame: &Rect,
    content_offset: (i32, i32),
    pane_offset: (i32, i32),
    pane: &Rect,
) -> Option<Rect> {
    let origin_x = frame.x + content_offset.0 + pane_offset.0;
    let origin_y = frame.y + content_offset.1 + pane_offset.1;

    let left = (origin_x + pane.x).max(frame.x);
    let top = (origin_y + pane.y).max(frame.y);
    let right = (origin_x + pane.x + pane.width).min(frame.right());
    let bottom = (origin_y + pane.y + pane.height).min(frame.bottom());

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
        let result = pane_rect(&FRAME, (10, 40), (0, 0), &Rect::new(300, 0, 500, 760)).unwrap();
        assert_eq!(result, Rect::new(410, 240, 500, 760));
    }

    #[test]
    fn wezterm_split_offset_shifts_origin() {
        let result = pane_rect(&FRAME, (0, 0), (600, 0), &Rect::new(0, 0, 400, 300)).unwrap();
        assert_eq!(result, Rect::new(700, 200, 400, 300));
    }

    #[test]
    fn result_is_clamped_to_frame() {
        // Pane extends past the right/bottom of the frame
        let result = pane_rect(&FRAME, (0, 0), (0, 0), &Rect::new(1000, 700, 500, 500)).unwrap();
        assert_eq!(result, Rect::new(1100, 900, 200, 100));
    }

    #[test]
    fn empty_after_clamping_returns_none() {
        // Pane entirely outside the frame
        assert!(pane_rect(&FRAME, (0, 0), (0, 0), &Rect::new(2000, 0, 100, 100)).is_none());
        // Zero-size pane
        assert!(pane_rect(&FRAME, (0, 0), (0, 0), &Rect::new(0, 0, 0, 100)).is_none());
    }
}
