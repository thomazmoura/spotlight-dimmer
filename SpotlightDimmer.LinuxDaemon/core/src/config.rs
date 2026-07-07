//! Configuration model and lenient JSON parsing.
//!
//! Mirrors the parsing half of the GNOME extension's `configBridge.js` and the
//! C# `AppConfig` defaults. Parsing is deliberately lenient per-field: an
//! invalid value keeps that field's default instead of failing the whole file
//! (a strict serde derive would reject the entire document, which would be a
//! behavior change from configBridge.js). Keys the Linux daemon does not
//! consume (System, Profiles, ...) are ignored.

use serde_json::Value;

use crate::primitives::Color;

/// Dimming mode. `Unknown` preserves configBridge.js behavior for
/// unrecognized mode strings: the focused monitor gets no overlays while
/// non-focused monitors are still dimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DimmingMode {
    #[default]
    FullScreen,
    Partial,
    PartialWithActive,
    Unknown,
}

impl DimmingMode {
    pub fn parse(s: &str) -> DimmingMode {
        match s {
            "FullScreen" => DimmingMode::FullScreen,
            "Partial" => DimmingMode::Partial,
            "PartialWithActive" => DimmingMode::PartialWithActive,
            _ => DimmingMode::Unknown,
        }
    }
}

/// Overlay appearance settings (the `Overlay` section of config.json).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayConfig {
    pub mode: DimmingMode,
    pub inactive_color: Color,
    /// 0-255; 153 is ~60%
    pub inactive_opacity: u8,
    pub active_color: Color,
    /// 0-255; 102 is ~40%
    pub active_opacity: u8,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        OverlayConfig {
            mode: DimmingMode::FullScreen,
            inactive_color: Color::BLACK,
            inactive_opacity: 153,
            active_color: Color::BLACK,
            active_opacity: 102,
        }
    }
}

/// One entry of the `AppIntegrations` section: spotlight an inner region of
/// matching windows (e.g. the focused tmux pane inside WezTerm).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppIntegration {
    pub wm_class: String,
    /// Currently only "tmux" is meaningful.
    pub provider: String,
    /// Pixel offset from the window frame origin to the terminal content
    /// origin (window decorations, tab bar, padding).
    pub content_offset_x: i32,
    pub content_offset_y: i32,
}

/// Full configuration consumed by the Linux daemon.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppConfig {
    pub overlay: OverlayConfig,
    pub app_integrations: Vec<AppIntegration>,
}

impl AppConfig {
    /// Parse config.json contents. Returns `Err` only for unparseable JSON;
    /// any recognized-but-invalid field silently keeps its default, matching
    /// configBridge.js.
    pub fn from_json(json: &str) -> Result<AppConfig, serde_json::Error> {
        let value: Value = serde_json::from_str(json)?;
        Ok(AppConfig::from_value(&value))
    }

    pub fn from_value(root: &Value) -> AppConfig {
        let mut config = AppConfig::default();

        if let Some(overlay) = root.get("Overlay") {
            parse_overlay(overlay, &mut config.overlay);
        }

        if let Some(integrations) = root.get("AppIntegrations") {
            config.app_integrations = parse_app_integrations(integrations);
        }

        config
    }

    /// Find the integration matching a window's WM_CLASS with the given
    /// provider. Port of `_matchIntegration` (appIntegrations.js).
    pub fn match_integration(&self, wm_class: &str, provider: &str) -> Option<&AppIntegration> {
        if wm_class.is_empty() {
            return None;
        }
        self.app_integrations
            .iter()
            .find(|i| i.wm_class == wm_class && i.provider == provider)
    }
}

fn parse_overlay(overlay: &Value, out: &mut OverlayConfig) {
    // Mode: only override when present and non-empty (JS truthiness check)
    if let Some(mode) = overlay.get("Mode").and_then(Value::as_str) {
        if !mode.is_empty() {
            out.mode = DimmingMode::parse(mode);
        }
    }

    if let Some(hex) = overlay.get("InactiveColor").and_then(Value::as_str) {
        if !hex.is_empty() {
            out.inactive_color = parse_hex_color(hex);
        }
    }

    if let Some(opacity) = overlay.get("InactiveOpacity").and_then(Value::as_f64) {
        out.inactive_opacity = clamp_opacity(opacity);
    }

    if let Some(hex) = overlay.get("ActiveColor").and_then(Value::as_str) {
        if !hex.is_empty() {
            out.active_color = parse_hex_color(hex);
        }
    }

    if let Some(opacity) = overlay.get("ActiveOpacity").and_then(Value::as_f64) {
        out.active_opacity = clamp_opacity(opacity);
    }
}

/// Entries without a non-empty string WmClass are dropped; Provider defaults
/// to "tmux"; offsets default to 0 and are rounded to integers.
fn parse_app_integrations(integrations: &Value) -> Vec<AppIntegration> {
    let Some(entries) = integrations.as_array() else {
        return Vec::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            let wm_class = entry.get("WmClass")?.as_str()?;
            if wm_class.is_empty() {
                return None;
            }

            let provider = entry
                .get("Provider")
                .and_then(Value::as_str)
                .filter(|p| !p.is_empty())
                .unwrap_or("tmux");

            Some(AppIntegration {
                wm_class: wm_class.to_string(),
                provider: provider.to_string(),
                content_offset_x: parse_offset(entry.get("ContentOffsetX")),
                content_offset_y: parse_offset(entry.get("ContentOffsetY")),
            })
        })
        .collect()
}

fn parse_offset(value: Option<&Value>) -> i32 {
    value
        .and_then(Value::as_f64)
        .map(|v| v.round() as i32)
        .unwrap_or(0)
}

/// Parse "#RRGGBB" (leading '#' optional). Invalid formats fall back to
/// black; each invalid component falls back to 0 (configBridge.js semantics).
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.replace('#', "");

    if hex.len() != 6 || !hex.is_ascii() {
        return Color::BLACK;
    }

    let component = |range| u8::from_str_radix(&hex[range], 16).unwrap_or(0);

    Color {
        r: component(0..2),
        g: component(2..4),
        b: component(4..6),
    }
}

fn clamp_opacity(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_configbridge_js() {
        let config = AppConfig::default();
        assert_eq!(config.overlay.mode, DimmingMode::FullScreen);
        assert_eq!(config.overlay.inactive_color, Color::BLACK);
        assert_eq!(config.overlay.inactive_opacity, 153);
        assert_eq!(config.overlay.active_color, Color::BLACK);
        assert_eq!(config.overlay.active_opacity, 102);
        assert!(config.app_integrations.is_empty());
    }

    #[test]
    fn parses_full_config() {
        let config = AppConfig::from_json(
            r##"{
                "Overlay": {
                    "Mode": "PartialWithActive",
                    "InactiveColor": "#102030",
                    "InactiveOpacity": 200,
                    "ActiveColor": "405060",
                    "ActiveOpacity": 50
                },
                "AppIntegrations": [
                    {
                        "WmClass": "org.wezfurlong.wezterm",
                        "Provider": "tmux",
                        "ContentOffsetX": 8,
                        "ContentOffsetY": 40.6
                    }
                ],
                "System": { "RendererBackend": "Composition" }
            }"##,
        )
        .unwrap();

        assert_eq!(config.overlay.mode, DimmingMode::PartialWithActive);
        assert_eq!(
            config.overlay.inactive_color,
            Color {
                r: 0x10,
                g: 0x20,
                b: 0x30
            }
        );
        assert_eq!(config.overlay.inactive_opacity, 200);
        // '#' prefix is optional
        assert_eq!(
            config.overlay.active_color,
            Color {
                r: 0x40,
                g: 0x50,
                b: 0x60
            }
        );
        assert_eq!(config.overlay.active_opacity, 50);

        assert_eq!(config.app_integrations.len(), 1);
        let integration = &config.app_integrations[0];
        assert_eq!(integration.wm_class, "org.wezfurlong.wezterm");
        assert_eq!(integration.provider, "tmux");
        assert_eq!(integration.content_offset_x, 8);
        // Fractional offsets are rounded
        assert_eq!(integration.content_offset_y, 41);
    }

    #[test]
    fn empty_document_keeps_defaults() {
        let config = AppConfig::from_json("{}").unwrap();
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(AppConfig::from_json("not json").is_err());
    }

    #[test]
    fn invalid_hex_color_falls_back_to_black() {
        let config = AppConfig::from_json(
            r##"{"Overlay": {"InactiveColor": "#12345", "ActiveColor": "zzzzzz"}}"##,
        )
        .unwrap();
        // Wrong length -> black
        assert_eq!(config.overlay.inactive_color, Color::BLACK);
        // Right length, invalid digits -> each component falls back to 0
        assert_eq!(config.overlay.active_color, Color::BLACK);
    }

    #[test]
    fn opacity_is_clamped_and_rounded() {
        let config = AppConfig::from_json(
            r#"{"Overlay": {"InactiveOpacity": 300, "ActiveOpacity": 101.5}}"#,
        )
        .unwrap();
        assert_eq!(config.overlay.inactive_opacity, 255);
        assert_eq!(config.overlay.active_opacity, 102);

        let config = AppConfig::from_json(r#"{"Overlay": {"InactiveOpacity": -5}}"#).unwrap();
        assert_eq!(config.overlay.inactive_opacity, 0);
    }

    #[test]
    fn non_numeric_opacity_keeps_default() {
        let config = AppConfig::from_json(r#"{"Overlay": {"InactiveOpacity": "dark"}}"#).unwrap();
        assert_eq!(config.overlay.inactive_opacity, 153);
    }

    #[test]
    fn unknown_mode_string_parses_to_unknown() {
        let config = AppConfig::from_json(r#"{"Overlay": {"Mode": "Sideways"}}"#).unwrap();
        assert_eq!(config.overlay.mode, DimmingMode::Unknown);
    }

    #[test]
    fn integrations_without_wm_class_are_dropped_and_provider_defaults() {
        let config = AppConfig::from_json(
            r#"{"AppIntegrations": [
                {"Provider": "tmux"},
                {"WmClass": ""},
                {"WmClass": "kitty"}
            ]}"#,
        )
        .unwrap();
        assert_eq!(config.app_integrations.len(), 1);
        assert_eq!(config.app_integrations[0].wm_class, "kitty");
        assert_eq!(config.app_integrations[0].provider, "tmux");
        assert_eq!(config.app_integrations[0].content_offset_x, 0);
    }

    #[test]
    fn match_integration_requires_class_and_provider() {
        let config = AppConfig::from_json(
            r#"{"AppIntegrations": [{"WmClass": "kitty", "Provider": "tmux"}]}"#,
        )
        .unwrap();
        assert!(config.match_integration("kitty", "tmux").is_some());
        assert!(config.match_integration("kitty", "other").is_none());
        assert!(config.match_integration("alacritty", "tmux").is_none());
        assert!(config.match_integration("", "tmux").is_none());
    }
}
