use crate::config::OverlayColor;
use crate::platform::DisplayInfo;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HWND, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{CreateSolidBrush, RGB};
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetWindowLongPtrW, PostQuitMessage, RegisterClassExW, SetLayeredWindowAttributes,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, GWL_EXSTYLE, LWA_ALPHA, MSG, SW_HIDE,
    SW_SHOW, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};

const CLASS_NAME: &str = "SpotlightDimmerOverlay";

/// Manager for overlay windows
pub struct OverlayManager {
    overlays: HashMap<String, HWND>,
    overlay_color: OverlayColor,
}

impl OverlayManager {
    pub fn new(overlay_color: OverlayColor) -> Result<Self, String> {
        // Register window class
        unsafe {
            let class_name = to_wstring(CLASS_NAME);
            let hinstance = GetModuleHandleW(ptr::null());

            let wnd_class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: ptr::null_mut(),
                hCursor: ptr::null_mut(),
                hbrBackground: CreateSolidBrush(RGB(0, 0, 0)) as HBRUSH,
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: ptr::null_mut(),
            };

            if RegisterClassExW(&wnd_class) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                // Class already registered is OK
                if err != 1410 {
                    // ERROR_CLASS_ALREADY_EXISTS
                    return Err(format!("Failed to register window class: error {}", err));
                }
            }
        }

        Ok(Self {
            overlays: HashMap::new(),
            overlay_color,
        })
    }

    /// Create overlays for all displays
    pub fn create_overlays(&mut self, displays: &[DisplayInfo]) -> Result<(), String> {
        for display in displays {
            self.create_overlay(display)?;
        }
        Ok(())
    }

    /// Create a single overlay window for a display
    fn create_overlay(&mut self, display: &DisplayInfo) -> Result<(), String> {
        unsafe {
            let class_name = to_wstring(CLASS_NAME);
            let window_name = to_wstring(&format!("Overlay {}", display.id));
            let hinstance = GetModuleHandleW(ptr::null());

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_POPUP,
                display.x,
                display.y,
                display.width,
                display.height,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance,
                ptr::null_mut(),
            );

            if hwnd.is_null() {
                let err = winapi::um::errhandlingapi::GetLastError();
                return Err(format!("Failed to create overlay window: error {}", err));
            }

            // Set the transparency
            let alpha = self.overlay_color.to_alpha_byte();
            let colorref = self.overlay_color.to_colorref();

            if SetLayeredWindowAttributes(hwnd, colorref, alpha, LWA_ALPHA) == 0 {
                DestroyWindow(hwnd);
                return Err("Failed to set window transparency".to_string());
            }

            // Show the window initially (will be hidden for active display later)
            ShowWindow(hwnd, SW_SHOW);

            println!(
                "[Overlay] Created for display {} at ({}, {}) size {}x{} with alpha {}",
                display.id, display.x, display.y, display.width, display.height, alpha
            );

            self.overlays.insert(display.id.clone(), hwnd);
        }

        Ok(())
    }

    /// Update overlay visibility based on active display
    pub fn update_visibility(&self, active_display_id: &str) {
        for (display_id, &hwnd) in &self.overlays {
            let should_show = display_id != active_display_id;
            unsafe {
                ShowWindow(hwnd, if should_show { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    /// Close all overlays
    pub fn close_all(&mut self) {
        unsafe {
            for (_, &hwnd) in &self.overlays {
                DestroyWindow(hwnd);
            }
        }
        self.overlays.clear();
        println!("[Overlay] Closed all overlays");
    }

    /// Recreate overlays with new displays
    pub fn recreate_overlays(&mut self, displays: &[DisplayInfo]) -> Result<(), String> {
        self.close_all();
        self.create_overlays(displays)
    }

    /// Update overlay color
    pub fn set_color(&mut self, color: OverlayColor, displays: &[DisplayInfo]) -> Result<(), String> {
        self.overlay_color = color;
        // Recreate overlays with new color
        self.recreate_overlays(displays)
    }

    /// Get current overlay count
    pub fn count(&self) -> usize {
        self.overlays.len()
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        self.close_all();
    }
}

/// Window procedure for overlay windows
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        winapi::um::winuser::WM_PAINT => {
            // No painting needed - transparency handled by layered window
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Convert Rust string to null-terminated wide string
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}