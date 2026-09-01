//! Live preview of the current overlay settings.
//!
//! Two mock monitors side by side — a focused one holding an active window
//! and an unfocused one — because that is the only framing under which all
//! three modes say something. A single-monitor preview would be blank in
//! FullScreen mode, which is the default.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::cairo::{Context, FillRule, FontSlant, FontWeight};
use gtk::prelude::*;
use gtk4 as gtk;

use spotlight_dimmer_core::config::{DimmingMode, OverlayConfig};
use spotlight_dimmer_core::primitives::Color;

const WIDTH: i32 = 300;
const HEIGHT: i32 = 118;

pub struct Preview {
    area: gtk::DrawingArea,
    config: Rc<RefCell<OverlayConfig>>,
}

impl Preview {
    pub fn new() -> Preview {
        let config = Rc::new(RefCell::new(OverlayConfig::default()));
        let area = gtk::DrawingArea::builder()
            .content_width(WIDTH)
            .content_height(HEIGHT)
            .halign(gtk::Align::Center)
            .tooltip_text("How the current settings look on a focused and an unfocused monitor")
            .build();

        let for_draw = config.clone();
        area.set_draw_func(move |area, cr, width, height| {
            // The mock monitors are deliberately fixed colours (they stand in
            // for a real desktop), but the captions belong to the window, so
            // they take the widget's own foreground colour and follow the
            // system light/dark theme.
            draw(
                cr,
                width as f64,
                height as f64,
                &for_draw.borrow(),
                area.color(),
            );
        });

        Preview { area, config }
    }

    pub fn widget(&self) -> &gtk::DrawingArea {
        &self.area
    }

    pub fn update(&self, config: OverlayConfig) {
        *self.config.borrow_mut() = config;
        self.area.queue_draw();
    }
}

fn draw(
    cr: &Context,
    width: f64,
    height: f64,
    config: &OverlayConfig,
    caption: gtk::gdk::RGBA,
) {
    let gap = 14.0;
    let label_space = 16.0;
    let monitor_width = (width - gap) / 2.0;
    let monitor_height = height - label_space;

    draw_monitor(cr, 0.0, 0.0, monitor_width, monitor_height, config, true);
    draw_monitor(
        cr,
        monitor_width + gap,
        0.0,
        monitor_width,
        monitor_height,
        config,
        false,
    );

    cr.set_source_rgba(
        caption.red() as f64,
        caption.green() as f64,
        caption.blue() as f64,
        caption.alpha() as f64 * 0.75,
    );
    cr.select_font_face("sans", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(10.0);
    centered_text(cr, "Focused monitor", monitor_width / 2.0, height - 3.0);
    centered_text(
        cr,
        "Unfocused monitor",
        monitor_width + gap + monitor_width / 2.0,
        height - 3.0,
    );
}

fn centered_text(cr: &Context, text: &str, center_x: f64, y: f64) {
    let Ok(extents) = cr.text_extents(text) else {
        return;
    };
    cr.move_to(center_x - extents.width() / 2.0, y);
    let _ = cr.show_text(text);
}

fn draw_monitor(
    cr: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    config: &OverlayConfig,
    focused: bool,
) {
    draw_desktop(cr, x, y, width, height);

    // The active window, inset inside the focused monitor.
    let window = (
        x + width * 0.18,
        y + height * 0.22,
        width * 0.64,
        height * 0.56,
    );

    if focused {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(window.0, window.1, window.2, window.3);
        let _ = cr.fill();
        cr.set_source_rgb(0.31, 0.44, 0.72);
        cr.rectangle(window.0, window.1, window.2, 6.0);
        let _ = cr.fill();
    }

    let inactive = rgba(config.inactive_color, config.inactive_opacity);
    let active = rgba(config.active_color, config.active_opacity);

    if !focused {
        // Every mode dims monitors that do not hold the focused window.
        set_rgba(cr, inactive);
        cr.rectangle(x, y, width, height);
        let _ = cr.fill();
    } else {
        match config.mode {
            // Unknown behaves like FullScreen on the focused monitor: no
            // overlays there, other monitors still dimmed.
            DimmingMode::FullScreen | DimmingMode::Unknown => {}
            DimmingMode::Partial | DimmingMode::PartialWithActive => {
                // Everything except the active window, as one even-odd
                // filled path (the daemon draws it as four separate rects).
                set_rgba(cr, inactive);
                cr.set_fill_rule(FillRule::EvenOdd);
                cr.rectangle(x, y, width, height);
                cr.rectangle(window.0, window.1, window.2, window.3);
                let _ = cr.fill();
                cr.set_fill_rule(FillRule::Winding);

                if config.mode == DimmingMode::PartialWithActive {
                    set_rgba(cr, active);
                    cr.rectangle(window.0, window.1, window.2, window.3);
                    let _ = cr.fill();
                }
            }
        }
    }

    cr.set_source_rgb(0.55, 0.55, 0.58);
    cr.set_line_width(1.0);
    cr.rectangle(x + 0.5, y + 0.5, width - 1.0, height - 1.0);
    let _ = cr.stroke();
}

/// A neutral stand-in desktop, with enough contrast that the difference
/// between 40% and 60% dimming is actually visible.
fn draw_desktop(cr: &Context, x: f64, y: f64, width: f64, height: f64) {
    cr.set_source_rgb(0.85, 0.88, 0.93);
    cr.rectangle(x, y, width, height);
    let _ = cr.fill();

    cr.set_source_rgb(0.62, 0.70, 0.84);
    cr.rectangle(x, y + height * 0.62, width, height * 0.38);
    let _ = cr.fill();

    cr.set_source_rgb(0.97, 0.97, 0.99);
    cr.rectangle(
        x + width * 0.06,
        y + height * 0.10,
        width * 0.30,
        height * 0.34,
    );
    let _ = cr.fill();

    cr.set_source_rgb(0.20, 0.22, 0.26);
    cr.rectangle(
        x + width * 0.58,
        y + height * 0.50,
        width * 0.34,
        height * 0.34,
    );
    let _ = cr.fill();
}

fn rgba(color: Color, opacity: u8) -> (f64, f64, f64, f64) {
    (
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        opacity as f64 / 255.0,
    )
}

fn set_rgba(cr: &Context, (r, g, b, a): (f64, f64, f64, f64)) {
    cr.set_source_rgba(r, g, b, a);
}
