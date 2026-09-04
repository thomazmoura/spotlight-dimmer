//! The "herdr" provider: the focused pane's cell layout, read from a Herdr
//! session socket.
//!
//! Herdr has no `set-hook` equivalent that could push geometry the way the
//! tmux snippet does, but it does expose a JSON socket API with an event
//! stream. So the push side lives here instead of in a hook script: one
//! background thread holds an `events.subscribe` connection and re-reads
//! `pane.layout` whenever focus or layout changes. Still no polling.
//!
//! Two things differ from the tmux provider:
//! - geometry arrives in terminal **cells**, not pixels (Herdr reports no
//!   cell pixel size outside its experimental graphics API), so the join in
//!   `core::pane::pane_rect_from_cells` scales the grid onto the window
//! - the join key is the workspace published in the window title
//!   (`core::title::HERDR_MARKER`), because Herdr's panes are its own ptys
//!   and there is no client tty to match on
//!
//! The protocol is one JSON request per line; a request connection answers
//! with a single line, while a subscription keeps streaming events.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use async_channel::Sender;
use serde::Deserialize;

use spotlight_dimmer_core::primitives::Rect;

use crate::events::Event;

/// Retry cadence while Herdr is not running (it is a long-lived session, so
/// there is no hurry).
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Events after which the focused layout may have moved. `pane.updated` is
/// deliberately absent: it fires on every title/agent-status change.
const SUBSCRIPTIONS: [&str; 7] = [
    "pane.focused",
    "tab.focused",
    "workspace.focused",
    "layout.updated",
    "pane.created",
    "pane.closed",
    "pane.moved",
];

/// The focused pane as Herdr sees it, in cell coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HerdrLayout {
    pub workspace_id: String,
    pub workspace_label: String,
    /// Total cell grid Herdr draws on (columns, rows), sidebar and tab bar
    /// included.
    pub grid: (i32, i32),
    /// Focused pane rect within that grid.
    pub pane: Rect,
}

/// Start the reader thread. Runs for the process lifetime, reconnecting
/// whenever the Herdr session goes away and comes back.
pub fn spawn(socket_path: &str, tx: Sender<Event>) {
    let path = resolve_socket_path(socket_path);
    println!(
        "SpotlightDimmer: herdr provider watching {}",
        path.display()
    );

    thread::Builder::new()
        .name(String::from("herdr"))
        .spawn(move || loop {
            read_session(&path, &tx);
            // The session ended (or never started): fall back to the
            // whole-window spotlight until it comes back.
            let _ = tx.send_blocking(Event::HerdrLayout(None));
            thread::sleep(RECONNECT_DELAY);
        })
        .expect("failed to spawn the herdr reader thread");
}

/// `SocketPath` when configured, else `$HERDR_SOCKET_PATH` (set inside Herdr
/// panes), else the default session socket. Named sessions keep theirs under
/// `~/.config/herdr/sessions/<name>/herdr.sock`.
fn resolve_socket_path(configured: &str) -> PathBuf {
    if !configured.is_empty() {
        return PathBuf::from(configured);
    }

    if let Some(from_env) = std::env::var_os("HERDR_SOCKET_PATH") {
        return PathBuf::from(from_env);
    }

    glib::user_config_dir().join("herdr").join("herdr.sock")
}

/// Hold one subscription connection and republish the layout after every
/// event, until the connection drops.
fn read_session(path: &Path, tx: &Sender<Event>) {
    let Ok(stream) = UnixStream::connect(path) else {
        return;
    };

    let request = format!(
        r#"{{"id":"spotlight-dimmer","method":"events.subscribe","params":{{"subscriptions":[{}]}}}}"#,
        SUBSCRIPTIONS
            .iter()
            .map(|kind| format!(r#"{{"type":"{kind}"}}"#))
            .collect::<Vec<_>>()
            .join(",")
    );

    let mut writer = &stream;
    if writeln!(writer, "{request}").is_err() {
        return;
    }

    println!("SpotlightDimmer: herdr session connected");
    let mut last: Option<HerdrLayout> = None;

    // Subscribing replays a backlog, so the same layout arrives repeatedly:
    // only publish actual changes to keep overlay recomputes down.
    for line in BufReader::new(&stream).lines() {
        if line.is_err() {
            break;
        }

        let layout = query_layout(path);
        if layout != last {
            last = layout.clone();
            if tx.send_blocking(Event::HerdrLayout(layout)).is_err() {
                return;
            }
        }
    }

    println!("SpotlightDimmer: herdr session disconnected");
}

/// Read the focused layout (and its workspace label) over two short-lived
/// connections. `None` whenever Herdr cannot name a focused pane.
fn query_layout(path: &Path) -> Option<HerdrLayout> {
    let response = request_line(path, r#"{"id":"sd","method":"pane.layout","params":{}}"#)?;
    let mut layout = parse_layout(&response)?;

    layout.workspace_label = query_workspace_label(path, &layout.workspace_id).unwrap_or_default();
    Some(layout)
}

/// Turn a `pane.layout` response into the focused layout, without the
/// workspace label (which takes a second request).
fn parse_layout(response: &str) -> Option<HerdrLayout> {
    let response: Response<LayoutResult> = serde_json::from_str(response).ok()?;
    let layout = response.result?.layout;

    let pane = layout.panes.iter().find(|p| p.focused)?;
    let area = &layout.area;

    Some(HerdrLayout {
        workspace_id: layout.workspace_id.clone(),
        workspace_label: String::new(),
        // Herdr lays the pane area out after the sidebar (x) and the tab bar
        // (y), so the full surface is the area plus its own origin.
        grid: (area.x + area.width, area.y + area.height),
        pane: Rect::new(pane.rect.x, pane.rect.y, pane.rect.width, pane.rect.height),
    })
}

fn query_workspace_label(path: &Path, workspace_id: &str) -> Option<String> {
    let line = format!(
        r#"{{"id":"sd","method":"workspace.get","params":{{"workspace_id":"{workspace_id}"}}}}"#
    );
    let response = request_line(path, &line)?;
    let response: Response<WorkspaceResult> = serde_json::from_str(&response).ok()?;
    response.result?.workspace.label
}

/// One request, one response line, one connection.
fn request_line(path: &Path, line: &str) -> Option<String> {
    let stream = UnixStream::connect(path).ok()?;
    let mut writer = &stream;
    writeln!(writer, "{line}").ok()?;

    let mut response = String::new();
    BufReader::new(&stream).read_line(&mut response).ok()?;
    Some(response)
}

/// Only the fields this provider needs; an error response simply leaves
/// `result` empty.
#[derive(Deserialize)]
struct Response<T> {
    result: Option<T>,
}

#[derive(Deserialize)]
struct LayoutResult {
    layout: Layout,
}

#[derive(Deserialize)]
struct Layout {
    workspace_id: String,
    area: CellRect,
    panes: Vec<PaneEntry>,
}

#[derive(Deserialize)]
struct PaneEntry {
    focused: bool,
    rect: CellRect,
}

#[derive(Deserialize)]
struct CellRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Deserialize)]
struct WorkspaceResult {
    workspace: WorkspaceInfo,
}

#[derive(Deserialize)]
struct WorkspaceInfo {
    label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Captured from `herdr pane layout` (Herdr 0.8.2, protocol 20) with the
    // pane area split in two: a 26-column sidebar and a one-row tab bar.
    const SPLIT_LAYOUT: &str = r#"{"id":"sd","result":{"type":"pane_layout","layout":{
        "workspace_id":"w18","tab_id":"w18:t1","zoomed":false,
        "area":{"x":26,"y":1,"width":187,"height":55},
        "focused_pane_id":"w18:p1",
        "panes":[
            {"pane_id":"w18:p1","focused":true,"rect":{"x":26,"y":1,"width":94,"height":55}},
            {"pane_id":"w18:p4","focused":false,"rect":{"x":120,"y":1,"width":93,"height":55}}
        ],
        "splits":[{"direction":"right","id":"split_0_root","ratio":0.5,
                   "rect":{"x":26,"y":1,"width":187,"height":55}}]}}}"#;

    #[test]
    fn reads_the_focused_pane_and_the_full_grid() {
        let layout = parse_layout(SPLIT_LAYOUT).unwrap();
        assert_eq!(layout.workspace_id, "w18");
        assert_eq!(layout.pane, Rect::new(26, 1, 94, 55));
        // Sidebar (26 columns) and tab bar (1 row) are part of the surface
        assert_eq!(layout.grid, (213, 56));
    }

    #[test]
    fn a_layout_without_a_focused_pane_is_ignored() {
        let unfocused = SPLIT_LAYOUT.replace("\"focused\":true", "\"focused\":false");
        assert!(parse_layout(&unfocused).is_none());
    }

    #[test]
    fn error_responses_are_ignored() {
        assert!(
            parse_layout(r#"{"id":"sd","error":{"code":"unknown","message":"nope"}}"#).is_none()
        );
        assert!(parse_layout("not json at all").is_none());
    }
}
