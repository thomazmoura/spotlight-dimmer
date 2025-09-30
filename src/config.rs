use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OverlayColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32, // opacity from 0.0 to 1.0
}

impl Default for OverlayColor {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0.5,
        }
    }
}

impl OverlayColor {
    pub fn to_colorref(&self) -> u32 {
        // Windows COLORREF is 0x00bbggrr
        ((self.b as u32) << 16) | ((self.g as u32) << 8) | (self.r as u32)
    }

    pub fn to_alpha_byte(&self) -> u8 {
        (self.a * 255.0) as u8
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub overlay_color: OverlayColor,
    pub is_dimming_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            overlay_color: OverlayColor::default(),
            is_dimming_enabled: true,
        }
    }
}

impl Config {
    /// Get the path to the config file in the user's AppData directory
    pub fn config_path() -> Result<PathBuf, String> {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Failed to get APPDATA environment variable".to_string())?;

        let config_dir = PathBuf::from(appdata).join("spotlight-dimmer");

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(config_dir.join("config.toml"))
    }

    /// Load configuration from disk, or create default if not found
    pub fn load() -> Self {
        match Self::config_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match toml::from_str(&content) {
                                Ok(config) => {
                                    println!("[Config] Loaded from {:?}", path);
                                    return config;
                                }
                                Err(e) => {
                                    eprintln!("[Config] Failed to parse config file: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[Config] Failed to read config file: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Config] Failed to get config path: {}", e);
            }
        }

        // Return default if loading failed
        println!("[Config] Using default configuration");
        Self::default()
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        println!("[Config] Saved to {:?}", path);
        Ok(())
    }

    /// Get the last modification time of the config file
    pub fn last_modified() -> Option<SystemTime> {
        if let Ok(path) = Self::config_path() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    return Some(modified);
                }
            }
        }
        None
    }

    /// Reload configuration if the file has been modified since last check
    /// Returns Some((new_config, new_modified_time)) if changed, None if unchanged or error
    pub fn reload_if_changed(last_modified: Option<SystemTime>) -> Option<(Self, SystemTime)> {
        // Check current modification time
        let current_modified = Self::last_modified()?;

        // If we have a last_modified time and it matches current, no change
        if let Some(last) = last_modified {
            if last == current_modified {
                return None;
            }
        }

        // File was modified or this is first check - reload
        let new_config = Self::load();
        println!("[Config] Reloaded from disk due to file change");
        Some((new_config, current_modified))
    }
}