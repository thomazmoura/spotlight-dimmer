mod platform;

use platform::{ActiveWindowInfo, DisplayInfo, DisplayManager, WindowManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State, LogicalPosition, WebviewUrl, WebviewWindowBuilder, Emitter};

// Global state for managing overlays and focus tracking
#[derive(Default)]
struct AppState {
    overlays: Arc<Mutex<HashMap<String, tauri::WebviewWindow>>>,
    is_dimming_enabled: Arc<Mutex<bool>>,
    current_active_display: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FocusChangeEvent {
    active_window: ActiveWindowInfo,
    active_display: DisplayInfo,
}

// Tauri commands
#[tauri::command]
async fn get_displays() -> Result<Vec<DisplayInfo>, String> {
    #[cfg(windows)]
    {
        let display_manager = platform::WindowsDisplayManager;
        display_manager.get_displays()
    }
    #[cfg(not(windows))]
    {
        Err("Platform not supported yet".to_string())
    }
}

#[tauri::command]
async fn get_active_window() -> Result<ActiveWindowInfo, String> {
    #[cfg(windows)]
    {
        let window_manager = platform::WindowsWindowManager;
        window_manager.get_active_window()
    }
    #[cfg(not(windows))]
    {
        Err("Platform not supported yet".to_string())
    }
}

#[tauri::command]
async fn toggle_dimming(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    // Fix: Use scope to drop mutex guard before await
    let is_enabled = {
        let mut enabled = state.is_dimming_enabled.lock().unwrap();
        *enabled = !*enabled;
        *enabled
    };

    if is_enabled {
        create_overlays(&app_handle, &state).await?;
    } else {
        close_all_overlays(&state).await?;
    }

    Ok(is_enabled)
}

#[tauri::command]
async fn is_dimming_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let is_enabled = state.is_dimming_enabled.lock().unwrap();
    Ok(*is_enabled)
}

// Helper functions
async fn create_overlays(app_handle: &AppHandle, state: &AppState) -> Result<(), String> {
    let displays = get_displays().await?;
    let mut overlays = state.overlays.lock().unwrap();

    // Close existing overlays first
    for (_, window) in overlays.drain() {
        let _ = window.close();
    }

    // Create new overlays for each display
    for display in displays {
        let overlay_id = format!("overlay_{}", display.id);

        match create_overlay_window(app_handle, &overlay_id, &display) {
            Ok(window) => {
                overlays.insert(display.id.clone(), window);
            }
            Err(e) => {
                log::warn!("Failed to create overlay for display {}: {}", display.id, e);
            }
        }
    }

    Ok(())
}

fn create_overlay_window(
    app_handle: &AppHandle,
    overlay_id: &str,
    display: &DisplayInfo,
) -> Result<tauri::WebviewWindow, String> {
    let window = WebviewWindowBuilder::new(app_handle, overlay_id, WebviewUrl::App("overlay.html".into()))
        .title("Spotlight Dimmer Overlay")
        .inner_size(display.width as f64, display.height as f64)
        .position(display.x as f64, display.y as f64)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .closable(false)
        .focusable(false)
        .build()
        .map_err(|e| format!("Failed to create overlay window: {}", e))?;

    // Make window click-through on Windows
    #[cfg(windows)]
    {
        println!("Making overlay window click-through...");
        if let Err(e) = make_window_click_through(&window) {
            println!("Failed to make window click-through: {}", e);
        } else {
            println!("Successfully made overlay window click-through");
        }
    }

    Ok(window)
}

#[cfg(windows)]
fn make_window_click_through(window: &tauri::WebviewWindow) -> Result<(), String> {
    use winapi::um::winuser::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_TOOLWINDOW, WS_EX_NOACTIVATE};

    let hwnd = window.hwnd().map_err(|e| e.to_string())?;

    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd.0 as _, GWL_EXSTYLE);
        SetWindowLongPtrW(
            hwnd.0 as _,
            GWL_EXSTYLE,
            ex_style | WS_EX_TRANSPARENT as isize | WS_EX_LAYERED as isize | WS_EX_TOOLWINDOW as isize | WS_EX_NOACTIVATE as isize,
        );
    }

    Ok(())
}

async fn close_all_overlays(state: &AppState) -> Result<(), String> {
    let mut overlays = state.overlays.lock().unwrap();
    for (_, window) in overlays.drain() {
        let _ = window.close();
    }
    Ok(())
}

async fn update_overlays(app_handle: &AppHandle, state: &AppState) -> Result<(), String> {
    let is_enabled = *state.is_dimming_enabled.lock().unwrap();
    if !is_enabled {
        return Ok(());
    }

    let active_window = get_active_window().await?;

    // Only update if the active display changed
    let should_update = {
        let mut current_display = state.current_active_display.lock().unwrap();
        if current_display.as_ref() != Some(&active_window.display_id) {
            *current_display = Some(active_window.display_id.clone());
            true
        } else {
            false
        }
    };

    if should_update {
        // Update overlay visibility
        {
            let overlays = state.overlays.lock().unwrap();
            for (display_id, window) in overlays.iter() {
                let should_hide = display_id == &active_window.display_id;
                if should_hide {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                }
            }
        }

        // Emit focus change event
        let displays = get_displays().await?;
        if let Some(active_display) = displays.iter().find(|d| d.id == active_window.display_id) {
            let focus_event = FocusChangeEvent {
                active_window,
                active_display: active_display.clone(),
            };

            if let Err(e) = app_handle.emit("focus-changed", &focus_event) {
                log::warn!("Failed to emit focus-changed event: {}", e);
            }
        }
    }

    Ok(())
}

// Synchronous version of update_overlays for background thread
fn update_overlays_sync(app_handle: &AppHandle, state: &AppState, active_window: &ActiveWindowInfo) -> Result<(), String> {
    let is_enabled = *state.is_dimming_enabled.lock().unwrap();
    if !is_enabled {
        return Ok(());
    }

    // Only update if the active display changed
    let should_update = {
        let mut current_display = state.current_active_display.lock().unwrap();
        if current_display.as_ref() != Some(&active_window.display_id) {
            *current_display = Some(active_window.display_id.clone());
            true
        } else {
            false
        }
    };

    if should_update {
        // Update overlay visibility
        {
            let overlays = state.overlays.lock().unwrap();
            for (display_id, window) in overlays.iter() {
                let should_hide = display_id == &active_window.display_id;
                if should_hide {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                }
            }
        }

        // Emit focus change event (sync version)
        let displays_result = {
            #[cfg(windows)]
            {
                let display_manager = platform::WindowsDisplayManager;
                display_manager.get_displays()
            }
            #[cfg(not(windows))]
            {
                Err("Platform not supported yet".to_string())
            }
        };

        if let Ok(displays) = displays_result {
            if let Some(active_display) = displays.iter().find(|d| d.id == active_window.display_id) {
                let focus_event = FocusChangeEvent {
                    active_window: active_window.clone(),
                    active_display: active_display.clone(),
                };

                if let Err(e) = app_handle.emit("focus-changed", &focus_event) {
                    println!("Failed to emit focus-changed event: {}", e);
                } else {
                    println!("Successfully emitted focus-changed event for: {}", active_window.window_title);
                }
            }
        }
    }

    Ok(())
}

// Focus monitoring setup
fn start_focus_monitoring(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let mut last_window_handle: Option<u64> = None;
        println!("Focus monitoring thread started!");

        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Create a sync version of get_active_window
            let active_window_result = {
                #[cfg(windows)]
                {
                    let window_manager = platform::WindowsWindowManager;
                    window_manager.get_active_window()
                }
                #[cfg(not(windows))]
                {
                    Err("Platform not supported yet".to_string())
                }
            };

            match active_window_result {
                Ok(active_window) => {
                    // Skip our own overlay windows to prevent focus stealing loops
                    if active_window.window_title.contains("Spotlight Dimmer Overlay") {
                        continue;
                    }

                    if Some(active_window.handle) != last_window_handle {
                        last_window_handle = Some(active_window.handle);
                        println!("Active window changed: {} ({})", active_window.window_title, active_window.process_name);

                        let state = app_handle.state::<AppState>();

                        // Use a blocking approach for updates
                        if let Err(e) = update_overlays_sync(&app_handle, &state, &active_window) {
                            println!("Failed to update overlays: {}", e);
                        }
                    }
                },
                Err(e) => {
                    if last_window_handle.is_some() {
                        println!("Failed to get active window: {}", e);
                        last_window_handle = None;
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build())
        .setup(|app| {

            // Start focus monitoring
            start_focus_monitoring(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_displays,
            get_active_window,
            toggle_dimming,
            is_dimming_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
