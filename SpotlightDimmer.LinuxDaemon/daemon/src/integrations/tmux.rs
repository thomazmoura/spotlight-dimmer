//! The tmux side shared by every tty source: what a resolved pane looks like,
//! and which ttys currently have a live tmux client attached.

use crate::integrations::proc::spawn_lines;

/// The resolved focused terminal pane hosting a tmux client.
/// `offset_x/y` is the pane's cell-grid origin within the terminal content
/// (non-zero for wezterm-native splits; 0 when tmux fills the surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePane {
    pub tty: String,
    pub offset_x: i32,
    pub offset_y: i32,
}

/// The ttys of all attached tmux clients, or None when tmux is not installed
/// and when no server is running (both make `tmux list-clients` fail).
pub async fn client_ttys() -> Option<Vec<String>> {
    spawn_lines(&["tmux", "list-clients", "-F", "#{client_tty}"]).await
}
