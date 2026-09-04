//! Events flowing from D-Bus handlers, file monitors and async queries into
//! the single-threaded daemon event loop, which owns all mutable state.

use spotlight_dimmer_core::primitives::Rect;
use spotlight_dimmer_core::state::Monitor;

use crate::integrations::herdr::HerdrLayout;
use crate::integrations::tmux::ActivePane;

#[derive(Debug)]
pub enum Event {
    /// An adapter called RegisterAdapter. `sender` is its unique bus name.
    AdapterRegistered {
        sender: String,
        compositor: String,
        renders_overlays: bool,
    },
    /// A renderer called RegisterRenderer (it already received the current
    /// snapshot synchronously; this is for logging/bookkeeping).
    RendererRegistered {
        sender: String,
    },
    /// A registered adapter's unique bus name vanished.
    AdapterLost {
        sender: String,
    },
    MonitorsUpdated {
        sender: String,
        monitors: Vec<Monitor>,
    },
    FocusChanged {
        sender: String,
        wm_class: String,
        title: String,
        frame: Rect,
        /// Client-area rect (decorations excluded); `None` from protocol v1
        /// adapters.
        client: Option<Rect>,
    },
    FocusCleared {
        sender: String,
    },
    GeometryChanged {
        sender: String,
        frame: Rect,
        client: Option<Rect>,
    },
    /// The focused window's title changed (tmux attach/detach and wezterm tab
    /// switches change the title without any focus/geometry event). The title
    /// itself matters for the window-title tty source.
    TitleChanged {
        sender: String,
        title: String,
    },
    /// Title-change debounce elapsed; requery the pane if still relevant.
    RequeryPane {
        epoch: u64,
    },
    /// PaneTracker D-Bus: focused tmux pane rect for a client tty (pixels
    /// relative to the terminal content origin).
    PaneUpdated {
        tty: String,
        rect: Rect,
    },
    PaneCleared {
        tty: String,
    },
    /// The Herdr reader thread published a new focused layout (in cells), or
    /// `None` when the session went away and the whole window applies again.
    HerdrLayout(Option<HerdrLayout>),
    /// The async pane query chain resolved (or failed -> None).
    PaneResolved {
        generation: u64,
        pane: Option<ActivePane>,
    },
    ConfigFileChanged,
    /// Enabled flag flipped via Toggle() or the Enabled property.
    EnabledChanged(bool),
}
