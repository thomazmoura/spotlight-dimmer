// This entire module is Windows-only
// Note: cfg(windows) is applied at the module level in lib.rs

use crate::config::OverlayColor;
use crate::platform::DisplayInfo;
use crate::tmux_overlay::TerminalGeometry;
use crate::tmux_watcher::TmuxPaneInfo;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
pub use winapi::shared::windef::{HBRUSH, HWND, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{CreateSolidBrush, RGB};
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, PostQuitMessage, RegisterClassExW,
    SetLayeredWindowAttributes, ShowWindow, LWA_ALPHA, SW_HIDE, SW_SHOW, WNDCLASSEXW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

// These constants are used in the OverlayManager implementation
#[allow(dead_code)]
const INACTIVE_CLASS_NAME: &str = "SpotlightDimmerInactiveOverlay";
#[allow(dead_code)]
const ACTIVE_CLASS_NAME: &str = "SpotlightDimmerActiveOverlay";
#[allow(dead_code)]
const PARTIAL_CLASS_NAME: &str = "SpotlightDimmerPartialOverlay";

/// Manager for overlay windows
/// Note: Used by main_new.rs binary, but appears unused due to cross-module analysis limitations
#[allow(dead_code)]
pub struct OverlayManager {
    inactive_overlays: HashMap<String, HWND>,
    active_overlays: HashMap<String, HWND>,
    partial_overlays: HashMap<String, Vec<HWND>>,
    tmux_overlays: Vec<HWND>, // Overlays for tmux pane dimming
    inactive_color: OverlayColor,
    active_color: Option<OverlayColor>,
}

// Methods used by main_new.rs binary but appear unused when compiling for config binary
#[allow(dead_code)]
impl OverlayManager {
    pub fn new(
        inactive_color: OverlayColor,
        active_color: Option<OverlayColor>,
    ) -> Result<Self, String> {
        // Register window classes for inactive, active, and partial overlays
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
                    return Err(format!(
                        "Failed to register inactive window class: error {}",
                        err
                    ));
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
                    return Err(format!(
                        "Failed to register active window class: error {}",
                        err
                    ));
                }
            }

            // Register partial overlay window class
            let partial_class_name = to_wstring(PARTIAL_CLASS_NAME);
            let partial_wnd_class = WNDCLASSEXW {
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
                lpszClassName: partial_class_name.as_ptr(),
                hIconSm: ptr::null_mut(),
            };

            if RegisterClassExW(&partial_wnd_class) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                // Class already registered is OK
                if err != 1410 {
                    // ERROR_CLASS_ALREADY_EXISTS
                    return Err(format!(
                        "Failed to register partial window class: error {}",
                        err
                    ));
                }
            }
        }

        Ok(Self {
            inactive_overlays: HashMap::new(),
            active_overlays: HashMap::new(),
            partial_overlays: HashMap::new(),
            tmux_overlays: Vec::new(),
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
            let window_name = to_wstring(&format!(
                "{} Overlay {}",
                if is_inactive { "Inactive" } else { "Active" },
                display.id
            ));
            let hinstance = GetModuleHandleW(ptr::null());

            // Create a brush with the overlay color
            let colorref = RGB(color.r, color.g, color.b);
            let brush = CreateSolidBrush(colorref);

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED
                    | WS_EX_TRANSPARENT
                    | WS_EX_TOPMOST
                    | WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE,
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
    /// Uses ShowWindow for safe visibility changes on layered windows
    pub fn update_visibility(&self, active_display_id: &str) {
        unsafe {
            // Manage inactive overlays - show on non-active displays only
            for (display_id, &hwnd) in &self.inactive_overlays {
                let should_show = display_id != active_display_id;
                ShowWindow(hwnd, if should_show { SW_SHOW } else { SW_HIDE });
            }

            // Manage active overlays - show on active display only
            for (display_id, &hwnd) in &self.active_overlays {
                let should_show = display_id == active_display_id;
                ShowWindow(hwnd, if should_show { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    /// Hide all overlays (inactive, active, and partial)
    /// Used during drag operations to prevent ghost window creation
    pub fn hide_all(&self) {
        unsafe {
            for &hwnd in self.inactive_overlays.values() {
                ShowWindow(hwnd, SW_HIDE);
            }
            for &hwnd in self.active_overlays.values() {
                ShowWindow(hwnd, SW_HIDE);
            }
            for hwnds in self.partial_overlays.values() {
                for &hwnd in hwnds {
                    ShowWindow(hwnd, SW_HIDE);
                }
            }
        }
    }

    /// Show all overlays (inactive, active, and partial)
    /// Used when unpausing to restore all overlays to visible state
    pub fn show_all(&self) {
        unsafe {
            for &hwnd in self.inactive_overlays.values() {
                ShowWindow(hwnd, SW_SHOW);
            }
            for &hwnd in self.active_overlays.values() {
                ShowWindow(hwnd, SW_SHOW);
            }
            for hwnds in self.partial_overlays.values() {
                for &hwnd in hwnds {
                    ShowWindow(hwnd, SW_SHOW);
                }
            }
        }
    }

    /// Close all overlays (inactive, active, and partial)
    pub fn close_all(&mut self) {
        unsafe {
            for &hwnd in self.inactive_overlays.values() {
                DestroyWindow(hwnd);
            }
            for &hwnd in self.active_overlays.values() {
                DestroyWindow(hwnd);
            }
            for hwnds in self.partial_overlays.values() {
                for &hwnd in hwnds {
                    DestroyWindow(hwnd);
                }
            }
            for &hwnd in &self.tmux_overlays {
                DestroyWindow(hwnd);
            }
        }
        self.inactive_overlays.clear();
        self.active_overlays.clear();
        self.partial_overlays.clear();
        self.tmux_overlays.clear();
        println!("[Overlay] Closed all overlays");
    }

    /// Close only inactive overlays
    pub fn close_inactive(&mut self) {
        unsafe {
            for &hwnd in self.inactive_overlays.values() {
                DestroyWindow(hwnd);
            }
        }
        self.inactive_overlays.clear();
        println!("[Overlay] Closed inactive overlays");
    }

    /// Close only active overlays
    pub fn close_active(&mut self) {
        unsafe {
            for &hwnd in self.active_overlays.values() {
                DestroyWindow(hwnd);
            }
        }
        self.active_overlays.clear();
        println!("[Overlay] Closed active overlays");
    }

    /// Resize and reposition an active overlay to match a window rectangle
    /// Used for windowed mode when partial dimming is enabled
    pub fn resize_active_overlay(&self, display_id: &str, window_rect: RECT) -> Result<(), String> {
        if let Some(&hwnd) = self.active_overlays.get(display_id) {
            unsafe {
                use winapi::um::winuser::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

                let width = window_rect.right - window_rect.left;
                let height = window_rect.bottom - window_rect.top;

                if SetWindowPos(
                    hwnd,
                    ptr::null_mut(),
                    window_rect.left,
                    window_rect.top,
                    width,
                    height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                ) == 0
                {
                    return Err("Failed to resize active overlay".to_string());
                }

                println!(
                    "[Overlay] Resized active overlay for display {} to match window at ({}, {}) size {}x{}",
                    display_id, window_rect.left, window_rect.top, width, height
                );
            }
            Ok(())
        } else {
            Err(format!(
                "No active overlay found for display {}",
                display_id
            ))
        }
    }

    /// Restore active overlay to full display size
    /// Used when switching from windowed to fullscreen mode
    pub fn restore_active_overlay_full_size(
        &self,
        display_id: &str,
        display: &DisplayInfo,
    ) -> Result<(), String> {
        if let Some(&hwnd) = self.active_overlays.get(display_id) {
            unsafe {
                use winapi::um::winuser::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

                if SetWindowPos(
                    hwnd,
                    ptr::null_mut(),
                    display.x,
                    display.y,
                    display.width,
                    display.height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                ) == 0
                {
                    return Err("Failed to restore active overlay to full size".to_string());
                }

                println!(
                    "[Overlay] Restored active overlay for display {} to full size at ({}, {}) size {}x{}",
                    display_id, display.x, display.y, display.width, display.height
                );
            }
            Ok(())
        } else {
            Err(format!(
                "No active overlay found for display {}",
                display_id
            ))
        }
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

    /// Update inactive overlay color
    pub fn set_inactive_color(
        &mut self,
        color: OverlayColor,
        displays: &[DisplayInfo],
    ) -> Result<(), String> {
        self.inactive_color = color;
        // Recreate inactive overlays with new color
        self.recreate_inactive_overlays(displays)
    }

    /// Update active overlay color
    pub fn set_active_color(
        &mut self,
        color: Option<OverlayColor>,
        displays: &[DisplayInfo],
    ) -> Result<(), String> {
        self.active_color = color;
        // Recreate active overlays with new color
        self.recreate_active_overlays(displays)
    }

    /// Update inactive color without recreating overlays (used when overlays are disabled)
    pub fn update_inactive_color_only(&mut self, color: OverlayColor) {
        self.inactive_color = color;
    }

    /// Update active color without recreating overlays (used when overlays are disabled)
    pub fn update_active_color_only(&mut self, color: Option<OverlayColor>) {
        self.active_color = color;
    }

    /// Get inactive overlay count
    pub fn inactive_count(&self) -> usize {
        self.inactive_overlays.len()
    }

    /// Get active overlay count
    pub fn active_count(&self) -> usize {
        self.active_overlays.len()
    }

    /// Create partial overlays for a specific display based on window bounds
    /// window_rect: The rectangle of the focused window (in screen coordinates)
    /// display: The display info for the active display
    pub fn create_partial_overlays(
        &mut self,
        display_id: &str,
        window_rect: RECT,
        display: &DisplayInfo,
    ) -> Result<(), String> {
        // Clear existing partial overlays for this display
        self.clear_partial_overlays_for_display(display_id);

        let mut new_overlays = Vec::new();

        // Define tolerance for edge detection (5 pixels)
        const EDGE_TOLERANCE: i32 = 5;

        // Calculate display bounds
        let display_left = display.x;
        let display_top = display.y;
        let display_right = display.x + display.width;
        let display_bottom = display.y + display.height;

        // Check if window is touching each edge
        let touches_left = (window_rect.left - display_left).abs() <= EDGE_TOLERANCE;
        let touches_right = (window_rect.right - display_right).abs() <= EDGE_TOLERANCE;
        let touches_top = (window_rect.top - display_top).abs() <= EDGE_TOLERANCE;
        let touches_bottom = (window_rect.bottom - display_bottom).abs() <= EDGE_TOLERANCE;

        // Create top overlay first (spans full width)
        let _top_overlay_height = if !touches_top && window_rect.top > display_top {
            let height = window_rect.top - display_top;
            if height > 0 {
                let hwnd = self.create_partial_overlay_window(
                    display_id,
                    "Top",
                    display_left,
                    display_top,
                    display.width,
                    height,
                )?;
                new_overlays.push(hwnd);
                height
            } else {
                0
            }
        } else {
            0
        };

        // Create bottom overlay (spans full width)
        let _bottom_overlay_height = if !touches_bottom && window_rect.bottom < display_bottom {
            let height = display_bottom - window_rect.bottom;
            if height > 0 {
                let hwnd = self.create_partial_overlay_window(
                    display_id,
                    "Bottom",
                    display_left,
                    window_rect.bottom,
                    display.width,
                    height,
                )?;
                new_overlays.push(hwnd);
                height
            } else {
                0
            }
        } else {
            0
        };

        // Calculate vertical bounds for left/right overlays (align with window edges)
        let vertical_start = window_rect.top;
        let vertical_end = window_rect.bottom;
        let vertical_height = vertical_end - vertical_start;

        // Create left overlay
        if !touches_left && window_rect.left > display_left && vertical_height > 0 {
            let width = window_rect.left - display_left;
            if width > 0 {
                let hwnd = self.create_partial_overlay_window(
                    display_id,
                    "Left",
                    display_left,
                    vertical_start,
                    width,
                    vertical_height,
                )?;
                new_overlays.push(hwnd);
            }
        }

        // Create right overlay
        if !touches_right && window_rect.right < display_right && vertical_height > 0 {
            let width = display_right - window_rect.right;
            if width > 0 {
                let hwnd = self.create_partial_overlay_window(
                    display_id,
                    "Right",
                    window_rect.right,
                    vertical_start,
                    width,
                    vertical_height,
                )?;
                new_overlays.push(hwnd);
            }
        }

        if !new_overlays.is_empty() {
            println!(
                "[Overlay] Created {} partial overlay(s) for display {}",
                new_overlays.len(),
                display_id
            );
            self.partial_overlays
                .insert(display_id.to_string(), new_overlays);
        }

        Ok(())
    }

    /// Create a single partial overlay window
    fn create_partial_overlay_window(
        &self,
        display_id: &str,
        side: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<HWND, String> {
        unsafe {
            let class_name = to_wstring(PARTIAL_CLASS_NAME);
            let window_name = to_wstring(&format!("Partial Overlay {} {}", display_id, side));
            let hinstance = GetModuleHandleW(ptr::null());

            let colorref = RGB(
                self.inactive_color.r,
                self.inactive_color.g,
                self.inactive_color.b,
            );
            let brush = CreateSolidBrush(colorref);

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED
                    | WS_EX_TRANSPARENT
                    | WS_EX_TOPMOST
                    | WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_POPUP,
                x,
                y,
                width,
                height,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance,
                ptr::null_mut(),
            );

            if hwnd.is_null() {
                let err = winapi::um::errhandlingapi::GetLastError();
                return Err(format!(
                    "Failed to create partial overlay window: error {}",
                    err
                ));
            }

            use winapi::um::winuser::{SetClassLongPtrW, GCLP_HBRBACKGROUND};
            SetClassLongPtrW(hwnd, GCLP_HBRBACKGROUND, brush as isize);

            let alpha = self.inactive_color.to_alpha_byte();
            if SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA) == 0 {
                DestroyWindow(hwnd);
                return Err("Failed to set partial overlay transparency".to_string());
            }

            ShowWindow(hwnd, SW_SHOW);

            println!(
                "[Overlay] Created partial overlay ({}) at ({}, {}) size {}x{}",
                side, x, y, width, height
            );

            Ok(hwnd)
        }
    }

    /// Clear partial overlays for a specific display
    pub fn clear_partial_overlays_for_display(&mut self, display_id: &str) {
        if let Some(hwnds) = self.partial_overlays.remove(display_id) {
            unsafe {
                for hwnd in hwnds {
                    DestroyWindow(hwnd);
                }
            }
            println!(
                "[Overlay] Cleared partial overlays for display {}",
                display_id
            );
        }
    }

    /// Clear all partial overlays
    pub fn clear_all_partial_overlays(&mut self) {
        unsafe {
            for hwnds in self.partial_overlays.values() {
                for &hwnd in hwnds {
                    DestroyWindow(hwnd);
                }
            }
        }
        self.partial_overlays.clear();
        println!("[Overlay] Cleared all partial overlays");
    }

    /// Update both partial overlays AND active overlay atomically (all at once)
    /// This prevents visual fragmentation by ensuring all overlays are repositioned simultaneously
    /// Returns true if overlays were successfully updated, false if they need to be recreated
    pub fn update_partial_and_active_overlays_atomic(
        &self,
        display_id: &str,
        window_rect: RECT,
        display: &DisplayInfo,
        update_active: bool,
        is_maximized: bool,
    ) -> Result<bool, String> {
        // Check if we have existing overlays for this display
        let existing_overlays = match self.partial_overlays.get(display_id) {
            Some(overlays) => overlays,
            None => return Ok(false), // No existing overlays, need to create
        };

        // Define tolerance for edge detection (5 pixels)
        const EDGE_TOLERANCE: i32 = 5;

        // Calculate display bounds
        let display_left = display.x;
        let display_top = display.y;
        let display_right = display.x + display.width;
        let display_bottom = display.y + display.height;

        // Check if window is touching each edge
        let touches_left = (window_rect.left - display_left).abs() <= EDGE_TOLERANCE;
        let touches_right = (window_rect.right - display_right).abs() <= EDGE_TOLERANCE;
        let touches_top = (window_rect.top - display_top).abs() <= EDGE_TOLERANCE;
        let touches_bottom = (window_rect.bottom - display_bottom).abs() <= EDGE_TOLERANCE;

        // Calculate which overlays we need (Top, Bottom, Left, Right)
        let mut needed_overlays = Vec::new();

        // Top overlay
        if !touches_top && window_rect.top > display_top {
            let height = window_rect.top - display_top;
            if height > 0 {
                needed_overlays.push((display_left, display_top, display.width, height));
            }
        }

        // Bottom overlay
        if !touches_bottom && window_rect.bottom < display_bottom {
            let height = display_bottom - window_rect.bottom;
            if height > 0 {
                needed_overlays.push((display_left, window_rect.bottom, display.width, height));
            }
        }

        // Calculate vertical bounds for left/right overlays
        let vertical_start = window_rect.top;
        let vertical_end = window_rect.bottom;
        let vertical_height = vertical_end - vertical_start;

        // Left overlay
        if !touches_left && window_rect.left > display_left && vertical_height > 0 {
            let width = window_rect.left - display_left;
            if width > 0 {
                needed_overlays.push((display_left, vertical_start, width, vertical_height));
            }
        }

        // Right overlay
        if !touches_right && window_rect.right < display_right && vertical_height > 0 {
            let width = display_right - window_rect.right;
            if width > 0 {
                needed_overlays.push((window_rect.right, vertical_start, width, vertical_height));
            }
        }

        // If the number of needed overlays doesn't match existing overlays, recreate
        if needed_overlays.len() != existing_overlays.len() {
            return Ok(false);
        }

        // Get active overlay if we need to update it
        let active_hwnd = if update_active {
            self.active_overlays.get(display_id).copied()
        } else {
            None
        };

        // Update all overlays atomically using deferred window positioning
        unsafe {
            use winapi::um::winuser::{
                BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, SWP_NOACTIVATE,
                SWP_NOREDRAW, SWP_NOZORDER,
            };

            // Calculate total number of windows to update
            let total_windows = existing_overlays.len() + if active_hwnd.is_some() { 1 } else { 0 };

            // Begin deferred window positioning for batch update
            let hdwp = BeginDeferWindowPos(total_windows as i32);
            if hdwp.is_null() {
                eprintln!("[Overlay] Warning: BeginDeferWindowPos failed");
                return Ok(false);
            }

            let mut current_hdwp = hdwp;

            // Defer each partial overlay position AND size change
            // SWP_NOREDRAW prevents individual redraws - all windows will redraw together after EndDeferWindowPos
            for (i, &hwnd) in existing_overlays.iter().enumerate() {
                if i >= needed_overlays.len() {
                    break;
                }

                let (x, y, width, height) = needed_overlays[i];

                current_hdwp = DeferWindowPos(
                    current_hdwp,
                    hwnd,
                    ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOREDRAW,
                );

                if current_hdwp.is_null() {
                    let err = winapi::um::errhandlingapi::GetLastError();
                    eprintln!(
                        "[Overlay] Warning: DeferWindowPos failed for partial overlay: error {}",
                        err
                    );
                    return Ok(false);
                }
            }

            // Defer active overlay position AND size change
            if let Some(hwnd) = active_hwnd {
                let (x, y, width, height) = if is_maximized {
                    // Restore to full display size
                    (display.x, display.y, display.width, display.height)
                } else {
                    // Match window size
                    let w = window_rect.right - window_rect.left;
                    let h = window_rect.bottom - window_rect.top;
                    (window_rect.left, window_rect.top, w, h)
                };

                current_hdwp = DeferWindowPos(
                    current_hdwp,
                    hwnd,
                    ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOREDRAW,
                );

                if current_hdwp.is_null() {
                    let err = winapi::um::errhandlingapi::GetLastError();
                    eprintln!(
                        "[Overlay] Warning: DeferWindowPos failed for active overlay: error {}",
                        err
                    );
                    return Ok(false);
                }
            }

            // Apply all deferred window positions atomically
            // This will trigger a single redraw of all windows simultaneously
            if EndDeferWindowPos(current_hdwp) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                eprintln!("[Overlay] Warning: EndDeferWindowPos failed: error {}", err);
                return Ok(false);
            }
        }

        Ok(true) // Successfully updated
    }

    /// Update (reposition/resize) existing partial overlays without recreating them
    /// Returns true if overlays were successfully updated, false if they need to be recreated
    pub fn update_partial_overlays(
        &self,
        display_id: &str,
        window_rect: RECT,
        display: &DisplayInfo,
    ) -> Result<bool, String> {
        // Check if we have existing overlays for this display
        let existing_overlays = match self.partial_overlays.get(display_id) {
            Some(overlays) => overlays,
            None => return Ok(false), // No existing overlays, need to create
        };

        // Define tolerance for edge detection (5 pixels)
        const EDGE_TOLERANCE: i32 = 5;

        // Calculate display bounds
        let display_left = display.x;
        let display_top = display.y;
        let display_right = display.x + display.width;
        let display_bottom = display.y + display.height;

        // Check if window is touching each edge
        let touches_left = (window_rect.left - display_left).abs() <= EDGE_TOLERANCE;
        let touches_right = (window_rect.right - display_right).abs() <= EDGE_TOLERANCE;
        let touches_top = (window_rect.top - display_top).abs() <= EDGE_TOLERANCE;
        let touches_bottom = (window_rect.bottom - display_bottom).abs() <= EDGE_TOLERANCE;

        // Calculate which overlays we need (Top, Bottom, Left, Right)
        let mut needed_overlays = Vec::new();

        // Top overlay
        if !touches_top && window_rect.top > display_top {
            let height = window_rect.top - display_top;
            if height > 0 {
                needed_overlays.push(("Top", display_left, display_top, display.width, height));
            }
        }

        // Bottom overlay
        if !touches_bottom && window_rect.bottom < display_bottom {
            let height = display_bottom - window_rect.bottom;
            if height > 0 {
                needed_overlays.push((
                    "Bottom",
                    display_left,
                    window_rect.bottom,
                    display.width,
                    height,
                ));
            }
        }

        // Calculate vertical bounds for left/right overlays
        let vertical_start = window_rect.top;
        let vertical_end = window_rect.bottom;
        let vertical_height = vertical_end - vertical_start;

        // Left overlay
        if !touches_left && window_rect.left > display_left && vertical_height > 0 {
            let width = window_rect.left - display_left;
            if width > 0 {
                needed_overlays.push((
                    "Left",
                    display_left,
                    vertical_start,
                    width,
                    vertical_height,
                ));
            }
        }

        // Right overlay
        if !touches_right && window_rect.right < display_right && vertical_height > 0 {
            let width = display_right - window_rect.right;
            if width > 0 {
                needed_overlays.push((
                    "Right",
                    window_rect.right,
                    vertical_start,
                    width,
                    vertical_height,
                ));
            }
        }

        // If the number of needed overlays doesn't match existing overlays, recreate
        if needed_overlays.len() != existing_overlays.len() {
            return Ok(false);
        }

        // Update all overlays atomically using deferred window positioning
        // This ensures all overlays are repositioned and rendered simultaneously
        unsafe {
            use winapi::um::winuser::{
                BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, SWP_NOACTIVATE,
                SWP_NOREDRAW, SWP_NOZORDER,
            };

            // Begin deferred window positioning for batch update
            let hdwp = BeginDeferWindowPos(existing_overlays.len() as i32);
            if hdwp.is_null() {
                eprintln!("[Overlay] Warning: BeginDeferWindowPos failed");
                return Ok(false);
            }

            let mut current_hdwp = hdwp;

            // Defer each window position AND size change
            // SWP_NOREDRAW prevents individual redraws - all windows will redraw together after EndDeferWindowPos
            for (i, &hwnd) in existing_overlays.iter().enumerate() {
                if i >= needed_overlays.len() {
                    break;
                }

                let (_side, x, y, width, height) = needed_overlays[i];

                current_hdwp = DeferWindowPos(
                    current_hdwp,
                    hwnd,
                    ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOREDRAW,
                );

                if current_hdwp.is_null() {
                    let err = winapi::um::errhandlingapi::GetLastError();
                    eprintln!("[Overlay] Warning: DeferWindowPos failed: error {}", err);
                    return Ok(false);
                }
            }

            // Apply all deferred window positions atomically
            if EndDeferWindowPos(current_hdwp) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                eprintln!("[Overlay] Warning: EndDeferWindowPos failed: error {}", err);
                return Ok(false);
            }
        }

        Ok(true) // Successfully updated
    }

    /// Create overlays for tmux inactive panes
    /// Uses tmux pane info and terminal geometry to calculate overlay positions
    pub fn create_tmux_overlays(
        &mut self,
        pane_info: &TmuxPaneInfo,
        terminal_window_rect: &RECT,
        terminal_geometry: &TerminalGeometry,
    ) -> Result<(), String> {
        // Clear existing tmux overlays first
        self.clear_tmux_overlays();

        // Calculate overlay rectangles for inactive pane areas
        let overlay_rects = crate::tmux_overlay::calculate_tmux_overlay_rects(
            pane_info,
            terminal_window_rect,
            terminal_geometry,
        );

        println!(
            "[Overlay] Creating {} tmux overlays for pane at ({},{}) to ({},{})",
            overlay_rects.len(),
            pane_info.pane_left,
            pane_info.pane_top,
            pane_info.pane_right,
            pane_info.pane_bottom
        );

        // Create overlay windows for each rectangle
        unsafe {
            let class_name = to_wstring(PARTIAL_CLASS_NAME);
            let window_name = to_wstring("Tmux Overlay");

            for (i, overlay_rect) in overlay_rects.iter().enumerate() {
                let hwnd = CreateWindowExW(
                    WS_EX_LAYERED
                        | WS_EX_TRANSPARENT
                        | WS_EX_TOPMOST
                        | WS_EX_TOOLWINDOW
                        | WS_EX_NOACTIVATE,
                    class_name.as_ptr(),
                    window_name.as_ptr(),
                    WS_POPUP,
                    overlay_rect.left,
                    overlay_rect.top,
                    overlay_rect.width(),
                    overlay_rect.height(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    GetModuleHandleW(ptr::null()),
                    ptr::null_mut(),
                );

                if hwnd.is_null() {
                    return Err(format!("Failed to create tmux overlay window {}", i));
                }

                // Set transparency
                let colorref = ((self.inactive_color.b as u32) << 16)
                    | ((self.inactive_color.g as u32) << 8)
                    | (self.inactive_color.r as u32);
                let alpha = (self.inactive_color.a * 255.0) as u8;

                SetLayeredWindowAttributes(hwnd, colorref, alpha, LWA_ALPHA);

                // Show the overlay
                ShowWindow(hwnd, SW_SHOW);

                self.tmux_overlays.push(hwnd);

                println!(
                    "[Overlay] Created tmux overlay {} at ({},{}) size {}x{}",
                    i,
                    overlay_rect.left,
                    overlay_rect.top,
                    overlay_rect.width(),
                    overlay_rect.height()
                );
            }
        }

        Ok(())
    }

    /// Clear all tmux overlays
    pub fn clear_tmux_overlays(&mut self) {
        if self.tmux_overlays.is_empty() {
            return;
        }

        unsafe {
            for &hwnd in &self.tmux_overlays {
                DestroyWindow(hwnd);
            }
        }

        println!(
            "[Overlay] Cleared {} tmux overlays",
            self.tmux_overlays.len()
        );
        self.tmux_overlays.clear();
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        self.close_all();
    }
}

/// Window procedure for overlay windows
/// Used as callback in RegisterClassExW - appears unused due to function pointer limitations
#[allow(dead_code)]
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
/// Used throughout overlay creation - appears unused due to inlining
#[allow(dead_code)]
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
