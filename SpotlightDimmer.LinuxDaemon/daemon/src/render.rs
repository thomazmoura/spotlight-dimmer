//! Layer-shell overlay renderer for compositors that support the
//! wlr-layer-shell protocol (KDE Plasma / KWin, wlroots compositors).
//!
//! One full-output GTK window per monitor on the Overlay layer: overlays are
//! flat rectangles, so a single surface painting 0-6 rects per output is
//! simpler and cheaper for the compositor than six layer surfaces, and
//! updates atomically. GNOME does not support layer-shell — there the GNOME
//! extension registers as the renderer and this module stays idle.
//!
//! Click-through: the input region is set empty on every map (the surface
//! can be recreated), and keyboard interactivity is None — the equivalent of
//! the Windows client's WS_EX_TRANSPARENT and the extension's reactive:false.
//!
//! Windows with no visible overlays are unmapped rather than painted empty,
//! which hands direct scanout back to fullscreen apps on that output.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gdk;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use spotlight_dimmer_core::calculator::OverlayDef;
use spotlight_dimmer_core::primitives::Rect;
use spotlight_dimmer_core::state::{Monitor, OverlaysPayload};

enum InitState {
    Uninitialized,
    Ready,
    Unavailable,
}

struct OverlayWindow {
    window: gtk::Window,
    /// Overlays in window-local coordinates, read by the draw func.
    overlays: Rc<RefCell<Vec<OverlayDef>>>,
    area: gtk::DrawingArea,
    /// Monitor geometry this window was created for; a change forces
    /// recreation so the layer surface re-anchors on the right output.
    geometry: Rect,
}

pub struct LayerShellRenderer {
    init_state: InitState,
    windows: HashMap<String, OverlayWindow>,
}

impl LayerShellRenderer {
    pub fn new() -> LayerShellRenderer {
        LayerShellRenderer {
            init_state: InitState::Uninitialized,
            windows: HashMap::new(),
        }
    }

    /// Apply a computed payload. `monitors` is the adapter-reported monitor
    /// list (origins for global->local translation and GdkMonitor mapping).
    pub fn apply(&mut self, payload: &OverlaysPayload, monitors: &[Monitor]) {
        if !self.ensure_init() {
            return;
        }

        self.sync_windows(monitors);

        for monitor_overlays in &payload.monitors {
            let Some(ow) = self.windows.get(&monitor_overlays.key) else {
                continue;
            };

            let origin = (ow.geometry.x, ow.geometry.y);
            let local: Vec<OverlayDef> = monitor_overlays
                .overlays
                .iter()
                .filter(|d| d.visible)
                .map(|d| OverlayDef {
                    x: d.x - origin.0,
                    y: d.y - origin.1,
                    ..d.clone()
                })
                .collect();

            let any_visible = !local.is_empty();
            *ow.overlays.borrow_mut() = local;

            if any_visible {
                ow.window.set_visible(true);
                ow.area.queue_draw();
            } else {
                ow.window.set_visible(false);
            }
        }
    }

    /// Unmap all overlay windows (renderer adapter took over, all adapters
    /// gone, or dimming paused).
    pub fn hide_all(&mut self) {
        for ow in self.windows.values() {
            ow.window.set_visible(false);
        }
    }

    /// Lazy GTK init: the daemon may run on GNOME (where the extension
    /// renders) or be started outside a Wayland session; only pay for GTK
    /// and fail gracefully when this renderer is actually needed.
    fn ensure_init(&mut self) -> bool {
        match self.init_state {
            InitState::Ready => true,
            InitState::Unavailable => false,
            InitState::Uninitialized => {
                if gtk::init().is_err() {
                    eprintln!(
                        "SpotlightDimmer: GTK init failed (no display?); daemon-side rendering disabled"
                    );
                    self.init_state = InitState::Unavailable;
                    return false;
                }

                if !gtk4_layer_shell::is_supported() {
                    eprintln!(
                        "SpotlightDimmer: layer-shell not supported by this compositor (GNOME?); daemon-side rendering disabled"
                    );
                    self.init_state = InitState::Unavailable;
                    return false;
                }

                // Overlay windows must not paint the theme background
                if let Some(display) = gdk::Display::default() {
                    let provider = gtk::CssProvider::new();
                    provider.load_from_string(
                        "window.spotlight-dimmer-overlay { background: transparent; }",
                    );
                    gtk::style_context_add_provider_for_display(
                        &display,
                        &provider,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                }

                println!("SpotlightDimmer: layer-shell renderer initialized");
                self.init_state = InitState::Ready;
                true
            }
        }
    }

    /// Reconcile the window set with the reported monitors: drop windows for
    /// gone monitors, recreate on geometry change, create missing ones.
    fn sync_windows(&mut self, monitors: &[Monitor]) {
        let alive: HashSet<&str> = monitors.iter().map(|m| m.key.as_str()).collect();
        self.windows.retain(|key, ow| {
            let keep = alive.contains(key.as_str());
            if !keep {
                ow.window.destroy();
            }
            keep
        });

        for monitor in monitors {
            let stale = self
                .windows
                .get(&monitor.key)
                .is_some_and(|ow| ow.geometry != monitor.geometry);
            if stale {
                if let Some(ow) = self.windows.remove(&monitor.key) {
                    ow.window.destroy();
                }
            }

            if !self.windows.contains_key(&monitor.key) {
                self.windows
                    .insert(monitor.key.clone(), create_window(monitor));
            }
        }
    }
}

fn create_window(monitor: &Monitor) -> OverlayWindow {
    let window = gtk::Window::new();
    window.add_css_class("spotlight-dimmer-overlay");

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        window.set_anchor(edge, true);
    }
    // Ignore other surfaces' exclusive zones: cover panels/docks too, exactly
    // like the fullscreen overlay does on the other platforms
    window.set_exclusive_zone(-1);
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_namespace(Some("spotlight-dimmer"));

    if let Some(gdk_monitor) = find_gdk_monitor(monitor) {
        window.set_monitor(Some(&gdk_monitor));
    } else {
        eprintln!(
            "SpotlightDimmer: no GdkMonitor match for '{}'; compositor will pick an output",
            monitor.key
        );
    }

    let overlays: Rc<RefCell<Vec<OverlayDef>>> = Rc::new(RefCell::new(Vec::new()));

    let area = gtk::DrawingArea::new();
    let draw_overlays = overlays.clone();
    area.set_draw_func(move |_, cr, _width, _height| {
        for def in draw_overlays.borrow().iter() {
            cr.set_source_rgba(
                def.color.r as f64 / 255.0,
                def.color.g as f64 / 255.0,
                def.color.b as f64 / 255.0,
                def.opacity as f64 / 255.0,
            );
            cr.rectangle(
                def.x as f64,
                def.y as f64,
                def.width as f64,
                def.height as f64,
            );
            let _ = cr.fill();
        }
    });
    window.set_child(Some(&area));

    // Click-through: empty input region, reapplied on every map because the
    // Wayland surface is recreated when the window is unmapped/remapped
    window.connect_map(|w| {
        if let Some(surface) = w.surface() {
            surface.set_input_region(Some(&gtk::cairo::Region::create()));
        }
    });

    OverlayWindow {
        window,
        overlays,
        area,
        geometry: monitor.geometry,
    }
}

/// Map an adapter-reported monitor to a GdkMonitor: by connector name first
/// (KWin keys are connector names like "DP-1"), then by geometry.
fn find_gdk_monitor(monitor: &Monitor) -> Option<gdk::Monitor> {
    let display = gdk::Display::default()?;
    let monitors = display.monitors();

    let mut geometry_match = None;
    for i in 0..monitors.n_items() {
        let gdk_monitor = monitors.item(i)?.downcast::<gdk::Monitor>().ok()?;

        if gdk_monitor
            .connector()
            .is_some_and(|c| c == monitor.key.as_str())
        {
            return Some(gdk_monitor);
        }

        let g = gdk_monitor.geometry();
        if g.x() == monitor.geometry.x
            && g.y() == monitor.geometry.y
            && g.width() == monitor.geometry.width
            && g.height() == monitor.geometry.height
        {
            geometry_match = Some(gdk_monitor);
        }
    }

    geometry_match
}
