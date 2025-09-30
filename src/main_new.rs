mod config;
mod overlay;
mod platform;

use config::Config;
use overlay::OverlayManager;
use platform::{DisplayManager, WindowManager, WindowsDisplayManager, WindowsWindowManager};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

fn main() {
    println!("[Main] Spotlight Dimmer starting...");

    // Load configuration
    let config = Arc::new(Mutex::new(Config::load()));

    // Initialize display and window managers
    let display_manager = WindowsDisplayManager;
    let window_manager = WindowsWindowManager;

    // Get initial display list
    let displays = match display_manager.get_displays() {
        Ok(displays) => displays,
        Err(e) => {
            eprintln!("[Main] Failed to get displays: {}", e);
            return;
        }
    };

    println!("[Main] Found {} display(s)", displays.len());

    // Create overlay manager
    let overlay_color = {
        let cfg = config.lock().unwrap();
        cfg.overlay_color.clone()
    };

    let overlay_manager = Arc::new(Mutex::new(
        match OverlayManager::new(overlay_color) {
            Ok(manager) => manager,
            Err(e) => {
                eprintln!("[Main] Failed to create overlay manager: {}", e);
                return;
            }
        },
    ));

    // Create initial overlays if dimming is enabled
    {
        let cfg = config.lock().unwrap();
        if cfg.is_dimming_enabled {
            let mut manager = overlay_manager.lock().unwrap();
            if let Err(e) = manager.create_overlays(&displays) {
                eprintln!("[Main] Failed to create initial overlays: {}", e);
                return;
            }
            println!("[Main] Created {} overlay(s)", manager.count());
        }
    }

    // Track last known state
    let mut last_window_handle: Option<u64> = None;
    let mut last_display_id: Option<String> = None;
    let mut last_display_count = displays.len();

    // Track config file modification time for periodic reloading
    let mut last_config_modified = Config::last_modified();
    let mut loop_counter: u32 = 0;

    println!("[Main] Starting focus monitoring loop...");

    // Main monitoring loop
    loop {
        thread::sleep(Duration::from_millis(100));
        loop_counter = loop_counter.wrapping_add(1);

        // Check for config file changes every 2 seconds (20 iterations of 100ms)
        if loop_counter % 20 == 0 {
            if let Some((new_config, new_modified_time)) = Config::reload_if_changed(last_config_modified) {
                let mut cfg = config.lock().unwrap();
                let old_dimming_enabled = cfg.is_dimming_enabled;
                let old_color = cfg.overlay_color.clone();

                // Update config
                *cfg = new_config.clone();
                drop(cfg); // Release lock before potentially recreating overlays

                last_config_modified = Some(new_modified_time);

                // Handle dimming enable/disable toggle
                if old_dimming_enabled != new_config.is_dimming_enabled {
                    if new_config.is_dimming_enabled {
                        println!("[Main] Dimming enabled via config change");
                        if let Ok(current_displays) = display_manager.get_displays() {
                            let mut manager = overlay_manager.lock().unwrap();
                            if let Err(e) = manager.create_overlays(&current_displays) {
                                eprintln!("[Main] Failed to create overlays: {}", e);
                            }
                        }
                    } else {
                        println!("[Main] Dimming disabled via config change");
                        let mut manager = overlay_manager.lock().unwrap();
                        manager.close_all();
                    }
                }
                // Handle color change (only if dimming is enabled)
                else if new_config.is_dimming_enabled &&
                        (old_color.r != new_config.overlay_color.r ||
                         old_color.g != new_config.overlay_color.g ||
                         old_color.b != new_config.overlay_color.b ||
                         old_color.a != new_config.overlay_color.a) {
                    println!("[Main] Overlay color changed via config");
                    if let Ok(current_displays) = display_manager.get_displays() {
                        let mut manager = overlay_manager.lock().unwrap();
                        if let Err(e) = manager.set_color(new_config.overlay_color.clone(), &current_displays) {
                            eprintln!("[Main] Failed to update overlay color: {}", e);
                        }
                    }
                }
            }
        }

        // Check if dimming is enabled
        let is_dimming_enabled = {
            let cfg = config.lock().unwrap();
            cfg.is_dimming_enabled
        };

        if !is_dimming_enabled {
            thread::sleep(Duration::from_millis(400)); // Longer sleep when disabled
            continue;
        }

        // Check for display configuration changes
        if let Ok(current_display_count) = display_manager.get_display_count() {
            if current_display_count != last_display_count {
                println!(
                    "[Main] Display configuration changed: {} -> {} displays",
                    last_display_count, current_display_count
                );

                // Get new display list
                if let Ok(new_displays) = display_manager.get_displays() {
                    let mut manager = overlay_manager.lock().unwrap();
                    if let Err(e) = manager.recreate_overlays(&new_displays) {
                        eprintln!("[Main] Failed to recreate overlays: {}", e);
                    } else {
                        println!("[Main] Recreated {} overlay(s)", manager.count());
                    }
                }

                last_display_count = current_display_count;
                last_window_handle = None;
                last_display_id = None;
                continue;
            }
        }

        // Get active window
        match window_manager.get_active_window() {
            Ok(active_window) => {
                // Skip our own overlay windows
                if active_window.window_title.contains("Spotlight Dimmer Overlay") {
                    continue;
                }

                // Check if window or display changed
                let window_changed = Some(active_window.handle) != last_window_handle;
                let display_changed = last_display_id.as_ref() != Some(&active_window.display_id);

                if window_changed || display_changed {
                    if window_changed {
                        println!(
                            "[Main] Active window: {} ({})",
                            active_window.window_title, active_window.process_name
                        );
                    }
                    if display_changed && !window_changed {
                        println!(
                            "[Main] Window moved to display: {}",
                            active_window.display_id
                        );
                    }

                    // Update overlay visibility
                    let manager = overlay_manager.lock().unwrap();
                    manager.update_visibility(&active_window.display_id);

                    last_window_handle = Some(active_window.handle);
                    last_display_id = Some(active_window.display_id);
                }
            }
            Err(_) => {
                // Silently ignore errors to avoid spam
                if last_window_handle.is_some() {
                    last_window_handle = None;
                    last_display_id = None;
                }
            }
        }
    }
}