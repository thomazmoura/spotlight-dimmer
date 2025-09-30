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

const INACTIVE_CLASS_NAME: &str = "SpotlightDimmerInactiveOverlay";
const ACTIVE_CLASS_NAME: &str = "SpotlightDimmerActiveOverlay";

/// Manager for overlay windows
pub struct OverlayManager {
    inactive_overlays: HashMap<String, HWND>,
    active_overlays: HashMap<String, HWND>,
    inactive_color: OverlayColor,
    active_color: Option<OverlayColor>,
}

impl OverlayManager {
    pub fn new(inactive_color: OverlayColor, active_color: Option<OverlayColor>) -> Result<Self, String> {
        // Register window classes for both inactive and active overlays
        unsafe {
            let hinstance = GetModuleHandleW(ptr::null());

            // Register inactive overlay window class
            let inactive_class_name = to_wstring(INACTIVE_CLASS_NAME);
            let inactive_wnd_class = WNDCLASSEXW {
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
                lpszClassName: inactive_class_name.as_ptr(),
                hIconSm: ptr::null_mut(),
            };

            if RegisterClassExW(&inactive_wnd_class) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                // Class already registered is OK
                if err != 1410 {
                    // ERROR_CLASS_ALREADY_EXISTS
                    return Err(format!("Failed to register inactive window class: error {}", err));
                }
            }

            // Register active overlay window class
            let active_class_name = to_wstring(ACTIVE_CLASS_NAME);
            let active_wnd_class = WNDCLASSEXW {
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
                lpszClassName: active_class_name.as_ptr(),
                hIconSm: ptr::null_mut(),
            };

            if RegisterClassExW(&active_wnd_class) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                // Class already registered is OK
                if err != 1410 {
                    // ERROR_CLASS_ALREADY_EXISTS
                    return Err(format!("Failed to register active window class: error {}", err));
                }
            }
        }

        Ok(Self {
            inactive_overlays: HashMap::new(),
            active_overlays: HashMap::new(),
            inactive_color,
            active_color,
        })
    }

    /// Create inactive overlays for all displays
    pub fn create_inactive_overlays(&mut self, displays: &[DisplayInfo]) -> Result<(), String> {
        for display in displays {
            self.create_overlay(display, true)?;
        }
        Ok(())
    }

    /// Create active overlays for all displays
    pub fn create_active_overlays(&mut self, displays: &[DisplayInfo]) -> Result<(), String> {
        if self.active_color.is_none() {
            return Ok(()); // No active overlay color configured
        }
        for display in displays {
            self.create_overlay(display, false)?;
        }
        Ok(())
    }

    /// Create a single overlay window for a display
    /// is_inactive: true for inactive overlays, false for active overlays
    fn create_overlay(&mut self, display: &DisplayInfo, is_inactive: bool) -> Result<(), String> {
        let (color, overlay_type, class_name_str) = if is_inactive {
            (&self.inactive_color, "inactive", INACTIVE_CLASS_NAME)
        } else {
            match &self.active_color {
                Some(c) => (c, "active", ACTIVE_CLASS_NAME),
                None => return Ok(()), // Skip if no active color
            }
        };

        unsafe {
            let class_name = to_wstring(class_name_str);
            let window_name = to_wstring(&format!("{} Overlay {}",
                if is_inactive { "Inactive" } else { "Active" },
                display.id));
            let hinstance = GetModuleHandleW(ptr::null());

            // Create a brush with the overlay color
            let colorref = RGB(color.r, color.g, color.b);
            let brush = CreateSolidBrush(colorref);

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

            // Set window background color using WM_PAINT handling with our brush
            // We need to set the class background
            use winapi::um::winuser::{SetClassLongPtrW, GCLP_HBRBACKGROUND};
            SetClassLongPtrW(hwnd, GCLP_HBRBACKGROUND, brush as isize);

            // Set the transparency (alpha only)
            let alpha = color.to_alpha_byte();

            if SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA) == 0 {
                DestroyWindow(hwnd);
                return Err("Failed to set window transparency".to_string());
            }

            // Show the window initially (visibility will be managed later)
            ShowWindow(hwnd, SW_SHOW);

            println!(
                "[Overlay] Created {} overlay for display {} at ({}, {}) size {}x{} with color RGB({}, {}, {}) alpha {}",
                overlay_type, display.id, display.x, display.y, display.width, display.height,
                color.r, color.g, color.b, alpha
            );

            if is_inactive {
                self.inactive_overlays.insert(display.id.clone(), hwnd);
            } else {
                self.active_overlays.insert(display.id.clone(), hwnd);
            }
        }

        Ok(())
    }

    /// Update overlay visibility based on active display
    pub fn update_visibility(&self, active_display_id: &str) {
        // Manage inactive overlays - show on non-active displays only
        for (display_id, &hwnd) in &self.inactive_overlays {
            let should_show = display_id != active_display_id;
            unsafe {
                ShowWindow(hwnd, if should_show { SW_SHOW } else { SW_HIDE });
            }
        }

        // Manage active overlays - show on active display only
        for (display_id, &hwnd) in &self.active_overlays {
            let should_show = display_id == active_display_id;
            unsafe {
                ShowWindow(hwnd, if should_show { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    /// Close all overlays (both inactive and active)
    pub fn close_all(&mut self) {
        unsafe {
            for (_, &hwnd) in &self.inactive_overlays {
                DestroyWindow(hwnd);
            }
            for (_, &hwnd) in &self.active_overlays {
                DestroyWindow(hwnd);
            }
        }
        self.inactive_overlays.clear();
        self.active_overlays.clear();
        println!("[Overlay] Closed all overlays");
    }

    /// Close only inactive overlays
    pub fn close_inactive(&mut self) {
        unsafe {
            for (_, &hwnd) in &self.inactive_overlays {
                DestroyWindow(hwnd);
            }
        }
        self.inactive_overlays.clear();
        println!("[Overlay] Closed inactive overlays");
    }

    /// Close only active overlays
    pub fn close_active(&mut self) {
        unsafe {
            for (_, &hwnd) in &self.active_overlays {
                DestroyWindow(hwnd);
            }
        }
        self.active_overlays.clear();
        println!("[Overlay] Closed active overlays");
    }

    /// Recreate inactive overlays with new displays
    pub fn recreate_inactive_overlays(&mut self, displays: &[DisplayInfo]) -> Result<(), String> {
        self.close_inactive();
        self.create_inactive_overlays(displays)
    }

    /// Recreate active overlays with new displays
    pub fn recreate_active_overlays(&mut self, displays: &[DisplayInfo]) -> Result<(), String> {
        self.close_active();
        self.create_active_overlays(displays)
    }

    /// Recreate all overlays with new displays
    pub fn recreate_all_overlays(&mut self, displays: &[DisplayInfo]) -> Result<(), String> {
        self.close_all();
        self.create_inactive_overlays(displays)?;
        self.create_active_overlays(displays)?;
        Ok(())
    }

    /// Update inactive overlay color
    pub fn set_inactive_color(&mut self, color: OverlayColor, displays: &[DisplayInfo]) -> Result<(), String> {
        self.inactive_color = color;
        // Recreate inactive overlays with new color
        self.recreate_inactive_overlays(displays)
    }

    /// Update active overlay color
    pub fn set_active_color(&mut self, color: Option<OverlayColor>, displays: &[DisplayInfo]) -> Result<(), String> {
        self.active_color = color;
        // Recreate active overlays with new color
        self.recreate_active_overlays(displays)
    }

    /// Get current overlay count (inactive + active)
    pub fn count(&self) -> usize {
        self.inactive_overlays.len() + self.active_overlays.len()
    }

    /// Get inactive overlay count
    pub fn inactive_count(&self) -> usize {
        self.inactive_overlays.len()
    }

    /// Get active overlay count
    pub fn active_count(&self) -> usize {
        self.active_overlays.len()
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