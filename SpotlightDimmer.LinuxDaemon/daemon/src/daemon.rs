//! The single-threaded event loop that owns all mutable daemon state.
//!
//! Every input (D-Bus calls, config file changes, finished async queries)
//! arrives as an Event; each handler mutates state and calls `apply()`,
//! which recomputes overlays and publishes the result.

use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, Sender};
use zbus::object_server::SignalEmitter;
use zbus::Connection;

use spotlight_dimmer_core::state::{AppState, Focus, Monitor};

use crate::config_watch;
use crate::dbus::daemon_iface::RendererIface;
use crate::dbus::DAEMON_PATH;
use crate::events::Event;
use crate::integrations::IntegrationState;
use crate::shared::{AdapterInfo, Shared};

/// Debounce for window title changes (terminals update titles frequently).
const TITLE_DEBOUNCE_MS: u64 = 150;

pub struct Daemon {
    state: AppState,
    integration: IntegrationState,
    shared: Arc<Shared>,
    tx: Sender<Event>,
    emitter: SignalEmitter<'static>,
    /// Epoch-based debounce for TitleChanged -> RequeryPane.
    title_epoch: u64,
    #[cfg(feature = "render")]
    renderer: crate::render::LayerShellRenderer,
}

impl Daemon {
    pub fn new(conn: &Connection, shared: Arc<Shared>, tx: Sender<Event>) -> zbus::Result<Daemon> {
        let emitter = SignalEmitter::new(conn, DAEMON_PATH)?.into_owned();

        let config = config_watch::load(&config_watch::config_path()).unwrap_or_default();
        let mut state = AppState::new(config);
        state.set_enabled(shared.enabled());
        state.monitors = load_monitor_cache();

        Ok(Daemon {
            state,
            integration: IntegrationState::new(),
            shared,
            tx,
            emitter,
            title_epoch: 0,
            #[cfg(feature = "render")]
            renderer: crate::render::LayerShellRenderer::new(),
        })
    }

    pub async fn run(mut self, rx: Receiver<Event>) {
        println!("SpotlightDimmer: daemon event loop started");

        while let Ok(event) = rx.recv().await {
            self.handle(event).await;
        }
    }

    async fn handle(&mut self, event: Event) {
        match event {
            Event::AdapterRegistered {
                sender,
                compositor,
                renders_overlays,
            } => {
                println!(
                    "SpotlightDimmer: adapter registered: {compositor} ({sender}), renders_overlays={renders_overlays}"
                );
                self.shared.register_adapter(
                    sender,
                    AdapterInfo {
                        compositor,
                        renders_overlays,
                    },
                );
                // Re-evaluate who renders (a GNOME renderer appearing must
                // silence the layer-shell path, and vice versa)
                self.apply().await;
            }

            Event::RendererRegistered { sender } => {
                println!("SpotlightDimmer: renderer registered: {sender}");
            }

            Event::AdapterLost { sender } => {
                if let Some(info) = self.shared.remove_adapter(&sender) {
                    println!(
                        "SpotlightDimmer: adapter lost: {} ({sender})",
                        info.compositor
                    );
                    self.apply().await;
                }
            }

            Event::MonitorsUpdated { sender, monitors } => {
                self.ensure_registered(&sender);
                println!("SpotlightDimmer: {} monitor(s) reported", monitors.len());
                save_monitor_cache(&monitors);
                self.state.monitors = monitors;
                self.apply().await;
            }

            Event::FocusChanged {
                sender,
                wm_class,
                title,
                frame,
            } => {
                self.ensure_registered(&sender);
                self.state.focus = Some(Focus {
                    wm_class: wm_class.clone(),
                    title,
                    frame,
                });
                self.integration
                    .set_focused_window(&self.state.config, Some(&wm_class), &self.tx);
                self.apply().await;
            }

            Event::FocusCleared { sender } => {
                self.ensure_registered(&sender);
                self.state.focus = None;
                self.integration
                    .set_focused_window(&self.state.config, None, &self.tx);
                self.apply().await;
            }

            Event::GeometryChanged { sender, frame } => {
                self.ensure_registered(&sender);
                if let Some(focus) = self.state.focus.as_mut() {
                    focus.frame = frame;
                    self.apply().await;
                }
            }

            Event::TitleChanged { sender } => {
                self.ensure_registered(&sender);
                if !self.integration.is_active() {
                    return;
                }

                // Debounce: only the timeout holding the latest epoch requeries
                self.title_epoch += 1;
                let epoch = self.title_epoch;
                let tx = self.tx.clone();
                glib::timeout_add_local_once(Duration::from_millis(TITLE_DEBOUNCE_MS), move || {
                    let _ = tx.send_blocking(Event::RequeryPane { epoch });
                });
            }

            Event::RequeryPane { epoch } => {
                if epoch == self.title_epoch {
                    self.integration.requery(&self.tx);
                }
            }

            Event::PaneUpdated { tty, rect } => {
                self.integration.update_pane_data(tty, rect);
                self.apply().await;
            }

            Event::PaneCleared { tty } => {
                if self.integration.clear_pane_data(&tty) {
                    self.apply().await;
                }
            }

            Event::PaneResolved { generation, pane } => {
                if self.integration.on_pane_resolved(generation, pane) {
                    self.apply().await;
                }
            }

            Event::ConfigFileChanged => {
                if let Some(config) = config_watch::load(&config_watch::config_path()) {
                    println!("SpotlightDimmer: config reloaded");
                    self.state.config = config;

                    // Re-match the focused window against the new
                    // AppIntegrations and refresh the pane query
                    let wm_class = self.state.focus.as_ref().map(|f| f.wm_class.clone());
                    self.integration.set_focused_window(
                        &self.state.config,
                        wm_class.as_deref(),
                        &self.tx,
                    );

                    self.apply().await;
                }
            }

            Event::EnabledChanged(enabled) => {
                println!(
                    "SpotlightDimmer: dimming {}",
                    if enabled { "enabled" } else { "paused" }
                );
                self.state.set_enabled(enabled);
                self.apply().await;
            }
        }
    }

    /// Adapter calls arriving from a sender we don't know (daemon restarted
    /// underneath a running KWin script) count as implicit registration; the
    /// monitor cache covers the monitor list until the adapter's next
    /// UpdateMonitors.
    fn ensure_registered(&mut self, sender: &str) {
        if sender.is_empty() || self.shared.is_adapter(sender) {
            return;
        }

        println!("SpotlightDimmer: implicit adapter registration for {sender}");
        self.shared.register_adapter(
            sender.to_string(),
            AdapterInfo {
                compositor: String::from("unknown"),
                renders_overlays: false,
            },
        );
    }

    /// Recompute overlays and publish: store the snapshot for late-joining
    /// renderers and emit OverlaysChanged. A `None` recompute means the
    /// anti-flicker freeze: keep the previous overlays untouched.
    async fn apply(&mut self) {
        self.state.inner_rect = self
            .integration
            .resolve_inner_rect(self.state.focus.as_ref());

        let Some(payload) = self.state.recompute() else {
            return;
        };

        let json = match serde_json::to_string(&payload) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("SpotlightDimmer: failed to serialize overlays payload: {e}");
                return;
            }
        };

        self.shared.set_last_payload(json.clone());

        if let Err(e) =
            RendererIface::overlays_changed(&self.emitter, payload.serial as u32, &json).await
        {
            eprintln!("SpotlightDimmer: failed to emit OverlaysChanged: {e}");
        }

        // Render locally via layer-shell only when a compositor adapter is
        // present and none of them renders overlays itself (i.e. KDE/wlroots,
        // not GNOME). With no adapters there is nothing meaningful to show.
        #[cfg(feature = "render")]
        {
            if self.shared.has_adapters() && !self.shared.has_renderer_adapter() {
                self.renderer.apply(&payload, &self.state.monitors);
            } else {
                self.renderer.hide_all();
            }
        }
    }
}

/// `$XDG_RUNTIME_DIR/spotlight-dimmer/monitors.json`. The runtime dir is
/// wiped at logout, which matches the cache's validity: monitor state only
/// means something within the session that reported it.
fn monitor_cache_path() -> std::path::PathBuf {
    glib::user_runtime_dir()
        .join("spotlight-dimmer")
        .join("monitors.json")
}

/// Adapters only send monitors on registration and on screen changes, so a
/// daemon restarted mid-session (upgrade, crash recovery) would otherwise
/// sit monitor-less — unable to compute overlays — until a hotplug event.
fn load_monitor_cache() -> Vec<Monitor> {
    let path = monitor_cache_path();
    let Ok(json) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };

    match serde_json::from_str::<Vec<Monitor>>(&json) {
        Ok(monitors) => {
            if !monitors.is_empty() {
                println!(
                    "SpotlightDimmer: restored {} monitor(s) from {}",
                    monitors.len(),
                    path.display()
                );
            }
            monitors
        }
        Err(e) => {
            eprintln!("SpotlightDimmer: ignoring invalid monitor cache: {e}");
            Vec::new()
        }
    }
}

/// Best-effort: a failed write only degrades restart recovery.
fn save_monitor_cache(monitors: &[Monitor]) {
    let path = monitor_cache_path();
    let Ok(json) = serde_json::to_string(monitors) else {
        return;
    };

    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, json) {
        eprintln!(
            "SpotlightDimmer: failed to write monitor cache {}: {e}",
            path.display()
        );
    }
}
