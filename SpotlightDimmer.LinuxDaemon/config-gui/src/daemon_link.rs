//! Best-effort D-Bus link to a running daemon.
//!
//! Strictly optional: every failure degrades to "daemon not running" and
//! never blocks editing the file. The daemon is D-Bus-activatable, so the
//! link deliberately probes `NameHasOwner` before talking to it — otherwise
//! merely opening the settings window would launch the daemon as a side
//! effect.

use async_channel::{Receiver, Sender};
use std::pin::Pin;

use futures_lite::{stream, Stream, StreamExt};
use zbus::names::BusName;
use zbus::Connection;

const DAEMON_NAME: &str = "org.spotlightdimmer.Daemon";
const DAEMON_PATH: &str = "/org/spotlightdimmer/Daemon";
const DAEMON_IFACE: &str = "org.spotlightdimmer.Daemon1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonStatus {
    NotRunning,
    Running { enabled: bool, protocol: u32 },
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    SetEnabled(bool),
}

pub struct DaemonLink {
    pub status: Receiver<DaemonStatus>,
    commands: Sender<Command>,
}

impl DaemonLink {
    /// Starts the D-Bus worker on its own thread. The GTK main loop reads
    /// `status` with `glib::spawn_future_local`; commands go the other way.
    pub fn start() -> DaemonLink {
        let (status_tx, status) = async_channel::bounded(8);
        let (commands, command_rx) = async_channel::bounded(8);

        std::thread::Builder::new()
            .name("spotlight-dimmer-dbus".to_string())
            .spawn(move || futures_lite::future::block_on(run(status_tx, command_rx)))
            .expect("spawning the D-Bus thread");

        DaemonLink { status, commands }
    }

    pub fn send(&self, command: Command) {
        // Dropping a command when the queue is full is fine: the next status
        // refresh re-syncs the switch with the daemon's real state.
        let _ = self.commands.try_send(command);
    }
}

async fn run(status_tx: Sender<DaemonStatus>, commands: Receiver<Command>) {
    let connection = match Connection::session().await {
        Ok(connection) => connection,
        Err(e) => {
            eprintln!("SpotlightDimmer: no session bus ({e}); daemon controls disabled");
            let _ = status_tx.send(DaemonStatus::NotRunning).await;
            return;
        }
    };

    let _ = status_tx.send(probe(&connection).await).await;

    let Some(mut wakes) = wake_stream(&connection, commands).await else {
        return;
    };

    while let Some(wake) = wakes.next().await {
        if let Wake::Command(Command::SetEnabled(value)) = wake {
            set_enabled(&connection, value).await;
        }

        if status_tx.send(probe(&connection).await).await.is_err() {
            return; // window closed
        }
    }
}

enum Wake {
    Bus,
    Command(Command),
}

/// Merges the two signals that can change the daemon's state — the daemon
/// starting or stopping, and `Enabled` being flipped elsewhere (the
/// Meta+Shift+D shortcut) — with commands from the UI.
///
/// `PropertiesChanged` is matched by path rather than through a proxy on
/// purpose: building a signal stream against a well-known destination
/// requires resolving its owner, which fails outright while the daemon is
/// down. A path match rule survives the daemon restarting.
async fn wake_stream<'a>(
    connection: &'a Connection,
    commands: Receiver<Command>,
) -> Option<Pin<Box<dyn Stream<Item = Wake> + 'a>>> {
    let dbus = zbus::fdo::DBusProxy::new(connection)
        .await
        .map_err(|e| eprintln!("SpotlightDimmer: DBus proxy unavailable: {e}"))
        .ok()?;

    let owner_changes = dbus
        .receive_name_owner_changed_with_args(&[(0, DAEMON_NAME)])
        .await
        .map_err(|e| eprintln!("SpotlightDimmer: cannot watch the daemon's bus name: {e}"))
        .ok()?;

    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .interface("org.freedesktop.DBus.Properties")
        .ok()?
        .member("PropertiesChanged")
        .ok()?
        .path(DAEMON_PATH)
        .ok()?
        .build();

    let properties = zbus::MessageStream::for_match_rule(rule, connection, Some(8))
        .await
        .map_err(|e| eprintln!("SpotlightDimmer: cannot watch daemon properties: {e}"))
        .ok()?;

    let bus = stream::or(
        owner_changes.map(|_| Wake::Bus),
        properties.map(|_| Wake::Bus),
    );

    Some(Box::pin(stream::or(bus, commands.map(Wake::Command))))
}

async fn probe(connection: &Connection) -> DaemonStatus {
    let Ok(name) = BusName::try_from(DAEMON_NAME) else {
        return DaemonStatus::NotRunning;
    };

    let Ok(dbus) = zbus::fdo::DBusProxy::new(connection).await else {
        return DaemonStatus::NotRunning;
    };

    // Never activate the daemon just by opening this window.
    if !dbus.name_has_owner(name).await.unwrap_or(false) {
        return DaemonStatus::NotRunning;
    }

    let Ok(proxy) = daemon_proxy(connection).await else {
        return DaemonStatus::NotRunning;
    };

    DaemonStatus::Running {
        enabled: proxy.get_property("Enabled").await.unwrap_or(true),
        protocol: proxy.get_property("ProtocolVersion").await.unwrap_or(0),
    }
}

async fn set_enabled(connection: &Connection, value: bool) {
    let Ok(proxy) = daemon_proxy(connection).await else {
        return;
    };
    if let Err(e) = proxy.set_property("Enabled", value).await {
        eprintln!("SpotlightDimmer: could not set Enabled on the daemon: {e}");
    }
}

async fn daemon_proxy(connection: &Connection) -> zbus::Result<zbus::Proxy<'_>> {
    zbus::Proxy::new(connection, DAEMON_NAME, DAEMON_PATH, DAEMON_IFACE).await
}
