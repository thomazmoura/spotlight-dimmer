//! spotlight-dimmer-config — GTK4 settings window for SpotlightDimmer on Linux.
//!
//! Edits the Linux-relevant parts of ~/.config/SpotlightDimmer/config.json
//! (the `Overlay` section and `AppIntegrations`), which the daemon
//! hot-reloads. Every other key in the file is preserved untouched.

mod daemon_link;
mod document;
mod fuzzy;
mod general_tab;
mod integrations_tab;
mod preview;
mod profiles_section;
mod widgets;
mod window;

use gio::prelude::*;
use gtk::prelude::*;
use gtk4 as gtk;

const APP_ID: &str = "org.spotlightdimmer.Config";

/// App action a second `--toggle` launch activates on the running instance.
const TOGGLE_ACTION: &str = "toggle-window";

fn main() -> glib::ExitCode {
    // Handled before GTK sees them; GTK would reject the flags outright.
    // The window takes no positional arguments, so the first is all there is.
    let mut toggle = false;
    if let Some(argument) = std::env::args().nth(1) {
        match argument.as_str() {
            "-h" | "--help" => {
                println!(
                    "Usage: spotlight-dimmer-config [--toggle]\n\n\
                     Settings window for SpotlightDimmer. Edits {}.\n\n\
                     Options:\n  \
                     -t, --toggle     Close the window if it is focused, otherwise open or raise it\n  \
                     -h, --help       Show this help\n  \
                     -V, --version    Show the version",
                    document::config_path().display()
                );
                return glib::ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("spotlight-dimmer-config {}", env!("CARGO_PKG_VERSION"));
                return glib::ExitCode::SUCCESS;
            }
            "-t" | "--toggle" => toggle = true,
            other => {
                eprintln!("spotlight-dimmer-config: unrecognized argument '{other}'");
                return glib::ExitCode::FAILURE;
            }
        }
    }

    let app = gtk::Application::builder().application_id(APP_ID).build();

    // A second launch activates the first instance instead of opening a
    // second window that would fight it over the same file.
    app.connect_activate(|app| {
        if let Some(existing) = app.active_window() {
            existing.present();
            return;
        }
        window::build(app).present();
    });

    // Bound to Super+Alt+Shift+D: the same keypress opens the window and,
    // once it has focus, closes it again. A window buried behind others is
    // raised rather than closed, since the user evidently wants to see it.
    let toggle_action = gio::SimpleAction::new(TOGGLE_ACTION, None);
    {
        let app = app.downgrade();
        toggle_action.connect_activate(move |_, _| {
            let Some(app) = app.upgrade() else {
                return;
            };
            match app.active_window() {
                Some(existing) if existing.is_active() => existing.close(),
                Some(existing) => {
                    existing.present();
                    // The shortcut is for switching profiles: land in the
                    // picker even when the window was left on another tab.
                    WidgetExt::activate_action(&existing, window::FOCUS_PROFILES_ACTION, None)
                        .ok();
                }
                None => window::build(&app).present(),
            }
        });
    }
    app.add_action(&toggle_action);

    if toggle {
        if let Err(e) = app.register(gio::Cancellable::NONE) {
            eprintln!("spotlight-dimmer-config: could not register the application: {e}");
            return glib::ExitCode::FAILURE;
        }
        if app.is_remote() {
            app.activate_action(TOGGLE_ACTION, None);
            // The remote call is queued asynchronously; make sure it leaves
            // before this process exits.
            if let Some(connection) = app.dbus_connection() {
                let _ = connection.flush_sync(gio::Cancellable::NONE);
            }
            return glib::ExitCode::SUCCESS;
        }
        // No running instance: fall through and open the window normally.
    }

    app.run_with_args::<&str>(&[])
}
