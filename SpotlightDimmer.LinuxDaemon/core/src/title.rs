//! Extract a tmux client tty published in the terminal window title.
//!
//! Terminals without a pane-query CLI (Ghostty) cannot tell the daemon which
//! tty the focused surface hosts, so tmux publishes it instead: the shipped
//! `set-titles-string` snippet appends `<U+2063>sd:#{client_tty}` to the
//! title. U+2063 (INVISIBLE SEPARATOR) renders as nothing, so the window
//! title still looks untouched in the window list.

/// Marker introducing the tty in a window title.
pub const TTY_MARKER: &str = "\u{2063}sd:";

/// The tty announced by the last marker in `title`, if any. Values that are
/// not device paths are rejected so an ordinary title (or a stale marker with
/// an empty format expansion) can never be mistaken for a tty.
pub fn parse_tty(title: &str) -> Option<&str> {
    let start = title.rfind(TTY_MARKER)? + TTY_MARKER.len();
    let tty = title[start..]
        .split(|c: char| c.is_whitespace())
        .next()
        .unwrap_or("");

    tty.starts_with("/dev/").then_some(tty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_marker_yields_nothing() {
        assert_eq!(parse_tty("nvim ~/code/spotlight-dimmer"), None);
        assert_eq!(parse_tty(""), None);
    }

    #[test]
    fn reads_a_marker_at_the_end_of_the_title() {
        assert_eq!(
            parse_tty("0:1:zsh - \"nvim\"\u{2063}sd:/dev/pts/7"),
            Some("/dev/pts/7")
        );
    }

    #[test]
    fn stops_at_the_first_space_after_the_marker() {
        assert_eq!(
            parse_tty("session\u{2063}sd:/dev/pts/3 trailing words"),
            Some("/dev/pts/3")
        );
    }

    #[test]
    fn the_last_marker_wins() {
        // A pane title echoing an old marker must not shadow the real one
        assert_eq!(
            parse_tty("\u{2063}sd:/dev/pts/1 - shell\u{2063}sd:/dev/pts/9"),
            Some("/dev/pts/9")
        );
    }

    #[test]
    fn non_device_values_are_rejected() {
        // tmux expanded the format outside a client, or the title was truncated
        assert_eq!(parse_tty("zsh\u{2063}sd:"), None);
        assert_eq!(parse_tty("zsh\u{2063}sd: /dev/pts/7"), None);
        assert_eq!(parse_tty("zsh\u{2063}sd:none"), None);
    }
}
