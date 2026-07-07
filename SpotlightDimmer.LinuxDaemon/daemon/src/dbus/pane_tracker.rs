//! The org.spotlightdimmer.PaneTracker interface.
//!
//! Signature-identical to the interface previously exported by the GNOME
//! extension (appIntegrations.js) so the existing tmux hook script
//! (tools/spotlight-dimmer-tmux-report.sh) keeps working unchanged.

use async_channel::Sender;

use spotlight_dimmer_core::primitives::Rect;

use crate::events::Event;

pub struct PaneTracker {
    tx: Sender<Event>,
}

impl PaneTracker {
    pub fn new(tx: Sender<Event>) -> Self {
        PaneTracker { tx }
    }
}

#[zbus::interface(name = "org.spotlightdimmer.PaneTracker")]
impl PaneTracker {
    /// Receive the focused pane rect for a tmux client, in pixels relative
    /// to the terminal content origin. Invalid input is silently ignored,
    /// matching the previous implementation.
    fn update_pane_geometry(&self, tty: String, x: i32, y: i32, width: i32, height: i32) {
        if tty.is_empty() || width <= 0 || height <= 0 {
            return;
        }

        let _ = self.tx.send_blocking(Event::PaneUpdated {
            tty,
            rect: Rect::new(x, y, width, height),
        });
    }

    /// Forget the pane rect for a tmux client.
    fn clear_pane(&self, tty: String) {
        let _ = self.tx.send_blocking(Event::PaneCleared { tty });
    }
}
