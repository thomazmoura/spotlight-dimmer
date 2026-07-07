//! spotlight-dimmer-daemon: shared SpotlightDimmer daemon for Wayland
//! compositors.
//!
//! Owns configuration, overlay calculation and the wezterm/tmux integration.
//! Compositor adapters (GNOME Shell extension, KWin script) feed it focus and
//! monitor data over D-Bus; it publishes computed overlay definitions back
//! (GNOME renders them) or renders them itself via layer-shell (KDE, M3).
//!
//! Threading model: zbus runs handlers on its own executor threads, but they
//! only push events into a channel; the glib main-context loop on this thread
//! owns all mutable state (see daemon.rs).

mod config_watch;
mod daemon;
mod dbus;
mod events;
mod integrations;
#[cfg(feature = "render")]
mod render;
mod shared;

use std::process::ExitCode;
use std::sync::Arc;

fn main() -> ExitCode {
    let main_context = glib::MainContext::default();
    let _context_guard = main_context
        .acquire()
        .expect("failed to acquire main context");

    let (tx, rx) = async_channel::unbounded::<events::Event>();
    let shared = Arc::new(shared::Shared::new());

    // Claim bus names and export interfaces before entering the main loop;
    // failure here usually means another daemon instance is running.
    let conn = match zbus::block_on(dbus::connect(tx.clone(), shared.clone())) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("SpotlightDimmer: failed to connect to session bus: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!(
        "SpotlightDimmer: owning {} and {}",
        dbus::DAEMON_NAME,
        dbus::PANE_TRACKER_NAME
    );

    let daemon = match daemon::Daemon::new(&conn, shared.clone(), tx.clone()) {
        Ok(daemon) => daemon,
        Err(e) => {
            eprintln!("SpotlightDimmer: failed to initialize: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Keep the config directory monitor alive for the process lifetime
    let _config_monitor = config_watch::watch(tx.clone());

    main_context.spawn_local(dbus::watch_adapter_names(conn.clone(), shared, tx));
    main_context.spawn_local(daemon.run(rx));

    // No custom SIGINT/SIGTERM handling: default termination is safe here —
    // bus names are released when the connection closes and overlay windows
    // die with the process.
    let main_loop = glib::MainLoop::new(Some(&main_context), false);
    main_loop.run();
    ExitCode::SUCCESS
}
