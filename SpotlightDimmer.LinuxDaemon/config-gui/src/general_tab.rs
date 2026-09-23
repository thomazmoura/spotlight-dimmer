//! Dimming mode, overlay colours and opacity, plus the live preview.
//!
//! Mirrors the Windows "General" tab minus its Renderer/Logging/Experimental
//! groups, which the Rust daemon ignores entirely. Stacked in one page it made
//! the window too tall, so it is split into Mode / Inactive / Active pages.
//! Each page carries its own preview: the point of the preview is to watch it
//! while dragging a slider, and a GTK widget can only have one parent.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk4 as gtk;

use spotlight_dimmer_core::config::{AlwaysOnTopHandling, ChromeHandling, DimmingMode, OverlayConfig};

use crate::document::Document;
use crate::preview::Preview;
use crate::widgets::{hex_from_rgba, hint, labelled_row, opacity_label, section, to_rgba};

const MODES: [&str; 3] = ["FullScreen", "Partial", "PartialWithActive"];

/// Order must match the index mapping in `always_on_top_index`/`connect`.
const ALWAYS_ON_TOP: [&str; 3] = ["Ignore", "Highlight", "Dim"];

fn always_on_top_index(handling: AlwaysOnTopHandling) -> u32 {
    match handling {
        AlwaysOnTopHandling::Ignore => 0,
        AlwaysOnTopHandling::Highlight => 1,
        AlwaysOnTopHandling::Dim => 2,
    }
}

pub struct GeneralTab {
    pages: [(&'static str, gtk::Widget); 3],
    mode_model: gtk::StringList,
    mode: gtk::DropDown,
    chrome_dim: gtk::Switch,
    always_on_top: gtk::DropDown,
    inactive_color: gtk::ColorDialogButton,
    inactive_opacity: gtk::Scale,
    inactive_value: gtk::Label,
    active_color: gtk::ColorDialogButton,
    active_opacity: gtk::Scale,
    active_value: gtk::Label,
    previews: Rc<Previews>,
    loading: Rc<Cell<bool>>,
}

impl GeneralTab {
    /// `mode_header` sits above the Mode page's own controls (the profile
    /// switcher, so the window's first tab opens on it).
    pub fn new(
        document: &Document,
        loading: Rc<Cell<bool>>,
        mode_header: &impl IsA<gtk::Widget>,
    ) -> GeneralTab {
        let previews = Rc::new(Previews(std::array::from_fn(|_| Preview::new())));

        // --- Mode -----------------------------------------------------------
        let mode_model = gtk::StringList::new(&MODES);
        let mode = gtk::DropDown::builder()
            .model(&mode_model)
            .hexpand(true)
            .build();

        // Switch rather than a dropdown: the underlying ChromeHandling is
        // binary, and "off" is the default the user should land on.
        let chrome_dim = gtk::Switch::builder().halign(gtk::Align::Start).build();

        let (mode_frame, mode_box) = section("Dimming");
        mode_box.append(&labelled_row("Mode:", &mode));
        mode_box.append(&hint(
            "FullScreen dims whole unfocused monitors. Partial also dims the focused \
             monitor around the active window. PartialWithActive additionally tints the \
             active window itself.",
        ));
        let always_on_top = gtk::DropDown::builder()
            .model(&gtk::StringList::new(&ALWAYS_ON_TOP))
            .hexpand(true)
            .build();

        mode_box.append(&labelled_row("Dim shell chrome:", &chrome_dim));
        mode_box.append(&hint(
            "Off (the default) keeps notifications, popups, the panel and the dock fully \
             lit, because the dimming stacks below them. On dims them along with windows, \
             which is how every release before this one behaved \u{2014} note that a \
             notification overlapping the edge of the spotlight then looks lit on one side \
             and dimmed on the other.",
        ));
        mode_box.append(&labelled_row("Always-on-top windows:", &always_on_top));
        mode_box.append(&hint(
            "Dim (the default) covers windows you pinned always-on-top with the inactive \
             overlay; Highlight keeps them lit as part of the spotlight. Either one stops \
             such a window coming out lit on the part over the active window and dimmed \
             on the rest, and the focused window always keeps its spotlight. Ignore dims \
             them like any other window.",
        ));
        let mode_page = page(Some(mode_header.upcast_ref()), &mode_frame, &previews.0[0]);

        // --- Inactive overlay -----------------------------------------------
        let (inactive_color, inactive_opacity, inactive_value, inactive_frame) = overlay_section(
            "Inactive overlay (dimmed areas)",
            "Applied to unfocused monitors, and to the area around the active window in \
             the Partial modes.",
        );
        let inactive_page = page(None, &inactive_frame, &previews.0[1]);

        // --- Active overlay -------------------------------------------------
        let (active_color, active_opacity, active_value, active_frame) = overlay_section(
            "Active overlay (focused window)",
            "Used only in PartialWithActive mode. The controls stay editable in the other \
             modes so the values can be set up before switching.",
        );
        let active_page = page(None, &active_frame, &previews.0[2]);

        let tab = GeneralTab {
            pages: [
                ("Mode", mode_page),
                ("Inactive", inactive_page),
                ("Active", active_page),
            ],
            mode_model,
            mode,
            chrome_dim,
            always_on_top,
            inactive_color,
            inactive_opacity,
            inactive_value,
            active_color,
            active_opacity,
            active_value,
            previews,
            loading,
        };

        tab.connect(document);
        tab
    }

    /// Notebook pages as (tab label, content), in display order.
    pub fn pages(&self) -> &[(&'static str, gtk::Widget)] {
        &self.pages
    }

    fn connect(&self, document: &Document) {
        // Closures capture the document, the preview and the reentrancy flag
        // — never the tab itself, which would create a reference cycle with
        // the widgets the tab owns.
        let bind_color = |button: &gtk::ColorDialogButton, set: fn(&Document, &str)| {
            let document = document.clone();
            let loading = self.loading.clone();
            let previews = self.previews.clone();
            button.connect_rgba_notify(move |button| {
                if loading.get() {
                    return;
                }
                set(&document, &hex_from_rgba(button.rgba()));
                previews.update(document.config().overlay);
            });
        };

        let bind_opacity =
            |scale: &gtk::Scale, value_label: &gtk::Label, set: fn(&Document, u8)| {
                let document = document.clone();
                let loading = self.loading.clone();
                let previews = self.previews.clone();
                let value_label = value_label.clone();
                scale.connect_value_changed(move |scale| {
                    let value = scale.value().round().clamp(0.0, 255.0) as u8;
                    value_label.set_text(&opacity_label(value));
                    if loading.get() {
                        return;
                    }
                    set(&document, value);
                    previews.update(document.config().overlay);
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

        let document_for_chrome = document.clone();
        let loading = self.loading.clone();
        self.chrome_dim.connect_active_notify(move |switch| {
            if loading.get() {
                return;
            }
            document_for_chrome.set_chrome_handling(if switch.is_active() {
                "Dim"
            } else {
                "Highlight"
            });
        });

        let document_for_aot = document.clone();
        let loading = self.loading.clone();
        self.always_on_top.connect_selected_notify(move |dropdown| {
            if loading.get() {
                return;
            }
            let Some(value) = ALWAYS_ON_TOP.get(dropdown.selected() as usize) else {
                return;
            };
            document_for_aot.set_always_on_top_handling(value);
        });

        let document_for_mode = document.clone();
        let loading = self.loading.clone();
        let previews = self.previews.clone();
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
            previews.update(document_for_mode.config().overlay);
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
        self.chrome_dim
            .set_active(overlay.chrome_handling == ChromeHandling::Dim);
        self.always_on_top
            .set_selected(always_on_top_index(overlay.always_on_top_handling));
        self.inactive_color.set_rgba(&to_rgba(overlay.inactive_color));
        self.active_color.set_rgba(&to_rgba(overlay.active_color));
        self.inactive_opacity.set_value(overlay.inactive_opacity as f64);
        self.active_opacity.set_value(overlay.active_opacity as f64);
        self.inactive_value
            .set_text(&opacity_label(overlay.inactive_opacity));
        self.active_value
            .set_text(&opacity_label(overlay.active_opacity));

        self.loading.set(was_loading);
        self.previews.update(overlay);
    }
}

/// One preview per page, kept in step.
struct Previews([Preview; 3]);

impl Previews {
    fn update(&self, config: OverlayConfig) {
        for preview in &self.0 {
            preview.update(config.clone());
        }
    }
}

/// A page's settings frame with the preview beneath it, optionally below a
/// header widget.
fn page(header: Option<&gtk::Widget>, settings: &gtk::Frame, preview: &Preview) -> gtk::Widget {
    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let (preview_frame, preview_box) = section("Preview");
    preview_box.append(preview.widget());

    if let Some(header) = header {
        root.append(header);
    }
    root.append(settings);
    root.append(&preview_frame);
    root.upcast()
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
