//! App integrations: spotlight an inner region of the focused window.
//!
//! The "tmux" provider joins two data sources (appIntegrations.js port):
//! - push: tmux hooks report the focused pane rect per client tty over the
//!   PaneTracker D-Bus interface (arrives as Event::PaneUpdated/PaneCleared)
//! - pull: on focus/title changes the focused pane's tty is resolved and
//!   verified against the live tmux clients
//!
//! The tty is the join key, and how it is found depends on the terminal
//! (`TtySource`): WezTerm is asked over its CLI, while terminals without one
//! (Ghostty) have tmux publish the tty in the window title.

pub mod herdr;
pub mod proc;
pub mod title;
pub mod tmux;
pub mod wezterm;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use async_channel::Sender;

use spotlight_dimmer_core::config::{AppConfig, AppIntegration, TtySource};
use spotlight_dimmer_core::pane;
use spotlight_dimmer_core::primitives::Rect;
use spotlight_dimmer_core::state::Focus;
// Aliased: this module also has a `title` submodule (the tty source).
use spotlight_dimmer_core::title as core_title;

use crate::events::Event;
use herdr::HerdrLayout;
use tmux::ActivePane;

/// Per-focused-window integration state, owned by the event loop.
pub struct IntegrationState {
    /// Latest pane rect per tmux client tty (pixels relative to the terminal
    /// content origin), fed by the PaneTracker D-Bus interface.
    pane_data_by_tty: HashMap<String, Rect>,
    /// Config entries matching the focused window. A window class can carry
    /// one entry per provider (the same terminal may run tmux in one window
    /// and Herdr in another); each provider recognizes its own windows by the
    /// marker in the title, so they are simply tried in order.
    matched: Vec<AppIntegration>,
    /// The focused window's current title (the tty source for terminals
    /// where tmux publishes it there).
    title: String,
    /// Focused terminal pane resolved by the query chain, if any.
    active_pane: Option<ActivePane>,
    /// Latest focused layout published by the Herdr reader thread (cells).
    herdr_layout: Option<HerdrLayout>,
    /// The Herdr reader thread is started once, on the first config that
    /// asks for the provider, and runs for the process lifetime.
    herdr_started: bool,
    /// Invalidates in-flight async queries when focus moves on. Shared with
    /// spawned query futures (single-threaded, hence Rc<Cell>).
    generation: Rc<Cell<u64>>,
}

impl IntegrationState {
    pub fn new() -> IntegrationState {
        IntegrationState {
            pane_data_by_tty: HashMap::new(),
            matched: Vec::new(),
            title: String::new(),
            active_pane: None,
            herdr_layout: None,
            herdr_started: false,
            generation: Rc::new(Cell::new(0)),
        }
    }

    /// True when the focused window has a matching integration (title
    /// changes are only interesting in that case).
    pub fn is_active(&self) -> bool {
        !self.matched.is_empty()
    }

    /// Update state for a newly focused window (or refreshed config) and
    /// start a pane query when the window matches an integration.
    /// Port of `setFocusedWindow` (appIntegrations.js).
    pub fn set_focused_window(
        &mut self,
        config: &AppConfig,
        wm_class: Option<&str>,
        title: &str,
        tx: &Sender<Event>,
    ) {
        // Invalidate any in-flight query
        self.generation.set(self.generation.get() + 1);

        self.matched = wm_class
            .map(|c| config.match_window(c).cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        self.title = title.to_string();
        self.active_pane = None;

        if self.has_provider("tmux") {
            self.spawn_query(tx);
        }
    }

    /// Start the data sources the current config asks for. Idempotent: the
    /// Herdr reader owns its own reconnect loop, so it is started once and
    /// never restarted on config reloads.
    pub fn ensure_providers(&mut self, config: &AppConfig, tx: &Sender<Event>) {
        if self.herdr_started || !config.uses_provider("herdr") {
            return;
        }

        let socket_path = config
            .app_integrations
            .iter()
            .find(|i| i.provider == "herdr")
            .map(|i| i.socket_path.clone())
            .unwrap_or_default();

        herdr::spawn(&socket_path, tx.clone());
        self.herdr_started = true;
    }

    fn has_provider(&self, provider: &str) -> bool {
        self.matched.iter().any(|i| i.provider == provider)
    }

    /// Re-run the pane query for the current focus (title changed, tmux hook
    /// fired, config reloaded). The title is refreshed first: for the
    /// window-title tty source it *is* the query input.
    /// Returns true when the caller should recompute right away: the Herdr
    /// join reads the workspace straight out of the title, so a new title is
    /// already the new answer — there is nothing to wait for.
    pub fn requery(&mut self, title: &str, tx: &Sender<Event>) -> bool {
        if self.matched.is_empty() {
            return false;
        }

        self.title = title.to_string();
        if self.has_provider("tmux") {
            self.spawn_query(tx);
            return false;
        }

        true
    }

    /// Store the layout published by the Herdr reader thread. Returns true
    /// when it changed anything.
    pub fn update_herdr_layout(&mut self, layout: Option<HerdrLayout>) -> bool {
        if self.herdr_layout == layout {
            return false;
        }

        self.herdr_layout = layout;
        true
    }

    /// Result of a finished query chain; ignored when stale.
    pub fn on_pane_resolved(&mut self, generation: u64, pane: Option<ActivePane>) -> bool {
        if generation != self.generation.get() {
            return false;
        }
        self.active_pane = pane;
        true
    }

    pub fn update_pane_data(&mut self, tty: String, rect: Rect) {
        self.pane_data_by_tty.insert(tty, rect);
    }

    /// Returns true when the tty had recorded pane data.
    pub fn clear_pane_data(&mut self, tty: &str) -> bool {
        self.pane_data_by_tty.remove(tty).is_some()
    }

    /// Screen-space rect of the focused inner region, or None to use the
    /// whole window. Port of `getPaneRect` (the join itself lives in
    /// core::pane so it is unit-tested).
    pub fn resolve_inner_rect(&self, focus: Option<&Focus>) -> Option<Rect> {
        let focus = focus?;

        // First provider that recognizes this window wins; the rest fall
        // through to the whole-window spotlight.
        self.matched.iter().find_map(|integration| {
            if integration.provider == "herdr" {
                self.resolve_herdr_rect(focus, integration)
            } else {
                self.resolve_tmux_rect(focus, integration)
            }
        })
    }

    /// tmux side of `resolve_inner_rect`.
    fn resolve_tmux_rect(&self, focus: &Focus, integration: &AppIntegration) -> Option<Rect> {
        let active_pane = self.active_pane.as_ref()?;
        let pane_data = self.pane_data_by_tty.get(&active_pane.tty)?;

        pane::pane_rect(
            &focus.frame,
            focus.client.as_ref(),
            (integration.content_offset_x, integration.content_offset_y),
            (active_pane.offset_x, active_pane.offset_y),
            pane_data,
        )
    }

    /// Herdr side of `resolve_inner_rect`: the window title says which Herdr
    /// workspace this window shows, and the reader thread says where the
    /// focused pane sits in that workspace's cell grid. A window without the
    /// marker (a plain shell, or Herdr not running) keeps the whole-window
    /// spotlight.
    fn resolve_herdr_rect(&self, focus: &Focus, integration: &AppIntegration) -> Option<Rect> {
        let layout = self.herdr_layout.as_ref()?;
        let key = core_title::parse_herdr_key(&self.title)?;

        if !core_title::herdr_key_matches(key, &layout.workspace_id, &layout.workspace_label) {
            return None;
        }

        pane::pane_rect_from_cells(
            &focus.frame,
            focus.client.as_ref(),
            (integration.content_offset_x, integration.content_offset_y),
            layout.grid,
            &layout.pane,
        )
    }

    fn spawn_query(&mut self, tx: &Sender<Event>) {
        self.generation.set(self.generation.get() + 1);
        let generation = self.generation.get();
        let generation_cell = self.generation.clone();
        let tx = tx.clone();

        let tty_source = self
            .matched
            .iter()
            .find(|i| i.provider == "tmux")
            .map(|i| i.tty_source)
            .unwrap_or_default();
        let title = self.title.clone();

        glib::spawn_future_local(async move {
            let pane = match tty_source {
                TtySource::WezTermCli => {
                    wezterm::query_active_pane(&generation_cell, generation).await
                }
                TtySource::WindowTitle => {
                    title::query_active_pane(&title, &generation_cell, generation).await
                }
            };

            // Don't overwrite a newer query's result with a stale abort
            if generation_cell.get() == generation {
                let _ = tx.send(Event::PaneResolved { generation, pane }).await;
            }
        });
    }
}
