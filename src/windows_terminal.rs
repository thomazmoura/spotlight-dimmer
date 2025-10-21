use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::mem;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use winapi::um::wingdi::{
    CreateFontW, DeleteObject, GetTextMetricsW, SelectObject, FW_NORMAL, TEXTMETRICW,
};
#[cfg(windows)]
use winapi::um::winuser::{GetDC, ReleaseDC};

/// Windows Terminal settings extracted from settings.json
#[derive(Debug, Clone)]
pub struct TerminalSettings {
    pub font_face: String,
    pub font_size_pt: f32, // Font size in points
    pub padding_left: i32,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
}

/// Font metrics in pixels
#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub width_px: u32,
    pub height_px: u32,
}

/// Find Windows Terminal settings.json file
pub fn find_settings_file() -> Result<PathBuf, String> {
    // Windows Terminal settings location:
    // %LOCALAPPDATA%\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json
    let local_appdata = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Failed to get LOCALAPPDATA environment variable".to_string())?;

    let settings_path = PathBuf::from(local_appdata)
        .join("Packages")
        .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
        .join("LocalState")
        .join("settings.json");

    if settings_path.exists() {
        Ok(settings_path)
    } else {
        Err(format!(
            "Windows Terminal settings.json not found at: {}",
            settings_path.display()
        ))
    }
}

/// Parse Windows Terminal settings.json and extract font/padding settings
/// If profile_name is None, reads from "defaults" section
/// If profile_name is Some, reads from that specific profile
pub fn parse_settings(profile_name: Option<&str>) -> Result<TerminalSettings, String> {
    let settings_path = find_settings_file()?;
    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    // Try to find settings in profile or defaults
    let settings_source = if let Some(profile) = profile_name {
        // Find specific profile
        find_profile_settings(&json, profile)?
    } else {
        // Use defaults
        json.get("profiles")
            .and_then(|p| p.get("defaults"))
            .ok_or_else(|| "No 'profiles.defaults' section found in settings.json".to_string())?
            .clone()
    };

    // Extract font settings (try modern format first, then legacy)
    let (font_face, font_size_pt) = if let Some(font_obj) = settings_source.get("font") {
        // Modern format: font.face and font.size
        let face = font_obj
            .get("face")
            .and_then(|v| v.as_str())
            .unwrap_or("Consolas")
            .to_string();
        let size = font_obj
            .get("size")
            .and_then(|v| v.as_f64())
            .unwrap_or(12.0) as f32;
        (face, size)
    } else {
        // Legacy format: fontFace and fontSize
        let face = settings_source
            .get("fontFace")
            .and_then(|v| v.as_str())
            .unwrap_or("Consolas")
            .to_string();
        let size = settings_source
            .get("fontSize")
            .and_then(|v| v.as_f64())
            .unwrap_or(12.0) as f32;
        (face, size)
    };

    // Extract padding (can be string or array)
    let (padding_left, padding_top, padding_right, padding_bottom) =
        if let Some(padding_val) = settings_source.get("padding") {
            parse_padding(padding_val)?
        } else {
            (8, 8, 8, 8) // Default padding
        };

    Ok(TerminalSettings {
        font_face,
        font_size_pt,
        padding_left,
        padding_top,
        padding_right,
        padding_bottom,
    })
}

/// Find settings for a specific profile by name
fn find_profile_settings(json: &Value, profile_name: &str) -> Result<Value, String> {
    let profiles_list = json
        .get("profiles")
        .and_then(|p| p.get("list"))
        .and_then(|l| l.as_array())
        .ok_or_else(|| "No 'profiles.list' found in settings.json".to_string())?;

    for profile in profiles_list {
        if let Some(name) = profile.get("name").and_then(|n| n.as_str()) {
            if name == profile_name {
                return Ok(profile.clone());
            }
        }
    }

    Err(format!("Profile '{}' not found", profile_name))
}

/// Parse padding value (can be string "#", "#, #", or "#, #, #, #")
fn parse_padding(padding_val: &Value) -> Result<(i32, i32, i32, i32), String> {
    let padding_str = padding_val
        .as_str()
        .ok_or_else(|| "Padding must be a string".to_string())?;

    let parts: Vec<&str> = padding_str.split(',').map(|s| s.trim()).collect();

    match parts.len() {
        1 => {
            // Single value: all sides
            let val = parts[0]
                .parse::<i32>()
                .map_err(|_| "Invalid padding value".to_string())?;
            Ok((val, val, val, val))
        }
        2 => {
            // Two values: left-right, top-bottom
            let horizontal = parts[0]
                .parse::<i32>()
                .map_err(|_| "Invalid padding horizontal value".to_string())?;
            let vertical = parts[1]
                .parse::<i32>()
                .map_err(|_| "Invalid padding vertical value".to_string())?;
            Ok((horizontal, vertical, horizontal, vertical))
        }
        4 => {
            // Four values: left, top, right, bottom
            let left = parts[0]
                .parse::<i32>()
                .map_err(|_| "Invalid padding left value".to_string())?;
            let top = parts[1]
                .parse::<i32>()
                .map_err(|_| "Invalid padding top value".to_string())?;
            let right = parts[2]
                .parse::<i32>()
                .map_err(|_| "Invalid padding right value".to_string())?;
            let bottom = parts[3]
                .parse::<i32>()
                .map_err(|_| "Invalid padding bottom value".to_string())?;
            Ok((left, top, right, bottom))
        }
        _ => Err("Padding must have 1, 2, or 4 values".to_string()),
    }
}

/// Calculate actual font metrics in pixels using Windows GDI API
#[cfg(windows)]
pub fn calculate_font_metrics(settings: &TerminalSettings) -> Result<FontMetrics, String> {
    unsafe {
        // Get device context for the desktop
        let hdc = GetDC(ptr::null_mut());
        if hdc.is_null() {
            return Err("Failed to get device context".to_string());
        }

        // Convert font face to wide string
        let font_face_wide: Vec<u16> = OsStr::new(&settings.font_face)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Calculate font height in logical units from points
        // Formula: -MulDiv(point_size, GetDeviceCaps(hdc, LOGPIXELSY), 72)
        // For simplicity, we'll use the common 96 DPI: height = -point_size * 96 / 72
        let logical_height = -((settings.font_size_pt * 96.0 / 72.0) as i32);

        // Create font
        let hfont = CreateFontW(
            logical_height, // Height
            0,              // Width (0 = default ratio)
            0,              // Escapement
            0,              // Orientation
            FW_NORMAL,      // Weight
            0,              // Italic
            0,              // Underline
            0,              // StrikeOut
            1,              // ANSI_CHARSET (1) for Western fonts
            0,              // OUT_DEFAULT_PRECIS
            0,              // CLIP_DEFAULT_PRECIS
            5,              // CLEARTYPE_QUALITY (5)
            49,             // FIXED_PITCH | FF_MODERN (49)
            font_face_wide.as_ptr(),
        );

        if hfont.is_null() {
            ReleaseDC(ptr::null_mut(), hdc);
            return Err("Failed to create font".to_string());
        }

        // Select font into device context
        let old_font = SelectObject(hdc, hfont as *mut _);

        // Get text metrics
        let mut tm: TEXTMETRICW = mem::zeroed();
        let result = GetTextMetricsW(hdc, &mut tm);

        // Clean up
        SelectObject(hdc, old_font);
        DeleteObject(hfont as *mut _);
        ReleaseDC(ptr::null_mut(), hdc);

        if result == 0 {
            return Err("Failed to get text metrics".to_string());
        }

        // Return metrics
        Ok(FontMetrics {
            width_px: tm.tmAveCharWidth as u32,
            height_px: tm.tmHeight as u32,
        })
    }
}

#[cfg(not(windows))]
pub fn calculate_font_metrics(_settings: &TerminalSettings) -> Result<FontMetrics, String> {
    Err("Font metrics calculation is only available on Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_padding_single() {
        let json = serde_json::json!("8");
        let result = parse_padding(&json).unwrap();
        assert_eq!(result, (8, 8, 8, 8));
    }

    #[test]
    fn test_parse_padding_two() {
        let json = serde_json::json!("10, 20");
        let result = parse_padding(&json).unwrap();
        assert_eq!(result, (10, 20, 10, 20));
    }

    #[test]
    fn test_parse_padding_four() {
        let json = serde_json::json!("5, 10, 15, 20");
        let result = parse_padding(&json).unwrap();
        assert_eq!(result, (5, 10, 15, 20));
    }

    #[test]
    fn test_parse_padding_invalid() {
        let json = serde_json::json!("5, 10, 15");
        let result = parse_padding(&json);
        assert!(result.is_err());
    }
}
