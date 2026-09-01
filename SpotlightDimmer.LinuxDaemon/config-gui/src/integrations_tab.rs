//! Terminal pane spotlight (`AppIntegrations`).
//!
//! Mirrors the Windows "Integrations" tab, with `WmClass` in place of
//! `ProcessName` plus the Linux-only `TtySource`.
//!
//! The list is built from the raw JSON array rather than from the parsed
//! `AppConfig`: the parser drops entries with an empty `WmClass`, so a row
//! that was just added and not yet filled in would otherwise disappear the
//! instant it was created.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk4 as gtk;
use serde_json::json;

use spotlight_dimmer_core::config::TtySource;

use crate::document::Document;
use crate::widgets::{hint, labelled_row, section};

const PROVIDERS: [&str; 1] = ["tmux"];
const TTY_SOURCES: [&str; 2] = ["WezTerm CLI (wezterm)", "Window title (title)"];

const WM_CLASS_TOOLTIP: &str = "The window's WM_CLASS, matched case-sensitively.\n\
    KDE:   qdbus6 org.kde.KWin /KWin org.kde.KWin.queryWindowInfo\n\
    GNOME: Looking Glass (Alt+F2, 'lg') → Windows → wm_class\n\
    Known values: org.wezfurlong.wezterm, com.mitchellh.ghostty";

pub struct IntegrationsTab {
    root: gtk::Widget,
    list: gtk::ListBox,
    remove_button: gtk::Button,
    detail: gtk::Box,
    wm_class: gtk::Entry,
    provider: gtk::DropDown,
    tty_source: gtk::DropDown,
    offset_x: gtk::SpinButton,
    offset_y: gtk::SpinButton,
    document: Document,
    loading: Rc<Cell<bool>>,
    /// Index into the raw `AppIntegrations` array, or None with no selection.
    selected: Cell<Option<usize>>,
}

impl IntegrationsTab {
    pub fn new(document: &Document, loading: Rc<Cell<bool>>) -> Rc<IntegrationsTab> {
        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        root.append(&hint(
            "Terminal pane spotlight — match a terminal by its WM_CLASS so the spotlight \
             follows the focused tmux pane instead of the whole window. Requires the tmux \
             hooks from docs/TMUX_INTEGRATION.md.",
        ));

        // --- list + buttons ---------------------------------------------------
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_height(120)
            .vexpand(true)
            .build();
        scroller.add_css_class("frame");

        let add_button = gtk::Button::with_label("Add");
        let remove_button = gtk::Button::with_label("Remove");
        remove_button.set_sensitive(false);

        let buttons = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk::Align::End)
            .build();
        buttons.append(&add_button);
        buttons.append(&remove_button);

        let list_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .build();
        list_box.append(&scroller);
        list_box.append(&buttons);
        root.append(&list_box);

        // --- detail pane ------------------------------------------------------
        let wm_class = gtk::Entry::builder()
            .hexpand(true)
            .placeholder_text("org.wezfurlong.wezterm")
            .tooltip_text(WM_CLASS_TOOLTIP)
            .build();

        let provider = gtk::DropDown::from_strings(&PROVIDERS);
        provider.set_hexpand(true);

        let tty_source = gtk::DropDown::from_strings(&TTY_SOURCES);
        tty_source.set_hexpand(true);

        let offset_x = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);
        let offset_y = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);

        let (detail_frame, detail) = section("Integration details");
        detail.append(&labelled_row("WM_CLASS:", &wm_class));
        detail.append(&labelled_row("Provider:", &provider));
        detail.append(&labelled_row("Tty source:", &tty_source));
        detail.append(&labelled_row("Content offset X:", &offset_x));
        detail.append(&labelled_row("Content offset Y:", &offset_y));
        detail.append(&hint(
            "Tty source: how the focused pane's tty is discovered. WezTerm CLI queries \
             `wezterm cli`; Window title reads it from the title tmux publishes via \
             set-titles-string (use this for terminals without a pane-query CLI, such as \
             Ghostty).",
        ));
        detail.append(&hint(
            "Offsets: pixels from the window's client area edge to the terminal cell grid \
             (padding for X, padding plus tab bar height for Y). Window decorations are \
             reported separately by the compositor adapter and must not be added here.",
        ));
        detail.set_sensitive(false);
        root.append(&detail_frame);

        let tab = Rc::new(IntegrationsTab {
            root: root.upcast(),
            list,
            remove_button,
            detail,
            wm_class,
            provider,
            tty_source,
            offset_x,
            offset_y,
            document: document.clone(),
            loading,
            selected: Cell::new(None),
        });

        tab.connect(&add_button);
        tab.reload(document);
        tab
    }

    pub fn widget(&self) -> &gtk::Widget {
        &self.root
    }

    fn connect(self: &Rc<Self>, add_button: &gtk::Button) {
        // Weak handles throughout: the tab owns these widgets, so a strong
        // capture would keep the whole tab alive forever.
        let weak = Rc::downgrade(self);
        self.list.connect_row_selected(move |_, row| {
            if let Some(tab) = weak.upgrade() {
                tab.selected.set(row.map(|row| row.index() as usize));
                tab.populate_detail();
            }
        });

        let weak = Rc::downgrade(self);
        add_button.connect_clicked(move |_| {
            if let Some(tab) = weak.upgrade() {
                let index = tab.document.add_integration();
                tab.refresh_list(Some(index));
            }
        });

        let weak = Rc::downgrade(self);
        self.remove_button.connect_clicked(move |_| {
            let Some(tab) = weak.upgrade() else { return };
            let Some(index) = tab.selected.get() else {
                return;
            };
            tab.document.remove_integration(index);
            let remaining = tab.document.integrations().len();
            let next = if remaining == 0 {
                None
            } else {
                Some(index.min(remaining - 1))
            };
            tab.refresh_list(next);
        });

        // The entry commits on focus-out and on Enter, skipping the write
        // when the text is unchanged — an unconditional write would relabel
        // the list row (and reselect it) on every focus change.
        let weak = Rc::downgrade(self);
        self.wm_class.connect_activate(move |_| {
            if let Some(tab) = weak.upgrade() {
                tab.commit_wm_class();
            }
        });

        let focus = gtk::EventControllerFocus::new();
        let weak = Rc::downgrade(self);
        focus.connect_leave(move |_| {
            if let Some(tab) = weak.upgrade() {
                tab.commit_wm_class();
            }
        });
        self.wm_class.add_controller(focus);

        let weak = Rc::downgrade(self);
        self.provider.connect_selected_notify(move |dropdown| {
            let Some(tab) = weak.upgrade() else { return };
            let Some(provider) = PROVIDERS.get(dropdown.selected() as usize) else {
                return;
            };
            tab.set_field("Provider", json!(provider));
            // The provider is part of each row's label.
            tab.refresh_list(tab.selected.get());
        });

        let weak = Rc::downgrade(self);
        self.tty_source.connect_selected_notify(move |dropdown| {
            let Some(tab) = weak.upgrade() else { return };
            // The parser treats anything that is not "title" as the WezTerm
            // CLI; "wezterm" is the readable spelling of that default.
            let value = if dropdown.selected() == 1 {
                "title"
            } else {
                "wezterm"
            };
            tab.set_field("TtySource", json!(value));
        });

        for (spin, key) in [
            (&self.offset_x, "ContentOffsetX"),
            (&self.offset_y, "ContentOffsetY"),
        ] {
            let weak = Rc::downgrade(self);
            spin.connect_value_changed(move |spin| {
                if let Some(tab) = weak.upgrade() {
                    tab.set_field(key, json!(spin.value().round() as i32));
                }
            });
        }
    }

    /// Repopulate from the document, preserving the selected row.
    pub fn reload(&self, _document: &Document) {
        self.refresh_list(self.selected.get());
    }

    fn refresh_list(&self, select: Option<usize>) {
        let was_loading = self.loading.replace(true);

        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }

        let integrations = self.document.integrations();
        for integration in &integrations {
            let name = if integration.wm_class.is_empty() {
                "(new entry)"
            } else {
                &integration.wm_class
            };
            let label = gtk::Label::builder()
                .label(format!("{name}  ({})", integration.provider))
                .xalign(0.0)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(8)
                .margin_end(8)
                .build();
            self.list.append(&label);
        }

        // Default to the first entry rather than leaving the detail pane
        // disabled: with entries present there is always something worth
        // showing, and an empty greyed-out pane reads as broken.
        let target = select
            .filter(|index| *index < integrations.len())
            .or(if integrations.is_empty() {
                None
            } else {
                Some(0)
            });
        self.selected.set(target);
        if let Some(index) = target {
            if let Some(row) = self.list.row_at_index(index as i32) {
                self.list.select_row(Some(&row));
            }
        }

        self.loading.set(was_loading);
        self.populate_detail();
    }

    /// Repopulated from both loading and non-loading contexts (selection
    /// changes), so the flag is saved and restored rather than cleared —
    /// the same pattern the Windows form uses in PopulateIntegrationDetails.
    fn populate_detail(&self) {
        let was_loading = self.loading.replace(true);

        let integrations = self.document.integrations();
        let current = self
            .selected
            .get()
            .and_then(|index| integrations.get(index).cloned());

        self.detail.set_sensitive(current.is_some());
        self.remove_button.set_sensitive(current.is_some());

        match current {
            Some(integration) => {
                self.wm_class.set_text(&integration.wm_class);
                let provider = PROVIDERS
                    .iter()
                    .position(|p| *p == integration.provider)
                    .unwrap_or(0);
                self.provider.set_selected(provider as u32);
                self.tty_source.set_selected(match integration.tty_source {
                    TtySource::WindowTitle => 1,
                    TtySource::WezTermCli => 0,
                });
                // Hand-edited values outside the spin range are clamped here
                // and written back on the next change to that entry.
                self.offset_x
                    .set_value(integration.content_offset_x.clamp(0, 1000) as f64);
                self.offset_y
                    .set_value(integration.content_offset_y.clamp(0, 1000) as f64);
            }
            None => {
                self.wm_class.set_text("");
                self.provider.set_selected(0);
                self.tty_source.set_selected(0);
                self.offset_x.set_value(0.0);
                self.offset_y.set_value(0.0);
            }
        }

        self.loading.set(was_loading);
    }

    fn commit_wm_class(&self) {
        if self.loading.get() {
            return;
        }
        let Some(index) = self.selected.get() else {
            return;
        };

        let text = self.wm_class.text().to_string();
        let integrations = self.document.integrations();
        if integrations.get(index).is_some_and(|i| i.wm_class == text) {
            return;
        }

        self.document
            .set_integration_field(index, "WmClass", json!(text));
        self.refresh_list(Some(index));
    }

    fn set_field(&self, key: &str, value: serde_json::Value) {
        if self.loading.get() {
            return;
        }
        let Some(index) = self.selected.get() else {
            return;
        };
        self.document.set_integration_field(index, key, value);
    }
}
