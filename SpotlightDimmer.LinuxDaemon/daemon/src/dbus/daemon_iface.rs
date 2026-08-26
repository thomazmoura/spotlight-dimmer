//! The org.spotlightdimmer.{Daemon1,Adapter1,Renderer1} interfaces, all
//! served at /org/spotlightdimmer/Daemon.
//!
//! Handlers are thin: they validate, push a typed Event into the main-thread
//! event loop and (where the contract requires) reply from Shared state.

use std::collections::HashMap;
use std::sync::Arc;

use async_channel::Sender;
use zbus::message::Header;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::OwnedValue;

use spotlight_dimmer_core::primitives::Rect;
use spotlight_dimmer_core::state::Monitor;

use crate::events::Event;
use crate::shared::Shared;

fn sender_of(header: &Header<'_>) -> String {
    header.sender().map(|s| s.to_string()).unwrap_or_default()
}

/// The FocusChanged2/GeometryChanged2 rects payload:
/// `{"frame":{"x":..,"y":..,"width":..,"height":..},"client":{...}}`.
/// JSON (like UpdateMonitors) because KWin's callDBus cannot marshal nested
/// structs and truncates calls with more than 9 arguments.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowRectsJson {
    frame: Rect,
    #[serde(default)]
    client: Option<Rect>,
}

impl WindowRectsJson {
    fn parse(method: &str, rects_json: &str) -> Option<WindowRectsJson> {
        match serde_json::from_str::<WindowRectsJson>(rects_json) {
            Ok(mut rects) => {
                // A degenerate client rect means the adapter couldn't
                // determine the client area; fall back to the frame.
                if rects
                    .client
                    .as_ref()
                    .is_some_and(|c| c.width <= 0 || c.height <= 0)
                {
                    rects.client = None;
                }
                Some(rects)
            }
            Err(e) => {
                eprintln!("SpotlightDimmer: invalid {method} payload: {e}");
                None
            }
        }
    }
}

/// One monitor as received in the UpdateMonitors JSON payload. JSON (rather
/// than nested D-Bus structs) because KWin's callDBus only marshals basic
/// types reliably; the same format keeps the GNOME adapter and busctl
/// debugging simple.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MonitorJson {
    key: String,
    geometry: Rect,
    work_area: Rect,
    #[serde(default = "default_scale")]
    scale: f64,
}

fn default_scale() -> f64 {
    1.0
}

/// Global control: toggle/enabled state and protocol version.
pub struct DaemonIface {
    tx: Sender<Event>,
    shared: Arc<Shared>,
}

impl DaemonIface {
    pub fn new(tx: Sender<Event>, shared: Arc<Shared>) -> Self {
        DaemonIface { tx, shared }
    }
}

#[zbus::interface(name = "org.spotlightdimmer.Daemon1")]
impl DaemonIface {
    /// Flip dimming on/off (bound to Super+Shift+D on GNOME, Meta+Shift+D on
    /// KDE). Returns the new enabled state.
    fn toggle(&self) -> bool {
        let enabled = self.shared.toggle_enabled();
        let _ = self.tx.send_blocking(Event::EnabledChanged(enabled));
        enabled
    }

    #[zbus(property)]
    fn enabled(&self) -> bool {
        self.shared.enabled()
    }

    #[zbus(property)]
    fn set_enabled(&self, value: bool) {
        self.shared.set_enabled(value);
        let _ = self.tx.send_blocking(Event::EnabledChanged(value));
    }

    #[zbus(property)]
    fn protocol_version(&self) -> u32 {
        crate::dbus::PROTOCOL_VERSION
    }
}

/// Inbound compositor events: registration, monitors, focus, geometry, title.
pub struct AdapterIface {
    tx: Sender<Event>,
}

impl AdapterIface {
    pub fn new(tx: Sender<Event>) -> Self {
        AdapterIface { tx }
    }
}

#[zbus::interface(name = "org.spotlightdimmer.Adapter1")]
impl AdapterIface {
    /// Register the calling adapter. `compositor` is "gnome" or "kwin";
    /// capabilities currently understands "renders_overlays" (b).
    /// Returns the daemon's protocol version.
    fn register_adapter(
        &self,
        compositor: String,
        capabilities: HashMap<String, OwnedValue>,
        #[zbus(header)] header: Header<'_>,
    ) -> u32 {
        let renders_overlays = capabilities
            .get("renders_overlays")
            .and_then(|v| bool::try_from(v).ok())
            .unwrap_or(false);

        let _ = self.tx.send_blocking(Event::AdapterRegistered {
            sender: sender_of(&header),
            compositor,
            renders_overlays,
        });

        crate::dbus::PROTOCOL_VERSION
    }

    /// Full monitor list as a JSON array:
    /// `[{"key":"DP-1","geometry":{"x":0,"y":0,"width":1920,"height":1080},
    ///    "workArea":{...},"scale":1.0}]`
    /// Rects are logical global compositor coordinates. Invalid JSON is
    /// logged and ignored.
    fn update_monitors(&self, monitors_json: String, #[zbus(header)] header: Header<'_>) {
        let monitors: Vec<MonitorJson> = match serde_json::from_str(&monitors_json) {
            Ok(monitors) => monitors,
            Err(e) => {
                eprintln!("SpotlightDimmer: invalid UpdateMonitors payload: {e}");
                return;
            }
        };

        let monitors = monitors
            .into_iter()
            .map(|m| Monitor {
                key: m.key,
                geometry: m.geometry,
                work_area: m.work_area,
                scale: m.scale,
            })
            .collect();

        let _ = self.tx.send_blocking(Event::MonitorsUpdated {
            sender: sender_of(&header),
            monitors,
        });
    }

    // Flat args are mandated by the wire contract (KWin-safe basic types)
    #[allow(clippy::too_many_arguments)]
    fn focus_changed(
        &self,
        wm_class: String,
        title: String,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        #[zbus(header)] header: Header<'_>,
    ) {
        let _ = self.tx.send_blocking(Event::FocusChanged {
            sender: sender_of(&header),
            wm_class,
            title,
            frame: Rect::new(x, y, width, height),
            client: None,
        });
    }

    /// Protocol v2 FocusChanged: `rects_json` also carries the client-area
    /// rect (decorations excluded), which anchors inner-pane resolution so
    /// the highlight stays aligned in both windowed and maximized states.
    /// Invalid JSON is logged and ignored.
    fn focus_changed2(
        &self,
        wm_class: String,
        title: String,
        rects_json: String,
        #[zbus(header)] header: Header<'_>,
    ) {
        let Some(rects) = WindowRectsJson::parse("FocusChanged2", &rects_json) else {
            return;
        };

        let _ = self.tx.send_blocking(Event::FocusChanged {
            sender: sender_of(&header),
            wm_class,
            title,
            frame: rects.frame,
            client: rects.client,
        });
    }

    fn focus_cleared(&self, #[zbus(header)] header: Header<'_>) {
        let _ = self.tx.send_blocking(Event::FocusCleared {
            sender: sender_of(&header),
        });
    }

    fn geometry_changed(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        #[zbus(header)] header: Header<'_>,
    ) {
        let _ = self.tx.send_blocking(Event::GeometryChanged {
            sender: sender_of(&header),
            frame: Rect::new(x, y, width, height),
            client: None,
        });
    }

    /// Protocol v2 GeometryChanged: frame plus client-area rect as JSON.
    /// Invalid JSON is logged and ignored.
    fn geometry_changed2(&self, rects_json: String, #[zbus(header)] header: Header<'_>) {
        let Some(rects) = WindowRectsJson::parse("GeometryChanged2", &rects_json) else {
            return;
        };

        let _ = self.tx.send_blocking(Event::GeometryChanged {
            sender: sender_of(&header),
            frame: rects.frame,
            client: rects.client,
        });
    }

    fn title_changed(&self, title: String, #[zbus(header)] header: Header<'_>) {
        // The change itself triggers a tmux requery (appIntegrations.js
        // semantics); the new title is also the tty source for terminals
        // where tmux publishes it there.
        let _ = self.tx.send_blocking(Event::TitleChanged {
            sender: sender_of(&header),
            title,
        });
    }
}

/// Outbound overlay definitions for adapter-side renderers (GNOME).
pub struct RendererIface {
    tx: Sender<Event>,
    shared: Arc<Shared>,
}

impl RendererIface {
    pub fn new(tx: Sender<Event>, shared: Arc<Shared>) -> Self {
        RendererIface { tx, shared }
    }
}

#[zbus::interface(name = "org.spotlightdimmer.Renderer1")]
impl RendererIface {
    /// Returns the current overlays payload so a (re)connecting renderer can
    /// paint immediately without waiting for the next OverlaysChanged.
    fn register_renderer(&self, #[zbus(header)] header: Header<'_>) -> String {
        let _ = self.tx.send_blocking(Event::RendererRegistered {
            sender: sender_of(&header),
        });
        self.shared.last_payload()
    }

    /// Emitted on every recompute with the serialized OverlaysPayload.
    /// `serial` duplicates the payload's serial for cheap stale-signal
    /// filtering after reconnects.
    #[zbus(signal)]
    pub async fn overlays_changed(
        emitter: &SignalEmitter<'_>,
        serial: u32,
        overlays_json: &str,
    ) -> zbus::Result<()>;
}
