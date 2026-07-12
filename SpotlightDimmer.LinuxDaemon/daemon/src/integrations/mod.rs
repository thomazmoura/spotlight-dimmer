//! App integrations: spotlight an inner region of the focused window.
//!
//! The "tmux" provider joins two data sources (appIntegrations.js port):
//! - push: tmux hooks report the focused pane rect per client tty over the
//!   PaneTracker D-Bus interface (arrives as Event::PaneUpdated/PaneCleared)
//! - pull: on focus/title changes the wezterm CLI is queried asynchronously
//!   to find the focused pane's tty and verify a live tmux client owns it

pub mod wezterm;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use async_channel::Sender;

use spotlight_dimmer_core::config::{AppConfig, AppIntegration};
use spotlight_dimmer_core::pane;
use spotlight_dimmer_core::primitives::Rect;
use spotlight_dimmer_core::state::Focus;

use crate::events::Event;
use wezterm::ActivePane;

/// Per-focused-window integration state, owned by the event loop.
pub struct IntegrationState {
    /// Latest pane rect per tmux client tty (pixels relative to the terminal
    /// content origin), fed by the PaneTracker D-Bus interface.
    pane_data_by_tty: HashMap<String, Rect>,
    /// Config entry matching the focused window, if any.
    matched: Option<AppIntegration>,
    /// Focused wezterm pane resolved by the query chain, if any.
    active_pane: Option<ActivePane>,
    /// Invalidates in-flight async queries when focus moves on. Shared with
    /// spawned query futures (single-threaded, hence Rc<Cell>).
    generation: Rc<Cell<u64>>,
}

impl IntegrationState {
    pub fn new() -> IntegrationState {
        IntegrationState {
            pane_data_by_tty: HashMap::new(),
            matched: None,
            active_pane: None,
            generation: Rc::new(Cell::new(0)),
        }
    }

    /// True when the focused window has a matching integration (title
    /// changes are only interesting in that case).
    pub fn is_active(&self) -> bool {
        self.matched.is_some()
    }

    /// Update state for a newly focused window (or refreshed config) and
    /// start a wezterm query when the window matches an integration.
    /// Port of `setFocusedWindow` (appIntegrations.js).
    pub fn set_focused_window(
        &mut self,
        config: &AppConfig,
        wm_class: Option<&str>,
        tx: &Sender<Event>,
    ) {
        // Invalidate any in-flight query
        self.generation.set(self.generation.get() + 1);

        self.matched = wm_class
            .and_then(|c| config.match_integration(c, "tmux"))
            .cloned();
        self.active_pane = None;

        if self.matched.is_some() {
            self.spawn_query(tx);
        }
    }

    /// Re-run the wezterm query for the current focus (title changed, tmux
    /// hook fired, config reloaded).
    pub fn requery(&mut self, tx: &Sender<Event>) {
        if self.matched.is_some() {
            self.spawn_query(tx);
        }
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
        let integration = self.matched.as_ref()?;
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

    fn spawn_query(&mut self, tx: &Sender<Event>) {
        self.generation.set(self.generation.get() + 1);
        let generation = self.generation.get();
        let generation_cell = self.generation.clone();
        let tx = tx.clone();

        glib::spawn_future_local(async move {
            let pane = wezterm::query_active_pane(&generation_cell, generation).await;

            // Don't overwrite a newer query's result with a stale abort
            if generation_cell.get() == generation {
                let _ = tx.send(Event::PaneResolved { generation, pane }).await;
            }
        });
    }
}
