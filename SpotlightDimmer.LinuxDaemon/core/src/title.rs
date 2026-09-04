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

/// Marker introducing the Herdr session key in a window title.
///
/// Herdr has no tty to publish (its panes are its own ptys), so the marker
/// carries the focused workspace instead: `ui.window_title` in Herdr's
/// config.toml is a template, and appending `\u{2063}sd:herdr:{workspace}`
/// both flags the window as a Herdr client and says which workspace it is
/// showing. See docs/HERDR_INTEGRATION.md.
pub const HERDR_MARKER: &str = "\u{2063}sd:herdr:";

/// The Herdr session key announced by the last marker in `title`, if any.
pub fn parse_herdr_key(title: &str) -> Option<&str> {
    let start = title.rfind(HERDR_MARKER)? + HERDR_MARKER.len();
    let key = title[start..]
        .split(|c: char| c.is_whitespace())
        .next()
        .unwrap_or("");

    (!key.is_empty()).then_some(key)
}

/// Whether a title marker key identifies the workspace Herdr currently has
/// focused. `*` (or a template Herdr left unexpanded) matches anything, which
/// is what a single-window setup wants; otherwise the key must name the
/// workspace by label or id. Labels are matched by prefix because the marker
/// stops at the first space, so a label with spaces is truncated in the title.
pub fn herdr_key_matches(key: &str, workspace_id: &str, workspace_label: &str) -> bool {
    key == "*"
        || key == "{workspace}"
        || key == workspace_id
        || (!key.is_empty() && workspace_label.starts_with(key))
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

#[cfg(test)]
mod herdr_tests {
    use super::*;

    #[test]
    fn reads_the_workspace_key() {
        assert_eq!(
            parse_herdr_key("abelha1621: spotlight-dimmer\u{2063}sd:herdr:spotlight-dimmer"),
            Some("spotlight-dimmer")
        );
        assert_eq!(parse_herdr_key("plain shell title"), None);
        // Marker with an empty expansion is not a key
        assert_eq!(parse_herdr_key("herdr\u{2063}sd:herdr:"), None);
    }

    #[test]
    fn the_tty_parser_ignores_a_herdr_marker() {
        // Both markers share the `sd:` prefix; a Herdr key must never be
        // mistaken for a tmux client tty.
        assert_eq!(parse_tty("herdr\u{2063}sd:herdr:default"), None);
    }

    #[test]
    fn keys_match_by_id_label_or_wildcard() {
        assert!(herdr_key_matches("*", "w18", "spotlight-dimmer"));
        assert!(herdr_key_matches("{workspace}", "w18", "spotlight-dimmer"));
        assert!(herdr_key_matches("w18", "w18", "spotlight-dimmer"));
        assert!(herdr_key_matches(
            "spotlight-dimmer",
            "w18",
            "spotlight-dimmer"
        ));
        // Truncated at the first space, so a spaced label matches by prefix
        assert!(herdr_key_matches("dev", "w17", "dev environment"));
        assert!(!herdr_key_matches("other", "w18", "spotlight-dimmer"));
    }
}
