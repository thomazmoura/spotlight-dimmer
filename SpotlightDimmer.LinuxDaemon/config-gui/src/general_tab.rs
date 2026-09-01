//! Dimming mode, overlay colours and opacity, plus the live preview.
//!
//! Mirrors the Windows "General" tab minus its Renderer/Logging/Experimental
//! groups, which the Rust daemon ignores entirely.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk4 as gtk;

use spotlight_dimmer_core::config::DimmingMode;

use crate::document::Document;
use crate::preview::Preview;
use crate::widgets::{hex_from_rgba, hint, labelled_row, opacity_label, section, to_rgba};

const MODES: [&str; 3] = ["FullScreen", "Partial", "PartialWithActive"];

pub struct GeneralTab {
    root: gtk::Widget,
    mode_model: gtk::StringList,
    mode: gtk::DropDown,
    inactive_color: gtk::ColorDialogButton,
    inactive_opacity: gtk::Scale,
    inactive_value: gtk::Label,
    active_color: gtk::ColorDialogButton,
    active_opacity: gtk::Scale,
    active_value: gtk::Label,
    preview: Rc<Preview>,
    loading: Rc<Cell<bool>>,
}

impl GeneralTab {
    pub fn new(document: &Document, loading: Rc<Cell<bool>>) -> GeneralTab {
        let preview = Rc::new(Preview::new());

        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // --- Dimming mode ---------------------------------------------------
        let mode_model = gtk::StringList::new(&MODES);
        let mode = gtk::DropDown::builder()
            .model(&mode_model)
            .hexpand(true)
            .build();

        let (mode_frame, mode_box) = section("Dimming");
        mode_box.append(&labelled_row("Mode:", &mode));
        mode_box.append(&hint(
            "FullScreen dims whole unfocused monitors. Partial also dims the focused \
             monitor around the active window. PartialWithActive additionally tints the \
             active window itself.",
        ));
        root.append(&mode_frame);

        // --- Inactive overlay -----------------------------------------------
        let (inactive_color, inactive_opacity, inactive_value, inactive_frame) = overlay_section(
            "Inactive overlay (dimmed areas)",
            "Applied to unfocused monitors, and to the area around the active window in \
             the Partial modes.",
        );
        root.append(&inactive_frame);

        // --- Active overlay -------------------------------------------------
        let (active_color, active_opacity, active_value, active_frame) = overlay_section(
            "Active overlay (focused window)",
            "Used only in PartialWithActive mode. The controls stay editable in the other \
             modes so the values can be set up before switching.",
        );
        root.append(&active_frame);

        // --- Preview --------------------------------------------------------
        let (preview_frame, preview_box) = section("Preview");
        preview_box.append(preview.widget());
        root.append(&preview_frame);

        let tab = GeneralTab {
            root: root.upcast(),
            mode_model,
            mode,
            inactive_color,
            inactive_opacity,
            inactive_value,
            active_color,
            active_opacity,
            active_value,
            preview,
            loading,
        };

        tab.connect(document);
        tab
    }

    pub fn widget(&self) -> &gtk::Widget {
        &self.root
    }

    fn connect(&self, document: &Document) {
        // Closures capture the document, the preview and the reentrancy flag
        // — never the tab itself, which would create a reference cycle with
        // the widgets the tab owns.
        let bind_color = |button: &gtk::ColorDialogButton, set: fn(&Document, &str)| {
            let document = document.clone();
            let loading = self.loading.clone();
            let preview = self.preview.clone();
            button.connect_rgba_notify(move |button| {
                if loading.get() {
                    return;
                }
                set(&document, &hex_from_rgba(button.rgba()));
                preview.update(document.config().overlay);
            });
        };

        let bind_opacity =
            |scale: &gtk::Scale, value_label: &gtk::Label, set: fn(&Document, u8)| {
                let document = document.clone();
                let loading = self.loading.clone();
                let preview = self.preview.clone();
                let value_label = value_label.clone();
                scale.connect_value_changed(move |scale| {
                    let value = scale.value().round().clamp(0.0, 255.0) as u8;
                    value_label.set_text(&opacity_label(value));
                    if loading.get() {
                        return;
                    }
                    set(&document, value);
                    preview.update(document.config().overlay);
                });
            };

        bind_color(&self.inactive_color, |d, hex| d.set_inactive_color(hex));
        bind_color(&self.active_color, |d, hex| d.set_active_color(hex));
        bind_opacity(&self.inactive_opacity, &self.inactive_value, |d, v| {
            d.set_inactive_opacity(v)
        });
        bind_opacity(&self.active_opacity, &self.active_value, |d, v| {
            d.set_active_opacity(v)
        });

        let document_for_mode = document.clone();
        let loading = self.loading.clone();
        let preview = self.preview.clone();
        let model = self.mode_model.clone();
        self.mode.connect_selected_notify(move |dropdown| {
            if loading.get() {
                return;
            }
            let selected = dropdown.selected();
            // The extra "Unknown" entry, when present, sits past the three
            // real modes; selecting it must not rewrite the user's value.
            let Some(mode) = MODES.get(selected as usize) else {
                return;
            };
            document_for_mode.set_mode(mode);
            drop_unknown_entry(&model, dropdown);
            preview.update(document_for_mode.config().overlay);
        });
    }

    /// Repopulate every control from the document. Guarded by the loading
    /// flag so the programmatic updates below do not write back.
    pub fn reload(&self, document: &Document) {
        let was_loading = self.loading.replace(true);

        let overlay = document.config().overlay;

        sync_mode(
            &self.mode_model,
            &self.mode,
            overlay.mode,
            document.overlay_raw_string("Mode"),
        );
        self.inactive_color.set_rgba(&to_rgba(overlay.inactive_color));
        self.active_color.set_rgba(&to_rgba(overlay.active_color));
        self.inactive_opacity.set_value(overlay.inactive_opacity as f64);
        self.active_opacity.set_value(overlay.active_opacity as f64);
        self.inactive_value
            .set_text(&opacity_label(overlay.inactive_opacity));
        self.active_value
            .set_text(&opacity_label(overlay.active_opacity));

        self.loading.set(was_loading);
        self.preview.update(overlay);
    }
}

/// Colour button + 0-255 opacity slider + value readout.
fn overlay_section(
    title: &str,
    description: &str,
) -> (gtk::ColorDialogButton, gtk::Scale, gtk::Label, gtk::Frame) {
    let dialog = gtk::ColorDialog::builder().with_alpha(false).build();
    let color = gtk::ColorDialogButton::builder().dialog(&dialog).build();

    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 255.0, 1.0);
    scale.set_digits(0);
    scale.set_draw_value(false);
    scale.set_hexpand(true);
    // Marks at each 20% step: fewer than the Windows tick frequency, but
    // they line up with the percentage shown in the value readout.
    for step in 0..=5 {
        scale.add_mark(step as f64 * 51.0, gtk::PositionType::Bottom, None);
    }

    let value = gtk::Label::builder()
        .label(opacity_label(0))
        .xalign(0.0)
        .width_chars(10)
        .build();

    let opacity_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .build();
    opacity_row.append(&scale);
    opacity_row.append(&value);

    let (frame, content) = section(title);
    content.append(&labelled_row("Colour:", &color));
    content.append(&labelled_row("Opacity:", &opacity_row));
    content.append(&hint(description));

    (color, scale, value, frame)
}

/// Show an unrecognised `Mode` string as a fourth entry rather than silently
/// coercing the file to a known value. `DimmingMode::Unknown` is a real
/// daemon behaviour (dim unfocused monitors only), so it is worth naming.
fn sync_mode(
    model: &gtk::StringList,
    dropdown: &gtk::DropDown,
    mode: DimmingMode,
    raw: Option<String>,
) {
    drop_unknown_entry(model, dropdown);

    match mode {
        DimmingMode::FullScreen => dropdown.set_selected(0),
        DimmingMode::Partial => dropdown.set_selected(1),
        DimmingMode::PartialWithActive => dropdown.set_selected(2),
        DimmingMode::Unknown => {
            let raw = raw.unwrap_or_default();
            model.append(&format!("Unknown ({raw}) — dims unfocused monitors only"));
            dropdown.set_selected(MODES.len() as u32);
        }
    }
}

fn drop_unknown_entry(model: &gtk::StringList, dropdown: &gtk::DropDown) {
    while model.n_items() > MODES.len() as u32 {
        // Removing the selected item would reset the selection, so move off
        // it first.
        if dropdown.selected() >= MODES.len() as u32 {
            dropdown.set_selected(0);
        }
        model.remove(MODES.len() as u32);
    }
}
