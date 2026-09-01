//! spotlight-dimmer-config — GTK4 settings window for SpotlightDimmer on Linux.
//!
//! Edits the Linux-relevant parts of ~/.config/SpotlightDimmer/config.json
//! (the `Overlay` section and `AppIntegrations`), which the daemon
//! hot-reloads. Every other key in the file is preserved untouched.

mod daemon_link;
mod document;
mod general_tab;
mod integrations_tab;
mod preview;
mod widgets;
mod window;

use gtk::prelude::*;
use gtk4 as gtk;

const APP_ID: &str = "org.spotlightdimmer.Config";

fn main() -> glib::ExitCode {
    // Handled before GTK sees them; GTK would reject the flags outright.
    // The window takes no positional arguments, so the first is all there is.
    if let Some(argument) = std::env::args().nth(1) {
        match argument.as_str() {
            "-h" | "--help" => {
                println!(
                    "Usage: spotlight-dimmer-config\n\n\
                     Settings window for SpotlightDimmer. Edits {}.\n\n\
                     Options:\n  \
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

    app.run_with_args::<&str>(&[])
}
