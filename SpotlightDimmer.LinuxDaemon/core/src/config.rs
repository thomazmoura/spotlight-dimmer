//! Configuration model and lenient JSON parsing.
//!
//! Mirrors the parsing half of the GNOME extension's `configBridge.js` and the
//! C# `AppConfig` defaults. Parsing is deliberately lenient per-field: an
//! invalid value keeps that field's default instead of failing the whole file
//! (a strict serde derive would reject the entire document, which would be a
//! behavior change from configBridge.js). Keys the Linux daemon does not
//! consume (System, Profiles, ...) are ignored.

use std::io;
use std::path::Path;

use serde_json::{Map, Value};

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

/// How surfaces the shell or compositor draws *above* application windows —
/// notification banners, OSD, panel menus, docks — are treated.
///
/// These are not application windows and no supported API enumerates their
/// geometry, so the only portable lever is where the overlays stack relative
/// to them. That makes the choice binary and exact: either the overlays sit
/// below such surfaces (never dimming them) or above (always dimming them).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChromeHandling {
    /// Stack the overlays below shell chrome, so notifications and popups are
    /// always fully lit. The default, because with the overlays on top a
    /// banner straddling the spotlight edge is painted dark on one side and
    /// left untouched on the other, which reads as a rendering bug.
    #[default]
    Highlight,
    /// Stack the overlays above shell chrome, dimming panels, docks and
    /// notifications along with everything else. The behavior of every
    /// release before this one.
    Dim,
}

impl ChromeHandling {
    /// Unrecognized values keep the default, matching the lenient per-field
    /// parsing used everywhere else in this module.
    pub fn parse(s: &str) -> ChromeHandling {
        match s {
            "Dim" => ChromeHandling::Dim,
            _ => ChromeHandling::Highlight,
        }
    }

    /// The wire form sent to renderers in the overlays payload.
    pub fn as_str(self) -> &'static str {
        match self {
            ChromeHandling::Highlight => "Highlight",
            ChromeHandling::Dim => "Dim",
        }
    }
}

/// What happens to application windows the user marked always-on-top, which
/// float above other windows but are ordinary windows the compositor can
/// enumerate (unlike shell chrome — see [`ChromeHandling`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlwaysOnTopHandling {
    /// Dim them like any other window, which is what every release up to
    /// 0.6.0 did. Cheapest: the adapters never enumerate windows at all.
    Ignore,
    /// Treat them as part of the spotlight: the active overlay covers them
    /// and the inactive overlay is cut away underneath.
    Highlight,
    /// Cover them with the inactive overlay uniformly, so one never comes out
    /// dimmed on the part over a dimmed area and lit on the part over the
    /// active window. The default: that split rendering reads as a bug, and
    /// the focused window is exempt either way, so the keyboard focus stays
    /// visible.
    #[default]
    Dim,
}

impl AlwaysOnTopHandling {
    /// Unrecognized values keep the default, matching the lenient per-field
    /// parsing used everywhere else in this module.
    pub fn parse(s: &str) -> AlwaysOnTopHandling {
        match s {
            "Highlight" => AlwaysOnTopHandling::Highlight,
            "Ignore" => AlwaysOnTopHandling::Ignore,
            _ => AlwaysOnTopHandling::Dim,
        }
    }

    /// Whether floating rects need reporting at all. When this is false the
    /// adapters skip enumerating and diffing windows entirely.
    pub fn tracks_windows(self) -> bool {
        self != AlwaysOnTopHandling::Ignore
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
    /// Persisted on/off state of the dimming (`Overlay.Enabled`). Flipped by
    /// the toggle shortcut and the settings window; `true` when absent.
    pub enabled: bool,
    /// Whether shell chrome (notifications, OSD, panels, docks) is dimmed
    /// along with windows, or stays lit.
    pub chrome_handling: ChromeHandling,
    /// Whether always-on-top application windows are exempted from the
    /// dimming, covered by it uniformly, or treated as ordinary windows.
    pub always_on_top_handling: AlwaysOnTopHandling,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        OverlayConfig {
            mode: DimmingMode::FullScreen,
            inactive_color: Color::BLACK,
            inactive_opacity: 153,
            active_color: Color::BLACK,
            active_opacity: 102,
            enabled: true,
            chrome_handling: ChromeHandling::Highlight,
            always_on_top_handling: AlwaysOnTopHandling::Dim,
        }
    }
}

/// How the focused pane's tty (the join key against the geometry pushed by
/// the tmux hooks) is discovered for a window class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TtySource {
    /// Ask the WezTerm CLI which pane is focused and which tty it owns.
    #[default]
    WezTermCli,
    /// Read the tty out of the window title, where tmux published it via
    /// `set-titles-string`. For terminals without a pane-query CLI (Ghostty).
    WindowTitle,
}

impl TtySource {
    /// Unrecognized values keep the default, matching the lenient
    /// per-field parsing used everywhere else in this module.
    pub fn parse(s: &str) -> TtySource {
        match s {
            "title" => TtySource::WindowTitle,
            _ => TtySource::WezTermCli,
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
    /// Where the focused pane's tty comes from for this window class.
    pub tty_source: TtySource,
    /// Pixel offset from the window client-area origin (decorations
    /// excluded) to the terminal cell grid: tab bar, window padding. Window
    /// decorations are reported separately by the adapter (protocol v2) and
    /// must not be folded in here.
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

    if let Some(enabled) = overlay.get("Enabled").and_then(Value::as_bool) {
        out.enabled = enabled;
    }

    if let Some(chrome) = overlay.get("ChromeHandling").and_then(Value::as_str) {
        if !chrome.is_empty() {
            out.chrome_handling = ChromeHandling::parse(chrome);
        }
    }

    if let Some(aot) = overlay.get("AlwaysOnTopHandling").and_then(Value::as_str) {
        if !aot.is_empty() {
            out.always_on_top_handling = AlwaysOnTopHandling::parse(aot);
        }
    }
}

/// Persist `Overlay.Enabled` into the config file at `path`, touching no
/// other key. A missing file is created; an unparseable one is left alone
/// (returns `InvalidData`) so a hand-edit in progress is never clobbered.
pub fn write_overlay_enabled(path: &Path, enabled: bool) -> io::Result<()> {
    let mut root = match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str::<Value>(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Value::Object(Map::new()),
        Err(e) => return Err(e),
    };

    let Some(object) = root.as_object_mut() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "config root is not an object",
        ));
    };
    let overlay = object
        .entry("Overlay")
        .or_insert_with(|| Value::Object(Map::new()));
    if !overlay.is_object() {
        *overlay = Value::Object(Map::new());
    }
    overlay
        .as_object_mut()
        .expect("Overlay is an object")
        .insert("Enabled".to_string(), Value::Bool(enabled));

    let mut contents = serde_json::to_string_pretty(&root)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    contents.push('\n');
    write_atomically(path, &contents)
}

/// Write `contents` to `path` atomically: temp file in the same directory,
/// fsync, rename. A file watcher on the directory sees a single `Created`
/// rather than a truncated intermediate state.
pub fn write_atomically(path: &Path, contents: &str) -> io::Result<()> {
    use std::io::Write;

    let dir = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "config path has no parent"))?;
    std::fs::create_dir_all(dir)?;

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config.json".to_string());
    let temp = dir.join(format!("{name}.tmp-{}", std::process::id()));
    {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
    }

    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}

/// Entries without a non-empty string WmClass are dropped; Provider defaults
/// to "tmux"; TtySource defaults to the WezTerm CLI; offsets default to 0 and
/// are rounded to integers.
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

            let tty_source = entry
                .get("TtySource")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(TtySource::parse)
                .unwrap_or_default();

            Some(AppIntegration {
                wm_class: wm_class.to_string(),
                provider: provider.to_string(),
                tty_source,
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
        assert!(config.overlay.enabled);
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
                    },
                    {
                        "WmClass": "com.mitchellh.ghostty",
                        "Provider": "tmux",
                        "TtySource": "title",
                        "ContentOffsetX": 2,
                        "ContentOffsetY": 2
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

        assert_eq!(config.app_integrations.len(), 2);
        let integration = &config.app_integrations[0];
        assert_eq!(integration.wm_class, "org.wezfurlong.wezterm");
        assert_eq!(integration.provider, "tmux");
        // Omitted TtySource keeps the WezTerm CLI query chain
        assert_eq!(integration.tty_source, TtySource::WezTermCli);
        assert_eq!(integration.content_offset_x, 8);
        // Fractional offsets are rounded
        assert_eq!(integration.content_offset_y, 41);

        let ghostty = &config.app_integrations[1];
        assert_eq!(ghostty.wm_class, "com.mitchellh.ghostty");
        assert_eq!(ghostty.tty_source, TtySource::WindowTitle);
    }

    #[test]
    fn enabled_parses_booleans_and_ignores_other_types() {
        let config = AppConfig::from_json(r#"{"Overlay": {"Enabled": false}}"#).unwrap();
        assert!(!config.overlay.enabled);

        let config = AppConfig::from_json(r#"{"Overlay": {"Enabled": "no"}}"#).unwrap();
        assert!(config.overlay.enabled);
    }

    fn temp_config(test: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "spotlight-dimmer-core-{test}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("config.json")
    }

    #[test]
    fn write_overlay_enabled_preserves_other_keys() {
        let path = temp_config("preserve");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            r#"{"ConfigVersion":"0.8.5","Overlay":{"Mode":"Partial"},"Profiles":[1]}"#,
        )
        .unwrap();

        write_overlay_enabled(&path, false).unwrap();

        let value: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["ConfigVersion"], "0.8.5");
        assert_eq!(value["Profiles"][0], 1);
        assert_eq!(value["Overlay"]["Mode"], "Partial");
        assert_eq!(value["Overlay"]["Enabled"], false);
        assert!(!AppConfig::from_value(&value).overlay.enabled);
    }

    #[test]
    fn write_overlay_enabled_creates_missing_file_and_refuses_bad_json() {
        let path = temp_config("create");
        write_overlay_enabled(&path, true).unwrap();
        let value: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["Overlay"]["Enabled"], true);

        std::fs::write(&path, "{ half-typed").unwrap();
        assert!(write_overlay_enabled(&path, false).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ half-typed");
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
    fn chrome_handling_defaults_to_highlight_and_parses_leniently() {
        assert_eq!(
            OverlayConfig::default().chrome_handling,
            ChromeHandling::Highlight
        );

        let dim = AppConfig::from_json(r#"{"Overlay":{"ChromeHandling":"Dim"}}"#).unwrap();
        assert_eq!(dim.overlay.chrome_handling, ChromeHandling::Dim);

        // Unrecognized, empty and absent all keep the default, like every
        // other field in this module.
        for json in [
            r#"{"Overlay":{"ChromeHandling":"Sideways"}}"#,
            r#"{"Overlay":{"ChromeHandling":""}}"#,
            r#"{"Overlay":{"ChromeHandling":7}}"#,
            r#"{"Overlay":{}}"#,
        ] {
            let config = AppConfig::from_json(json).unwrap();
            assert_eq!(
                config.overlay.chrome_handling,
                ChromeHandling::Highlight,
                "{json}"
            );
        }
    }

    #[test]
    fn always_on_top_handling_defaults_to_dim_and_parses_leniently() {
        assert_eq!(
            OverlayConfig::default().always_on_top_handling,
            AlwaysOnTopHandling::Dim
        );
        assert_eq!(AlwaysOnTopHandling::default(), AlwaysOnTopHandling::Dim);
        assert!(!AlwaysOnTopHandling::Ignore.tracks_windows());
        assert!(AlwaysOnTopHandling::Highlight.tracks_windows());
        assert!(AlwaysOnTopHandling::Dim.tracks_windows());

        for (json, expected) in [
            (
                r#"{"Overlay":{"AlwaysOnTopHandling":"Highlight"}}"#,
                AlwaysOnTopHandling::Highlight,
            ),
            (
                r#"{"Overlay":{"AlwaysOnTopHandling":"Dim"}}"#,
                AlwaysOnTopHandling::Dim,
            ),
            // Opting out explicitly must still work now that it is not the default.
            (
                r#"{"Overlay":{"AlwaysOnTopHandling":"Ignore"}}"#,
                AlwaysOnTopHandling::Ignore,
            ),
            (
                r#"{"Overlay":{"AlwaysOnTopHandling":"Sideways"}}"#,
                AlwaysOnTopHandling::Dim,
            ),
            (r#"{"Overlay":{}}"#, AlwaysOnTopHandling::Dim),
        ] {
            let config = AppConfig::from_json(json).unwrap();
            assert_eq!(config.overlay.always_on_top_handling, expected, "{json}");
        }
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
        assert_eq!(config.app_integrations[0].tty_source, TtySource::WezTermCli);
        assert_eq!(config.app_integrations[0].content_offset_x, 0);
    }

    #[test]
    fn unknown_or_empty_tty_source_keeps_the_wezterm_default() {
        let config = AppConfig::from_json(
            r#"{"AppIntegrations": [
                {"WmClass": "a", "TtySource": "smoke-signals"},
                {"WmClass": "b", "TtySource": ""},
                {"WmClass": "c", "TtySource": "wezterm"},
                {"WmClass": "d", "TtySource": "title"}
            ]}"#,
        )
        .unwrap();
        let sources: Vec<TtySource> = config
            .app_integrations
            .iter()
            .map(|i| i.tty_source)
            .collect();
        assert_eq!(
            sources,
            vec![
                TtySource::WezTermCli,
                TtySource::WezTermCli,
                TtySource::WezTermCli,
                TtySource::WindowTitle
            ]
        );
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
