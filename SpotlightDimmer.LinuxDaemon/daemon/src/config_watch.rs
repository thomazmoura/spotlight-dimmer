//! Config file loading and hot-reload watching.
//!
//! Mirrors configBridge.js: watch the parent *directory* (the file may not
//! exist yet), filter on the basename, debounce 100 ms. Parsing itself lives
//! in spotlight_dimmer_core::config.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use async_channel::Sender;
use gio::prelude::*;

use spotlight_dimmer_core::config::AppConfig;

use crate::events::Event;

const CONFIG_DIR: &str = "SpotlightDimmer";
const CONFIG_FILE: &str = "config.json";
const DEBOUNCE_MS: u64 = 100;

/// `~/.config/SpotlightDimmer/config.json` (same file as the GNOME extension
/// used; the Windows client reads the same schema from %AppData%).
pub fn config_path() -> PathBuf {
    glib::user_config_dir().join(CONFIG_DIR).join(CONFIG_FILE)
}

/// Load and parse the config file. Returns None when the file is missing or
/// unparseable so callers keep the previous configuration (configBridge.js
/// semantics).
pub fn load(path: &Path) -> Option<AppConfig> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            println!(
                "SpotlightDimmer: config file not found, keeping current config (expected {})",
                path.display()
            );
            return None;
        }
    };

    match AppConfig::from_json(&contents) {
        Ok(config) => Some(config),
        Err(e) => {
            eprintln!("SpotlightDimmer: error parsing config: {e}");
            None
        }
    }
}

/// Watch the config directory; sends Event::ConfigFileChanged (debounced).
/// The returned FileMonitor must be kept alive for the watch to persist.
pub fn watch(tx: Sender<Event>) -> Option<gio::FileMonitor> {
    let path = config_path();
    let dir = path.parent()?;

    let monitor = gio::File::for_path(dir)
        .monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
        .map_err(|e| eprintln!("SpotlightDimmer: error watching config dir: {e}"))
        .ok()?;

    // Debounce via an epoch counter instead of source removal: each change
    // bumps the epoch, and only the timeout holding the latest epoch fires.
    let epoch = Rc::new(Cell::new(0u64));

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

        let current = epoch.get() + 1;
        epoch.set(current);

        let epoch = epoch.clone();
        let tx = tx.clone();
        glib::timeout_add_local_once(Duration::from_millis(DEBOUNCE_MS), move || {
            if epoch.get() == current {
                let _ = tx.send_blocking(Event::ConfigFileChanged);
            }
        });
    });

    println!("SpotlightDimmer: watching {} for changes", path.display());
    Some(monitor)
}
