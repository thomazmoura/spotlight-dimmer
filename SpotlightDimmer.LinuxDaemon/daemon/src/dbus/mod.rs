//! D-Bus service setup: name ownership, interface registration and the
//! adapter-liveness watcher.

pub mod daemon_iface;
pub mod pane_tracker;

use std::sync::Arc;

use async_channel::Sender;
use futures_lite::StreamExt;
use zbus::Connection;

use crate::events::Event;
use crate::shared::Shared;

pub const DAEMON_NAME: &str = "org.spotlightdimmer.Daemon";
pub const DAEMON_PATH: &str = "/org/spotlightdimmer/Daemon";
pub const PANE_TRACKER_NAME: &str = "org.spotlightdimmer.PaneTracker";
pub const PANE_TRACKER_PATH: &str = "/org/spotlightdimmer/PaneTracker";
pub const PROTOCOL_VERSION: u32 = 1;

/// Connect to the session bus, export all interfaces and claim both
/// well-known names. Fails if another daemon instance already owns them.
pub async fn connect(tx: Sender<Event>, shared: Arc<Shared>) -> zbus::Result<Connection> {
    let conn = zbus::connection::Builder::session()?
        .name(DAEMON_NAME)?
        .serve_at(
            DAEMON_PATH,
            daemon_iface::DaemonIface::new(tx.clone(), shared.clone()),
        )?
        .serve_at(DAEMON_PATH, daemon_iface::AdapterIface::new(tx.clone()))?
        .serve_at(
            DAEMON_PATH,
            daemon_iface::RendererIface::new(tx.clone(), shared.clone()),
        )?
        .serve_at(PANE_TRACKER_PATH, pane_tracker::PaneTracker::new(tx))?
        .build()
        .await?;

    // Second well-known name on the same connection. Deliberately NOT
    // D-Bus-activatable (no .service file): tmux hooks must stay silent
    // no-ops when the daemon is down and must never auto-start it.
    conn.request_name(PANE_TRACKER_NAME).await?;

    Ok(conn)
}

/// Watch NameOwnerChanged and report registered adapters whose unique bus
/// name vanished (extension disabled, gnome-shell restart, KWin exit).
pub async fn watch_adapter_names(conn: Connection, shared: Arc<Shared>, tx: Sender<Event>) {
    let proxy = match zbus::fdo::DBusProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("SpotlightDimmer: failed to create DBus proxy for name watching: {e}");
            return;
        }
    };

    let mut stream = match proxy.receive_name_owner_changed().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("SpotlightDimmer: failed to subscribe to NameOwnerChanged: {e}");
            return;
        }
    };

    while let Some(signal) = stream.next().await {
        let Ok(args) = signal.args() else {
            continue;
        };

        if args.new_owner().is_none() {
            let name = args.name().to_string();
            if shared.is_adapter(&name) {
                let _ = tx.send(Event::AdapterLost { sender: name }).await;
            }
        }
    }
}
