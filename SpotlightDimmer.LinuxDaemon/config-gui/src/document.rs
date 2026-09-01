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

use spotlight_dimmer_core::config::{AppConfig, TtySource};

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

/// One `AppIntegrations` entry as the *editor* sees it.
///
/// Deliberately not `core`'s `AppIntegration`: the parser drops entries with
/// an empty `WmClass`, so a freshly added row would vanish from the list the
/// moment it was created. This view keeps every array element, applying the
/// same per-field defaults the daemon applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationView {
    pub wm_class: String,
    pub provider: String,
    pub tty_source: TtySource,
    pub content_offset_x: i32,
    pub content_offset_y: i32,
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

    pub fn set_mode(&self, mode: &str) {
        self.set_overlay("Mode", json!(mode));
    }

    pub fn set_inactive_color(&self, hex: &str) {
        self.set_overlay("InactiveColor", json!(hex));
    }

    pub fn set_inactive_opacity(&self, opacity: u8) {
        self.set_overlay("InactiveOpacity", json!(opacity));
    }

    pub fn set_active_color(&self, hex: &str) {
        self.set_overlay("ActiveColor", json!(hex));
    }

    pub fn set_active_opacity(&self, opacity: u8) {
        self.set_overlay("ActiveOpacity", json!(opacity));
    }

    /// Mutates a single leaf under `Overlay`, creating that object only if it
    /// is absent and never touching a sibling key.
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
        self.schedule_save();
    }

    // ----------------------------------------------------------- integrations

    /// Every element of the `AppIntegrations` array, including entries the
    /// daemon would drop (empty `WmClass`), so the editor can show a row that
    /// has not been filled in yet.
    pub fn integrations(&self) -> Vec<IntegrationView> {
        let root = self.0.root.borrow();
        let Some(entries) = root.get("AppIntegrations").and_then(Value::as_array) else {
            return Vec::new();
        };

        entries
            .iter()
            .map(|entry| IntegrationView {
                wm_class: string_field(entry, "WmClass").unwrap_or_default(),
                provider: string_field(entry, "Provider").unwrap_or_else(|| "tmux".to_string()),
                tty_source: string_field(entry, "TtySource")
                    .map(|s| TtySource::parse(&s))
                    .unwrap_or_default(),
                content_offset_x: offset_field(entry, "ContentOffsetX"),
                content_offset_y: offset_field(entry, "ContentOffsetY"),
            })
            .collect()
    }

    /// Appends an entry and returns its index. Written with explicit
    /// defaults so the resulting JSON is self-documenting rather than
    /// relying on the parser's implicit fallbacks.
    pub fn add_integration(&self) -> usize {
        let index = {
            let mut root = self.0.root.borrow_mut();
            let entries = integrations_array(&mut root);
            entries.push(json!({
                "WmClass": "",
                "Provider": "tmux",
                "TtySource": "wezterm",
                "ContentOffsetX": 0,
                "ContentOffsetY": 0
            }));
            entries.len() - 1
        };
        self.schedule_save();
        index
    }

    pub fn remove_integration(&self, index: usize) {
        {
            let mut root = self.0.root.borrow_mut();
            let entries = integrations_array(&mut root);
            if index >= entries.len() {
                return;
            }
            entries.remove(index);
        }
        self.schedule_save();
    }

    /// Mutates one field of one entry, leaving every other key of that entry
    /// (including keys only the Windows client understands) untouched.
    pub fn set_integration_field(&self, index: usize, key: &str, value: Value) {
        {
            let mut root = self.0.root.borrow_mut();
            let entries = integrations_array(&mut root);
            let Some(entry) = entries.get_mut(index) else {
                return;
            };
            if !entry.is_object() {
                *entry = Value::Object(Map::new());
            }
            entry
                .as_object_mut()
                .expect("entry is an object")
                .insert(key.to_string(), value);
        }
        self.schedule_save();
    }

    // ------------------------------------------------------------------ write

    /// Coalesces bursts of widget changes into one write, using the same
    /// epoch-counter debounce as config_watch.rs (no source removal, so no
    /// race between a fired and a cancelled timeout).
    pub fn schedule_save(&self) {
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

fn write_atomically(path: &Path, contents: &str) -> std::io::Result<()> {
    use std::io::Write;

    let dir = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "config path has no parent")
    })?;
    std::fs::create_dir_all(dir)?;

    let temp = dir.join(format!("{CONFIG_FILE}.tmp-{}", std::process::id()));
    {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
    }

    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn integrations_are_editable_and_keep_unknown_entry_keys() {
        let temp = TempConfig::new("integrations", Some(FULL_CONFIG));
        let document = Document::load_from(temp.path());

        // Existing entry: change one field, leave the rest alone.
        document.set_integration_field(0, "TtySource", json!("title"));
        // New entry, filled in the way the detail pane fills one.
        let index = document.add_integration();
        document.set_integration_field(index, "WmClass", json!("com.mitchellh.ghostty"));
        document.set_integration_field(index, "ContentOffsetX", json!(2));
        document.set_integration_field(index, "ContentOffsetY", json!(2));
        document.flush();

        let written = temp.read();
        let entries = written["AppIntegrations"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["WmClass"], json!("org.wezfurlong.wezterm"));
        assert_eq!(entries[0]["Provider"], json!("tmux"));
        assert_eq!(entries[0]["TtySource"], json!("title"));

        let parsed = AppConfig::from_json(&std::fs::read_to_string(temp.path()).unwrap()).unwrap();
        let ghostty = parsed.match_integration("com.mitchellh.ghostty", "tmux").unwrap();
        assert_eq!(ghostty.tty_source, TtySource::WezTermCli);
        assert_eq!(ghostty.content_offset_x, 2);
        assert_eq!(ghostty.content_offset_y, 2);
    }

    #[test]
    fn a_new_entry_stays_visible_before_its_wm_class_is_typed() {
        let temp = TempConfig::new("newentry", Some("{}"));
        let document = Document::load_from(temp.path());

        document.add_integration();

        // The parser drops it, but the editor must still show the row.
        assert!(document.config().app_integrations.is_empty());
        assert_eq!(document.integrations().len(), 1);
        assert_eq!(document.integrations()[0].provider, "tmux");
    }

    #[test]
    fn removing_an_entry_shifts_the_rest() {
        let temp = TempConfig::new("remove", Some(FULL_CONFIG));
        let document = Document::load_from(temp.path());

        let index = document.add_integration();
        document.set_integration_field(index, "WmClass", json!("kitty"));
        document.remove_integration(0);
        document.flush();

        let entries = document.integrations();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].wm_class, "kitty");
        assert_eq!(temp.read()["AppIntegrations"].as_array().unwrap().len(), 1);
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
