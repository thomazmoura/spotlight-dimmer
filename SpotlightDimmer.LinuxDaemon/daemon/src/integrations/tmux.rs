//! The tmux side shared by every tty source: what a resolved pane looks like,
//! which ttys currently have a live tmux client attached, and asking the
//! report script for fresh pane geometry.

use std::path::PathBuf;

use spotlight_dimmer_core::primitives::Rect;

use crate::integrations::proc::{spawn_capture, spawn_lines};

const REPORT_SCRIPT: &str = "spotlight-dimmer-tmux-report.sh";

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

/// The installed report script: the per-user copy (`make install-tools`)
/// first, then the .deb location.
fn report_script() -> Option<PathBuf> {
    let user = glib::user_config_dir()
        .join("SpotlightDimmer")
        .join("tools")
        .join(REPORT_SCRIPT);
    let system = PathBuf::from("/usr/share/spotlight-dimmer/tools").join(REPORT_SCRIPT);

    [user, system].into_iter().find(|p| p.is_file())
}

/// Run the report script for one tmux client and return what it would have
/// sent over D-Bus: the client's focused pane (narrowed to its neovim split,
/// when one is published) in pixels relative to the terminal content origin.
/// This is the same computation the tmux hooks trigger, requested from the
/// daemon side for changes no tmux hook sees (a neovim over ssh switching
/// splits only changes its pane title).
pub async fn report_pane(tty: &str) -> Option<Rect> {
    let script = report_script()?;
    let script = script.to_str()?;
    let stdout = spawn_capture(&[script, "--client", tty, "--print"]).await?;
    parse_report(&stdout, tty)
}

/// Parse `--print` output (`<tty> <x> <y> <width> <height>`). The tty must
/// match the one asked about; an empty or malformed answer means the script
/// had nothing to report.
fn parse_report(stdout: &str, tty: &str) -> Option<Rect> {
    let mut fields = stdout.split_whitespace();
    if fields.next()? != tty {
        return None;
    }

    let mut next = || fields.next()?.parse::<i32>().ok();
    let (x, y, width, height) = (next()?, next()?, next()?, next()?);
    (width > 0 && height > 0).then(|| Rect::new(x, y, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_print_output() {
        assert_eq!(
            parse_report("/dev/pts/3 800 0 420 800\n", "/dev/pts/3"),
            Some(Rect::new(800, 0, 420, 800))
        );
    }

    #[test]
    fn rejects_other_ttys_and_garbage() {
        assert_eq!(parse_report("/dev/pts/4 800 0 420 800", "/dev/pts/3"), None);
        assert_eq!(parse_report("", "/dev/pts/3"), None);
        assert_eq!(parse_report("/dev/pts/3 800 0 x 800", "/dev/pts/3"), None);
        assert_eq!(parse_report("/dev/pts/3 800 0 0 800", "/dev/pts/3"), None);
    }
}
