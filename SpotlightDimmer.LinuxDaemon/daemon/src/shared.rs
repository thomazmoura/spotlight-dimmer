//! State shared between the zbus executor threads and the main-thread event
//! loop. Kept minimal: only what D-Bus handlers must read/write synchronously
//! to produce method replies; everything else lives in the event loop.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterInfo {
    pub compositor: String,
    pub renders_overlays: bool,
}

#[derive(Debug)]
pub struct Shared {
    /// Dimming enabled/paused flag; written by Toggle()/Enabled property and
    /// synced into AppState via Event::EnabledChanged.
    enabled: AtomicBool,
    /// Latest emitted overlays payload JSON; returned by RegisterRenderer so
    /// a (re)connecting renderer can paint immediately.
    last_payload: Mutex<String>,
    /// Registered adapters by unique bus name. Mutated by the event loop,
    /// read by the NameOwnerChanged watcher for filtering.
    adapters: Mutex<HashMap<String, AdapterInfo>>,
}

impl Shared {
    pub fn new() -> Shared {
        Shared {
            enabled: AtomicBool::new(true),
            last_payload: Mutex::new(String::from(r#"{"serial":0,"enabled":true,"monitors":[]}"#)),
            adapters: Mutex::new(HashMap::new()),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn set_enabled(&self, value: bool) {
        self.enabled.store(value, Ordering::SeqCst);
    }

    /// Flip the enabled flag, returning the new value.
    pub fn toggle_enabled(&self) -> bool {
        !self.enabled.fetch_xor(true, Ordering::SeqCst)
    }

    pub fn last_payload(&self) -> String {
        self.last_payload.lock().unwrap().clone()
    }

    pub fn set_last_payload(&self, json: String) {
        *self.last_payload.lock().unwrap() = json;
    }

    pub fn register_adapter(&self, sender: String, info: AdapterInfo) {
        self.adapters.lock().unwrap().insert(sender, info);
    }

    pub fn remove_adapter(&self, sender: &str) -> Option<AdapterInfo> {
        self.adapters.lock().unwrap().remove(sender)
    }

    pub fn is_adapter(&self, sender: &str) -> bool {
        self.adapters.lock().unwrap().contains_key(sender)
    }

    /// True when a registered adapter renders overlays itself (the GNOME
    /// extension); the daemon then suppresses its own layer-shell rendering.
    /// Consumed by the layer-shell renderer (render feature).
    #[allow(dead_code)]
    pub fn has_renderer_adapter(&self) -> bool {
        self.adapters
            .lock()
            .unwrap()
            .values()
            .any(|a| a.renders_overlays)
    }

    /// True when at least one compositor adapter is registered.
    /// Consumed by the layer-shell renderer (render feature).
    #[allow(dead_code)]
    pub fn has_adapters(&self) -> bool {
        !self.adapters.lock().unwrap().is_empty()
    }
}
