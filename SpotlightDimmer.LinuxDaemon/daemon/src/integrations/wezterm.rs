//! Resolve the focused wezterm pane and verify it hosts a live tmux client.
//!
//! Port of `_queryWezTerm` from the GNOME extension's appIntegrations.js.
//! Three async subprocesses, chained; every failure path resolves to `None`
//! so the caller falls back to whole-window highlighting:
//! 1. `wezterm cli list-clients` -> focused_pane_id
//! 2. `wezterm cli list`         -> that pane's tty_name and cell origin
//! 3. `tmux list-clients`        -> tty must belong to a live tmux client

use std::cell::Cell;
use std::ffi::OsStr;

use serde_json::Value;

/// The resolved focused wezterm pane hosting a tmux client.
/// `offset_x/y` is the pane's cell-grid origin within the terminal content
/// (non-zero for wezterm-native splits; 0 when tmux fills the tab).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePane {
    pub tty: String,
    pub offset_x: i32,
    pub offset_y: i32,
}

/// Run the 3-stage query. `current`/`generation` implement the invalidation
/// pattern from the JS: the caller bumps `current` whenever focus or title
/// moves on, and the chain aborts at the next checkpoint (skipping the
/// remaining subprocess spawns).
pub async fn query_active_pane(current: &Cell<u64>, generation: u64) -> Option<ActivePane> {
    let clients = spawn_json(&["wezterm", "cli", "list-clients", "--format", "json"]).await;
    if current.get() != generation {
        return None;
    }

    let clients = clients?;
    let focused_pane_id = clients.as_array()?.iter().find_map(|c| {
        let id = c.get("focused_pane_id")?;
        (!id.is_null()).then(|| id.clone())
    })?;

    let panes = spawn_json(&["wezterm", "cli", "list", "--format", "json"]).await;
    if current.get() != generation {
        return None;
    }

    let panes = panes?;
    let pane = panes
        .as_array()?
        .iter()
        .find(|p| p.get("pane_id") == Some(&focused_pane_id))?;
    let tty = pane.get("tty_name")?.as_str()?;
    if tty.is_empty() {
        return None;
    }

    // Offset of this wezterm pane's cell grid within the window; cell pixel
    // size is derived from the pane's own reported size.
    let size = pane.get("size");
    let cell_size = |pixels: &str, cells: &str| -> f64 {
        let Some(size) = size else { return 0.0 };
        let count = size.get(cells).and_then(Value::as_f64).unwrap_or(0.0);
        if count > 0.0 {
            size.get(pixels).and_then(Value::as_f64).unwrap_or(0.0) / count
        } else {
            0.0
        }
    };
    let cell_w = cell_size("pixel_width", "cols");
    let cell_h = cell_size("pixel_height", "rows");
    let left_col = pane.get("left_col").and_then(Value::as_f64).unwrap_or(0.0);
    let top_row = pane.get("top_row").and_then(Value::as_f64).unwrap_or(0.0);
    let offset_x = (left_col * cell_w).round() as i32;
    let offset_y = (top_row * cell_h).round() as i32;

    let ttys = spawn_lines(&["tmux", "list-clients", "-F", "#{client_tty}"]).await;
    if current.get() != generation {
        return None;
    }

    // Focused wezterm pane must be a live tmux client (not a plain shell
    // tab, and the tmux server must still be running).
    let ttys = ttys?;
    ttys.iter().any(|t| t == tty).then(|| ActivePane {
        tty: tty.to_string(),
        offset_x,
        offset_y,
    })
}

/// Spawn a subprocess and parse its stdout as JSON (None on any failure).
async fn spawn_json(argv: &[&str]) -> Option<Value> {
    let stdout = spawn_capture(argv).await?;
    serde_json::from_str(&stdout).ok()
}

/// Spawn a subprocess and split its stdout into trimmed non-empty lines.
async fn spawn_lines(argv: &[&str]) -> Option<Vec<String>> {
    let stdout = spawn_capture(argv).await?;
    Some(
        stdout
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

/// Spawn a subprocess asynchronously; resolves to its stdout, or None when
/// the binary is missing or the process fails. Never blocks the event loop.
async fn spawn_capture(argv: &[&str]) -> Option<String> {
    let argv_os: Vec<&OsStr> = argv.iter().map(OsStr::new).collect();

    let process = gio::Subprocess::newv(
        &argv_os,
        gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_SILENCE,
    )
    .ok()?;

    let (stdout, _stderr) = process.communicate_utf8_future(None).await.ok()?;

    if !process.is_successful() {
        return None;
    }

    stdout.map(|s| s.to_string())
}
