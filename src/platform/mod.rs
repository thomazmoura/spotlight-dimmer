// Platform-specific modules
#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
#[allow(unused_imports)] // Re-export all Windows platform items
pub use windows::*;

// Future Linux support
// #[cfg(unix)]
// pub mod linux;

// #[cfg(unix)]
// pub use linux::*;

// Cross-platform types and traits
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    pub handle: u64,
    pub display_id: String,
    pub process_name: String,
    pub window_title: String,
}

// Platform abstraction traits
#[allow(dead_code)]
pub trait DisplayManager {
    fn get_displays(&self) -> Result<Vec<DisplayInfo>, String>;
    fn get_primary_display(&self) -> Result<DisplayInfo, String>;
    fn get_display_count(&self) -> Result<usize, String>;
}

#[allow(dead_code)]
pub trait WindowManager {
    fn get_active_window(&self) -> Result<ActiveWindowInfo, String>;
    fn get_window_display(&self, window_handle: u64) -> Result<DisplayInfo, String>;

    #[cfg(windows)]
    fn get_window_rect(&self, window_handle: u64) -> Result<winapi::shared::windef::RECT, String>;

    #[cfg(windows)]
    fn is_window_maximized(&self, window_handle: u64) -> Result<bool, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_info_creation() {
        let display = DisplayInfo {
            id: "display-1".to_string(),
            name: "Primary Monitor".to_string(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        };

        assert_eq!(display.id, "display-1");
        assert_eq!(display.name, "Primary Monitor");
        assert_eq!(display.x, 0);
        assert_eq!(display.y, 0);
        assert_eq!(display.width, 1920);
        assert_eq!(display.height, 1080);
        assert!(display.is_primary);
    }

    #[test]
    fn test_display_info_clone() {
        let display1 = DisplayInfo {
            id: "display-1".to_string(),
            name: "Monitor 1".to_string(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        };

        let display2 = display1.clone();

        assert_eq!(display1.id, display2.id);
        assert_eq!(display1.width, display2.width);
        assert_eq!(display1.is_primary, display2.is_primary);
    }

    #[test]
    fn test_display_info_serialization() {
        let display = DisplayInfo {
            id: "test".to_string(),
            name: "Test Display".to_string(),
            x: 100,
            y: 200,
            width: 1024,
            height: 768,
            is_primary: false,
        };

        // Test serialization
        let json = serde_json::to_string(&display).expect("Failed to serialize");
        assert!(json.contains("\"id\":\"test\""));
        assert!(json.contains("\"width\":1024"));

        // Test deserialization
        let deserialized: DisplayInfo = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.id, display.id);
        assert_eq!(deserialized.width, display.width);
    }

    #[test]
    fn test_active_window_info_creation() {
        let window = ActiveWindowInfo {
            handle: 12345678,
            display_id: "display-1".to_string(),
            process_name: "notepad.exe".to_string(),
            window_title: "Untitled - Notepad".to_string(),
        };

        assert_eq!(window.handle, 12345678);
        assert_eq!(window.display_id, "display-1");
        assert_eq!(window.process_name, "notepad.exe");
        assert_eq!(window.window_title, "Untitled - Notepad");
    }

    #[test]
    fn test_active_window_info_clone() {
        let window1 = ActiveWindowInfo {
            handle: 98765,
            display_id: "display-2".to_string(),
            process_name: "chrome.exe".to_string(),
            window_title: "Google Chrome".to_string(),
        };

        let window2 = window1.clone();

        assert_eq!(window1.handle, window2.handle);
        assert_eq!(window1.display_id, window2.display_id);
        assert_eq!(window1.process_name, window2.process_name);
    }

    #[test]
    fn test_active_window_info_serialization() {
        let window = ActiveWindowInfo {
            handle: 999,
            display_id: "test-display".to_string(),
            process_name: "test.exe".to_string(),
            window_title: "Test Window".to_string(),
        };

        // Test serialization
        let json = serde_json::to_string(&window).expect("Failed to serialize");
        assert!(json.contains("\"handle\":999"));
        assert!(json.contains("\"process_name\":\"test.exe\""));

        // Test deserialization
        let deserialized: ActiveWindowInfo =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.handle, window.handle);
        assert_eq!(deserialized.process_name, window.process_name);
    }

    #[test]
    fn test_display_info_negative_coordinates() {
        // Test secondary monitor with negative coordinates (common on Windows)
        let display = DisplayInfo {
            id: "display-2".to_string(),
            name: "Secondary Monitor".to_string(),
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: false,
        };

        assert_eq!(display.x, -1920);
        assert_eq!(display.y, 0);
        assert!(!display.is_primary);
    }

    #[test]
    fn test_display_info_vertical_layout() {
        // Test vertical monitor arrangement
        let display = DisplayInfo {
            id: "display-top".to_string(),
            name: "Top Monitor".to_string(),
            x: 0,
            y: -1080,
            width: 1920,
            height: 1080,
            is_primary: false,
        };

        assert_eq!(display.y, -1080);
    }

    #[test]
    fn test_display_info_various_resolutions() {
        // 4K display
        let display_4k = DisplayInfo {
            id: "4k".to_string(),
            name: "4K Monitor".to_string(),
            x: 0,
            y: 0,
            width: 3840,
            height: 2160,
            is_primary: true,
        };
        assert_eq!(display_4k.width, 3840);
        assert_eq!(display_4k.height, 2160);

        // 1080p display
        let display_1080p = DisplayInfo {
            id: "1080p".to_string(),
            name: "Full HD".to_string(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: false,
        };
        assert_eq!(display_1080p.width, 1920);

        // Ultrawide display
        let display_ultrawide = DisplayInfo {
            id: "ultrawide".to_string(),
            name: "Ultrawide".to_string(),
            x: 0,
            y: 0,
            width: 3440,
            height: 1440,
            is_primary: false,
        };
        assert_eq!(display_ultrawide.width, 3440);
        assert_eq!(display_ultrawide.height, 1440);
    }

    #[test]
    fn test_active_window_info_empty_title() {
        let window = ActiveWindowInfo {
            handle: 1,
            display_id: "display-1".to_string(),
            process_name: "system.exe".to_string(),
            window_title: "".to_string(),
        };

        assert_eq!(window.window_title, "");
    }

    #[test]
    fn test_active_window_info_special_characters() {
        let window = ActiveWindowInfo {
            handle: 1,
            display_id: "display-1".to_string(),
            process_name: "app.exe".to_string(),
            window_title: "File: C:\\Users\\Test\\Document.txt - Editor".to_string(),
        };

        assert!(window.window_title.contains("\\"));
        assert!(window.window_title.contains(":"));
    }

    #[test]
    fn test_display_info_debug_format() {
        let display = DisplayInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            is_primary: true,
        };

        let debug_str = format!("{:?}", display);
        assert!(debug_str.contains("DisplayInfo"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_active_window_info_debug_format() {
        let window = ActiveWindowInfo {
            handle: 123,
            display_id: "test".to_string(),
            process_name: "test.exe".to_string(),
            window_title: "Test".to_string(),
        };

        let debug_str = format!("{:?}", window);
        assert!(debug_str.contains("ActiveWindowInfo"));
        assert!(debug_str.contains("123"));
    }
}
