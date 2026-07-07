//! Overlay geometry calculator.
//!
//! Port of the GNOME extension's `calculator.js`, which itself ports
//! `SpotlightDimmer.Core/AppState.cs`. Pure functions, no side effects.

use serde::Serialize;

use crate::config::{DimmingMode, OverlayConfig};
use crate::primitives::{Color, Rect};

/// Overlay region indices matching the C# `OverlayRegion` enum and the
/// St.Widget slot order in the GNOME extension's overlayManager.js.
pub mod region {
    pub const FULLSCREEN: u8 = 0;
    pub const TOP: u8 = 1;
    pub const BOTTOM: u8 = 2;
    pub const LEFT: u8 = 3;
    pub const RIGHT: u8 = 4;
    pub const CENTER: u8 = 5;
}

/// A single overlay to render. Field names match the definitions consumed by
/// the GNOME extension's `overlayManager.updateMonitor()` so the serialized
/// JSON can be applied there without transformation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OverlayDef {
    pub region: u8,
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub color: Color,
    pub opacity: u8,
}

/// Calculate overlays for a single monitor.
///
/// `monitor` is the monitor's *work area* (panels/docks excluded).
/// `window_rect` is the spotlight target on the focused monitor (window frame
/// or resolved inner pane rect), `None` on non-focused monitors.
///
/// Returns `None` to signal "keep existing state": the window reported 0x0
/// dimensions, which happens transiently during focus changes and would cause
/// flicker if applied (see calculator.js:52-54).
pub fn calculate(
    config: &OverlayConfig,
    monitor: &Rect,
    window_rect: Option<&Rect>,
    is_focused_monitor: bool,
) -> Option<Vec<OverlayDef>> {
    if let Some(w) = window_rect {
        if w.width == 0 || w.height == 0 {
            return None;
        }
    }

    // Non-focused monitors always get a full-screen overlay
    if !is_focused_monitor {
        return Some(full_screen_overlay(monitor, config));
    }

    // Focused monitor behavior depends on mode
    match config.mode {
        // No overlays on focused monitor in FullScreen mode
        DimmingMode::FullScreen => Some(Vec::new()),
        DimmingMode::Partial => Some(partial_overlays(
            monitor,
            window_rect,
            config.inactive_color,
            config.inactive_opacity,
        )),
        DimmingMode::PartialWithActive => {
            Some(partial_with_active_overlays(monitor, window_rect, config))
        }
        // Unknown mode, no overlays
        DimmingMode::Unknown => Some(Vec::new()),
    }
}

/// Single full-screen overlay for a non-focused monitor.
fn full_screen_overlay(monitor: &Rect, config: &OverlayConfig) -> Vec<OverlayDef> {
    vec![OverlayDef {
        region: region::FULLSCREEN,
        visible: true,
        x: monitor.x,
        y: monitor.y,
        width: monitor.width,
        height: monitor.height,
        color: config.inactive_color,
        opacity: config.inactive_opacity,
    }]
}

/// The 4 edge overlays (Top, Bottom, Left, Right) around a focused window.
/// Port of `AppState.UpdatePartialOverlays()`.
fn partial_overlays(
    monitor: &Rect,
    window: Option<&Rect>,
    color: Color,
    opacity: u8,
) -> Vec<OverlayDef> {
    let Some(window) = window else {
        return Vec::new();
    };

    let clamped = window.clamp_to(monitor);

    // Window not visible on this monitor: no overlays
    if clamped.width <= 0 || clamped.height <= 0 {
        return Vec::new();
    }

    // Window fills the entire monitor (maximized/fullscreen): no edge overlays
    if clamped == *monitor {
        return Vec::new();
    }

    let mut overlays = Vec::with_capacity(4);

    // Top overlay: full width, from display top to window top
    let top_height = clamped.y - monitor.y;
    if top_height > 0 {
        overlays.push(OverlayDef {
            region: region::TOP,
            visible: true,
            x: monitor.x,
            y: monitor.y,
            width: monitor.width,
            height: top_height,
            color,
            opacity,
        });
    }

    // Bottom overlay: full width, from window bottom to display bottom
    let bottom_y = clamped.bottom();
    let bottom_height = monitor.bottom() - bottom_y;
    if bottom_height > 0 {
        overlays.push(OverlayDef {
            region: region::BOTTOM,
            visible: true,
            x: monitor.x,
            y: bottom_y,
            width: monitor.width,
            height: bottom_height,
            color,
            opacity,
        });
    }

    // Left overlay: window height, from display left to window left
    let left_width = clamped.x - monitor.x;
    if left_width > 0 {
        overlays.push(OverlayDef {
            region: region::LEFT,
            visible: true,
            x: monitor.x,
            y: clamped.y,
            width: left_width,
            height: clamped.height,
            color,
            opacity,
        });
    }

    // Right overlay: window height, from window right to display right
    let right_x = clamped.right();
    let right_width = monitor.right() - right_x;
    if right_width > 0 {
        overlays.push(OverlayDef {
            region: region::RIGHT,
            visible: true,
            x: right_x,
            y: clamped.y,
            width: right_width,
            height: clamped.height,
            color,
            opacity,
        });
    }

    overlays
}

/// 4 edge overlays plus a center overlay on the focused window.
/// Port of `AppState.UpdatePartialWithActiveOverlays()`.
fn partial_with_active_overlays(
    monitor: &Rect,
    window: Option<&Rect>,
    config: &OverlayConfig,
) -> Vec<OverlayDef> {
    let mut overlays = partial_overlays(
        monitor,
        window,
        config.inactive_color,
        config.inactive_opacity,
    );

    if let Some(window) = window {
        let clamped = window.clamp_to(monitor);

        if clamped.width > 0 && clamped.height > 0 {
            overlays.push(OverlayDef {
                region: region::CENTER,
                visible: true,
                x: clamped.x,
                y: clamped.y,
                width: clamped.width,
                height: clamped.height,
                color: config.active_color,
                opacity: config.active_opacity,
            });
        }
    }

    overlays
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONITOR: Rect = Rect::new(0, 32, 2560, 1408);

    fn config(mode: DimmingMode) -> OverlayConfig {
        OverlayConfig {
            mode,
            ..OverlayConfig::default()
        }
    }

    fn find(defs: &[OverlayDef], region: u8) -> Option<&OverlayDef> {
        defs.iter().find(|d| d.region == region)
    }

    #[test]
    fn non_focused_monitor_gets_full_screen_overlay_in_every_mode() {
        for mode in [
            DimmingMode::FullScreen,
            DimmingMode::Partial,
            DimmingMode::PartialWithActive,
        ] {
            let defs = calculate(&config(mode), &MONITOR, None, false).unwrap();
            assert_eq!(defs.len(), 1);
            let d = &defs[0];
            assert_eq!(d.region, region::FULLSCREEN);
            assert!(d.visible);
            assert_eq!(
                (d.x, d.y, d.width, d.height),
                (MONITOR.x, MONITOR.y, MONITOR.width, MONITOR.height)
            );
            assert_eq!(d.opacity, 153);
            assert_eq!(d.color, Color::BLACK);
        }
    }

    #[test]
    fn fullscreen_mode_focused_monitor_has_no_overlays() {
        let window = Rect::new(100, 100, 800, 600);
        let defs = calculate(
            &config(DimmingMode::FullScreen),
            &MONITOR,
            Some(&window),
            true,
        )
        .unwrap();
        assert!(defs.is_empty());
    }

    #[test]
    fn zero_size_window_freezes_state() {
        let window = Rect::new(100, 100, 0, 600);
        assert!(calculate(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).is_none());

        let window = Rect::new(100, 100, 800, 0);
        assert!(calculate(
            &config(DimmingMode::FullScreen),
            &MONITOR,
            Some(&window),
            true
        )
        .is_none());
    }

    #[test]
    fn partial_mode_centered_window_produces_four_edges() {
        let window = Rect::new(500, 400, 800, 600);
        let defs = calculate(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();
        assert_eq!(defs.len(), 4);

        let top = find(&defs, region::TOP).unwrap();
        assert_eq!((top.x, top.y, top.width, top.height), (0, 32, 2560, 368));

        let bottom = find(&defs, region::BOTTOM).unwrap();
        assert_eq!(
            (bottom.x, bottom.y, bottom.width, bottom.height),
            (0, 1000, 2560, 440)
        );

        let left = find(&defs, region::LEFT).unwrap();
        assert_eq!(
            (left.x, left.y, left.width, left.height),
            (0, 400, 500, 600)
        );

        let right = find(&defs, region::RIGHT).unwrap();
        assert_eq!(
            (right.x, right.y, right.width, right.height),
            (1300, 400, 1260, 600)
        );
    }

    #[test]
    fn partial_mode_window_at_top_left_corner_omits_top_and_left() {
        let window = Rect::new(0, 32, 800, 600);
        let defs = calculate(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();
        assert_eq!(defs.len(), 2);
        assert!(find(&defs, region::TOP).is_none());
        assert!(find(&defs, region::LEFT).is_none());
        assert!(find(&defs, region::BOTTOM).is_some());
        assert!(find(&defs, region::RIGHT).is_some());
    }

    #[test]
    fn partial_mode_maximized_window_has_no_edge_overlays() {
        let defs = calculate(
            &config(DimmingMode::Partial),
            &MONITOR,
            Some(&MONITOR),
            true,
        )
        .unwrap();
        assert!(defs.is_empty());
    }

    #[test]
    fn partial_mode_window_overflowing_monitor_is_clamped() {
        // Window extends past the right and bottom edges
        let window = Rect::new(2000, 1000, 1000, 1000);
        let defs = calculate(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();

        // Only top and left edges remain
        assert_eq!(defs.len(), 2);
        let top = find(&defs, region::TOP).unwrap();
        assert_eq!((top.x, top.y, top.width, top.height), (0, 32, 2560, 968));
        let left = find(&defs, region::LEFT).unwrap();
        assert_eq!(
            (left.x, left.y, left.width, left.height),
            (0, 1000, 2000, 440)
        );
    }

    #[test]
    fn partial_mode_window_entirely_off_monitor_produces_no_overlays() {
        let window = Rect::new(5000, 5000, 800, 600);
        let defs = calculate(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();
        assert!(defs.is_empty());
    }

    #[test]
    fn partial_with_active_adds_center_overlay_with_active_style() {
        let cfg = OverlayConfig {
            mode: DimmingMode::PartialWithActive,
            active_color: Color {
                r: 10,
                g: 20,
                b: 30,
            },
            active_opacity: 42,
            ..OverlayConfig::default()
        };
        let window = Rect::new(500, 400, 800, 600);
        let defs = calculate(&cfg, &MONITOR, Some(&window), true).unwrap();
        assert_eq!(defs.len(), 5);

        let center = find(&defs, region::CENTER).unwrap();
        assert_eq!(
            (center.x, center.y, center.width, center.height),
            (500, 400, 800, 600)
        );
        assert_eq!(
            center.color,
            Color {
                r: 10,
                g: 20,
                b: 30
            }
        );
        assert_eq!(center.opacity, 42);
    }

    #[test]
    fn partial_with_active_maximized_window_keeps_center_only() {
        let defs = calculate(
            &config(DimmingMode::PartialWithActive),
            &MONITOR,
            Some(&MONITOR),
            true,
        )
        .unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].region, region::CENTER);
    }

    #[test]
    fn partial_with_active_center_is_clamped_to_monitor() {
        let window = Rect::new(-100, 0, 800, 600);
        let defs = calculate(
            &config(DimmingMode::PartialWithActive),
            &MONITOR,
            Some(&window),
            true,
        )
        .unwrap();
        let center = find(&defs, region::CENTER).unwrap();
        assert_eq!(
            (center.x, center.y, center.width, center.height),
            (0, 32, 700, 568)
        );
    }

    #[test]
    fn unknown_mode_produces_no_overlays_on_focused_monitor() {
        let window = Rect::new(500, 400, 800, 600);
        let defs = calculate(&config(DimmingMode::Unknown), &MONITOR, Some(&window), true).unwrap();
        assert!(defs.is_empty());
    }
}
