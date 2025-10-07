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
    #[allow(dead_code)]
    pub fn to_colorref(&self) -> u32 {
        // Windows COLORREF is 0x00bbggrr
        ((self.b as u32) << 16) | ((self.g as u32) << 8) | (self.r as u32)
    }

    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        profiles.insert(
            "light-mode".to_string(),
            Profile {
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
            },
        );

        // Add default dark mode profile
        profiles.insert(
            "dark-mode".to_string(),
            Profile {
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
            },
        );

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

        println!("[Config] Saved to {:?}", path);
        Ok(())
    }

    /// Get the last modification time of the config file
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn list_profiles(&self) -> Vec<String> {
        let mut names: Vec<String> = self.profiles.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get a profile by name
    #[allow(dead_code)]
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Save current config as a profile
    #[allow(dead_code)]
    pub fn save_profile(&mut self, name: String) {
        let profile = Profile::from_config(self);
        self.profiles.insert(name, profile);
    }

    /// Load a profile by name and apply it to current config
    #[allow(dead_code)]
    pub fn load_profile(&mut self, name: &str) -> Result<(), String> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| format!("Profile '{}' not found", name))?
            .clone();

        profile.apply_to_config(self);
        Ok(())
    }

    /// Delete a profile by name
    #[allow(dead_code)]
    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        self.profiles
            .remove(name)
            .ok_or_else(|| format!("Profile '{}' not found", name))?;
        Ok(())
    }

    /// Add default profiles if they don't exist
    fn add_default_profiles(&mut self) {
        // Add default light mode profile
        self.profiles.insert(
            "light-mode".to_string(),
            Profile {
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
            },
        );

        // Add default dark mode profile
        self.profiles.insert(
            "dark-mode".to_string(),
            Profile {
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
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a temporary config directory for testing
    #[allow(dead_code)]
    fn setup_test_config_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    /// Helper function to create a custom config path for testing
    #[allow(dead_code)]
    fn create_test_config(dir: &TempDir, content: &str) -> PathBuf {
        let config_path = dir.path().join("config.toml");
        let mut file = fs::File::create(&config_path).expect("Failed to create test config");
        file.write_all(content.as_bytes())
            .expect("Failed to write test config");
        config_path
    }

    #[test]
    fn test_overlay_color_default() {
        let color = OverlayColor::default();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 0.5);
    }

    #[test]
    fn test_overlay_color_to_colorref() {
        // Test black color
        let black = OverlayColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0.5,
        };
        assert_eq!(black.to_colorref(), 0x00000000);

        // Test white color
        let white = OverlayColor {
            r: 255,
            g: 255,
            b: 255,
            a: 1.0,
        };
        assert_eq!(white.to_colorref(), 0x00FFFFFF);

        // Test red color (Windows COLORREF is BGR format: 0x00bbggrr)
        let red = OverlayColor {
            r: 255,
            g: 0,
            b: 0,
            a: 0.5,
        };
        assert_eq!(red.to_colorref(), 0x000000FF);

        // Test green color
        let green = OverlayColor {
            r: 0,
            g: 255,
            b: 0,
            a: 0.5,
        };
        assert_eq!(green.to_colorref(), 0x0000FF00);

        // Test blue color
        let blue = OverlayColor {
            r: 0,
            g: 0,
            b: 255,
            a: 0.5,
        };
        assert_eq!(blue.to_colorref(), 0x00FF0000);

        // Test mixed color
        let purple = OverlayColor {
            r: 128,
            g: 0,
            b: 128,
            a: 0.7,
        };
        assert_eq!(purple.to_colorref(), 0x00800080);
    }

    #[test]
    fn test_overlay_color_to_alpha_byte() {
        let transparent = OverlayColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0.0,
        };
        assert_eq!(transparent.to_alpha_byte(), 0);

        let half = OverlayColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0.5,
        };
        assert_eq!(half.to_alpha_byte(), 127);

        let opaque = OverlayColor {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
        };
        assert_eq!(opaque.to_alpha_byte(), 255);

        let quarter = OverlayColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0.25,
        };
        assert_eq!(quarter.to_alpha_byte(), 63);
    }

    #[test]
    fn test_overlay_color_equality() {
        let color1 = OverlayColor {
            r: 100,
            g: 150,
            b: 200,
            a: 0.8,
        };
        let color2 = OverlayColor {
            r: 100,
            g: 150,
            b: 200,
            a: 0.8,
        };
        let color3 = OverlayColor {
            r: 100,
            g: 150,
            b: 201,
            a: 0.8,
        };

        assert_eq!(color1, color2);
        assert_ne!(color1, color3);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert_eq!(config.overlay_color, OverlayColor::default());
        assert!(config.is_dimming_enabled);
        assert_eq!(config.active_overlay_color, None);
        assert!(!config.is_active_overlay_enabled);
        assert!(!config.is_paused);
        assert!(!config.is_partial_dimming_enabled);

        // Check default profiles
        assert!(config.profiles.contains_key("light-mode"));
        assert!(config.profiles.contains_key("dark-mode"));
        assert_eq!(config.profiles.len(), 2);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize");

        // Verify it contains expected keys
        assert!(toml_str.contains("is_dimming_enabled"));
        assert!(toml_str.contains("overlay_color"));
        assert!(toml_str.contains("[profiles."));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_content = r#"
            is_dimming_enabled = true
            is_active_overlay_enabled = false
            is_paused = false
            is_partial_dimming_enabled = false

            [overlay_color]
            r = 50
            g = 100
            b = 150
            a = 0.7
        "#;

        let config: Config = toml::from_str(toml_content).expect("Failed to deserialize");

        assert!(config.is_dimming_enabled);
        assert_eq!(config.overlay_color.r, 50);
        assert_eq!(config.overlay_color.g, 100);
        assert_eq!(config.overlay_color.b, 150);
        assert_eq!(config.overlay_color.a, 0.7);
    }

    #[test]
    fn test_config_deserialization_with_defaults() {
        // Test that missing fields use #[serde(default)]
        let toml_content = r#"
            is_dimming_enabled = true
            is_active_overlay_enabled = false

            [overlay_color]
            r = 0
            g = 0
            b = 0
            a = 0.5
        "#;

        let config: Config = toml::from_str(toml_content).expect("Failed to deserialize");

        // These should use defaults since they're missing
        assert!(!config.is_paused);
        assert!(!config.is_partial_dimming_enabled);
        assert_eq!(config.profiles.len(), 0); // Should be empty, will be filled on load
    }

    #[test]
    fn test_profile_from_config() {
        let config = Config {
            overlay_color: OverlayColor {
                r: 10,
                g: 20,
                b: 30,
                a: 0.6,
            },
            is_dimming_enabled: false,
            is_partial_dimming_enabled: true,
            ..Default::default()
        };

        let profile = Profile::from_config(&config);

        assert_eq!(profile.overlay_color.r, 10);
        assert_eq!(profile.overlay_color.g, 20);
        assert_eq!(profile.overlay_color.b, 30);
        assert_eq!(profile.overlay_color.a, 0.6);
        assert!(!profile.is_dimming_enabled);
        assert!(profile.is_partial_dimming_enabled);
    }

    #[test]
    fn test_profile_apply_to_config() {
        let mut config = Config::default();

        let profile = Profile {
            overlay_color: OverlayColor {
                r: 100,
                g: 100,
                b: 100,
                a: 0.9,
            },
            is_dimming_enabled: false,
            active_overlay_color: Some(OverlayColor {
                r: 50,
                g: 50,
                b: 50,
                a: 0.2,
            }),
            is_active_overlay_enabled: true,
            is_partial_dimming_enabled: true,
        };

        profile.apply_to_config(&mut config);

        assert_eq!(config.overlay_color.r, 100);
        assert!(!config.is_dimming_enabled);
        assert!(config.is_active_overlay_enabled);
        assert!(config.is_partial_dimming_enabled);
        assert!(config.active_overlay_color.is_some());
        assert_eq!(config.active_overlay_color.unwrap().r, 50);
    }

    #[test]
    fn test_config_list_profiles() {
        let config = Config::default();
        let profiles = config.list_profiles();

        // Should be sorted alphabetically
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0], "dark-mode");
        assert_eq!(profiles[1], "light-mode");
    }

    #[test]
    fn test_config_get_profile() {
        let config = Config::default();

        let light_mode = config.get_profile("light-mode");
        assert!(light_mode.is_some());
        assert_eq!(light_mode.unwrap().overlay_color.a, 0.5);

        let dark_mode = config.get_profile("dark-mode");
        assert!(dark_mode.is_some());
        assert_eq!(dark_mode.unwrap().overlay_color.a, 0.7);

        let nonexistent = config.get_profile("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_config_save_profile() {
        let mut config = Config {
            overlay_color: OverlayColor {
                r: 200,
                g: 100,
                b: 50,
                a: 0.8,
            },
            is_dimming_enabled: false,
            ..Default::default()
        };

        config.save_profile("custom".to_string());

        assert!(config.profiles.contains_key("custom"));
        let custom_profile = config.get_profile("custom").unwrap();
        assert_eq!(custom_profile.overlay_color.r, 200);
        assert!(!custom_profile.is_dimming_enabled);
    }

    #[test]
    fn test_config_load_profile_success() {
        let mut config = Config::default();

        // Load dark-mode profile
        let result = config.load_profile("dark-mode");
        assert!(result.is_ok());

        // Verify config was updated
        assert_eq!(config.overlay_color.a, 0.7);
        assert!(config.is_dimming_enabled);
        assert!(config.active_overlay_color.is_some());
    }

    #[test]
    fn test_config_load_profile_not_found() {
        let mut config = Config::default();

        let result = config.load_profile("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_config_delete_profile_success() {
        let mut config = Config::default();

        // Should have 2 default profiles
        assert_eq!(config.profiles.len(), 2);

        let result = config.delete_profile("light-mode");
        assert!(result.is_ok());
        assert_eq!(config.profiles.len(), 1);
        assert!(!config.profiles.contains_key("light-mode"));
    }

    #[test]
    fn test_config_delete_profile_not_found() {
        let mut config = Config::default();

        let result = config.delete_profile("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_profile_serialization() {
        let profile = Profile {
            overlay_color: OverlayColor {
                r: 1,
                g: 2,
                b: 3,
                a: 0.4,
            },
            is_dimming_enabled: true,
            active_overlay_color: None,
            is_active_overlay_enabled: false,
            is_partial_dimming_enabled: true,
        };

        let toml_str = toml::to_string(&profile).expect("Failed to serialize profile");
        assert!(toml_str.contains("is_dimming_enabled"));
        assert!(toml_str.contains("overlay_color"));
    }

    #[test]
    fn test_add_default_profiles() {
        let mut config = Config {
            overlay_color: OverlayColor::default(),
            is_dimming_enabled: true,
            active_overlay_color: None,
            is_active_overlay_enabled: false,
            is_paused: false,
            is_partial_dimming_enabled: false,
            profiles: HashMap::new(), // Empty profiles
        };

        assert_eq!(config.profiles.len(), 0);

        config.add_default_profiles();

        assert_eq!(config.profiles.len(), 2);
        assert!(config.profiles.contains_key("light-mode"));
        assert!(config.profiles.contains_key("dark-mode"));
    }

    #[test]
    fn test_overlay_color_clone() {
        let color1 = OverlayColor {
            r: 10,
            g: 20,
            b: 30,
            a: 0.4,
        };
        let color2 = color1.clone();

        assert_eq!(color1, color2);
        // Verify they're independent
        assert_eq!(color1.r, color2.r);
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::default();
        let config2 = config1.clone();

        assert_eq!(config1.is_dimming_enabled, config2.is_dimming_enabled);
        assert_eq!(config1.overlay_color, config2.overlay_color);
        assert_eq!(config1.profiles.len(), config2.profiles.len());
    }

    #[test]
    fn test_profile_with_active_overlay() {
        let profile = Profile {
            overlay_color: OverlayColor::default(),
            is_dimming_enabled: true,
            active_overlay_color: Some(OverlayColor {
                r: 255,
                g: 0,
                b: 0,
                a: 0.3,
            }),
            is_active_overlay_enabled: true,
            is_partial_dimming_enabled: false,
        };

        assert!(profile.active_overlay_color.is_some());
        assert_eq!(profile.active_overlay_color.unwrap().r, 255);
    }

    #[test]
    fn test_overlay_color_boundary_values() {
        // Test minimum values
        let min_color = OverlayColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0.0,
        };
        assert_eq!(min_color.to_alpha_byte(), 0);
        assert_eq!(min_color.to_colorref(), 0x00000000);

        // Test maximum values
        let max_color = OverlayColor {
            r: 255,
            g: 255,
            b: 255,
            a: 1.0,
        };
        assert_eq!(max_color.to_alpha_byte(), 255);
        assert_eq!(max_color.to_colorref(), 0x00FFFFFF);
    }

    #[test]
    fn test_config_roundtrip_serialization() {
        let original = Config::default();

        // Serialize to TOML
        let toml_str = toml::to_string(&original).expect("Failed to serialize");

        // Deserialize back
        let mut deserialized: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

        // Add default profiles if empty (mimics load() behavior)
        if deserialized.profiles.is_empty() {
            deserialized.add_default_profiles();
        }

        // Compare key fields
        assert_eq!(original.is_dimming_enabled, deserialized.is_dimming_enabled);
        assert_eq!(original.overlay_color, deserialized.overlay_color);
        assert_eq!(original.is_paused, deserialized.is_paused);
        assert_eq!(original.profiles.len(), deserialized.profiles.len());
    }
}
