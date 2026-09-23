//! The config.json document: key-preserving mutation, debounced atomic
//! writes and external-change watching.
//!
//! `spotlight_dimmer_core::config` is parse-only and deliberately ignores
//! keys the Linux daemon does not consume (`System`, `Profiles`,
//! `CurrentProfile`, `$schema`, `ConfigVersion`,
//! `Overlay.ExcludeFromScreenCapture`). Round-tripping the config through
//! `AppConfig` would therefore silently delete every one of them, so this
//! module keeps the full `serde_json::Value` document and mutates only the
//! individual leaves the GUI owns. `AppConfig::from_value` is then used on
//! that same document to derive the display state with exactly the daemon's
//! own lenient semantics.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use gio::prelude::*;
use serde_json::{json, Map, Value};

use spotlight_dimmer_core::config::{write_atomically, AppConfig, DimmingMode};

const CONFIG_DIR: &str = "SpotlightDimmer";
const CONFIG_FILE: &str = "config.json";
/// Above the daemon's own 100 ms read debounce, so a slider drag produces a
/// handful of writes rather than the per-tick write storm the Windows GUI
/// generates (ConfigForm.cs saves on every TrackBar.ValueChanged).
const SAVE_DEBOUNCE_MS: u64 = 150;
/// Mirrors config_watch.rs so both sides settle at the same rate.
const WATCH_DEBOUNCE_MS: u64 = 100;

pub fn config_path() -> PathBuf {
    glib::user_config_dir().join(CONFIG_DIR).join(CONFIG_FILE)
}

/// The editable fields of a built-in integration's `AppIntegrations` entry,
/// read with the same per-field defaults the daemon applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationView {
    pub content_offset_x: i32,
    pub content_offset_y: i32,
}

/// One `Profiles` entry: a named overlay preset shared with the Windows
/// client. Missing fields take the C# `Profile` class defaults, so a profile
/// written by either platform applies identically on both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileView {
    pub name: String,
    pub mode: String,
    pub inactive_color: String,
    pub inactive_opacity: u8,
    pub active_color: String,
    pub active_opacity: u8,
}

impl ProfileView {
    fn from_entry(entry: &Value) -> Option<ProfileView> {
        let name = string_field(entry, "Name")?;
        let opacity = |key, default| {
            entry
                .get(key)
                .and_then(Value::as_f64)
                .map(|v| v.round().clamp(0.0, 255.0) as u8)
                .unwrap_or(default)
        };
        Some(ProfileView {
            name,
            mode: string_field(entry, "Mode").unwrap_or_else(|| "FullScreen".to_string()),
            inactive_color: string_field(entry, "InactiveColor")
                .unwrap_or_else(|| "#000000".to_string()),
            inactive_opacity: opacity("InactiveOpacity", 153),
            active_color: string_field(entry, "ActiveColor")
                .unwrap_or_else(|| "#000000".to_string()),
            active_opacity: opacity("ActiveOpacity", 102),
        })
    }

    /// The five overlay leaves a profile carries, as JSON values.
    fn fields(&self) -> [(&'static str, Value); 5] {
        [
            ("Mode", json!(self.mode)),
            ("InactiveColor", json!(self.inactive_color)),
            ("InactiveOpacity", json!(self.inactive_opacity)),
            ("ActiveColor", json!(self.active_color)),
            ("ActiveOpacity", json!(self.active_opacity)),
        ]
    }
}

/// Why the document refuses to be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadProblem {
    /// The file exists but is not valid JSON. Saving would replace it, so
    /// writes stay blocked until the user explicitly confirms.
    Unparseable(String),
}

struct Inner {
    path: PathBuf,
    /// The FULL document, unknown keys included.
    root: RefCell<Value>,
    problem: RefCell<Option<LoadProblem>>,
    /// Set while a `LoadProblem` is outstanding and unconfirmed.
    write_blocked: Cell<bool>,
    /// Exact bytes of our last write, used to ignore our own change events.
    last_written: RefCell<Option<String>>,
    save_epoch: Cell<u64>,
    watch_epoch: Cell<u64>,
    on_reload: RefCell<Vec<Box<dyn Fn()>>>,
    on_problem: RefCell<Vec<Box<dyn Fn()>>>,
    on_edit: RefCell<Vec<Box<dyn Fn()>>>,
    monitor: RefCell<Option<gio::FileMonitor>>,
}

/// Shared handle to the config document. Cheap to clone into signal handlers.
#[derive(Clone)]
pub struct Document(Rc<Inner>);

impl Document {
    /// Read the config file. A missing file is not an error: the document
    /// starts from the daemon's own defaults and the directory is created on
    /// first save. Unparseable JSON keeps defaults in memory but blocks
    /// writing — silently clobbering a file with a typo in it is the one
    /// unrecoverable failure here.
    pub fn load() -> Document {
        Document::load_from(config_path())
    }

    pub fn load_from(path: PathBuf) -> Document {
        let (root, problem) = match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<Value>(&contents) {
                Ok(value @ Value::Object(_)) => (value, None),
                Ok(_) => (
                    default_root(),
                    Some(LoadProblem::Unparseable(
                        "the file's top level is not a JSON object".to_string(),
                    )),
                ),
                Err(e) => (
                    default_root(),
                    Some(LoadProblem::Unparseable(e.to_string())),
                ),
            },
            Err(_) => (default_root(), None),
        };

        let blocked = problem.is_some();
        Document(Rc::new(Inner {
            path,
            root: RefCell::new(root),
            problem: RefCell::new(problem),
            write_blocked: Cell::new(blocked),
            last_written: RefCell::new(None),
            save_epoch: Cell::new(0),
            watch_epoch: Cell::new(0),
            on_reload: RefCell::new(Vec::new()),
            on_problem: RefCell::new(Vec::new()),
            on_edit: RefCell::new(Vec::new()),
            monitor: RefCell::new(None),
        }))
    }

    pub fn path(&self) -> &Path {
        &self.0.path
    }

    /// The daemon's exact view of this document.
    pub fn config(&self) -> AppConfig {
        AppConfig::from_value(&self.0.root.borrow())
    }

    pub fn problem(&self) -> Option<LoadProblem> {
        self.0.problem.borrow().clone()
    }

    /// Accept that saving will replace the unparseable file, and write it.
    pub fn confirm_overwrite(&self) {
        self.0.write_blocked.set(false);
        *self.0.problem.borrow_mut() = None;
        self.flush();
        self.notify_problem();
    }

    /// Called after any widget-driven mutation.
    pub fn on_reload(&self, f: impl Fn() + 'static) {
        self.0.on_reload.borrow_mut().push(Box::new(f));
    }

    /// Called after every in-window mutation (each one schedules a save),
    /// before the debounce. Keep handlers cheap: a slider drag fires this
    /// per tick.
    pub fn on_edit(&self, f: impl Fn() + 'static) {
        self.0.on_edit.borrow_mut().push(Box::new(f));
    }

    /// Called when the write-blocked banner needs to appear or disappear.
    pub fn on_problem(&self, f: impl Fn() + 'static) {
        self.0.on_problem.borrow_mut().push(Box::new(f));
    }

    // ---------------------------------------------------------------- overlay

    /// The raw string stored under `Overlay.<key>`, for showing a value the
    /// parser could not make sense of instead of silently rewriting it.
    pub fn overlay_raw_string(&self, key: &str) -> Option<String> {
        self.0
            .root
            .borrow()
            .get("Overlay")
            .and_then(|o| o.get(key))
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    /// `Overlay.Enabled` with the daemon's own default (`true`).
    pub fn enabled(&self) -> bool {
        self.config().overlay.enabled
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.set_overlay("Enabled", json!(enabled));
        self.schedule_save();
    }

    /// Mirror a value the daemon already persisted into the in-memory
    /// document without writing. Closes the race where an unrelated edit
    /// saved before the file watcher caught up would write the stale flag
    /// back and undo a shortcut press.
    pub fn note_enabled(&self, enabled: bool) {
        self.set_overlay("Enabled", json!(enabled));
    }

    pub fn set_mode(&self, mode: &str) {
        self.set_overlay("Mode", json!(mode));
        self.schedule_save();
    }

    pub fn set_chrome_handling(&self, value: &str) {
        self.set_overlay("ChromeHandling", json!(value));
        self.schedule_save();
    }

    pub fn set_always_on_top_handling(&self, value: &str) {
        self.set_overlay("AlwaysOnTopHandling", json!(value));
        self.schedule_save();
    }

    pub fn set_inactive_color(&self, hex: &str) {
        self.set_overlay("InactiveColor", json!(hex));
        self.schedule_save();
    }

    pub fn set_inactive_opacity(&self, opacity: u8) {
        self.set_overlay("InactiveOpacity", json!(opacity));
        self.schedule_save();
    }

    pub fn set_active_color(&self, hex: &str) {
        self.set_overlay("ActiveColor", json!(hex));
        self.schedule_save();
    }

    pub fn set_active_opacity(&self, opacity: u8) {
        self.set_overlay("ActiveOpacity", json!(opacity));
        self.schedule_save();
    }

    /// Mutates a single leaf under `Overlay`, creating that object only if it
    /// is absent and never touching a sibling key. Does not save.
    fn set_overlay(&self, key: &str, value: Value) {
        {
            let mut root = self.0.root.borrow_mut();
            let object = root.as_object_mut().expect("root is an object");
            let overlay = object
                .entry("Overlay")
                .or_insert_with(|| Value::Object(Map::new()));
            if !overlay.is_object() {
                *overlay = Value::Object(Map::new());
            }
            overlay
                .as_object_mut()
                .expect("Overlay is an object")
                .insert(key.to_string(), value);
        }
    }

    // --------------------------------------------------------------- profiles
    //
    // Same `Profiles` / `CurrentProfile` shape as the Windows client, edited
    // leaf by leaf so keys this window does not know about survive.

    /// Every entry with a non-empty `Name`, in file order.
    pub fn profiles(&self) -> Vec<ProfileView> {
        let root = self.0.root.borrow();
        root.get("Profiles")
            .and_then(Value::as_array)
            .map(|entries| entries.iter().filter_map(ProfileView::from_entry).collect())
            .unwrap_or_default()
    }

    pub fn current_profile(&self) -> Option<String> {
        self.0
            .root
            .borrow()
            .get("CurrentProfile")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    /// Copy the named profile into `Overlay` and mark it current. Written
    /// straight through rather than debounced: Ctrl+Enter closes the window
    /// right after, and the app exits with its last window, taking any
    /// pending timeout with it. `Overlay.Enabled` is deliberately untouched.
    pub fn apply_profile(&self, name: &str) -> bool {
        let Some(profile) = self.profiles().into_iter().find(|p| p.name == name) else {
            return false;
        };
        for (key, value) in profile.fields() {
            self.set_overlay(key, value);
        }
        self.set_root("CurrentProfile", json!(name));
        // Supersede any debounced save still in flight; this write covers it.
        self.0.save_epoch.set(self.0.save_epoch.get() + 1);
        self.save_now();
        true
    }

    /// Store the current overlay under `name`: an existing entry is updated
    /// in place (its other keys kept), otherwise a new one is appended.
    pub fn save_profile(&self, name: &str) {
        let overlay = self.config().overlay;
        let raw_mode = self.overlay_raw_string("Mode");
        let profile = ProfileView {
            name: name.to_string(),
            // An unrecognised mode is stored verbatim rather than coerced.
            mode: raw_mode.unwrap_or_else(|| mode_name(overlay.mode).to_string()),
            inactive_color: crate::widgets::to_hex(overlay.inactive_color),
            inactive_opacity: overlay.inactive_opacity,
            active_color: crate::widgets::to_hex(overlay.active_color),
            active_opacity: overlay.active_opacity,
        };
        {
            let mut root = self.0.root.borrow_mut();
            let entries = profiles_array(&mut root);
            match entries
                .iter_mut()
                .find(|e| string_field(e, "Name").as_deref() == Some(name))
            {
                Some(entry) => {
                    let object = entry.as_object_mut().expect("a named entry is an object");
                    for (key, value) in profile.fields() {
                        object.insert(key.to_string(), value);
                    }
                }
                None => {
                    let mut object = Map::new();
                    object.insert("Name".to_string(), json!(name));
                    for (key, value) in profile.fields() {
                        object.insert(key.to_string(), value);
                    }
                    entries.push(Value::Object(object));
                }
            }
        }
        self.set_root("CurrentProfile", json!(name));
        self.schedule_save();
    }

    /// Remove every entry named `name`; clears `CurrentProfile` if it
    /// pointed there.
    pub fn delete_profile(&self, name: &str) {
        {
            let mut root = self.0.root.borrow_mut();
            profiles_array(&mut root).retain(|e| string_field(e, "Name").as_deref() != Some(name));
        }
        if self.current_profile().as_deref() == Some(name) {
            self.set_root("CurrentProfile", Value::Null);
        }
        self.schedule_save();
    }

    /// Whether the overlay still equals the named profile (Windows
    /// `DoesOverlayMatchProfile`), compared through the daemon's parser so
    /// `#abcdef` and `ABCDEF` count as the same colour.
    pub fn overlay_matches_profile(&self, name: &str) -> bool {
        let Some(profile) = self.profiles().into_iter().find(|p| p.name == name) else {
            return false;
        };
        let overlay: Map<String, Value> = profile
            .fields()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        let expected = AppConfig::from_value(&json!({ "Overlay": overlay })).overlay;
        let actual = self.config().overlay;
        expected.mode == actual.mode
            && expected.inactive_color == actual.inactive_color
            && expected.inactive_opacity == actual.inactive_opacity
            && expected.active_color == actual.active_color
            && expected.active_opacity == actual.active_opacity
    }

    /// Tell every view the document changed without coming from disk (a
    /// profile applied from one section must refresh all the others).
    pub fn notify_changed(&self) {
        self.notify_reload();
    }

    fn set_root(&self, key: &str, value: Value) {
        self.0
            .root
            .borrow_mut()
            .as_object_mut()
            .expect("root is an object")
            .insert(key.to_string(), value);
    }

    // ----------------------------------------------------------- integrations
    //
    // Entries are addressed by WM_CLASS, never by index: the editor only
    // shows the built-in integrations, so hand-written entries for other
    // terminals (or Windows `ProcessName` entries) sit hidden between them
    // and must survive every edit untouched.

    /// The first `tmux` entry for `wm_class`, as the daemon would match it.
    pub fn find_integration(&self, wm_class: &str) -> Option<IntegrationView> {
        let root = self.0.root.borrow();
        let entries = root.get("AppIntegrations").and_then(Value::as_array)?;
        let entry = entries.iter().find(|e| is_tmux_entry_for(e, wm_class))?;

        Some(IntegrationView {
            content_offset_x: offset_field(entry, "ContentOffsetX"),
            content_offset_y: offset_field(entry, "ContentOffsetY"),
        })
    }

    /// Adds the entry for a built-in integration. An entry that is already
    /// there is kept as-is, so hand-tuned offsets and extra keys survive a
    /// re-check. Written with explicit defaults so the resulting JSON is
    /// self-documenting rather than relying on the parser's fallbacks.
    pub fn enable_integration(&self, wm_class: &str, tty_source: &str, offset: (i32, i32)) {
        {
            let mut root = self.0.root.borrow_mut();
            let entries = integrations_array(&mut root);
            if entries.iter().any(|e| is_tmux_entry_for(e, wm_class)) {
                return;
            }
            entries.push(json!({
                "WmClass": wm_class,
                "Provider": "tmux",
                "TtySource": tty_source,
                "ContentOffsetX": offset.0,
                "ContentOffsetY": offset.1
            }));
        }
        self.schedule_save();
    }

    /// Removes every entry for `wm_class` (duplicates included — the daemon
    /// would otherwise keep matching the next one), leaving the rest alone.
    pub fn disable_integration(&self, wm_class: &str) {
        {
            let mut root = self.0.root.borrow_mut();
            let entries = integrations_array(&mut root);
            let before = entries.len();
            entries.retain(|e| !is_tmux_entry_for(e, wm_class));
            if entries.len() == before {
                return;
            }
        }
        self.schedule_save();
    }

    /// Mutates one field of the entry for `wm_class`, leaving every other key
    /// of that entry (including keys only the Windows client understands)
    /// untouched.
    pub fn set_integration_field(&self, wm_class: &str, key: &str, value: Value) {
        {
            let mut root = self.0.root.borrow_mut();
            let entries = integrations_array(&mut root);
            let Some(entry) = entries.iter_mut().find(|e| is_tmux_entry_for(e, wm_class)) else {
                return;
            };
            entry
                .as_object_mut()
                .expect("a matched entry is an object")
                .insert(key.to_string(), value);
        }
        self.schedule_save();
    }

    // ------------------------------------------------------------------ write

    /// Coalesces bursts of widget changes into one write, using the same
    /// epoch-counter debounce as config_watch.rs (no source removal, so no
    /// race between a fired and a cancelled timeout).
    pub fn schedule_save(&self) {
        for callback in self.0.on_edit.borrow().iter() {
            callback();
        }

        if self.0.write_blocked.get() {
            return;
        }

        // Debouncing needs a main context to attach the timeout to. Without
        // one (tests, and any non-GTK caller) write straight through rather
        // than dropping the change on the floor.
        if !glib::MainContext::default().is_owner() {
            self.save_now();
            return;
        }

        let current = self.0.save_epoch.get() + 1;
        self.0.save_epoch.set(current);

        let document = self.clone();
        glib::timeout_add_local_once(Duration::from_millis(SAVE_DEBOUNCE_MS), move || {
            if document.0.save_epoch.get() == current {
                document.save_now();
            }
        });
    }

    /// Serialize and write atomically: temp file in the same directory,
    /// fsync, rename. The daemon's watcher accepts `Created`, which is what
    /// gio's inotify backend reports for a file moved into a watched
    /// directory without WATCH_MOVES.
    /// Write immediately, bypassing the debounce. Used by tests and by the
    /// overwrite confirmation.
    pub fn flush(&self) {
        self.save_now();
    }

    fn save_now(&self) {
        if self.0.write_blocked.get() {
            return;
        }

        let contents = self.serialize();
        if let Err(e) = write_atomically(&self.0.path, &contents) {
            eprintln!(
                "SpotlightDimmer: could not write {}: {e}",
                self.0.path.display()
            );
            return;
        }
        *self.0.last_written.borrow_mut() = Some(contents);
    }

    fn serialize(&self) -> String {
        let mut contents = serde_json::to_string_pretty(&*self.0.root.borrow())
            .unwrap_or_else(|_| "{}".to_string());
        contents.push('\n');
        contents
    }

    // ------------------------------------------------------------------ watch

    /// Watch the config file for edits made outside this window (a text
    /// editor, another copy of the GUI). Mirrors config_watch.rs: monitor the
    /// parent *directory* because the file may not exist yet, filter on the
    /// basename, accept `Changed | Created`, debounce.
    pub fn start_watching(&self) {
        let Some(dir) = self.0.path.parent() else {
            return;
        };

        let monitor = match gio::File::for_path(dir)
            .monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
        {
            Ok(monitor) => monitor,
            Err(e) => {
                eprintln!("SpotlightDimmer: could not watch the config directory: {e}");
                return;
            }
        };

        let document = self.clone();
        monitor.connect_changed(move |_, file, _, event_type| {
            if file.basename().is_none_or(|b| b.as_os_str() != CONFIG_FILE) {
                return;
            }
            if !matches!(
                event_type,
                gio::FileMonitorEvent::Changed | gio::FileMonitorEvent::Created
            ) {
                return;
            }

            let current = document.0.watch_epoch.get() + 1;
            document.0.watch_epoch.set(current);

            let document = document.clone();
            glib::timeout_add_local_once(Duration::from_millis(WATCH_DEBOUNCE_MS), move || {
                if document.0.watch_epoch.get() == current {
                    document.reload_from_disk();
                }
            });
        });

        *self.0.monitor.borrow_mut() = Some(monitor);
    }

    fn reload_from_disk(&self) {
        let Ok(contents) = std::fs::read_to_string(&self.0.path) else {
            return;
        };

        // Our own write coming back at us.
        if self.0.last_written.borrow().as_deref() == Some(contents.as_str()) {
            return;
        }

        let (root, problem) = match serde_json::from_str::<Value>(&contents) {
            Ok(value @ Value::Object(_)) => (value, None),
            Ok(_) => (
                default_root(),
                Some(LoadProblem::Unparseable(
                    "the file's top level is not a JSON object".to_string(),
                )),
            ),
            Err(e) => {
                // Mid-edit garbage is common while another editor saves.
                // Surface it and stop writing rather than overwrite it.
                (default_root(), Some(LoadProblem::Unparseable(e.to_string())))
            }
        };

        // Adopt what is on disk and drop any write still pending, so an
        // external edit is never clobbered by an in-flight debounce.
        self.0.save_epoch.set(self.0.save_epoch.get() + 1);
        *self.0.root.borrow_mut() = root;
        self.0.write_blocked.set(problem.is_some());
        *self.0.problem.borrow_mut() = problem;
        *self.0.last_written.borrow_mut() = Some(contents);

        self.notify_problem();
        self.notify_reload();
    }

    fn notify_reload(&self) {
        for callback in self.0.on_reload.borrow().iter() {
            callback();
        }
    }

    fn notify_problem(&self) {
        for callback in self.0.on_problem.borrow().iter() {
            callback();
        }
    }
}

fn default_root() -> Value {
    // Seeded from OverlayConfig::default() so a first save produces a file a
    // user can read and reason about, not an empty object.
    json!({
        "Overlay": {
            "Mode": "FullScreen",
            "InactiveColor": "#000000",
            "InactiveOpacity": 153,
            "ActiveColor": "#000000",
            "ActiveOpacity": 102
        }
    })
}

fn integrations_array(root: &mut Value) -> &mut Vec<Value> {
    let object = root.as_object_mut().expect("root is an object");
    let entry = object
        .entry("AppIntegrations")
        .or_insert_with(|| Value::Array(Vec::new()));
    if !entry.is_array() {
        *entry = Value::Array(Vec::new());
    }
    entry.as_array_mut().expect("AppIntegrations is an array")
}

fn profiles_array(root: &mut Value) -> &mut Vec<Value> {
    let object = root.as_object_mut().expect("root is an object");
    let entry = object
        .entry("Profiles")
        .or_insert_with(|| Value::Array(Vec::new()));
    if !entry.is_array() {
        *entry = Value::Array(Vec::new());
    }
    entry.as_array_mut().expect("Profiles is an array")
}

fn mode_name(mode: DimmingMode) -> &'static str {
    match mode {
        DimmingMode::Partial => "Partial",
        DimmingMode::PartialWithActive => "PartialWithActive",
        DimmingMode::FullScreen | DimmingMode::Unknown => "FullScreen",
    }
}

/// The same test as `AppConfig::match_integration` with the "tmux" provider
/// (a missing or empty `Provider` defaults to it), so an entry the editor
/// shows as enabled is exactly one the daemon acts on.
fn is_tmux_entry_for(entry: &Value, wm_class: &str) -> bool {
    string_field(entry, "WmClass").as_deref() == Some(wm_class)
        && string_field(entry, "Provider").is_none_or(|p| p == "tmux")
}

fn string_field(entry: &Value, key: &str) -> Option<String> {
    entry
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn offset_field(entry: &Value, key: &str) -> i32 {
    entry
        .get(key)
        .and_then(Value::as_f64)
        .map(|v| v.round() as i32)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spotlight_dimmer_core::config::TtySource;

    /// A config with every key the Linux daemon does not consume, which a
    /// round-trip through `AppConfig` would silently delete.
    const FULL_CONFIG: &str = r##"{
  "$schema": "https://example.invalid/config.schema.json",
  "ConfigVersion": "0.8.5",
  "Overlay": {
    "Mode": "PartialWithActive",
    "InactiveColor": "#000000",
    "InactiveOpacity": 128,
    "ActiveColor": "#000000",
    "ActiveOpacity": 0,
    "ExcludeFromScreenCapture": false
  },
  "System": { "EnableLogging": true, "LogLevel": "Information", "LogRetentionDays": 7 },
  "Profiles": [ { "Name": "Dark Mode", "Mode": "Partial", "InactiveOpacity": 204 } ],
  "CurrentProfile": null,
  "AppIntegrations": [
    { "WmClass": "org.wezfurlong.wezterm", "Provider": "tmux", "ContentOffsetX": 0, "ContentOffsetY": 0 }
  ]
}"##;

    struct TempConfig {
        dir: PathBuf,
    }

    impl TempConfig {
        fn new(name: &str, contents: Option<&str>) -> TempConfig {
            let dir = std::env::temp_dir().join(format!("sd-config-gui-test-{name}"));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            if let Some(contents) = contents {
                std::fs::write(dir.join(CONFIG_FILE), contents).unwrap();
            }
            TempConfig { dir }
        }

        fn path(&self) -> PathBuf {
            self.dir.join(CONFIG_FILE)
        }

        fn read(&self) -> Value {
            serde_json::from_str(&std::fs::read_to_string(self.path()).unwrap()).unwrap()
        }
    }

    impl Drop for TempConfig {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn writing_preserves_every_key_the_daemon_ignores() {
        let temp = TempConfig::new("preserve", Some(FULL_CONFIG));
        let original: Value = serde_json::from_str(FULL_CONFIG).unwrap();

        let document = Document::load_from(temp.path());
        document.set_mode("Partial");
        document.set_inactive_opacity(200);
        document.flush();

        let written = temp.read();

        for key in ["$schema", "ConfigVersion", "System", "Profiles", "CurrentProfile"] {
            assert_eq!(written.get(key), original.get(key), "{key} was not preserved");
        }
        // Sibling keys inside Overlay survive too.
        assert_eq!(
            written["Overlay"]["ExcludeFromScreenCapture"],
            original["Overlay"]["ExcludeFromScreenCapture"]
        );
        assert_eq!(written["Overlay"]["ActiveOpacity"], json!(0));

        assert_eq!(written["Overlay"]["Mode"], json!("Partial"));
        assert_eq!(written["Overlay"]["InactiveOpacity"], json!(200));
    }

    #[test]
    fn edits_round_trip_through_the_daemons_own_parser() {
        let temp = TempConfig::new("roundtrip", Some(FULL_CONFIG));
        let document = Document::load_from(temp.path());

        document.set_mode("PartialWithActive");
        document.set_inactive_color("#102030");
        document.set_inactive_opacity(200);
        document.set_active_color("#405060");
        document.set_active_opacity(50);
        document.flush();

        let reloaded = AppConfig::from_json(&std::fs::read_to_string(temp.path()).unwrap()).unwrap();
        assert_eq!(reloaded.overlay, document.config().overlay);
        assert_eq!(reloaded.overlay.inactive_opacity, 200);
        assert_eq!(reloaded.overlay.active_opacity, 50);
    }

    const GHOSTTY: &str = "com.mitchellh.ghostty";
    const WEZTERM: &str = "org.wezfurlong.wezterm";

    #[test]
    fn enabling_writes_an_entry_the_daemon_matches() {
        let temp = TempConfig::new("enable", Some("{}"));
        let document = Document::load_from(temp.path());

        assert!(document.find_integration(GHOSTTY).is_none());
        document.enable_integration(GHOSTTY, "title", (2, 2));
        document.flush();

        let parsed = AppConfig::from_json(&std::fs::read_to_string(temp.path()).unwrap()).unwrap();
        let ghostty = parsed.match_integration(GHOSTTY, "tmux").unwrap();
        assert_eq!(ghostty.tty_source, TtySource::WindowTitle);
        assert_eq!((ghostty.content_offset_x, ghostty.content_offset_y), (2, 2));
        assert_eq!(
            document.find_integration(GHOSTTY),
            Some(IntegrationView {
                content_offset_x: 2,
                content_offset_y: 2
            })
        );
    }

    #[test]
    fn enabling_an_existing_entry_keeps_it_untouched() {
        let temp = TempConfig::new("reenable", Some(FULL_CONFIG));
        let document = Document::load_from(temp.path());
        document.set_integration_field(WEZTERM, "ContentOffsetX", json!(8));
        document.set_integration_field(WEZTERM, "ProcessName", json!("wezterm-gui.exe"));

        document.enable_integration(WEZTERM, "wezterm", (0, 0));
        document.flush();

        let entries = temp.read()["AppIntegrations"].as_array().unwrap().clone();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["ContentOffsetX"], json!(8));
        assert_eq!(entries[0]["ProcessName"], json!("wezterm-gui.exe"));
    }

    #[test]
    fn a_missing_provider_counts_as_tmux_like_the_daemon() {
        let contents = r#"{"AppIntegrations": [
            {"WmClass": "org.wezfurlong.wezterm", "ContentOffsetY": 5},
            {"WmClass": "com.mitchellh.ghostty", "Provider": "other"}
        ]}"#;
        let temp = TempConfig::new("noprovider", Some(contents));
        let document = Document::load_from(temp.path());

        let wezterm = document.find_integration(WEZTERM).unwrap();
        assert_eq!(wezterm.content_offset_y, 5);
        assert!(document.find_integration(GHOSTTY).is_none());
    }

    #[test]
    fn disabling_removes_duplicates_and_keeps_other_entries() {
        let contents = r#"{"AppIntegrations": [
            {"WmClass": "com.mitchellh.ghostty", "Provider": "tmux", "TtySource": "title"},
            {"WmClass": "kitty", "Provider": "tmux", "TtySource": "title"},
            {"ProcessName": "WindowsTerminal.exe", "Provider": "windows-terminal"},
            {"WmClass": "com.mitchellh.ghostty"}
        ]}"#;
        let temp = TempConfig::new("disable", Some(contents));
        let document = Document::load_from(temp.path());

        document.disable_integration(GHOSTTY);
        document.flush();

        assert!(document.find_integration(GHOSTTY).is_none());
        let entries = temp.read()["AppIntegrations"].as_array().unwrap().clone();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["WmClass"], json!("kitty"));
        assert_eq!(entries[1]["ProcessName"], json!("WindowsTerminal.exe"));
    }

    #[test]
    fn field_edits_reach_the_right_entry_past_hidden_ones() {
        let contents = r#"{"AppIntegrations": [
            {"WmClass": "kitty", "Provider": "tmux", "ContentOffsetX": 1},
            {"WmClass": "org.wezfurlong.wezterm", "Provider": "tmux", "ContentOffsetX": 0}
        ]}"#;
        let temp = TempConfig::new("offset", Some(contents));
        let document = Document::load_from(temp.path());

        document.set_integration_field(WEZTERM, "ContentOffsetX", json!(4));
        document.flush();

        let entries = temp.read()["AppIntegrations"].as_array().unwrap().clone();
        assert_eq!(entries[0]["ContentOffsetX"], json!(1));
        assert_eq!(entries[1]["ContentOffsetX"], json!(4));
    }

    const PROFILES_CONFIG: &str = r##"{
  "Overlay": { "Mode": "Partial", "InactiveOpacity": 10, "Enabled": false, "ExcludeFromScreenCapture": true },
  "Profiles": [
    { "Name": "Umbra", "Mode": "PartialWithActive", "InactiveColor": "#101010",
      "InactiveOpacity": 230, "ActiveColor": "#202020", "ActiveOpacity": 90, "Hotkey": "F9" },
    { "Name": "Sparse", "InactiveOpacity": 50 },
    { "Mode": "Partial" }
  ],
  "CurrentProfile": null
}"##;

    #[test]
    fn profiles_skip_nameless_entries_and_default_like_windows() {
        let temp = TempConfig::new("profiles-read", Some(PROFILES_CONFIG));
        let document = Document::load_from(temp.path());

        let profiles = document.profiles();
        assert_eq!(profiles.len(), 2);
        assert_eq!(
            profiles[1],
            ProfileView {
                name: "Sparse".to_string(),
                mode: "FullScreen".to_string(),
                inactive_color: "#000000".to_string(),
                inactive_opacity: 50,
                active_color: "#000000".to_string(),
                active_opacity: 102,
            }
        );
        assert_eq!(document.current_profile(), None);
    }

    #[test]
    fn applying_a_profile_writes_its_overlay_and_keeps_everything_else() {
        let temp = TempConfig::new("profiles-apply", Some(PROFILES_CONFIG));
        let document = Document::load_from(temp.path());

        assert!(document.apply_profile("Umbra"));
        assert!(!document.apply_profile("Nope"));

        // Written straight through, no flush needed.
        let written = temp.read();
        let overlay = &written["Overlay"];
        assert_eq!(overlay["Mode"], json!("PartialWithActive"));
        assert_eq!(overlay["InactiveColor"], json!("#101010"));
        assert_eq!(overlay["InactiveOpacity"], json!(230));
        assert_eq!(overlay["ActiveColor"], json!("#202020"));
        assert_eq!(overlay["ActiveOpacity"], json!(90));
        // The on/off state is not part of a profile.
        assert_eq!(overlay["Enabled"], json!(false));
        assert_eq!(overlay["ExcludeFromScreenCapture"], json!(true));
        assert_eq!(written["CurrentProfile"], json!("Umbra"));
        assert_eq!(written["Profiles"][0]["Hotkey"], json!("F9"));
        assert_eq!(written["Profiles"].as_array().unwrap().len(), 3);

        assert!(document.overlay_matches_profile("Umbra"));
        document.set_active_opacity(91);
        assert!(!document.overlay_matches_profile("Umbra"));
    }

    #[test]
    fn matching_compares_through_the_parser() {
        let contents = r##"{
          "Overlay": { "Mode": "FullScreen", "InactiveColor": "abcdef" },
          "Profiles": [ { "Name": "Lower", "InactiveColor": "#ABCDEF" } ]
        }"##;
        let temp = TempConfig::new("profiles-match", Some(contents));
        let document = Document::load_from(temp.path());
        assert!(document.overlay_matches_profile("Lower"));
        assert!(!document.overlay_matches_profile("Missing"));
    }

    #[test]
    fn saving_updates_in_place_or_appends() {
        let temp = TempConfig::new("profiles-save", Some(PROFILES_CONFIG));
        let document = Document::load_from(temp.path());

        document.save_profile("Umbra");
        document.save_profile("Fresh");
        document.flush();

        let written = temp.read();
        let entries = written["Profiles"].as_array().unwrap();
        assert_eq!(entries.len(), 4);
        // Updated in place: position and unknown keys survive.
        assert_eq!(entries[0]["Name"], json!("Umbra"));
        assert_eq!(entries[0]["Mode"], json!("Partial"));
        assert_eq!(entries[0]["InactiveOpacity"], json!(10));
        assert_eq!(entries[0]["Hotkey"], json!("F9"));
        assert_eq!(entries[3]["Name"], json!("Fresh"));
        assert_eq!(entries[3]["InactiveColor"], json!("#000000"));
        assert_eq!(written["CurrentProfile"], json!("Fresh"));

        assert!(document.overlay_matches_profile("Fresh"));
    }

    #[test]
    fn saving_keeps_an_unknown_mode_verbatim() {
        let temp = TempConfig::new("profiles-unknown", Some(r#"{"Overlay": {"Mode": "Sideways"}}"#));
        let document = Document::load_from(temp.path());
        document.save_profile("Odd");
        document.flush();
        assert_eq!(temp.read()["Profiles"][0]["Mode"], json!("Sideways"));
    }

    #[test]
    fn deleting_clears_the_current_profile_only_when_it_pointed_there() {
        let temp = TempConfig::new("profiles-delete", Some(PROFILES_CONFIG));
        let document = Document::load_from(temp.path());

        document.apply_profile("Umbra");
        document.delete_profile("Sparse");
        assert_eq!(document.current_profile().as_deref(), Some("Umbra"));

        document.delete_profile("Umbra");
        document.flush();
        let written = temp.read();
        assert_eq!(written["CurrentProfile"], Value::Null);
        // The nameless entry is not ours to remove.
        assert_eq!(written["Profiles"].as_array().unwrap().len(), 1);
        // The overlay keeps the deleted profile's values.
        assert_eq!(written["Overlay"]["InactiveOpacity"], json!(230));
    }

    #[test]
    fn an_unparseable_file_is_never_overwritten_until_confirmed() {
        let temp = TempConfig::new("broken", Some("{ \"Overlay\": }"));
        let document = Document::load_from(temp.path());

        assert!(matches!(
            document.problem(),
            Some(LoadProblem::Unparseable(_))
        ));

        document.set_inactive_opacity(10);
        document.flush();
        assert_eq!(
            std::fs::read_to_string(temp.path()).unwrap(),
            "{ \"Overlay\": }",
            "the broken file must survive untouched"
        );

        document.confirm_overwrite();
        assert!(document.problem().is_none());
        assert_eq!(temp.read()["Overlay"]["InactiveOpacity"], json!(10));
    }

    #[test]
    fn a_missing_file_is_created_with_the_daemon_defaults() {
        let temp = TempConfig::new("missing", None);
        let document = Document::load_from(temp.path());

        assert!(document.problem().is_none());
        assert_eq!(document.config(), AppConfig::default());

        document.set_mode("Partial");
        document.flush();
        assert_eq!(temp.read()["Overlay"]["InactiveOpacity"], json!(153));
        assert_eq!(temp.read()["Overlay"]["Mode"], json!("Partial"));
    }

    #[test]
    fn writes_end_in_a_newline_and_leave_no_temp_file_behind() {
        let temp = TempConfig::new("atomic", Some(FULL_CONFIG));
        let document = Document::load_from(temp.path());
        document.set_mode("Partial");
        document.flush();

        assert!(std::fs::read_to_string(temp.path()).unwrap().ends_with("}\n"));
        let leftovers: Vec<_> = std::fs::read_dir(&temp.dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| name != CONFIG_FILE)
            .collect();
        assert!(leftovers.is_empty(), "left behind: {leftovers:?}");
    }
}
