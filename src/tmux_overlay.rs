use crate::tmux_watcher::TmuxPaneInfo;

#[cfg(windows)]
use winapi::shared::windef::RECT;

/// Rectangle representing an overlay area in pixel coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl OverlayRect {
    /// Create a new overlay rectangle
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Get the width of the rectangle
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    /// Get the height of the rectangle
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    /// Check if the rectangle has valid dimensions (positive width and height)
    pub fn is_valid(&self) -> bool {
        self.width() > 0 && self.height() > 0
    }
}

#[cfg(windows)]
impl From<OverlayRect> for RECT {
    fn from(rect: OverlayRect) -> Self {
        RECT {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

/// Configuration for terminal geometry
#[derive(Debug, Clone, Copy)]
pub struct TerminalGeometry {
    /// Font character width in pixels
    pub font_width: u32,
    /// Font character height in pixels
    pub font_height: u32,
    /// Left padding in pixels
    pub padding_left: i32,
    /// Top padding in pixels (including title bar)
    pub padding_top: i32,
}

impl TerminalGeometry {
    /// Create a new terminal geometry configuration
    pub fn new(font_width: u32, font_height: u32, padding_left: i32, padding_top: i32) -> Self {
        Self {
            font_width,
            font_height,
            padding_left,
            padding_top,
        }
    }

    /// Convert character column to pixel x-coordinate (relative to window)
    pub fn col_to_x(&self, col: i32) -> i32 {
        self.padding_left + (col * self.font_width as i32)
    }

    /// Convert character row to pixel y-coordinate (relative to window)
    pub fn row_to_y(&self, row: i32) -> i32 {
        self.padding_top + (row * self.font_height as i32)
    }

    /// Convert pixel x-coordinate to character column
    #[allow(dead_code)]
    pub fn x_to_col(&self, x: i32) -> i32 {
        (x - self.padding_left) / self.font_width as i32
    }

    /// Convert pixel y-coordinate to character row
    #[allow(dead_code)]
    pub fn y_to_row(&self, y: i32) -> i32 {
        (y - self.padding_top) / self.font_height as i32
    }
}

/// Calculate overlay rectangles for inactive tmux panes
/// Returns up to 4 rectangles (top, bottom, left, right) that cover non-active pane areas
/// Coordinates are in absolute screen coordinates (not window-relative)
#[cfg(windows)]
pub fn calculate_tmux_overlay_rects(
    pane_info: &TmuxPaneInfo,
    terminal_window: &RECT,
    geometry: &TerminalGeometry,
) -> Vec<OverlayRect> {
    if !pane_info.is_valid() {
        eprintln!("[TmuxOverlay] Invalid pane info: {:?}", pane_info);
        return vec![];
    }

    let mut overlays = Vec::new();

    // Calculate window content area bounds (absolute screen coordinates)
    let window_left = terminal_window.left;
    let window_top = terminal_window.top;

    // Calculate active pane bounds in window-relative pixel coordinates
    let pane_left_px = geometry.col_to_x(pane_info.pane_left);
    let pane_top_px = geometry.row_to_y(pane_info.pane_top);
    let pane_right_px = geometry.col_to_x(pane_info.pane_right + 1); // +1 because right edge is inclusive
    let pane_bottom_px = geometry.row_to_y(pane_info.pane_bottom + 1); // +1 because bottom edge is inclusive

    // Convert to absolute screen coordinates
    let pane_abs_left = window_left + pane_left_px;
    let pane_abs_top = window_top + pane_top_px;
    let pane_abs_right = window_left + pane_right_px;
    let pane_abs_bottom = window_top + pane_bottom_px;

    // Calculate terminal content area bounds (where text is rendered)
    let content_left = window_left + geometry.padding_left;
    let content_top = window_top + geometry.padding_top;
    let content_right = window_left + geometry.col_to_x(pane_info.window_width);
    let content_bottom = window_top + geometry.row_to_y(pane_info.window_height);

    // Top overlay (above active pane, full terminal width)
    if pane_abs_top > content_top {
        let overlay = OverlayRect::new(content_left, content_top, content_right, pane_abs_top);
        if overlay.is_valid() {
            overlays.push(overlay);
        }
    }

    // Bottom overlay (below active pane, full terminal width)
    if pane_abs_bottom < content_bottom {
        let overlay =
            OverlayRect::new(content_left, pane_abs_bottom, content_right, content_bottom);
        if overlay.is_valid() {
            overlays.push(overlay);
        }
    }

    // Left overlay (left of active pane, between top and bottom of pane)
    if pane_abs_left > content_left {
        let overlay = OverlayRect::new(content_left, pane_abs_top, pane_abs_left, pane_abs_bottom);
        if overlay.is_valid() {
            overlays.push(overlay);
        }
    }

    // Right overlay (right of active pane, between top and bottom of pane)
    if pane_abs_right < content_right {
        let overlay =
            OverlayRect::new(pane_abs_right, pane_abs_top, content_right, pane_abs_bottom);
        if overlay.is_valid() {
            overlays.push(overlay);
        }
    }

    overlays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_rect_new() {
        let rect = OverlayRect::new(10, 20, 100, 200);
        assert_eq!(rect.left, 10);
        assert_eq!(rect.top, 20);
        assert_eq!(rect.right, 100);
        assert_eq!(rect.bottom, 200);
    }

    #[test]
    fn test_overlay_rect_dimensions() {
        let rect = OverlayRect::new(10, 20, 100, 200);
        assert_eq!(rect.width(), 90);
        assert_eq!(rect.height(), 180);
    }

    #[test]
    fn test_overlay_rect_is_valid() {
        let valid = OverlayRect::new(10, 20, 100, 200);
        assert!(valid.is_valid());

        let invalid_width = OverlayRect::new(100, 20, 10, 200);
        assert!(!invalid_width.is_valid());

        let invalid_height = OverlayRect::new(10, 200, 100, 20);
        assert!(!invalid_height.is_valid());

        let zero_width = OverlayRect::new(10, 20, 10, 200);
        assert!(!zero_width.is_valid());
    }

    #[test]
    fn test_terminal_geometry_col_to_x() {
        let geo = TerminalGeometry::new(10, 20, 5, 35);
        assert_eq!(geo.col_to_x(0), 5); // padding_left
        assert_eq!(geo.col_to_x(1), 15); // padding_left + font_width
        assert_eq!(geo.col_to_x(10), 105); // padding_left + 10 * font_width
    }

    #[test]
    fn test_terminal_geometry_row_to_y() {
        let geo = TerminalGeometry::new(10, 20, 5, 35);
        assert_eq!(geo.row_to_y(0), 35); // padding_top
        assert_eq!(geo.row_to_y(1), 55); // padding_top + font_height
        assert_eq!(geo.row_to_y(10), 235); // padding_top + 10 * font_height
    }

    #[test]
    fn test_terminal_geometry_x_to_col() {
        let geo = TerminalGeometry::new(10, 20, 5, 35);
        assert_eq!(geo.x_to_col(5), 0);
        assert_eq!(geo.x_to_col(15), 1);
        assert_eq!(geo.x_to_col(105), 10);
    }

    #[test]
    fn test_terminal_geometry_y_to_row() {
        let geo = TerminalGeometry::new(10, 20, 5, 35);
        assert_eq!(geo.y_to_row(35), 0);
        assert_eq!(geo.y_to_row(55), 1);
        assert_eq!(geo.y_to_row(235), 10);
    }

    #[cfg(windows)]
    #[test]
    fn test_calculate_tmux_overlay_rects_full_screen_pane() {
        // Pane that fills the entire terminal window
        let pane_info = TmuxPaneInfo {
            pane_left: 0,
            pane_top: 0,
            pane_right: 119, // 120 columns
            pane_bottom: 29, // 30 rows
            window_width: 120,
            window_height: 30,
        };

        let terminal_window = RECT {
            left: 100,
            top: 100,
            right: 1300, // 1200px wide
            bottom: 700, // 600px tall
        };

        let geometry = TerminalGeometry::new(10, 20, 0, 0);

        let overlays = calculate_tmux_overlay_rects(&pane_info, &terminal_window, &geometry);

        // No overlays should be created for a full-screen pane
        assert_eq!(overlays.len(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn test_calculate_tmux_overlay_rects_left_pane() {
        // Pane on left half of screen (vertical split)
        let pane_info = TmuxPaneInfo {
            pane_left: 0,
            pane_top: 0,
            pane_right: 59,  // Half width (60 columns)
            pane_bottom: 29, // Full height
            window_width: 120,
            window_height: 30,
        };

        let terminal_window = RECT {
            left: 0,
            top: 0,
            right: 1200,
            bottom: 600,
        };

        let geometry = TerminalGeometry::new(10, 20, 0, 0);

        let overlays = calculate_tmux_overlay_rects(&pane_info, &terminal_window, &geometry);

        // Should have only right overlay (covering right pane)
        assert_eq!(overlays.len(), 1);

        let right_overlay = &overlays[0];
        assert_eq!(right_overlay.left, 600); // After 60 columns
        assert_eq!(right_overlay.top, 0);
        assert_eq!(right_overlay.right, 1200); // Full width
        assert_eq!(right_overlay.bottom, 600); // Full height
    }

    #[cfg(windows)]
    #[test]
    fn test_calculate_tmux_overlay_rects_top_left_quadrant() {
        // Pane in top-left quadrant (both splits)
        let pane_info = TmuxPaneInfo {
            pane_left: 0,
            pane_top: 0,
            pane_right: 59,  // Half width
            pane_bottom: 14, // Half height
            window_width: 120,
            window_height: 30,
        };

        let terminal_window = RECT {
            left: 0,
            top: 0,
            right: 1200,
            bottom: 600,
        };

        let geometry = TerminalGeometry::new(10, 20, 0, 0);

        let overlays = calculate_tmux_overlay_rects(&pane_info, &terminal_window, &geometry);

        // Should have 2 overlays: bottom (below pane) and right (right of pane)
        assert_eq!(overlays.len(), 2);
    }

    #[cfg(windows)]
    #[test]
    fn test_calculate_tmux_overlay_rects_with_padding() {
        // Test with terminal padding
        let pane_info = TmuxPaneInfo {
            pane_left: 0,
            pane_top: 0,
            pane_right: 59,
            pane_bottom: 29,
            window_width: 120,
            window_height: 30,
        };

        let terminal_window = RECT {
            left: 100,
            top: 100,
            right: 1300, // 1200px content + padding
            bottom: 735, // 600px content + padding
        };

        let geometry = TerminalGeometry::new(10, 20, 5, 35);

        let overlays = calculate_tmux_overlay_rects(&pane_info, &terminal_window, &geometry);

        // Should have right overlay accounting for padding
        assert_eq!(overlays.len(), 1);

        let right_overlay = &overlays[0];
        assert_eq!(right_overlay.left, 100 + 5 + 600); // window_left + padding + pane_width
        assert_eq!(right_overlay.top, 100 + 35); // window_top + padding_top
    }

    #[cfg(windows)]
    #[test]
    fn test_calculate_tmux_overlay_rects_invalid_pane() {
        // Invalid pane info (negative coordinates)
        let pane_info = TmuxPaneInfo {
            pane_left: -1,
            pane_top: 0,
            pane_right: 59,
            pane_bottom: 29,
            window_width: 120,
            window_height: 30,
        };

        let terminal_window = RECT {
            left: 0,
            top: 0,
            right: 1200,
            bottom: 600,
        };

        let geometry = TerminalGeometry::new(10, 20, 0, 0);

        let overlays = calculate_tmux_overlay_rects(&pane_info, &terminal_window, &geometry);

        // Should return empty vec for invalid pane info
        assert_eq!(overlays.len(), 0);
    }
}
