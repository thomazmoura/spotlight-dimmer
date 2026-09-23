//! Profile quick-switcher: the first thing the settings window focuses.
//!
//! Built for the keyboard path "Super+Alt+Shift+D, type a few letters,
//! Enter": the entry fuzzy-filters the `Profiles` list below it, Up/Down move
//! the selection without leaving the entry, Enter applies the selected
//! profile and Ctrl+Enter applies it and closes the window. The list is
//! inline rather than a popover so typing never fights a Wayland popup grab.
//!
//! Profiles use the Windows client's `Profiles` / `CurrentProfile` keys, so
//! one config.json switches the same presets on both platforms.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gtk::prelude::*;
use gtk4 as gtk;

use crate::document::{Document, ProfileView};
use crate::fuzzy;
use crate::widgets::{hint, opacity_label, section};

pub struct ProfilesSection {
    frame: gtk::Frame,
    state: Rc<State>,
}

/// Everything the signal handlers need. Handlers hold it weakly: the widgets
/// below own those handlers, so a strong reference would be a cycle.
struct State {
    document: Document,
    entry: gtk::SearchEntry,
    list: gtk::ListBox,
    scroller: gtk::ScrolledWindow,
    status: gtk::Label,
    save: gtk::Button,
    delete: gtk::Button,
    /// Profile names in the order the list currently shows them.
    shown: RefCell<Vec<String>>,
}

impl ProfilesSection {
    pub fn new(document: &Document) -> ProfilesSection {
        let entry = gtk::SearchEntry::builder()
            .placeholder_text("Type to switch profile…")
            .hexpand(true)
            .build();

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .build();
        list.add_css_class("boxed-list");
        // Keeps the Mode tab from growing without bound with many profiles.
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(170)
            .child(&list)
            .build();

        let status = gtk::Label::builder().xalign(0.0).hexpand(true).build();
        status.add_css_class("dim-label");

        let save = gtk::Button::with_label("Save");
        let delete = gtk::Button::with_label("Delete");
        let buttons = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .build();
        buttons.append(&status);
        buttons.append(&save);
        buttons.append(&delete);

        let (frame, content) = section("Profile");
        content.append(&entry);
        content.append(&scroller);
        content.append(&buttons);
        content.append(&hint(
            "Enter applies the highlighted profile, Ctrl+Enter applies it and closes the \
             window. Save stores the current settings under the typed name.",
        ));

        let state = Rc::new(State {
            document: document.clone(),
            entry,
            list,
            scroller,
            status,
            save,
            delete,
            shown: RefCell::new(Vec::new()),
        });
        connect(&state);
        state.refresh();

        ProfilesSection { frame, state }
    }

    pub fn widget(&self) -> &gtk::Frame {
        &self.frame
    }

    /// Put the cursor in the entry, ready for typing.
    pub fn focus(&self) {
        self.state.entry.grab_focus();
    }

    /// Rebuild from the document (external edit, or a profile applied).
    pub fn reload(&self) {
        self.state.refresh();
    }

    /// Refresh only the "(modified)" status; cheap enough for every edit.
    pub fn update_status(&self) {
        self.state.update_status();
    }
}

fn connect(state: &Rc<State>) {
    let weak = Rc::downgrade(state);
    state.entry.connect_changed(move |_| {
        with(&weak, |s| s.refresh());
    });

    let weak = Rc::downgrade(state);
    state.entry.connect_stop_search(move |entry| {
        // Escape: clear the filter; a second Escape has nothing to clear.
        entry.set_text("");
        with(&weak, |s| s.refresh());
    });

    // Capture phase so these keys are handled before the entry's own
    // bindings (Up/Down would otherwise move focus out of the entry).
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = Rc::downgrade(state);
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        let Some(state) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        match key {
            gtk::gdk::Key::Down => state.move_selection(1),
            gtk::gdk::Key::Up => state.move_selection(-1),
            gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter | gtk::gdk::Key::ISO_Enter => {
                let close = modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK);
                state.apply_selected(close);
            }
            _ => return glib::Propagation::Proceed,
        }
        glib::Propagation::Stop
    });
    state.entry.add_controller(keys);

    let weak = Rc::downgrade(state);
    state.list.connect_row_activated(move |_, row| {
        with(&weak, |s| s.apply_index(row.index(), false));
    });

    let weak = Rc::downgrade(state);
    state.list.connect_selected_rows_changed(move |_| {
        with(&weak, |s| s.update_buttons());
    });

    let weak = Rc::downgrade(state);
    state.save.connect_clicked(move |_| {
        with(&weak, |s| s.save());
    });

    let weak = Rc::downgrade(state);
    state.delete.connect_clicked(move |_| {
        with(&weak, |s| s.confirm_delete());
    });
}

fn with(weak: &Weak<State>, f: impl FnOnce(&Rc<State>)) {
    if let Some(state) = weak.upgrade() {
        f(&state);
    }
}

impl State {
    fn typed(&self) -> String {
        self.entry.text().trim().to_string()
    }

    /// Re-filter the list against the entry text; the best match is
    /// selected so Enter always has a target.
    fn refresh(&self) {
        let profiles = self.document.profiles();
        let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        let order = fuzzy::rank(&self.typed(), &names);
        let current = self.document.current_profile();

        self.list.remove_all();
        for &index in &order {
            let profile = &profiles[index];
            self.list.append(&profile_row(
                profile,
                current.as_deref() == Some(&profile.name),
            ));
        }
        *self.shown.borrow_mut() = order.iter().map(|&i| profiles[i].name.clone()).collect();

        if profiles.is_empty() {
            self.list.set_placeholder(Some(&placeholder(
                "No profiles yet. Type a name and press Save to store the current settings.",
            )));
        } else {
            self.list
                .set_placeholder(Some(&placeholder("No profile matches.")));
        }

        if let Some(first) = self.list.row_at_index(0) {
            self.list.select_row(Some(&first));
        }
        self.update_status();
        self.update_buttons();
    }

    fn update_status(&self) {
        let text = match self.document.current_profile() {
            Some(name) if self.document.overlay_matches_profile(&name) => {
                format!("Current: {name}")
            }
            Some(name) => format!("Current: {name} (modified)"),
            None => "No profile applied".to_string(),
        };
        self.status.set_text(&text);
    }

    fn update_buttons(&self) {
        let typed = self.typed();
        let exists = |name: &str| self.document.profiles().iter().any(|p| p.name == name);
        let (label, sensitive) = if !typed.is_empty() {
            if exists(&typed) {
                (format!("Update “{typed}”"), true)
            } else {
                (format!("Save as “{typed}”"), true)
            }
        } else if let Some(current) = self.document.current_profile().filter(|c| exists(c)) {
            (format!("Update “{current}”"), true)
        } else {
            ("Save".to_string(), false)
        };
        self.save.set_label(&label);
        self.save.set_sensitive(sensitive);
        self.save.set_tooltip_text(Some(if sensitive {
            "Store the current mode, colours and opacities under this name"
        } else {
            "Type a name first"
        }));

        self.delete.set_sensitive(self.selected_name().is_some());
    }

    fn selected_name(&self) -> Option<String> {
        let row = self.list.selected_row()?;
        self.shown.borrow().get(row.index() as usize).cloned()
    }

    fn move_selection(&self, delta: i32) {
        let count = self.shown.borrow().len() as i32;
        if count == 0 {
            return;
        }
        let current = self.list.selected_row().map_or(0, |r| r.index());
        let next = (current + delta).clamp(0, count - 1);
        if let Some(row) = self.list.row_at_index(next) {
            self.list.select_row(Some(&row));
            self.scroll_to(&row);
        }
    }

    /// Rows never take focus (it stays in the entry), so GTK will not
    /// scroll them into view on its own.
    fn scroll_to(&self, row: &gtk::ListBoxRow) {
        let Some(bounds) = row.compute_bounds(&self.list) else {
            return;
        };
        let adjustment = self.scroller.vadjustment();
        let (top, bottom) = (bounds.y() as f64, (bounds.y() + bounds.height()) as f64);
        if top < adjustment.value() {
            adjustment.set_value(top);
        } else if bottom > adjustment.value() + adjustment.page_size() {
            adjustment.set_value(bottom - adjustment.page_size());
        }
    }

    fn apply_selected(&self, close: bool) {
        if let Some(row) = self.list.selected_row() {
            self.apply_index(row.index(), close);
        }
    }

    fn apply_index(&self, index: i32, close: bool) {
        let Some(name) = self.shown.borrow().get(index as usize).cloned() else {
            return;
        };
        if !self.document.apply_profile(&name) {
            return;
        }
        // Clear first so the refresh triggered below shows the full list.
        self.entry.set_text("");
        self.document.notify_changed();

        if close {
            if let Some(window) = self.window() {
                window.close();
            }
        } else {
            self.entry.grab_focus();
        }
    }

    fn save(&self) {
        let typed = self.typed();
        let name = if typed.is_empty() {
            match self.document.current_profile() {
                Some(current) => current,
                None => return,
            }
        } else {
            typed
        };
        self.document.save_profile(&name);
        self.entry.set_text("");
        self.document.notify_changed();
    }

    fn confirm_delete(self: &Rc<Self>) {
        let Some(name) = self.selected_name() else {
            return;
        };
        let dialog = gtk::AlertDialog::builder()
            .modal(true)
            .message(format!("Delete the profile “{name}”?"))
            .detail("The current dimming settings are kept; only the saved preset is removed.")
            .buttons(["Cancel", "Delete"])
            .cancel_button(0)
            .default_button(0)
            .build();

        let weak = Rc::downgrade(self);
        dialog.choose(
            self.window().as_ref(),
            gio::Cancellable::NONE,
            move |response| {
                if !matches!(response, Ok(1)) {
                    return;
                }
                with(&weak, |s| {
                    s.document.delete_profile(&name);
                    s.document.notify_changed();
                    s.entry.grab_focus();
                });
            },
        );
    }

    fn window(&self) -> Option<gtk::Window> {
        self.entry.root().and_downcast::<gtk::Window>()
    }
}

/// Name on the left, a short summary of what the profile does on the right.
fn profile_row(profile: &ProfileView, current: bool) -> gtk::ListBoxRow {
    let name = gtk::Label::builder()
        .label(&profile.name)
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    if current {
        name.add_css_class("heading");
    }

    let summary = gtk::Label::builder()
        .label(format!(
            "{} · {} / {}",
            profile.mode,
            opacity_label(profile.inactive_opacity),
            opacity_label(profile.active_opacity)
        ))
        .xalign(1.0)
        .build();
    summary.add_css_class("dim-label");

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(10)
        .margin_end(10)
        .build();
    content.append(&name);
    content.append(&summary);

    gtk::ListBoxRow::builder()
        .child(&content)
        // Focus stays in the entry; rows are reached with Up/Down or a click.
        .focusable(false)
        .build()
}

fn placeholder(text: &str) -> gtk::Label {
    let label = gtk::Label::builder()
        .label(text)
        .wrap(true)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    label.add_css_class("dim-label");
    label
}
