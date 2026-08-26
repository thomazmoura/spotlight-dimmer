//! Async subprocess helpers shared by the tty sources. Every failure path
//! resolves to `None` so callers fall back to whole-window highlighting.

use std::ffi::OsStr;

use serde_json::Value;

/// Spawn a subprocess and parse its stdout as JSON (None on any failure).
pub async fn spawn_json(argv: &[&str]) -> Option<Value> {
    let stdout = spawn_capture(argv).await?;
    serde_json::from_str(&stdout).ok()
}

/// Spawn a subprocess and split its stdout into trimmed non-empty lines.
pub async fn spawn_lines(argv: &[&str]) -> Option<Vec<String>> {
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
pub async fn spawn_capture(argv: &[&str]) -> Option<String> {
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
