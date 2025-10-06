// This entire module is Windows-only
#![cfg(windows)]

use crate::config::Config;
use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HWND, POINT};
use winapi::um::libloaderapi::{GetModuleFileNameW, GetModuleHandleW};
use winapi::um::shellapi::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use winapi::um::winuser::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    GetCursorPos, LoadImageW, PostQuitMessage, RegisterClassExW, SetForegroundWindow,
    TrackPopupMenu, CS_DBLCLKS, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, MF_STRING,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_COMMAND, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP,
    WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

const WM_TRAYICON: u32 = winapi::um::winuser::WM_USER + 1;
const CMD_EXIT: u16 = 1001;
const CMD_PROFILE_BASE: u16 = 2000; // Base ID for profile menu items

/// Struct to hold shared state pointers for window procedure
struct TrayState {
    exit_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
}

/// Get the directory where the executable is located
fn get_exe_directory() -> Result<PathBuf, String> {
    unsafe {
        let mut buffer: Vec<u16> = vec![0; 512];
        let len = GetModuleFileNameW(ptr::null_mut(), buffer.as_mut_ptr(), buffer.len() as u32);

        if len == 0 {
            return Err("Failed to get executable path".to_string());
        }

        buffer.truncate(len as usize);
        let exe_path = PathBuf::from(String::from_utf16_lossy(&buffer));

        exe_path
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "Failed to get executable directory".to_string())
    }
}

/// System tray icon manager
pub struct TrayIcon {
    hwnd: HWND,
    hicon_active: winapi::shared::windef::HICON,
    hicon_paused: winapi::shared::windef::HICON,
    exit_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
}

impl TrayIcon {
    /// Create a new system tray icon
    pub fn new(
        icon_path: &str,
        tooltip: &str,
        exit_flag: Arc<AtomicBool>,
        pause_flag: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        unsafe {
            // Register window class for hidden message window
            let class_name = to_wstring("SpotlightDimmerTrayWindow");
            let hinstance = GetModuleHandleW(ptr::null());

            let wnd_class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_DBLCLKS, // Enable double-click messages
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: ptr::null_mut(),
                hCursor: ptr::null_mut(),
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: ptr::null_mut(),
            };

            if RegisterClassExW(&wnd_class) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                // Class already registered is OK
                if err != 1410 {
                    return Err(format!(
                        "Failed to register tray window class: error {}",
                        err
                    ));
                }
            }

            // Create hidden message window
            let window_name = to_wstring("Spotlight Dimmer Tray");
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance,
                ptr::null_mut(),
            );

            if hwnd.is_null() {
                let err = winapi::um::errhandlingapi::GetLastError();
                return Err(format!("Failed to create tray window: error {}", err));
            }

            // Store tray state (both flags) in window user data for access in window proc
            let state = Box::new(TrayState {
                exit_flag: exit_flag.clone(),
                pause_flag: pause_flag.clone(),
            });
            winapi::um::winuser::SetWindowLongPtrW(
                hwnd,
                winapi::um::winuser::GWLP_USERDATA,
                Box::into_raw(state) as isize,
            );

            // Get absolute path to icon files (relative to executable)
            let exe_dir = get_exe_directory().map_err(|e| {
                DestroyWindow(hwnd);
                e
            })?;
            let icon_full_path = exe_dir.join(icon_path);
            let icon_paused_full_path = exe_dir.join("spotlight-dimmer-icon-paused.ico");

            if !icon_full_path.exists() {
                DestroyWindow(hwnd);
                return Err(format!(
                    "Icon file not found: {} (looked in: {})",
                    icon_path,
                    icon_full_path.display()
                ));
            }

            if !icon_paused_full_path.exists() {
                DestroyWindow(hwnd);
                return Err(format!(
                    "Paused icon file not found: spotlight-dimmer-icon-paused.ico (looked in: {})",
                    icon_paused_full_path.display()
                ));
            }

            // Load active icon from file using absolute path
            let icon_path_wide = to_wstring(&icon_full_path.to_string_lossy());
            let hicon_active = LoadImageW(
                ptr::null_mut(),
                icon_path_wide.as_ptr(),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            ) as winapi::shared::windef::HICON;

            if hicon_active.is_null() {
                DestroyWindow(hwnd);
                return Err(format!(
                    "Failed to load active icon from: {}",
                    icon_full_path.display()
                ));
            }

            // Load paused icon from file using absolute path
            let icon_paused_path_wide = to_wstring(&icon_paused_full_path.to_string_lossy());
            let hicon_paused = LoadImageW(
                ptr::null_mut(),
                icon_paused_path_wide.as_ptr(),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            ) as winapi::shared::windef::HICON;

            if hicon_paused.is_null() {
                winapi::um::winuser::DestroyIcon(hicon_active);
                DestroyWindow(hwnd);
                return Err(format!(
                    "Failed to load paused icon from: {}",
                    icon_paused_full_path.display()
                ));
            }

            // Create tray icon with active icon initially
            let tooltip_wide = to_wstring(tooltip);
            let mut nid: NOTIFYICONDATAW = mem::zeroed();
            nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = hicon_active;

            // Copy tooltip (max 128 chars)
            let tooltip_len = tooltip_wide.len().min(127);
            ptr::copy_nonoverlapping(tooltip_wide.as_ptr(), nid.szTip.as_mut_ptr(), tooltip_len);

            if Shell_NotifyIconW(NIM_ADD, &mut nid) == 0 {
                winapi::um::winuser::DestroyIcon(hicon_active);
                winapi::um::winuser::DestroyIcon(hicon_paused);
                DestroyWindow(hwnd);
                return Err("Failed to add tray icon".to_string());
            }

            println!("[Tray] System tray icon created successfully");

            Ok(Self {
                hwnd,
                hicon_active,
                hicon_paused,
                exit_flag,
                pause_flag,
            })
        }
    }

    /// Get the window handle for message processing
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Update the tray icon tooltip
    pub fn update_tooltip(&self, tooltip: &str) -> Result<(), String> {
        unsafe {
            let tooltip_wide = to_wstring(tooltip);
            let mut nid: NOTIFYICONDATAW = mem::zeroed();
            nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_TIP;

            // Copy tooltip (max 128 chars)
            let tooltip_len = tooltip_wide.len().min(127);
            ptr::copy_nonoverlapping(tooltip_wide.as_ptr(), nid.szTip.as_mut_ptr(), tooltip_len);

            if Shell_NotifyIconW(winapi::um::shellapi::NIM_MODIFY, &mut nid) == 0 {
                return Err("Failed to update tray icon tooltip".to_string());
            }

            Ok(())
        }
    }

    /// Update the tray icon and tooltip based on pause state
    pub fn update_icon(&self, is_paused: bool) -> Result<(), String> {
        unsafe {
            let tooltip = if is_paused {
                "Spotlight Dimmer (PAUSED)"
            } else {
                "Spotlight Dimmer"
            };
            let hicon = if is_paused {
                self.hicon_paused
            } else {
                self.hicon_active
            };

            let tooltip_wide = to_wstring(tooltip);
            let mut nid: NOTIFYICONDATAW = mem::zeroed();
            nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_ICON | NIF_TIP;
            nid.hIcon = hicon;

            // Copy tooltip (max 128 chars)
            let tooltip_len = tooltip_wide.len().min(127);
            ptr::copy_nonoverlapping(tooltip_wide.as_ptr(), nid.szTip.as_mut_ptr(), tooltip_len);

            if Shell_NotifyIconW(winapi::um::shellapi::NIM_MODIFY, &mut nid) == 0 {
                return Err("Failed to update tray icon".to_string());
            }

            Ok(())
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            // Remove tray icon
            let mut nid: NOTIFYICONDATAW = mem::zeroed();
            nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = 1;
            Shell_NotifyIconW(NIM_DELETE, &mut nid);

            // Cleanup resources
            winapi::um::winuser::DestroyIcon(self.hicon_active);
            winapi::um::winuser::DestroyIcon(self.hicon_paused);
            DestroyWindow(self.hwnd);

            // Cleanup tray state pointer stored in window user data
            let ptr = winapi::um::winuser::GetWindowLongPtrW(
                self.hwnd,
                winapi::um::winuser::GWLP_USERDATA,
            ) as *mut TrayState;
            if !ptr.is_null() {
                drop(Box::from_raw(ptr));
            }

            println!("[Tray] System tray icon removed");
        }
    }
}

/// Window procedure for tray icon message window
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            // Handle tray icon events
            let event = lparam as u32;
            match event {
                WM_RBUTTONUP => {
                    // Show context menu on right-click
                    show_context_menu(hwnd);
                }
                WM_LBUTTONUP => {
                    // Optional: handle left-click (currently does nothing)
                }
                WM_LBUTTONDBLCLK => {
                    // Toggle pause state on double-click
                    let ptr = winapi::um::winuser::GetWindowLongPtrW(
                        hwnd,
                        winapi::um::winuser::GWLP_USERDATA,
                    ) as *const TrayState;
                    if !ptr.is_null() {
                        let state = &*ptr;
                        // Toggle pause state
                        let was_paused = state.pause_flag.load(Ordering::SeqCst);
                        state.pause_flag.store(!was_paused, Ordering::SeqCst);

                        // Save to config
                        let mut config = Config::load();
                        config.is_paused = !was_paused;
                        if let Err(e) = config.save() {
                            eprintln!("[Tray] Failed to save pause state: {}", e);
                        } else {
                            println!(
                                "[Tray] Pause state toggled: {}",
                                if !was_paused { "PAUSED" } else { "UNPAUSED" }
                            );
                        }
                    }
                }
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            // Handle menu commands
            let cmd_id = (wparam & 0xFFFF) as u16;
            if cmd_id == CMD_EXIT {
                println!("[Tray] Exit command received");
                // Get tray state from window user data
                let ptr = winapi::um::winuser::GetWindowLongPtrW(
                    hwnd,
                    winapi::um::winuser::GWLP_USERDATA,
                ) as *const TrayState;
                if !ptr.is_null() {
                    let state = &*ptr;
                    state.exit_flag.store(true, Ordering::SeqCst);
                }
            } else if cmd_id >= CMD_PROFILE_BASE && cmd_id < CMD_PROFILE_BASE + 100 {
                // Profile menu item clicked
                let profile_index = (cmd_id - CMD_PROFILE_BASE) as usize;
                let mut config = Config::load();
                let profiles = config.list_profiles();

                if profile_index < profiles.len() {
                    let profile_name = &profiles[profile_index];
                    println!("[Tray] Loading profile: {}", profile_name);

                    match config.load_profile(profile_name) {
                        Ok(_) => {
                            if let Err(e) = config.save() {
                                eprintln!("[Tray] Failed to save profile: {}", e);
                            } else {
                                println!("[Tray] Profile '{}' loaded successfully", profile_name);
                            }
                        }
                        Err(e) => {
                            eprintln!("[Tray] Failed to load profile: {}", e);
                        }
                    }
                }
            }
            0
        }
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Show context menu at cursor position
unsafe fn show_context_menu(hwnd: HWND) {
    let hmenu = CreatePopupMenu();
    if hmenu.is_null() {
        return;
    }

    // Load config to get profiles
    let config = Config::load();
    let profiles = config.list_profiles();

    // Add profile menu items if any exist
    if !profiles.is_empty() {
        for (index, profile_name) in profiles.iter().enumerate() {
            let menu_text = to_wstring(&format!("Profile: {}", profile_name));
            AppendMenuW(
                hmenu,
                MF_STRING,
                (CMD_PROFILE_BASE + index as u16) as usize,
                menu_text.as_ptr(),
            );
        }

        // Add separator
        AppendMenuW(hmenu, winapi::um::winuser::MF_SEPARATOR, 0, ptr::null());
    }

    // Add "Exit" menu item
    let exit_text = to_wstring("Exit");
    AppendMenuW(hmenu, MF_STRING, CMD_EXIT as usize, exit_text.as_ptr());

    // Get cursor position
    let mut pt: POINT = mem::zeroed();
    GetCursorPos(&mut pt);

    // Required for menu to close when clicking outside
    SetForegroundWindow(hwnd);

    // Show menu
    TrackPopupMenu(
        hmenu,
        TPM_BOTTOMALIGN | TPM_LEFTALIGN,
        pt.x,
        pt.y,
        0,
        hwnd,
        ptr::null(),
    );

    // Cleanup
    DestroyMenu(hmenu);
}

/// Convert Rust string to null-terminated wide string
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
