//! The settings window: header bar, tabs, the unparseable-file banner and
//! the daemon status footer.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk4 as gtk;

use crate::daemon_link::{Command, DaemonLink, DaemonStatus};
use crate::document::Document;
use crate::general_tab::GeneralTab;
use crate::integrations_tab::IntegrationsTab;

pub fn build(app: &gtk::Application) -> gtk::ApplicationWindow {
    let document = Document::load();
    // One flag shared by both tabs: programmatic widget updates must not be
    // mistaken for user edits and written back (the Windows form's
    // `_isLoading`).
    let loading = Rc::new(Cell::new(false));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Spotlight Dimmer Settings")
        .default_width(560)
        .default_height(680)
        .build();

    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    // --- unparseable-file banner --------------------------------------------
    let (banner, banner_message, banner_button) = build_banner();
    root.append(&banner);

    // --- tabs ----------------------------------------------------------------
    let general = Rc::new(GeneralTab::new(&document, loading.clone()));
    let integrations = IntegrationsTab::new(&document, loading.clone());

    let notebook = gtk::Notebook::builder().vexpand(true).build();
    notebook.append_page(general.widget(), Some(&gtk::Label::new(Some("General"))));
    notebook.append_page(
        integrations.widget(),
        Some(&gtk::Label::new(Some("Integrations"))),
    );
    root.append(&notebook);

    // --- footer --------------------------------------------------------------
    let daemon_status = gtk::Label::builder()
        .label("checking for the daemon…")
        .xalign(0.0)
        .hexpand(true)
        .build();
    daemon_status.add_css_class("dim-label");

    let path_label = gtk::Label::builder()
        .label(document.path().display().to_string())
        .xalign(1.0)
        .ellipsize(gtk::pango::EllipsizeMode::Start)
        .tooltip_text(document.path().display().to_string())
        .build();
    path_label.add_css_class("dim-label");

    let footer = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(6)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(12)
        .build();
    footer.append(&daemon_status);
    footer.append(&path_label);
    root.append(&footer);

    window.set_child(Some(&root));

    // --- header bar ----------------------------------------------------------
    let dimming_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .tooltip_text("Turn dimming on or off in the running daemon")
        .sensitive(false)
        .build();

    let header = gtk::HeaderBar::new();
    let switch_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    let switch_label = gtk::Label::new(Some("Dimming"));
    switch_label.add_css_class("dim-label");
    switch_box.append(&switch_label);
    switch_box.append(&dimming_switch);
    header.pack_end(&switch_box);
    window.set_titlebar(Some(&header));

    // --- wiring --------------------------------------------------------------
    general.reload(&document);
    update_banner(&document, &banner, &banner_message);

    {
        let document_for_reload = document.clone();
        let general = general.clone();
        let integrations = integrations.clone();
        document.on_reload(move || {
            general.reload(&document_for_reload);
            integrations.reload(&document_for_reload);
        });
    }

    {
        let document_for_problem = document.clone();
        let banner = banner.clone();
        let banner_message = banner_message.clone();
        document.on_problem(move || {
            update_banner(&document_for_problem, &banner, &banner_message);
        });
    }

    {
        let document = document.clone();
        banner_button.connect_clicked(move |_| document.confirm_overwrite());
    }

    document.start_watching();

    connect_daemon(&window, &dimming_switch, &daemon_status);

    window
}

/// Silently clobbering a config file with a typo in it is the one
/// unrecoverable failure this GUI could cause, so writes stay blocked behind
/// an explicit confirmation.
fn build_banner() -> (gtk::Box, gtk::Label, gtk::Button) {
    let message = gtk::Label::builder()
        .xalign(0.0)
        .hexpand(true)
        .wrap(true)
        .build();

    let button = gtk::Button::builder()
        .label("Replace the file")
        .valign(gtk::Align::Center)
        .build();

    let banner = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .visible(false)
        .build();
    banner.append(&message);
    banner.append(&button);
    banner.add_css_class("error");

    (banner, message, button)
}

fn update_banner(document: &Document, banner: &gtk::Box, message: &gtk::Label) {
    match document.problem() {
        Some(crate::document::LoadProblem::Unparseable(detail)) => {
            message.set_text(&format!(
                "config.json could not be parsed ({detail}). Nothing will be saved until you \
                 fix the file or choose to replace it with the settings shown here."
            ));
            banner.set_visible(true);
        }
        None => banner.set_visible(false),
    }
}

fn connect_daemon(
    window: &gtk::ApplicationWindow,
    dimming_switch: &gtk::Switch,
    status_label: &gtk::Label,
) {
    let link = Rc::new(DaemonLink::start());
    // The switch is also driven by the daemon's own state, so guard against
    // a programmatic update being echoed back as a user command.
    let updating = Rc::new(Cell::new(false));

    {
        let link = link.clone();
        let updating = updating.clone();
        dimming_switch.connect_active_notify(move |switch| {
            if updating.get() {
                return;
            }
            link.send(Command::SetEnabled(switch.is_active()));
        });
    }

    let status_rx = link.status.clone();
    let dimming_switch = dimming_switch.clone();
    let status_label = status_label.clone();
    // Keeps the link (and its worker thread's channel) alive for the
    // lifetime of the window.
    let keep_alive = link.clone();
    glib::spawn_future_local(async move {
        let _link = keep_alive;
        while let Ok(status) = status_rx.recv().await {
            updating.set(true);
            match status {
                DaemonStatus::Running { enabled, protocol } => {
                    dimming_switch.set_sensitive(true);
                    dimming_switch.set_active(enabled);
                    status_label.set_text(&format!("daemon running · protocol v{protocol}"));
                }
                DaemonStatus::NotRunning => {
                    dimming_switch.set_sensitive(false);
                    dimming_switch.set_active(false);
                    status_label.set_text("daemon not running");
                }
            }
            updating.set(false);
        }
    });

    // Nothing else to hold the window here; the borrow keeps the signature
    // honest about what this function attaches to.
    let _ = window;
}
