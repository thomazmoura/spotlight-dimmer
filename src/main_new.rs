mod config;
mod message_window;
mod overlay;
mod platform;
mod tray;

#[cfg(windows)]
use config::Config;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::{Duration, Instant};

#[cfg(windows)]
use message_window::MessageWindow;
#[cfg(windows)]
use overlay::OverlayManager;
#[cfg(windows)]
use platform::{DisplayManager, WindowManager, WindowsDisplayManager, WindowsWindowManager};
#[cfg(windows)]
use tray::TrayIcon;

#[cfg(windows)]
use winapi::um::winuser::{
    DispatchMessageW, PeekMessageW, PostMessageW, TranslateMessage, MSG, PM_REMOVE,
};

#[cfg(windows)]
use winapi::um::fileapi::{FindFirstChangeNotificationW, FindNextChangeNotification};
#[cfg(windows)]
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use winapi::um::synchapi::WaitForSingleObject;
#[cfg(windows)]
use winapi::um::winbase::WAIT_OBJECT_0;
#[cfg(windows)]
use winapi::um::winnt::{FILE_NOTIFY_CHANGE_LAST_WRITE, HANDLE};

#[cfg(windows)]
use widestring::U16CString;

#[cfg(windows)]
/// Sets up file watching for the config directory using Windows FindFirstChangeNotificationW
/// Returns a HANDLE that can be checked with WaitForSingleObject
fn setup_config_file_watching() -> Option<HANDLE> {
    // Get config file path and extract directory
    let config_path = Config::config_path().ok()?;
    let config_dir = config_path.parent()?;

    // Convert path to wide string for Windows API
    let wide_path = match U16CString::from_os_str(config_dir.as_os_str()) {
        Ok(path) => path,
        Err(e) => {
            eprintln!(
                "[FileWatch] Failed to convert config path to wide string: {}",
                e
            );
            return None;
        }
    };

    unsafe {
        let handle = FindFirstChangeNotificationW(
            wide_path.as_ptr(),
            0, // bWatchSubtree: FALSE - only watch the directory itself, not subdirectories
            FILE_NOTIFY_CHANGE_LAST_WRITE, // Watch for file modification events
        );

        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            eprintln!("[FileWatch] Failed to create file change notification");
            return None;
        }

        println!(
            "[FileWatch] Watching config directory for changes: {:?}",
            config_dir
        );
        Some(handle)
    }
}

#[cfg(windows)]
fn hide_console_if_not_launched_from_terminal() {
    use winapi::um::wincon::{FreeConsole, GetConsoleProcessList};

    unsafe {
        // Check how many processes are attached to this console
        let mut process_list = [0u32; 2];
        let count = GetConsoleProcessList(process_list.as_mut_ptr(), 2);

        // If count == 1, only this process is attached (launched from GUI)
        // If count > 1, launched from terminal with parent process
        if count == 1 {
            // Hide the console window since we weren't launched from a terminal
            FreeConsole();
        }
    }
}

#[cfg(windows)]
fn set_dpi_awareness() {
    use winapi::shared::windef::DPI_AWARENESS_CONTEXT;
    use winapi::um::winuser::SetProcessDpiAwarenessContext;

    // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = -4
    // This is the most advanced DPI awareness mode that ensures:
    // - Application receives physical pixels (not scaled)
    // - Per-monitor DPI awareness (each monitor can have different scaling)
    // - Automatic non-client area (title bar, borders) scaling
    const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: DPI_AWARENESS_CONTEXT =
        -4isize as DPI_AWARENESS_CONTEXT;

    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == 0 {
            eprintln!("[Main] Warning: Failed to set DPI awareness context");
            eprintln!(
                "[Main] Overlays may not align correctly with windows at non-100% display scales"
            );
        } else {
            println!("[Main] DPI awareness set to Per-Monitor V2");
        }
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn hide_console_if_not_launched_from_terminal() {
    // No-op on non-Windows platforms
}

// Phase 2 & 3: Event-driven infrastructure (commented out for incremental migration)
// These will be enabled in later phases after Phase 1 is complete and tested

// #[cfg(windows)]
// // Custom window messages for our event-driven architecture
// const WM_USER_FOREGROUND_CHANGED: UINT = 0x0400; // WM_USER base
// const WM_USER_WINDOW_MOVED: UINT = 0x0401;
// const WM_USER_CONFIG_CHANGED: UINT = 0x0402;

#[cfg(windows)]
// Process Windows messages in a non-blocking manner (used during transition phase)
// Returns the number of messages processed
fn process_windows_messages() -> u32 {
    let mut count = 0;
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
            count += 1;
        }
    }
    count
}

#[cfg(windows)]
fn main() {
    // Set DPI awareness FIRST - before any Windows API calls
    set_dpi_awareness();

    // Hide console if not launched from terminal
    hide_console_if_not_launched_from_terminal();

    println!("[Main] Spotlight Dimmer starting...");

    // Create exit flag for coordinating shutdown
    let exit_flag = Arc::new(AtomicBool::new(false));

    // Load configuration
    let config = Arc::new(Mutex::new(Config::load()));

    // Create pause flag from config
    let pause_flag = Arc::new(AtomicBool::new({
        let cfg = config.lock().unwrap();
        cfg.is_paused
    }));

    // Create system tray icon
    let tray_icon = match TrayIcon::new(
        "spotlight-dimmer-icon.ico",
        "Spotlight Dimmer",
        exit_flag.clone(),
        pause_flag.clone(),
    ) {
        Ok(tray) => tray,
        Err(e) => {
            eprintln!("[Main] Failed to create system tray icon: {}", e);
            eprintln!("[Main] Make sure spotlight-dimmer-icon.ico and spotlight-dimmer-icon-paused.ico exist in the same directory as the executable");
            return;
        }
    };

    // Phase 2: Create message-only window for event-driven communication
    let message_window = match MessageWindow::new() {
        Ok(window) => window,
        Err(e) => {
            eprintln!("[Main] Failed to create message window: {}", e);
            return;
        }
    };
    println!("[Main] Phase 2: Message window infrastructure initialized");

    // Set up file watching for config directory
    let config_watch_handle = setup_config_file_watching();
    if config_watch_handle.is_some() {
        println!("[Main] Config file watching enabled (event-driven detection)");
    } else {
        eprintln!("[Main] Warning: Config file watching failed to initialize - config changes may not be detected");
    }

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
    let (inactive_color, active_color) = {
        let cfg = config.lock().unwrap();
        (cfg.overlay_color.clone(), cfg.active_overlay_color.clone())
    };

    #[allow(clippy::arc_with_non_send_sync)] // Arc used only in single-threaded main loop
    let overlay_manager = Arc::new(Mutex::new(
        match OverlayManager::new(inactive_color, active_color) {
            Ok(manager) => manager,
            Err(e) => {
                eprintln!("[Main] Failed to create overlay manager: {}", e);
                return;
            }
        },
    ));

    // Create initial overlays based on enabled flags
    {
        let cfg = config.lock().unwrap();
        let mut manager = overlay_manager.lock().unwrap();

        if cfg.is_dimming_enabled {
            if let Err(e) = manager.create_inactive_overlays(&displays) {
                eprintln!("[Main] Failed to create inactive overlays: {}", e);
                return;
            }
            println!(
                "[Main] Created {} inactive overlay(s)",
                manager.inactive_count()
            );
        }

        if cfg.is_active_overlay_enabled {
            if let Err(e) = manager.create_active_overlays(&displays) {
                eprintln!("[Main] Failed to create active overlays: {}", e);
                return;
            }
            println!(
                "[Main] Created {} active overlay(s)",
                manager.active_count()
            );
        }
    }

    // Track last known state
    let mut last_window_handle: Option<u64> = None;
    let mut last_display_id: Option<String> = None;
    let mut last_display_count = displays.len();
    let mut last_paused = pause_flag.load(Ordering::SeqCst);
    let mut last_window_rect: Option<winapi::shared::windef::RECT> = None;

    // Track config file modification time for reloading when file watcher triggers
    let mut last_config_modified = Config::last_modified();

    // Phase 1: Message loop optimization metrics
    let mut total_messages_processed: u64 = 0;
    let mut metrics_start_time = Instant::now();
    let mut last_activity_time = Instant::now();

    // Phase 2: Test message timing
    let mut last_test_message_time = Instant::now();

    println!("[Main] Starting focus monitoring loop...");
    println!("[Main] Phase 1: Message loop optimization active - collecting baseline metrics");
    println!("[Main] Phase 2: Test messages will be posted every 10 seconds");
    if last_paused {
        println!("[Main] Application started in PAUSED state");
        let manager = overlay_manager.lock().unwrap();
        manager.hide_all();
        // Update tray icon and tooltip to show paused state
        if let Err(e) = tray_icon.update_icon(true) {
            eprintln!("[Main] Failed to update tray icon: {}", e);
        }
    }

    // Main monitoring loop
    loop {
        // Phase 1: Process Windows messages and count them
        let messages_this_iteration = process_windows_messages();
        total_messages_processed += messages_this_iteration as u64;

        // Track activity for adaptive sleep
        if messages_this_iteration > 0 {
            last_activity_time = Instant::now();
        }

        // Phase 1: Log metrics every 10 seconds
        let elapsed_since_metrics = metrics_start_time.elapsed().as_secs();
        if elapsed_since_metrics >= 10 {
            let messages_per_sec = total_messages_processed as f64 / elapsed_since_metrics as f64;
            println!(
                "[Phase1] Metrics: {:.2} messages/sec over {} seconds ({} total messages)",
                messages_per_sec, elapsed_since_metrics, total_messages_processed
            );
            // Reset metrics
            total_messages_processed = 0;
            metrics_start_time = Instant::now();
        }

        // Phase 2: Post test message every 10 seconds
        let elapsed_since_test = last_test_message_time.elapsed().as_secs();
        if elapsed_since_test >= 10 {
            unsafe {
                let result = PostMessageW(
                    message_window.hwnd(),
                    message_window::WM_USER_TEST,
                    42,   // Test wparam value
                    1337, // Test lparam value
                );
                if result != 0 {
                    println!(
                        "[Phase2] Test message posted to message window (wparam: 42, lparam: 1337)"
                    );
                } else {
                    eprintln!("[Phase2] Failed to post test message");
                }
            }
            last_test_message_time = Instant::now();
        }

        // Check if exit was requested via tray icon
        if exit_flag.load(Ordering::SeqCst) {
            println!("[Main] Exit requested, shutting down...");
            break;
        }

        // Check for pause state changes
        let current_paused = pause_flag.load(Ordering::SeqCst);
        if current_paused != last_paused {
            last_paused = current_paused;
            let manager = overlay_manager.lock().unwrap();
            if current_paused {
                println!("[Main] Application PAUSED - hiding all overlays");
                manager.hide_all();
            } else {
                println!("[Main] Application UNPAUSED - showing overlays");
                manager.show_all();
                // Force update visibility based on current active display
                if let Ok(active_window) = window_manager.get_active_window() {
                    manager.update_visibility(&active_window.display_id);
                }
            }
            // Update tray icon and tooltip to reflect pause state
            if let Err(e) = tray_icon.update_icon(current_paused) {
                eprintln!("[Main] Failed to update tray icon: {}", e);
            }
        }

        // Skip focus monitoring if paused
        if current_paused {
            thread::sleep(Duration::from_millis(400)); // Longer sleep when paused
            continue;
        }

        // Phase 1: Adaptive sleep based on recent activity
        // If we had activity in the last 2 seconds, use shorter sleep for better responsiveness
        // Otherwise, use longer sleep to reduce CPU usage when idle
        let time_since_activity = last_activity_time.elapsed().as_millis();
        let sleep_duration = if time_since_activity < 2000 {
            Duration::from_millis(50) // Active: 50ms for better responsiveness
        } else {
            Duration::from_millis(200) // Idle: 200ms for lower CPU usage
        };
        thread::sleep(sleep_duration);

        // Check for config file changes using file watching (event-driven detection)
        let config_changed = if let Some(handle) = config_watch_handle {
            unsafe {
                // Check if file change notification is signaled (non-blocking check with 0 timeout)
                let wait_result = WaitForSingleObject(handle, 0);
                if wait_result == WAIT_OBJECT_0 {
                    // File change detected! Re-arm the notification for next change
                    if FindNextChangeNotification(handle) == 0 {
                        eprintln!("[FileWatch] Failed to re-arm file change notification");
                    }
                    true
                } else {
                    false
                }
            }
        } else {
            false
        };

        if config_changed {
            println!("[FileWatch] Config file change detected");
            if let Some((new_config, new_modified_time)) =
                Config::reload_if_changed(last_config_modified)
            {
                let mut cfg = config.lock().unwrap();
                let old_dimming_enabled = cfg.is_dimming_enabled;
                let old_active_overlay_enabled = cfg.is_active_overlay_enabled;
                let old_inactive_color = cfg.overlay_color.clone();
                let old_active_color = cfg.active_overlay_color.clone();
                let old_partial_dimming_enabled = cfg.is_partial_dimming_enabled;

                // Update pause flag if it changed in config
                if cfg.is_paused != new_config.is_paused {
                    pause_flag.store(new_config.is_paused, Ordering::SeqCst);
                    println!(
                        "[Main] Pause state updated from config: {}",
                        if new_config.is_paused {
                            "PAUSED"
                        } else {
                            "UNPAUSED"
                        }
                    );
                }

                // Update config
                *cfg = new_config.clone();
                drop(cfg); // Release lock before potentially recreating overlays

                last_config_modified = Some(new_modified_time);

                // Determine what changed
                let dimming_enabled_changed = old_dimming_enabled != new_config.is_dimming_enabled;
                let inactive_color_changed = old_inactive_color.r != new_config.overlay_color.r
                    || old_inactive_color.g != new_config.overlay_color.g
                    || old_inactive_color.b != new_config.overlay_color.b
                    || old_inactive_color.a != new_config.overlay_color.a;
                let active_overlay_enabled_changed =
                    old_active_overlay_enabled != new_config.is_active_overlay_enabled;
                let active_color_changed = old_active_color != new_config.active_overlay_color;
                let partial_dimming_changed =
                    old_partial_dimming_enabled != new_config.is_partial_dimming_enabled;

                // Handle inactive overlay changes
                if dimming_enabled_changed || inactive_color_changed {
                    if new_config.is_dimming_enabled {
                        if dimming_enabled_changed && !old_dimming_enabled {
                            println!("[Main] Inactive dimming enabled via config change");
                        } else if inactive_color_changed {
                            println!("[Main] Inactive overlay color changed via config");
                        }
                        if let Ok(current_displays) = display_manager.get_displays() {
                            let mut manager = overlay_manager.lock().unwrap();
                            // Update color and recreate overlays
                            if let Err(e) = manager.set_inactive_color(
                                new_config.overlay_color.clone(),
                                &current_displays,
                            ) {
                                eprintln!("[Main] Failed to update inactive overlays: {}", e);
                            } else {
                                // Update visibility based on current active display
                                // If we don't have a cached display_id, get the current active window
                                if let Some(ref display_id) = last_display_id {
                                    manager.update_visibility(display_id);
                                } else if let Ok(active_window) = window_manager.get_active_window()
                                {
                                    manager.update_visibility(&active_window.display_id);
                                    last_display_id = Some(active_window.display_id.clone());
                                }
                            }
                        }
                    } else {
                        if dimming_enabled_changed {
                            println!("[Main] Inactive dimming disabled via config change");
                            let mut manager = overlay_manager.lock().unwrap();
                            manager.close_inactive();
                        }
                        // Update color in manager even when disabled, so it's ready when enabled
                        if inactive_color_changed {
                            let mut manager = overlay_manager.lock().unwrap();
                            manager.update_inactive_color_only(new_config.overlay_color.clone());
                        }
                    }
                }

                // Handle active overlay changes
                if active_overlay_enabled_changed || active_color_changed {
                    if new_config.is_active_overlay_enabled {
                        if active_overlay_enabled_changed && !old_active_overlay_enabled {
                            println!("[Main] Active overlay enabled via config change");
                        } else if active_color_changed {
                            println!("[Main] Active overlay color changed via config");
                        }
                        if let Ok(current_displays) = display_manager.get_displays() {
                            let mut manager = overlay_manager.lock().unwrap();
                            // Update color and recreate overlays
                            if let Err(e) = manager.set_active_color(
                                new_config.active_overlay_color.clone(),
                                &current_displays,
                            ) {
                                eprintln!("[Main] Failed to update active overlays: {}", e);
                            } else {
                                // Update visibility based on current active display
                                // If we don't have a cached display_id, get the current active window
                                if let Some(ref display_id) = last_display_id {
                                    manager.update_visibility(display_id);
                                } else if let Ok(active_window) = window_manager.get_active_window()
                                {
                                    manager.update_visibility(&active_window.display_id);
                                    last_display_id = Some(active_window.display_id.clone());
                                }
                            }
                        }
                    } else {
                        if active_overlay_enabled_changed {
                            println!("[Main] Active overlay disabled via config change");
                            let mut manager = overlay_manager.lock().unwrap();
                            manager.close_active();
                        }
                        // Update color in manager even when disabled, so it's ready when enabled
                        if active_color_changed {
                            let mut manager = overlay_manager.lock().unwrap();
                            manager
                                .update_active_color_only(new_config.active_overlay_color.clone());
                        }
                    }
                }

                // Handle partial dimming state change
                if partial_dimming_changed && !new_config.is_partial_dimming_enabled {
                    println!("[Main] Partial dimming disabled via config change");
                    let mut manager = overlay_manager.lock().unwrap();
                    manager.clear_all_partial_overlays();
                    last_window_rect = None;
                }
            }
        }

        // Check if any overlay type is enabled
        let (is_dimming_enabled, is_active_overlay_enabled, is_partial_dimming_enabled) = {
            let cfg = config.lock().unwrap();
            (
                cfg.is_dimming_enabled,
                cfg.is_active_overlay_enabled,
                cfg.is_partial_dimming_enabled,
            )
        };

        if !is_dimming_enabled && !is_active_overlay_enabled && !is_partial_dimming_enabled {
            thread::sleep(Duration::from_millis(400)); // Longer sleep when all disabled
            continue;
        }

        // Phase 3: Event-driven display configuration change detection
        // Two-stage stabilization: 500ms (fast) + 5000ms (safety net)
        if let Some(is_final_check) = message_window::check_display_change_ready() {
            let check_type = if is_final_check { "FINAL" } else { "FIRST" };
            println!(
                "[Phase3] Display stabilization {} check triggered",
                check_type
            );

            // Get new display list (now that Windows has finished reconfiguring)
            if let Ok(new_displays) = display_manager.get_displays() {
                let new_display_count = new_displays.len();

                // Check if display count actually changed
                let count_changed = new_display_count != last_display_count;

                if count_changed || is_final_check {
                    // Process if count changed OR this is the final check (force refresh)
                    println!(
                        "[Main] Display configuration changed: {} -> {} displays ({})",
                        last_display_count, new_display_count, check_type
                    );

                    let mut manager = overlay_manager.lock().unwrap();

                    if is_dimming_enabled {
                        if let Err(e) = manager.recreate_inactive_overlays(&new_displays) {
                            eprintln!("[Main] Failed to recreate inactive overlays: {}", e);
                        } else {
                            println!(
                                "[Main] Recreated {} inactive overlay(s)",
                                manager.inactive_count()
                            );
                        }
                    }

                    if is_active_overlay_enabled {
                        if let Err(e) = manager.recreate_active_overlays(&new_displays) {
                            eprintln!("[Main] Failed to recreate active overlays: {}", e);
                        } else {
                            println!(
                                "[Main] Recreated {} active overlay(s)",
                                manager.active_count()
                            );
                        }
                    }

                    last_display_count = new_display_count;
                    last_window_handle = None;
                    last_display_id = None;
                    last_window_rect = None;
                    continue;
                } else {
                    println!(
                        "[Phase3] Display count unchanged ({} displays) - waiting for final check",
                        new_display_count
                    );
                }
            }
        }

        // Phase 4: Check if foreground window changed (event-driven via EVENT_SYSTEM_FOREGROUND hook)
        let foreground_changed = message_window::check_and_reset_foreground_changed();
        if foreground_changed {
            println!("[Phase4] Foreground window changed (event-driven detection)");
        }

        // Always get active window (needed for rect polling even if foreground didn't change)
        match window_manager.get_active_window() {
            Ok(active_window) => {
                // Skip our own overlay windows
                if active_window
                    .window_title
                    .contains("Spotlight Dimmer Overlay")
                {
                    continue;
                }

                // Phase 4: Only process window/display changes when foreground actually changed
                // This replaces the polling-based change detection
                if foreground_changed {
                    let window_changed = Some(active_window.handle) != last_window_handle;
                    let display_changed =
                        last_display_id.as_ref() != Some(&active_window.display_id);

                    if window_changed || display_changed {
                        // Phase 1: Track activity for adaptive sleep
                        last_activity_time = Instant::now();

                        if window_changed {
                            println!(
                                "[Main] Active window: {} ({})",
                                active_window.window_title, active_window.process_name
                            );
                        }

                        if display_changed {
                            println!(
                                "[Main] Window moved to display: {}",
                                active_window.display_id
                            );
                        }

                        // Update overlays for any window or display change
                        let mut manager = overlay_manager.lock().unwrap();
                        manager.update_visibility(&active_window.display_id);

                        // Clear partial overlays when switching displays or windows
                        if display_changed {
                            manager.clear_all_partial_overlays();
                            last_window_rect = None;
                        }

                        last_window_handle = Some(active_window.handle);
                        last_display_id = Some(active_window.display_id.clone());
                    }
                }

                // Handle partial dimming (check for window movement/resize)
                if is_partial_dimming_enabled && last_display_id.is_some() {
                    if let Ok(current_rect) = window_manager.get_window_rect(active_window.handle) {
                        let rect_changed = match last_window_rect {
                            None => true,
                            Some(last_rect) => {
                                last_rect.left != current_rect.left
                                    || last_rect.top != current_rect.top
                                    || last_rect.right != current_rect.right
                                    || last_rect.bottom != current_rect.bottom
                            }
                        };

                        if rect_changed {
                            // Update overlays immediately when rect changes
                            if let Ok(display_info) =
                                window_manager.get_window_display(active_window.handle)
                            {
                                let mut manager = overlay_manager.lock().unwrap();

                                // Check if window is maximized (needed for atomic update)
                                let is_maximized = window_manager
                                    .is_window_maximized(active_window.handle)
                                    .unwrap_or(false);

                                // Try atomic update of both partial and active overlays (more efficient and no visual fragmentation)
                                let updated = if is_active_overlay_enabled {
                                    match manager.update_partial_and_active_overlays_atomic(
                                        &active_window.display_id,
                                        current_rect,
                                        &display_info,
                                        true, // Update active overlay
                                        is_maximized,
                                    ) {
                                        Ok(true) => true,   // Successfully updated all overlays atomically
                                        Ok(false) => false, // Need to recreate (topology changed)
                                        Err(e) => {
                                            eprintln!(
                                                "[Main] Failed to update overlays atomically: {}",
                                                e
                                            );
                                            false // Need to recreate on error
                                        }
                                    }
                                } else {
                                    // No active overlay, just update partial overlays
                                    match manager.update_partial_overlays(
                                        &active_window.display_id,
                                        current_rect,
                                        &display_info,
                                    ) {
                                        Ok(true) => true,
                                        Ok(false) => false,
                                        Err(e) => {
                                            eprintln!(
                                                "[Main] Failed to update partial overlays: {}",
                                                e
                                            );
                                            false
                                        }
                                    }
                                };

                                // If atomic update failed or overlays don't exist, recreate them
                                if !updated {
                                    if let Err(e) = manager.create_partial_overlays(
                                        &active_window.display_id,
                                        current_rect,
                                        &display_info,
                                    ) {
                                        eprintln!(
                                            "[Main] Failed to create partial overlays: {}",
                                            e
                                        );
                                    }

                                    // Resize active overlay separately if needed
                                    if is_active_overlay_enabled {
                                        if !is_maximized {
                                            if let Err(e) = manager.resize_active_overlay(
                                                &active_window.display_id,
                                                current_rect,
                                            ) {
                                                eprintln!(
                                                    "[Main] Failed to resize active overlay: {}",
                                                    e
                                                );
                                            }
                                        } else {
                                            // Window is maximized, restore active overlay to full display size
                                            if let Err(e) = manager
                                                .restore_active_overlay_full_size(
                                                    &active_window.display_id,
                                                    &display_info,
                                                )
                                            {
                                                eprintln!("[Main] Failed to restore active overlay to full size: {}", e);
                                            }
                                        }
                                    }
                                }
                            }

                            last_window_rect = Some(current_rect);
                        }
                    }
                } else if !is_partial_dimming_enabled && last_window_rect.is_some() {
                    // Clear partial overlays if feature was disabled
                    let mut manager = overlay_manager.lock().unwrap();
                    manager.clear_all_partial_overlays();
                    last_window_rect = None;

                    // Restore active overlay to full size when partial dimming is disabled
                    if is_active_overlay_enabled {
                        if let Ok(display_info) =
                            window_manager.get_window_display(active_window.handle)
                        {
                            if let Err(e) = manager.restore_active_overlay_full_size(
                                &active_window.display_id,
                                &display_info,
                            ) {
                                eprintln!(
                                    "[Main] Failed to restore active overlay to full size: {}",
                                    e
                                );
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Silently ignore errors to avoid spam
                if last_window_handle.is_some() {
                    last_window_handle = None;
                    last_display_id = None;
                    last_window_rect = None;
                    // Clear partial overlays when no active window
                    if is_partial_dimming_enabled {
                        let mut manager = overlay_manager.lock().unwrap();
                        manager.clear_all_partial_overlays();
                    }
                }
            }
        }
    }

    // Cleanup on exit
    println!("[Main] Performing cleanup...");

    // Close file watching handle
    if let Some(handle) = config_watch_handle {
        unsafe {
            CloseHandle(handle);
        }
        println!("[FileWatch] File watching handle closed");
    }

    drop(overlay_manager);
    drop(tray_icon);
    drop(message_window);
    println!("[Main] Spotlight Dimmer exited successfully");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Error: Spotlight Dimmer is only supported on Windows.");
    eprintln!("This application requires Windows API to function.");
    std::process::exit(1);
}
