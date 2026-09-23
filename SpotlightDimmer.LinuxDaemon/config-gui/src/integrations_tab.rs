//! Terminal pane spotlight (`AppIntegrations`).
//!
//! Integrations are not plugins: each one is a code path in the daemon, so
//! the tab lists the fixed set it implements as checkboxes rather than
//! asking the user to type a `WM_CLASS` and pick a tty source. Checking one
//! writes its `AppIntegrations` entry; unchecking removes it.
//!
//! Entries for anything else (a hand-configured terminal, the Windows
//! client's `ProcessName` entries) are not shown and are never touched, so
//! the JSON stays the escape hatch for setups the catalog does not cover.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk4 as gtk;
use serde_json::json;

use crate::document::Document;
use crate::widgets::{hint, labelled_row, section};

/// A terminal the daemon's tmux provider knows how to query.
struct KnownIntegration {
    name: &'static str,
    description: &'static str,
    wm_class: &'static str,
    /// `TtySource` value written into the entry.
    tty_source: &'static str,
    /// Offsets written when the integration is first enabled: the
    /// terminal's out-of-the-box padding.
    default_offset: (i32, i32),
}

const KNOWN_INTEGRATIONS: [KnownIntegration; 2] = [
    KnownIntegration {
        name: "WezTerm",
        description: "Spotlight the focused tmux pane (and neovim split) inside WezTerm. \
                      The focused pane is looked up with `wezterm cli`.",
        wm_class: "org.wezfurlong.wezterm",
        tty_source: "wezterm",
        default_offset: (0, 0),
    },
    KnownIntegration {
        name: "Ghostty",
        description: "Spotlight the focused tmux pane (and neovim split) inside Ghostty. \
                      Ghostty has no pane-query CLI, so tmux publishes the pane in the \
                      window title: also uncomment the Ghostty block in \
                      spotlight-dimmer.tmux.conf.",
        wm_class: "com.mitchellh.ghostty",
        tty_source: "title",
        default_offset: (2, 2),
    },
];

struct IntegrationRow {
    integration: &'static KnownIntegration,
    enabled: gtk::CheckButton,
    /// Holds the offset controls, greyed out while the box is unchecked.
    settings: gtk::Box,
    offset_x: gtk::SpinButton,
    offset_y: gtk::SpinButton,
}

pub struct IntegrationsTab {
    root: gtk::Widget,
    rows: Vec<IntegrationRow>,
    document: Document,
    loading: Rc<Cell<bool>>,
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
            "Terminal pane spotlight — make the spotlight follow the focused tmux pane \
             instead of the whole terminal window. Requires the tmux hooks from \
             docs/TMUX_INTEGRATION.md.",
        ));

        let rows: Vec<IntegrationRow> = KNOWN_INTEGRATIONS
            .iter()
            .map(|integration| {
                let row = build_row(integration);
                root.append(&row.0);
                row.1
            })
            .collect();

        root.append(&hint(
            "Offsets: pixels from the window's client area edge to the terminal cell grid \
             (padding for X, padding plus tab bar height for Y). Window decorations are \
             reported separately by the compositor adapter and must not be added here.",
        ));

        let tab = Rc::new(IntegrationsTab {
            root: root.upcast(),
            rows,
            document: document.clone(),
            loading,
        });

        tab.connect();
        tab.reload(document);
        tab
    }

    pub fn widget(&self) -> &gtk::Widget {
        &self.root
    }

    fn connect(self: &Rc<Self>) {
        // Weak handles throughout: the tab owns these widgets, so a strong
        // capture would keep the whole tab alive forever.
        for (index, row) in self.rows.iter().enumerate() {
            let weak = Rc::downgrade(self);
            row.enabled.connect_toggled(move |check| {
                let Some(tab) = weak.upgrade() else { return };
                let row = &tab.rows[index];
                row.settings.set_sensitive(check.is_active());
                if tab.loading.get() {
                    return;
                }

                let integration = row.integration;
                if check.is_active() {
                    // The spin buttons hold the defaults, or the values from
                    // before it was unchecked, so a quick uncheck/re-check
                    // does not lose hand-tuned offsets.
                    let offset = (spin_value(&row.offset_x), spin_value(&row.offset_y));
                    tab.document.enable_integration(
                        integration.wm_class,
                        integration.tty_source,
                        offset,
                    );
                } else {
                    tab.document.disable_integration(integration.wm_class);
                }
            });

            for (spin, key) in [
                (&row.offset_x, "ContentOffsetX"),
                (&row.offset_y, "ContentOffsetY"),
            ] {
                let weak = Rc::downgrade(self);
                spin.connect_value_changed(move |spin| {
                    let Some(tab) = weak.upgrade() else { return };
                    if tab.loading.get() {
                        return;
                    }
                    tab.document.set_integration_field(
                        tab.rows[index].integration.wm_class,
                        key,
                        json!(spin_value(spin)),
                    );
                });
            }
        }
    }

    /// Repopulate from the document. Called from both loading and
    /// non-loading contexts, so the flag is saved and restored rather than
    /// cleared.
    pub fn reload(&self, _document: &Document) {
        let was_loading = self.loading.replace(true);

        for row in &self.rows {
            let entry = self.document.find_integration(row.integration.wm_class);
            let (x, y) = entry
                .as_ref()
                .map(|e| (e.content_offset_x, e.content_offset_y))
                .unwrap_or(row.integration.default_offset);

            row.enabled.set_active(entry.is_some());
            row.settings.set_sensitive(entry.is_some());
            // Hand-edited values outside the spin range are clamped here and
            // written back on the next change to that offset.
            row.offset_x.set_value(x.clamp(0, 1000) as f64);
            row.offset_y.set_value(y.clamp(0, 1000) as f64);
        }

        self.loading.set(was_loading);
    }
}

fn build_row(integration: &'static KnownIntegration) -> (gtk::Frame, IntegrationRow) {
    let (frame, content) = section(integration.name);

    let enabled = gtk::CheckButton::with_label("Enabled");
    enabled.set_tooltip_text(Some(&format!(
        "Matches windows whose WM_CLASS is {}",
        integration.wm_class
    )));
    content.append(&enabled);
    content.append(&hint(integration.description));

    let offset_x = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);
    let offset_y = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);

    let settings = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .sensitive(false)
        .build();
    settings.append(&labelled_row("Content offset X:", &offset_x));
    settings.append(&labelled_row("Content offset Y:", &offset_y));
    content.append(&settings);

    let row = IntegrationRow {
        integration,
        enabled,
        settings,
        offset_x,
        offset_y,
    };
    (frame, row)
}

fn spin_value(spin: &gtk::SpinButton) -> i32 {
    spin.value().round() as i32
}
