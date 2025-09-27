// Platform-specific modules
#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
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
pub trait DisplayManager {
    fn get_displays(&self) -> Result<Vec<DisplayInfo>, String>;
    fn get_primary_display(&self) -> Result<DisplayInfo, String>;
}

pub trait WindowManager {
    fn get_active_window(&self) -> Result<ActiveWindowInfo, String>;
    fn get_window_display(&self, window_handle: u64) -> Result<DisplayInfo, String>;
}