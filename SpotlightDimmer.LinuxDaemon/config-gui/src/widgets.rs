//! Small layout helpers.
//!
//! Plain GTK4 is used rather than libadwaita: libadwaita ignores the system
//! GTK theme, so an Adwaita-styled window would look foreign on the KDE
//! Plasma half of the supported desktops. The only thing lost is
//! libadwaita's preference-row polish, which `labelled_row` replaces.

use gtk::prelude::*;
use gtk4 as gtk;

use spotlight_dimmer_core::primitives::Color;

/// A titled frame wrapping a vertical box of rows.
pub fn section(title: &str) -> (gtk::Frame, gtk::Box) {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();

    let frame = gtk::Frame::builder().label(title).child(&content).build();

    (frame, content)
}

/// A left-aligned label of fixed width followed by its control, so controls
/// line up down a section without needing a shared size group per tab.
pub fn labelled_row(label: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .build();

    let label = gtk::Label::builder()
        .label(label)
        .xalign(0.0)
        .width_chars(18)
        .max_width_chars(18)
        .build();

    row.append(&label);
    row.append(control);
    row
}

/// Secondary explanatory text.
pub fn hint(text: &str) -> gtk::Label {
    let label = gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::WordChar)
        .build();
    label.add_css_class("dim-label");
    label
}

pub fn to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

pub fn to_rgba(color: Color) -> gtk::gdk::RGBA {
    gtk::gdk::RGBA::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        1.0,
    )
}

/// `#RRGGBB` from a picker value; alpha is carried by the separate opacity
/// slider, so it is deliberately dropped here.
pub fn hex_from_rgba(rgba: gtk::gdk::RGBA) -> String {
    let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    to_hex(Color {
        r: channel(rgba.red()),
        g: channel(rgba.green()),
        b: channel(rgba.blue()),
    })
}

/// Opacity is stored 0-255 but users think in percent, so show both.
pub fn opacity_label(value: u8) -> String {
    let percent = (value as f64 * 100.0 / 255.0).round() as u32;
    format!("{value} ({percent}%)")
}
