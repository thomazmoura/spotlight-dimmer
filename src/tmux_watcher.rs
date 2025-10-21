use std::fs;
use std::path::Path;

/// Information about the active tmux pane in character coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct TmuxPaneInfo {
    /// Left edge of pane in character columns
    pub pane_left: i32,
    /// Top edge of pane in character rows
    pub pane_top: i32,
    /// Right edge of pane in character columns (inclusive)
    pub pane_right: i32,
    /// Bottom edge of pane in character rows (inclusive)
    pub pane_bottom: i32,
    /// Total window width in character columns
    pub window_width: i32,
    /// Total window height in character rows
    pub window_height: i32,
}

impl TmuxPaneInfo {
    /// Parse tmux pane info from a comma-separated string
    /// Expected format: "left,top,right,bottom,window_width,window_height"
    /// Example: "0,0,119,29,240,60" means pane at (0,0) to (119,29) in a 240x60 window
    pub fn parse(content: &str) -> Result<Self, String> {
        let trimmed = content.trim();

        if trimmed.is_empty() {
            return Err("Empty content".to_string());
        }

        let parts: Vec<&str> = trimmed.split(',').collect();

        if parts.len() != 6 {
            return Err(format!(
                "Expected 6 comma-separated values, got {}",
                parts.len()
            ));
        }

        let parse_int = |s: &str, name: &str| -> Result<i32, String> {
            s.trim()
                .parse::<i32>()
                .map_err(|e| format!("Failed to parse {}: {}", name, e))
        };

        Ok(Self {
            pane_left: parse_int(parts[0], "pane_left")?,
            pane_top: parse_int(parts[1], "pane_top")?,
            pane_right: parse_int(parts[2], "pane_right")?,
            pane_bottom: parse_int(parts[3], "pane_bottom")?,
            window_width: parse_int(parts[4], "window_width")?,
            window_height: parse_int(parts[5], "window_height")?,
        })
    }

    /// Read and parse tmux pane info from a file
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read tmux pane file: {}", e))?;

        Self::parse(&content)
    }

    /// Get the width of the active pane in character columns
    pub fn pane_width(&self) -> i32 {
        self.pane_right - self.pane_left + 1
    }

    /// Get the height of the active pane in character rows
    pub fn pane_height(&self) -> i32 {
        self.pane_bottom - self.pane_top + 1
    }

    /// Check if the pane coordinates are valid
    pub fn is_valid(&self) -> bool {
        self.pane_left >= 0
            && self.pane_top >= 0
            && self.pane_right >= self.pane_left
            && self.pane_bottom >= self.pane_top
            && self.window_width > 0
            && self.window_height > 0
            && self.pane_right < self.window_width
            && self.pane_bottom < self.window_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let info = TmuxPaneInfo::parse("0,0,119,29,240,60").unwrap();
        assert_eq!(info.pane_left, 0);
        assert_eq!(info.pane_top, 0);
        assert_eq!(info.pane_right, 119);
        assert_eq!(info.pane_bottom, 29);
        assert_eq!(info.window_width, 240);
        assert_eq!(info.window_height, 60);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let info = TmuxPaneInfo::parse(" 10, 5, 100, 25, 200, 50 ").unwrap();
        assert_eq!(info.pane_left, 10);
        assert_eq!(info.pane_top, 5);
    }

    #[test]
    fn test_parse_empty() {
        let result = TmuxPaneInfo::parse("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty"));
    }

    #[test]
    fn test_parse_invalid_count() {
        let result = TmuxPaneInfo::parse("1,2,3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 6"));
    }

    #[test]
    fn test_parse_invalid_number() {
        let result = TmuxPaneInfo::parse("a,b,c,d,e,f");
        assert!(result.is_err());
    }

    #[test]
    fn test_pane_width() {
        let info = TmuxPaneInfo::parse("10,5,50,25,100,50").unwrap();
        assert_eq!(info.pane_width(), 41); // 50 - 10 + 1
    }

    #[test]
    fn test_pane_height() {
        let info = TmuxPaneInfo::parse("10,5,50,25,100,50").unwrap();
        assert_eq!(info.pane_height(), 21); // 25 - 5 + 1
    }

    #[test]
    fn test_is_valid_true() {
        let info = TmuxPaneInfo::parse("0,0,119,29,240,60").unwrap();
        assert!(info.is_valid());
    }

    #[test]
    fn test_is_valid_false_negative_coords() {
        let info = TmuxPaneInfo {
            pane_left: -1,
            pane_top: 0,
            pane_right: 100,
            pane_bottom: 50,
            window_width: 200,
            window_height: 100,
        };
        assert!(!info.is_valid());
    }

    #[test]
    fn test_is_valid_false_right_less_than_left() {
        let info = TmuxPaneInfo {
            pane_left: 100,
            pane_top: 0,
            pane_right: 50,
            pane_bottom: 50,
            window_width: 200,
            window_height: 100,
        };
        assert!(!info.is_valid());
    }

    #[test]
    fn test_is_valid_false_out_of_bounds() {
        let info = TmuxPaneInfo {
            pane_left: 0,
            pane_top: 0,
            pane_right: 250, // Exceeds window_width
            pane_bottom: 50,
            window_width: 200,
            window_height: 100,
        };
        assert!(!info.is_valid());
    }

    #[test]
    fn test_equality() {
        let info1 = TmuxPaneInfo::parse("0,0,100,50,200,100").unwrap();
        let info2 = TmuxPaneInfo::parse("0,0,100,50,200,100").unwrap();
        let info3 = TmuxPaneInfo::parse("0,0,100,51,200,100").unwrap();

        assert_eq!(info1, info2);
        assert_ne!(info1, info3);
    }

    #[test]
    fn test_clone() {
        let info1 = TmuxPaneInfo::parse("0,0,100,50,200,100").unwrap();
        let info2 = info1.clone();
        assert_eq!(info1, info2);
    }
}
