//! Overlay geometry calculator.
//!
//! Port of the GNOME extension's `calculator.js`, which itself ports
//! `SpotlightDimmer.Core/AppState.cs`. Pure functions, no side effects.

use serde::Serialize;

use crate::config::{AlwaysOnTopHandling, DimmingMode, OverlayConfig};
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
    /// A surface floating above application windows (an always-on-top
    /// window), covered by its own overlay so it reads uniformly instead of
    /// picking up whatever happens to be beneath it. Unlike the others this
    /// is not a slot index: a monitor can carry several.
    pub const FLOATING: u8 = 6;
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
/// `monitor` is the monitor's full geometry (panels/docks included, so the
/// overlays cover them too, matching the Windows client).
/// `window_rect` is the spotlight target on the focused monitor (window frame
/// or resolved inner pane rect), `None` on non-focused monitors.
/// `floating` holds always-on-top window rects in stacking order, bottom
/// first; it is empty unless `AlwaysOnTopHandling` asks for them.
///
/// The result always covers the whole monitor: every pixel belongs to either
/// the active or the inactive overlay. That is what makes "dim or highlight
/// this floating surface" a single well-defined operation rather than an
/// alpha-compositing accident.
///
/// Returns `None` to signal "keep existing state": the window reported 0x0
/// dimensions, which happens transiently during focus changes and would cause
/// flicker if applied (see calculator.js:52-54).
pub fn calculate(
    config: &OverlayConfig,
    monitor: &Rect,
    window_rect: Option<&Rect>,
    is_focused_monitor: bool,
    floating: &[Rect],
) -> Option<Vec<OverlayDef>> {
    if let Some(w) = window_rect {
        if w.width == 0 || w.height == 0 {
            return None;
        }
    }

    let mut overlays = if is_focused_monitor {
        focused_overlays(config, monitor, window_rect)
    } else {
        // Non-focused monitors are entirely inactive in every mode.
        vec![fill(
            region::FULLSCREEN,
            monitor,
            config.inactive_color,
            config.inactive_opacity,
        )]
    };

    apply_floating(&mut overlays, monitor, config, floating);

    Some(overlays)
}

/// One overlay covering `rect`.
fn fill(region: u8, rect: &Rect, color: Color, opacity: u8) -> OverlayDef {
    OverlayDef {
        region,
        visible: true,
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
        color,
        opacity,
    }
}

/// Overlays for the monitor holding the focused window.
fn focused_overlays(
    config: &OverlayConfig,
    monitor: &Rect,
    window: Option<&Rect>,
) -> Vec<OverlayDef> {
    let spotlight = window
        .map(|w| w.clamp_to(monitor))
        .filter(|c| c.width > 0 && c.height > 0);

    let Some(spotlight) = spotlight else {
        // The focused window is not actually visible on this monitor; there
        // is no spotlight to carve out, so the monitor reads as inactive.
        return vec![fill(
            region::FULLSCREEN,
            monitor,
            config.inactive_color,
            config.inactive_opacity,
        )];
    };

    match config.mode {
        // The whole monitor is the spotlight.
        DimmingMode::FullScreen | DimmingMode::Unknown => vec![fill(
            region::FULLSCREEN,
            monitor,
            config.active_color,
            config.active_opacity,
        )],
        // Partial and PartialWithActive now render identically: the active
        // overlay is always emitted so the monitor is fully covered, and
        // `ActiveOpacity` decides how visible it is. Partial is kept as a
        // separate mode string so existing configs keep parsing.
        DimmingMode::Partial | DimmingMode::PartialWithActive => {
            let mut overlays = partial_overlays(
                monitor,
                &spotlight,
                config.inactive_color,
                config.inactive_opacity,
            );
            overlays.push(fill(
                region::CENTER,
                &spotlight,
                config.active_color,
                config.active_opacity,
            ));
            overlays
        }
    }
}

/// The 4 edge overlays (Top, Bottom, Left, Right) around the spotlight rect,
/// which must already be clamped to `monitor` and non-empty. Degenerate
/// edges (a maximized window) simply produce no overlay for that side.
/// Port of `AppState.UpdatePartialOverlays()`.
fn partial_overlays(monitor: &Rect, spotlight: &Rect, color: Color, opacity: u8) -> Vec<OverlayDef> {
    let mut overlays = Vec::with_capacity(4);

    // Top overlay: full width, from display top to window top
    let top_height = spotlight.y - monitor.y;
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
    let bottom_y = spotlight.bottom();
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
    let left_width = spotlight.x - monitor.x;
    if left_width > 0 {
        overlays.push(OverlayDef {
            region: region::LEFT,
            visible: true,
            x: monitor.x,
            y: spotlight.y,
            width: left_width,
            height: spotlight.height,
            color,
            opacity,
        });
    }

    // Right overlay: window height, from window right to display right
    let right_x = spotlight.right();
    let right_width = monitor.right() - right_x;
    if right_width > 0 {
        overlays.push(OverlayDef {
            region: region::RIGHT,
            visible: true,
            x: right_x,
            y: spotlight.y,
            width: right_width,
            height: spotlight.height,
            color,
            opacity,
        });
    }

    overlays
}

/// Give each floating surface an overlay of its own.
///
/// The surface's rect is cut out of everything already computed and then
/// covered by exactly one overlay, so it reads uniformly. Both directions
/// need the cut: a transparent overlay on top cannot undo the dim beneath
/// it, and a second dim overlay on top would composite into a darker patch.
fn apply_floating(
    overlays: &mut Vec<OverlayDef>,
    monitor: &Rect,
    config: &OverlayConfig,
    floating: &[Rect],
) {
    let (color, opacity) = match config.always_on_top_handling {
        // Always-on-top windows are dimmed like any other window: nothing to
        // do, and no work spent on rects the adapters would not even report.
        AlwaysOnTopHandling::Ignore => return,
        AlwaysOnTopHandling::Highlight => (config.active_color, config.active_opacity),
        AlwaysOnTopHandling::Dim => (config.inactive_color, config.inactive_opacity),
    };

    for rect in floating {
        let hole = rect.clamp_to(monitor);
        if hole.width <= 0 || hole.height <= 0 {
            continue;
        }

        // Cut the hole out of every overlay computed so far. Earlier
        // floating rects are included, so when two always-on-top windows
        // overlap the one later in stacking order wins.
        let mut carved = Vec::with_capacity(overlays.len() + 3);
        for def in overlays.iter() {
            let def_rect = Rect::new(def.x, def.y, def.width, def.height);
            for piece in def_rect.subtract(&hole) {
                carved.push(OverlayDef {
                    x: piece.x,
                    y: piece.y,
                    width: piece.width,
                    height: piece.height,
                    ..def.clone()
                });
            }
        }
        carved.push(fill(region::FLOATING, &hole, color, opacity));
        *overlays = carved;
    }
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

    /// `calculate` with no floating surfaces, which is the default.
    fn calc(
        config: &OverlayConfig,
        monitor: &Rect,
        window: Option<&Rect>,
        focused: bool,
    ) -> Option<Vec<OverlayDef>> {
        calculate(config, monitor, window, focused, &[])
    }

    fn find(defs: &[OverlayDef], region: u8) -> Option<&OverlayDef> {
        defs.iter().find(|d| d.region == region)
    }

    fn rect_of(def: &OverlayDef) -> Rect {
        Rect::new(def.x, def.y, def.width, def.height)
    }

    /// Every mode must produce a total cover of the monitor: the overlays
    /// tile it exactly, with no gap and no double-covered pixel.
    fn assert_covers_monitor(defs: &[OverlayDef], monitor: &Rect) {
        let total: i64 = defs
            .iter()
            .map(|d| d.width as i64 * d.height as i64)
            .sum();
        assert_eq!(
            total,
            monitor.width as i64 * monitor.height as i64,
            "overlays do not tile the monitor: {defs:?}"
        );

        for (i, a) in defs.iter().enumerate() {
            for b in defs.iter().skip(i + 1) {
                assert_eq!(
                    rect_of(a).overlap_area(&rect_of(b)),
                    0,
                    "overlays overlap: {a:?} {b:?}"
                );
            }
        }
    }

    #[test]
    fn non_focused_monitor_gets_full_screen_overlay_in_every_mode() {
        for mode in [
            DimmingMode::FullScreen,
            DimmingMode::Partial,
            DimmingMode::PartialWithActive,
        ] {
            let defs = calc(&config(mode), &MONITOR, None, false).unwrap();
            assert_eq!(defs.len(), 1);
            let d = &defs[0];
            assert_eq!(d.region, region::FULLSCREEN);
            assert!(d.visible);
            assert_eq!(rect_of(d), MONITOR);
            assert_eq!(d.opacity, 153);
            assert_eq!(d.color, Color::BLACK);
        }
    }

    #[test]
    fn every_mode_covers_the_whole_monitor() {
        let window = Rect::new(500, 400, 800, 600);
        for mode in [
            DimmingMode::FullScreen,
            DimmingMode::Partial,
            DimmingMode::PartialWithActive,
            DimmingMode::Unknown,
        ] {
            for focused in [true, false] {
                let defs = calc(&config(mode), &MONITOR, Some(&window), focused).unwrap();
                assert_covers_monitor(&defs, &MONITOR);
            }
        }
    }

    #[test]
    fn fullscreen_mode_covers_the_focused_monitor_with_the_active_overlay() {
        let cfg = OverlayConfig {
            mode: DimmingMode::FullScreen,
            active_opacity: 42,
            ..OverlayConfig::default()
        };
        let window = Rect::new(100, 100, 800, 600);
        let defs = calc(&cfg, &MONITOR, Some(&window), true).unwrap();

        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].region, region::FULLSCREEN);
        assert_eq!(rect_of(&defs[0]), MONITOR);
        assert_eq!(defs[0].opacity, 42);
    }

    #[test]
    fn zero_size_window_freezes_state() {
        let window = Rect::new(100, 100, 0, 600);
        assert!(calc(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).is_none());

        let window = Rect::new(100, 100, 800, 0);
        assert!(calc(&config(DimmingMode::FullScreen), &MONITOR, Some(&window), true).is_none());
    }

    #[test]
    fn partial_mode_centered_window_produces_four_edges_plus_the_active_rect() {
        let window = Rect::new(500, 400, 800, 600);
        let defs = calc(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();
        assert_eq!(defs.len(), 5);

        let top = find(&defs, region::TOP).unwrap();
        assert_eq!(rect_of(top), Rect::new(0, 32, 2560, 368));

        let bottom = find(&defs, region::BOTTOM).unwrap();
        assert_eq!(rect_of(bottom), Rect::new(0, 1000, 2560, 440));

        let left = find(&defs, region::LEFT).unwrap();
        assert_eq!(rect_of(left), Rect::new(0, 400, 500, 600));

        let right = find(&defs, region::RIGHT).unwrap();
        assert_eq!(rect_of(right), Rect::new(1300, 400, 1260, 600));

        let center = find(&defs, region::CENTER).unwrap();
        assert_eq!(rect_of(center), window);
    }

    #[test]
    fn partial_and_partial_with_active_render_identically() {
        let window = Rect::new(500, 400, 800, 600);
        let partial = calc(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();
        let with_active = calc(
            &config(DimmingMode::PartialWithActive),
            &MONITOR,
            Some(&window),
            true,
        )
        .unwrap();
        assert_eq!(partial, with_active);
    }

    #[test]
    fn partial_mode_window_at_top_left_corner_omits_top_and_left() {
        let window = Rect::new(0, 32, 800, 600);
        let defs = calc(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();
        assert_eq!(defs.len(), 3);
        assert!(find(&defs, region::TOP).is_none());
        assert!(find(&defs, region::LEFT).is_none());
        assert!(find(&defs, region::BOTTOM).is_some());
        assert!(find(&defs, region::RIGHT).is_some());
        assert!(find(&defs, region::CENTER).is_some());
    }

    #[test]
    fn partial_mode_maximized_window_keeps_the_active_rect_only() {
        let defs = calc(
            &config(DimmingMode::Partial),
            &MONITOR,
            Some(&MONITOR),
            true,
        )
        .unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].region, region::CENTER);
        assert_eq!(rect_of(&defs[0]), MONITOR);
    }

    #[test]
    fn partial_mode_window_overflowing_monitor_is_clamped() {
        // Window extends past the right and bottom edges
        let window = Rect::new(2000, 1000, 1000, 1000);
        let defs = calc(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();

        // Top and left edges, plus the clamped active rect
        assert_eq!(defs.len(), 3);
        let top = find(&defs, region::TOP).unwrap();
        assert_eq!(rect_of(top), Rect::new(0, 32, 2560, 968));
        let left = find(&defs, region::LEFT).unwrap();
        assert_eq!(rect_of(left), Rect::new(0, 1000, 2000, 440));
        let center = find(&defs, region::CENTER).unwrap();
        assert_eq!(rect_of(center), Rect::new(2000, 1000, 560, 440));
    }

    #[test]
    fn window_entirely_off_monitor_leaves_the_monitor_inactive() {
        let window = Rect::new(5000, 5000, 800, 600);
        let defs = calc(&config(DimmingMode::Partial), &MONITOR, Some(&window), true).unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].region, region::FULLSCREEN);
        assert_eq!(defs[0].opacity, 153);
    }

    #[test]
    fn active_rect_uses_the_active_style() {
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
        let defs = calc(&cfg, &MONITOR, Some(&window), true).unwrap();
        assert_eq!(defs.len(), 5);

        let center = find(&defs, region::CENTER).unwrap();
        assert_eq!(rect_of(center), window);
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
    fn active_rect_is_clamped_to_monitor() {
        let window = Rect::new(-100, 0, 800, 600);
        let defs = calc(
            &config(DimmingMode::PartialWithActive),
            &MONITOR,
            Some(&window),
            true,
        )
        .unwrap();
        let center = find(&defs, region::CENTER).unwrap();
        assert_eq!(rect_of(center), Rect::new(0, 32, 700, 568));
    }

    #[test]
    fn unknown_mode_covers_the_focused_monitor_like_fullscreen() {
        let window = Rect::new(500, 400, 800, 600);
        let unknown = calc(&config(DimmingMode::Unknown), &MONITOR, Some(&window), true).unwrap();
        let full = calc(
            &config(DimmingMode::FullScreen),
            &MONITOR,
            Some(&window),
            true,
        )
        .unwrap();
        assert_eq!(unknown, full);
    }

    // --- floating surfaces ------------------------------------------------

    fn floating_config(handling: AlwaysOnTopHandling) -> OverlayConfig {
        OverlayConfig {
            mode: DimmingMode::Partial,
            always_on_top_handling: handling,
            active_opacity: 0,
            ..OverlayConfig::default()
        }
    }

    /// The reported case: a banner straddling the spotlight edge.
    const SPOTLIGHT: Rect = Rect::new(500, 400, 800, 600);
    const STRADDLING: Rect = Rect::new(300, 300, 400, 200);

    #[test]
    fn ignore_leaves_floating_surfaces_dimmed_like_any_window() {
        let cfg = floating_config(AlwaysOnTopHandling::Ignore);
        let with = calculate(&cfg, &MONITOR, Some(&SPOTLIGHT), true, &[STRADDLING]).unwrap();
        let without = calculate(&cfg, &MONITOR, Some(&SPOTLIGHT), true, &[]).unwrap();

        assert_eq!(with, without, "Ignore must be byte-identical to no floating");
        assert!(find(&with, region::FLOATING).is_none());
    }

    #[test]
    fn a_straddling_surface_gets_one_overlay_at_a_single_opacity() {
        for (handling, expected) in [
            (AlwaysOnTopHandling::Highlight, 0),
            (AlwaysOnTopHandling::Dim, 153),
        ] {
            let cfg = floating_config(handling);
            let defs = calculate(&cfg, &MONITOR, Some(&SPOTLIGHT), true, &[STRADDLING]).unwrap();

            let floating: Vec<&OverlayDef> = defs
                .iter()
                .filter(|d| d.region == region::FLOATING)
                .collect();
            assert_eq!(floating.len(), 1, "{handling:?}");
            assert_eq!(rect_of(floating[0]), STRADDLING, "{handling:?}");
            assert_eq!(floating[0].opacity, expected, "{handling:?}");

            // Nothing else may touch the surface's area, so it reads
            // uniformly rather than picking up the band beneath it.
            for def in defs.iter().filter(|d| d.region != region::FLOATING) {
                assert_eq!(
                    rect_of(def).overlap_area(&STRADDLING),
                    0,
                    "{handling:?}: {def:?} still covers part of the surface"
                );
            }

            assert_covers_monitor(&defs, &MONITOR);
        }
    }

    #[test]
    fn a_surface_inside_the_spotlight_only_carves_the_active_rect() {
        let cfg = floating_config(AlwaysOnTopHandling::Dim);
        let inside = Rect::new(600, 500, 100, 100);
        let defs = calculate(&cfg, &MONITOR, Some(&SPOTLIGHT), true, &[inside]).unwrap();

        assert_covers_monitor(&defs, &MONITOR);
        let floating = find(&defs, region::FLOATING).unwrap();
        assert_eq!(rect_of(floating), inside);
        assert_eq!(floating.opacity, 153);
    }

    #[test]
    fn a_surface_off_this_monitor_is_skipped_and_one_overlapping_is_clamped() {
        let cfg = floating_config(AlwaysOnTopHandling::Highlight);

        let elsewhere = Rect::new(4000, 4000, 300, 300);
        let defs = calculate(&cfg, &MONITOR, Some(&SPOTLIGHT), true, &[elsewhere]).unwrap();
        assert!(find(&defs, region::FLOATING).is_none());
        assert_covers_monitor(&defs, &MONITOR);

        let spanning = Rect::new(-100, 0, 400, 400);
        let defs = calculate(&cfg, &MONITOR, Some(&SPOTLIGHT), true, &[spanning]).unwrap();
        let floating = find(&defs, region::FLOATING).unwrap();
        assert_eq!(rect_of(floating), spanning.clamp_to(&MONITOR));
        assert_covers_monitor(&defs, &MONITOR);
    }

    #[test]
    fn overlapping_surfaces_resolve_to_the_topmost() {
        let cfg = floating_config(AlwaysOnTopHandling::Dim);
        let lower = Rect::new(300, 300, 400, 200);
        let upper = Rect::new(400, 350, 400, 200);

        // Stacking order, bottom first: the later rect wins the shared area.
        let defs = calculate(&cfg, &MONITOR, Some(&SPOTLIGHT), true, &[lower, upper]).unwrap();
        assert_covers_monitor(&defs, &MONITOR);

        let upper_def = defs
            .iter()
            .find(|d| d.region == region::FLOATING && rect_of(d) == upper)
            .expect("topmost surface keeps its full rect");
        assert_eq!(rect_of(upper_def), upper);
    }

    #[test]
    fn floating_surfaces_apply_on_unfocused_monitors_too() {
        let cfg = floating_config(AlwaysOnTopHandling::Highlight);
        let defs = calculate(&cfg, &MONITOR, None, false, &[STRADDLING]).unwrap();

        let floating = find(&defs, region::FLOATING).unwrap();
        assert_eq!(rect_of(floating), STRADDLING);
        assert_eq!(floating.opacity, 0);
        assert_covers_monitor(&defs, &MONITOR);
    }
}
