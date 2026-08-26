//! Resolve the focused pane's tty from the window title.
//!
//! For terminals with no pane-query CLI (Ghostty): tmux publishes the client
//! tty in the title via `set-titles-string`, so the title the compositor
//! already reports *is* the answer. Only the liveness check needs a
//! subprocess, and Ghostty-native splits are not represented, so the pane
//! origin is always (0, 0) — tmux is assumed to fill the surface.

use std::cell::Cell;

use spotlight_dimmer_core::title;

use crate::integrations::tmux::{self, ActivePane};

/// Parse the tty out of `window_title` and verify a live tmux client owns it.
/// `current`/`generation` abort the chain when focus moved on meanwhile, as
/// in the wezterm source.
pub async fn query_active_pane(
    window_title: &str,
    current: &Cell<u64>,
    generation: u64,
) -> Option<ActivePane> {
    // Cheap and synchronous: bail out before spawning anything.
    let tty = title::parse_tty(window_title)?.to_string();

    let ttys = tmux::client_ttys().await;
    if current.get() != generation {
        return None;
    }

    // A stale title (tmux detached or the server exited) must not keep the
    // spotlight pinned to a pane that no longer exists.
    let ttys = ttys?;
    ttys.contains(&tty).then_some(ActivePane {
        tty,
        offset_x: 0,
        offset_y: 0,
    })
}
