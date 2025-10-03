use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
pub struct Profile {
    pub overlay_color: OverlayColor,
    pub is_dimming_enabled: bool,
    pub active_overlay_color: Option<OverlayColor>,
    pub is_active_overlay_enabled: bool,
    pub is_partial_dimming_enabled: bool,
}

impl Profile {
    /// Create a profile from current config settings
    pub fn from_config(config: &Config) -> Self {
        Self {
            overlay_color: config.overlay_color.clone(),
            is_dimming_enabled: config.is_dimming_enabled,
            active_overlay_color: config.active_overlay_color.clone(),
            is_active_overlay_enabled: config.is_active_overlay_enabled,
            is_partial_dimming_enabled: config.is_partial_dimming_enabled,
        }
    }

    /// Apply this profile to a config
    pub fn apply_to_config(&self, config: &mut Config) {
        config.overlay_color = self.overlay_color.clone();
        config.is_dimming_enabled = self.is_dimming_enabled;
        config.active_overlay_color = self.active_overlay_color.clone();
        config.is_active_overlay_enabled = self.is_active_overlay_enabled;
        config.is_partial_dimming_enabled = self.is_partial_dimming_enabled;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub overlay_color: OverlayColor,
    pub is_dimming_enabled: bool,
    pub active_overlay_color: Option<OverlayColor>,
    pub is_active_overlay_enabled: bool,
    #[serde(default)]
    pub is_paused: bool,
    #[serde(default)]
    pub is_partial_dimming_enabled: bool,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        
        // Add default light mode profile
        profiles.insert("light-mode".to_string(), Profile {
            overlay_color: OverlayColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0.5,
            },
            is_dimming_enabled: true,
            active_overlay_color: None,
            is_active_overlay_enabled: false,
            is_partial_dimming_enabled: true,
        });
        
        // Add default dark mode profile
        profiles.insert("dark-mode".to_string(), Profile {
            overlay_color: OverlayColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0.7,
            },
            is_dimming_enabled: true,
            active_overlay_color: Some(OverlayColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0.3,
            }),
            is_active_overlay_enabled: true,
            is_partial_dimming_enabled: true,
        });

        Self {
            overlay_color: OverlayColor::default(),
            is_dimming_enabled: true,
            active_overlay_color: None,
            is_active_overlay_enabled: false,
            is_paused: false,
            is_partial_dimming_enabled: false,
            profiles,
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
                            match toml::from_str::<Config>(&content) {
                                Ok(mut config) => {
                                    println!("[Config] Loaded from {:?}", path);
                                    // Add default profiles if profiles HashMap is empty
                                    if config.profiles.is_empty() {
                                        config.add_default_profiles();
                                    }
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

    /// Get a list of all profile names
    pub fn list_profiles(&self) -> Vec<String> {
        let mut names: Vec<String> = self.profiles.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Save current config as a profile
    pub fn save_profile(&mut self, name: String) {
        let profile = Profile::from_config(self);
        self.profiles.insert(name, profile);
    }

    /// Load a profile by name and apply it to current config
    pub fn load_profile(&mut self, name: &str) -> Result<(), String> {
        let profile = self.profiles.get(name)
            .ok_or_else(|| format!("Profile '{}' not found", name))?
            .clone();
        
        profile.apply_to_config(self);
        Ok(())
    }

    /// Delete a profile by name
    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        self.profiles.remove(name)
            .ok_or_else(|| format!("Profile '{}' not found", name))?;
        Ok(())
    }

    /// Add default profiles if they don't exist
    fn add_default_profiles(&mut self) {
        // Add default light mode profile
        self.profiles.insert("light-mode".to_string(), Profile {
            overlay_color: OverlayColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0.5,
            },
            is_dimming_enabled: true,
            active_overlay_color: None,
            is_active_overlay_enabled: false,
            is_partial_dimming_enabled: true,
        });
        
        // Add default dark mode profile
        self.profiles.insert("dark-mode".to_string(), Profile {
            overlay_color: OverlayColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0.7,
            },
            is_dimming_enabled: true,
            active_overlay_color: Some(OverlayColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0.3,
            }),
            is_active_overlay_enabled: true,
            is_partial_dimming_enabled: true,
        });
    }
}