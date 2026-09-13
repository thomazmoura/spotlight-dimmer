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

/// Marker introducing a focused neovim split in a pane title.
///
/// A neovim on the other end of an ssh cannot reach the local tmux, so the
/// SpotlightDimmer neovim plugin publishes the split in its terminal title
/// instead; tmux copies the pane title into the window title (`#T` in
/// `set-titles-string`). The daemon cannot use the rect itself (it is in
/// cells, relative to a tmux pane), but a change to it is the only signal
/// that the split moved, so the daemon asks tmux for fresh pane geometry.
pub const NVIM_MARKER: &str = "sd-nvim=";

/// The neovim split payload (`cols,rows,col,row,width,height`) announced by
/// the last marker in `title`, if any. Anything other than exactly six
/// comma-separated unsigned integers is rejected.
pub fn parse_nvim_rect(title: &str) -> Option<&str> {
    let start = title.rfind(NVIM_MARKER)? + NVIM_MARKER.len();
    let rect = title[start..]
        .split(|c: char| !(c.is_ascii_digit() || c == ','))
        .next()
        .unwrap_or("");

    let mut fields = 0;
    for field in rect.split(',') {
        if field.is_empty() {
            return None;
        }
        fields += 1;
    }

    (fields == 6).then_some(rect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nvim_rect_is_read_from_anywhere_in_the_title() {
        assert_eq!(
            parse_nvim_rect("s:1:w - \"nvim-nav=hl sd-nvim=160,41,0,0,81,40\" \u{2063}sd:/dev/pts/3"),
            Some("160,41,0,0,81,40")
        );
        assert_eq!(parse_nvim_rect("sd-nvim=80,24,40,0,40,23"), Some("80,24,40,0,40,23"));
    }

    #[test]
    fn nvim_rect_missing_or_malformed_yields_nothing() {
        assert_eq!(parse_nvim_rect("nvim-nav=hl"), None);
        assert_eq!(parse_nvim_rect("sd-nvim="), None);
        assert_eq!(parse_nvim_rect("sd-nvim=1,2,3,4,5"), None);
        assert_eq!(parse_nvim_rect("sd-nvim=1,2,3,4,5,6,7"), None);
        assert_eq!(parse_nvim_rect("sd-nvim=1,2,,4,5,6"), None);
        assert_eq!(parse_nvim_rect("sd-nvim=-1,2,3,4,5,6"), None);
    }

    #[test]
    fn nvim_rect_last_marker_wins() {
        assert_eq!(
            parse_nvim_rect("sd-nvim=1,1,1,1,1,1 then sd-nvim=2,2,2,2,2,2"),
            Some("2,2,2,2,2,2")
        );
    }

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
