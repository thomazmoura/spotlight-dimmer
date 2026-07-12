//! Central application state: monitors, focus, resolved pane rect, enabled
//! flag. Produces the overlay payload consumed by renderers (the daemon's
//! layer-shell renderer or the GNOME extension over D-Bus).

use serde::{Deserialize, Serialize};

use crate::calculator::{self, OverlayDef};
use crate::config::AppConfig;
use crate::primitives::Rect;

/// A monitor as reported by a compositor adapter.
/// All rects are logical global compositor coordinates.
/// Serde: serialized to the daemon's runtime monitor cache so a restarted
/// daemon keeps working before the adapter re-sends monitors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monitor {
    /// Adapter-defined identity: Mutter monitor index as a string on GNOME
    /// ("0"), output connector name on KWin ("DP-1").
    pub key: String,
    /// Full monitor geometry; used to determine the focused monitor and as
    /// the layer-shell surface origin.
    pub geometry: Rect,
    /// Geometry minus panels/docks. Informational: the calculator dims the
    /// full geometry so panels get covered too (matching the Windows client),
    /// but adapters still report the work area as part of the D-Bus contract.
    pub work_area: Rect,
    pub scale: f64,
}

/// The focused window as reported by a compositor adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct Focus {
    pub wm_class: String,
    pub title: String,
    pub frame: Rect,
    /// Client-area rect (decorations excluded); `None` when the adapter
    /// doesn't report one (protocol v1). Used as the base origin for inner
    /// pane resolution so window decorations don't skew the highlight.
    pub client: Option<Rect>,
}

/// Overlays for one monitor, addressed by the adapter's monitor key.
#[derive(Debug, Clone, Serialize)]
pub struct MonitorOverlays {
    pub key: String,
    pub overlays: Vec<OverlayDef>,
}

/// The versioned payload sent to renderers, serialized as JSON.
#[derive(Debug, Clone, Serialize)]
pub struct OverlaysPayload {
    pub serial: u64,
    pub enabled: bool,
    pub monitors: Vec<MonitorOverlays>,
}

/// Central state manager. Mutate the public fields, then call [`recompute`]
/// to obtain a fresh payload (or `None` when the current state must not be
/// applied, preserving the previous overlays).
///
/// [`recompute`]: AppState::recompute
#[derive(Debug, Default)]
pub struct AppState {
    pub config: AppConfig,
    pub monitors: Vec<Monitor>,
    pub focus: Option<Focus>,
    /// Resolved inner spotlight rect (e.g. the focused tmux pane) in screen
    /// space; substitutes the window frame when present.
    pub inner_rect: Option<Rect>,
    enabled: bool,
    serial: u64,
}

impl AppState {
    pub fn new(config: AppConfig) -> AppState {
        AppState {
            config,
            monitors: Vec::new(),
            focus: None,
            inner_rect: None,
            enabled: true,
            serial: 0,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Toggle the enabled flag, returning the new value.
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    /// Index of the monitor with the largest overlap with the focused window
    /// frame (matching the Windows client's display resolution logic).
    /// Ties resolve to the first monitor; `None` when there is no focus, no
    /// monitors, or the window overlaps no monitor at all.
    pub fn focused_monitor_index(&self) -> Option<usize> {
        let focus = self.focus.as_ref()?;

        let mut best: Option<(usize, i64)> = None;
        for (i, monitor) in self.monitors.iter().enumerate() {
            let area = focus.frame.overlap_area(&monitor.geometry);
            if area > 0 && best.is_none_or(|(_, best_area)| area > best_area) {
                best = Some((i, area));
            }
        }

        best.map(|(i, _)| i)
    }

    /// Recalculate overlays for all monitors.
    ///
    /// Returns `None` to signal "keep the previous overlays": the focused
    /// window reported transient 0x0 dimensions (the calculator's
    /// anti-flicker freeze). Otherwise returns a payload with a fresh serial.
    pub fn recompute(&mut self) -> Option<OverlaysPayload> {
        if !self.enabled {
            // Paused: every monitor gets an explicit empty overlay list so
            // renderers hide everything.
            let monitors = self
                .monitors
                .iter()
                .map(|m| MonitorOverlays {
                    key: m.key.clone(),
                    overlays: Vec::new(),
                })
                .collect();

            self.serial += 1;
            return Some(OverlaysPayload {
                serial: self.serial,
                enabled: false,
                monitors,
            });
        }

        // The spotlight target: resolved inner pane rect when available,
        // otherwise the window frame (extension.js:_updateAllOverlays).
        let window_rect: Option<Rect> = self
            .focus
            .as_ref()
            .map(|f| self.inner_rect.unwrap_or(f.frame));

        // Anti-flicker freeze (calculator.js:52-54): a transient 0x0 window
        // must not be applied. Checked here as well as in the calculator
        // because a 0x0 frame has zero overlap with every monitor, which
        // would otherwise resolve to "no focused monitor" and skip the
        // calculator's own check.
        if let Some(rect) = &window_rect {
            if rect.width == 0 || rect.height == 0 {
                return None;
            }
        }

        let focused_index = self.focused_monitor_index();

        let mut monitors = Vec::with_capacity(self.monitors.len());
        for (i, monitor) in self.monitors.iter().enumerate() {
            let is_focused = focused_index == Some(i);
            let overlays = calculator::calculate(
                &self.config.overlay,
                &monitor.geometry,
                if is_focused {
                    window_rect.as_ref()
                } else {
                    None
                },
                is_focused,
            )?; // 0x0 window: freeze everything until the next geometry event

            monitors.push(MonitorOverlays {
                key: monitor.key.clone(),
                overlays,
            });
        }

        self.serial += 1;
        Some(OverlaysPayload {
            serial: self.serial,
            enabled: true,
            monitors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculator::region;
    use crate::config::DimmingMode;

    fn two_monitor_state() -> AppState {
        let mut state = AppState::new(AppConfig::default());
        state.monitors = vec![
            Monitor {
                key: "0".into(),
                geometry: Rect::new(0, 0, 1920, 1080),
                work_area: Rect::new(0, 32, 1920, 1048),
                scale: 1.0,
            },
            Monitor {
                key: "1".into(),
                geometry: Rect::new(1920, 0, 2560, 1440),
                work_area: Rect::new(1920, 0, 2560, 1440),
                scale: 1.0,
            },
        ];
        state
    }

    fn focus(frame: Rect) -> Option<Focus> {
        Some(Focus {
            wm_class: "test".into(),
            title: "Test".into(),
            frame,
            client: None,
        })
    }

    #[test]
    fn focused_monitor_is_resolved_by_max_overlap() {
        let mut state = two_monitor_state();

        state.focus = focus(Rect::new(100, 100, 800, 600));
        assert_eq!(state.focused_monitor_index(), Some(0));

        // Straddling the boundary, mostly on monitor 1
        state.focus = focus(Rect::new(1800, 100, 800, 600));
        assert_eq!(state.focused_monitor_index(), Some(1));

        // No overlap with any monitor
        state.focus = focus(Rect::new(10000, 10000, 100, 100));
        assert_eq!(state.focused_monitor_index(), None);

        state.focus = None;
        assert_eq!(state.focused_monitor_index(), None);
    }

    #[test]
    fn fullscreen_mode_dims_only_unfocused_monitors() {
        let mut state = two_monitor_state();
        state.focus = focus(Rect::new(100, 100, 800, 600));

        let payload = state.recompute().unwrap();
        assert!(payload.enabled);
        assert_eq!(payload.serial, 1);
        assert_eq!(payload.monitors.len(), 2);

        // Focused monitor 0: no overlays
        assert_eq!(payload.monitors[0].key, "0");
        assert!(payload.monitors[0].overlays.is_empty());

        // Monitor 1: overlay covering the full monitor geometry
        let overlays = &payload.monitors[1].overlays;
        assert_eq!(overlays.len(), 1);
        assert_eq!(overlays[0].region, region::FULLSCREEN);
        assert_eq!(
            (
                overlays[0].x,
                overlays[0].y,
                overlays[0].width,
                overlays[0].height
            ),
            (1920, 0, 2560, 1440)
        );
    }

    #[test]
    fn no_focus_dims_all_monitors() {
        let mut state = two_monitor_state();
        let payload = state.recompute().unwrap();
        for monitor in &payload.monitors {
            assert_eq!(monitor.overlays.len(), 1);
            assert_eq!(monitor.overlays[0].region, region::FULLSCREEN);
        }
    }

    #[test]
    fn inner_rect_substitutes_window_frame() {
        let mut state = two_monitor_state();
        state.config.overlay.mode = DimmingMode::Partial;
        state.focus = focus(Rect::new(100, 132, 800, 600));
        state.inner_rect = Some(Rect::new(300, 232, 400, 300));

        let payload = state.recompute().unwrap();
        let overlays = &payload.monitors[0].overlays;

        // Edge overlays surround the pane, not the window frame
        let top = overlays.iter().find(|d| d.region == region::TOP).unwrap();
        assert_eq!(top.height, 232); // from geometry top (0) to pane top (232)
    }

    #[test]
    fn panel_area_outside_work_area_is_dimmed() {
        // Monitor "0" has a 32px top panel (geometry starts at y=0, work area
        // at y=32). A window maximized to the work area must still produce a
        // Top overlay covering the panel strip.
        let mut state = two_monitor_state();
        state.config.overlay.mode = DimmingMode::Partial;
        state.focus = focus(Rect::new(0, 32, 1920, 1048));

        let payload = state.recompute().unwrap();
        let overlays = &payload.monitors[0].overlays;
        assert_eq!(overlays.len(), 1);
        let top = &overlays[0];
        assert_eq!(top.region, region::TOP);
        assert_eq!((top.x, top.y, top.width, top.height), (0, 0, 1920, 32));
    }

    #[test]
    fn zero_size_focused_window_freezes_and_keeps_serial() {
        let mut state = two_monitor_state();
        state.focus = focus(Rect::new(100, 100, 800, 600));
        assert_eq!(state.recompute().unwrap().serial, 1);

        state.focus = focus(Rect::new(100, 100, 0, 0));
        assert!(state.recompute().is_none());

        // Serial did not advance during the freeze
        state.focus = focus(Rect::new(100, 100, 800, 600));
        assert_eq!(state.recompute().unwrap().serial, 2);
    }

    #[test]
    fn disabled_state_emits_empty_overlays_for_all_monitors() {
        let mut state = two_monitor_state();
        state.focus = focus(Rect::new(100, 100, 800, 600));
        assert!(!state.toggle());

        let payload = state.recompute().unwrap();
        assert!(!payload.enabled);
        assert_eq!(payload.monitors.len(), 2);
        assert!(payload.monitors.iter().all(|m| m.overlays.is_empty()));

        assert!(state.toggle());
        assert!(state.recompute().unwrap().enabled);
    }

    #[test]
    fn payload_serializes_with_expected_field_names() {
        let mut state = two_monitor_state();
        state.focus = focus(Rect::new(100, 100, 800, 600));

        let payload = state.recompute().unwrap();
        let json = serde_json::to_string(&payload).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["serial"], 1);
        assert_eq!(value["enabled"], true);
        assert_eq!(value["monitors"][1]["key"], "1");

        let overlay = &value["monitors"][1]["overlays"][0];
        assert_eq!(overlay["region"], 0);
        assert_eq!(overlay["visible"], true);
        assert_eq!(overlay["x"], 1920);
        assert_eq!(overlay["width"], 2560);
        assert_eq!(overlay["color"]["r"], 0);
        assert_eq!(overlay["opacity"], 153);
    }
}
