// This entire module is Windows-only
// Message window infrastructure for event-driven Windows API communication
// Phase 2 of polling-to-events migration

use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, WM_DISPLAYCHANGE,
    WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_POPUP,
};

// Custom message constants for event-driven architecture
// Phase 2: Testing infrastructure
pub const WM_USER_TEST: UINT = 0x0400; // WM_USER base

// Phase 3: Display configuration change events
pub const WM_USER_DISPLAY_CHANGED: UINT = 0x0401;

// Reserved for future phases (commented out until needed)
// Phase 4: Foreground window change events
// const WM_USER_FOREGROUND_CHANGED: UINT = 0x0402;

// Phase 5: Window location/size change events
// const WM_USER_WINDOW_MOVED: UINT = 0x0403;

// Phase 6: Configuration file change events
// const WM_USER_CONFIG_CHANGED: UINT = 0x0404;

/// Thread-safe wrapper for HWND (Windows window handle)
///
/// HWND is a raw pointer (*mut HWND__) which doesn't implement Send by default.
/// However, HWND values are opaque handles that can be safely sent between threads
/// as they're just integer identifiers to kernel objects. Windows API functions
/// that use HWND are internally thread-safe.
///
/// Safety: This wrapper stores HWND as usize, making it Send. The actual HWND
/// operations are performed in the thread that owns the message window's message loop.
pub struct MessageWindowHandle(usize);

unsafe impl Send for MessageWindowHandle {}

impl MessageWindowHandle {
    /// Create a new MessageWindowHandle from an HWND
    pub fn new(hwnd: HWND) -> Self {
        Self(hwnd as usize)
    }

    /// Convert back to HWND for Windows API calls
    pub fn as_hwnd(&self) -> HWND {
        self.0 as HWND
    }
}

/// Statistics for message window (used for debugging and metrics)
static MESSAGE_COUNT: AtomicU32 = AtomicU32::new(0);

/// Get the total number of messages processed by the message window
pub fn get_message_count() -> u32 {
    MESSAGE_COUNT.load(Ordering::Relaxed)
}

/// Reset the message counter (useful for testing)
pub fn reset_message_count() {
    MESSAGE_COUNT.store(0, Ordering::Relaxed);
}

/// Phase 3: Display change timestamp for event-driven display configuration monitoring
/// Stores the millisecond timestamp when WM_DISPLAYCHANGE was received
/// Value of 0 means no pending display change
static DISPLAY_CHANGE_TIMESTAMP_MS: AtomicU64 = AtomicU64::new(0);

/// Flag to track if we've already processed the first stabilization check
/// 0 = no change pending, 1 = first check completed, needs verification
static DISPLAY_CHANGE_FIRST_CHECK_DONE: AtomicU32 = AtomicU32::new(0);

/// Two-stage stabilization delays:
/// - First check at 500ms (fast path for normal scenarios)
/// - Second check at 5000ms (safety net for complex reconfigurations)
const DISPLAY_STABILIZATION_DELAY_FIRST_MS: u64 = 500;
const DISPLAY_STABILIZATION_DELAY_SECOND_MS: u64 = 5000;

/// Get current time in milliseconds since UNIX_EPOCH
fn get_current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Record that a display change event occurred
/// Called by WM_DISPLAYCHANGE handler
fn record_display_change() {
    DISPLAY_CHANGE_TIMESTAMP_MS.store(get_current_time_ms(), Ordering::SeqCst);
    DISPLAY_CHANGE_FIRST_CHECK_DONE.store(0, Ordering::SeqCst); // Reset first check flag
}

/// Check if display change occurred and stabilization delay has passed
/// Two-stage approach:
/// 1. First check at 500ms (returns Some(false) to indicate first attempt)
/// 2. Second check at 5000ms (returns Some(true) to indicate final attempt)
///
/// Returns None if no pending change
pub fn check_display_change_ready() -> Option<bool> {
    let timestamp = DISPLAY_CHANGE_TIMESTAMP_MS.load(Ordering::SeqCst);
    if timestamp == 0 {
        return None; // No pending change
    }

    let current_time = get_current_time_ms();
    let elapsed = current_time.saturating_sub(timestamp);
    let first_check_done = DISPLAY_CHANGE_FIRST_CHECK_DONE.load(Ordering::SeqCst);

    if first_check_done == 0 && elapsed >= DISPLAY_STABILIZATION_DELAY_FIRST_MS {
        // First check (500ms) - mark as done
        DISPLAY_CHANGE_FIRST_CHECK_DONE.store(1, Ordering::SeqCst);
        Some(false) // Return false = first check
    } else if first_check_done == 1 && elapsed >= DISPLAY_STABILIZATION_DELAY_SECOND_MS {
        // Second check (5000ms) - reset everything
        DISPLAY_CHANGE_TIMESTAMP_MS.store(0, Ordering::SeqCst);
        DISPLAY_CHANGE_FIRST_CHECK_DONE.store(0, Ordering::SeqCst);
        Some(true) // Return true = final check
    } else {
        None // Still waiting
    }
}

/// Hidden top-level window for receiving Windows broadcast messages
///
/// IMPORTANT: This is NOT a message-only window (HWND_MESSAGE) because message-only
/// windows cannot receive broadcast messages like WM_DISPLAYCHANGE.
///
/// Instead, this is a hidden top-level window that receives:
/// - Broadcast messages (WM_DISPLAYCHANGE, WM_SETTINGCHANGE, etc.)
/// - Custom messages from event hooks
/// - Inter-thread communication
/// - Timer messages
///
/// The window is never shown and uses WS_EX_TOOLWINDOW to avoid taskbar/Alt+Tab.
pub struct MessageWindow {
    hwnd: HWND,
}

impl MessageWindow {
    /// Create a new hidden top-level window for receiving broadcast messages
    ///
    /// Returns a MessageWindow instance or an error string describing the failure.
    ///
    /// CRITICAL: Uses NULL parent (not HWND_MESSAGE) to create a top-level window
    /// that can receive broadcast messages like WM_DISPLAYCHANGE. The window is
    /// hidden and uses WS_EX_TOOLWINDOW to avoid appearing in taskbar/Alt+Tab.
    pub fn new() -> Result<Self, String> {
        unsafe {
            // Register window class for hidden broadcast receiver window
            let class_name = to_wstring("SpotlightDimmerMessageWindow");
            let hinstance = GetModuleHandleW(ptr::null());

            let wnd_class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(message_window_proc),
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
                // Class already registered is OK (error 1410)
                if err != 1410 {
                    return Err(format!(
                        "Failed to register message window class: error {}",
                        err
                    ));
                }
            }

            // Create hidden top-level window (NULL parent to receive broadcasts)
            // WS_EX_TOOLWINDOW prevents taskbar/Alt+Tab appearance
            let window_name = to_wstring("Spotlight Dimmer Broadcast Receiver");

            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,     // dwExStyle - prevents taskbar/Alt+Tab appearance
                class_name.as_ptr(),  // lpClassName
                window_name.as_ptr(), // lpWindowName
                WS_POPUP,             // dwStyle - popup window (no decorations)
                0,                    // x (off-screen is fine, never shown)
                0,                    // y
                0,                    // width
                0,                    // height
                ptr::null_mut(),      // hWndParent - NULL creates top-level window!
                ptr::null_mut(),      // hMenu
                hinstance,            // hInstance
                ptr::null_mut(),      // lpParam
            );

            if hwnd.is_null() {
                let err = winapi::um::errhandlingapi::GetLastError();
                return Err(format!(
                    "Failed to create broadcast receiver window: error {}",
                    err
                ));
            }

            // DO NOT call ShowWindow - keep it hidden
            // The window exists as a top-level window but is never visible

            println!(
                "[MessageWindow] Hidden broadcast receiver window created (HWND: {:?})",
                hwnd
            );

            Ok(Self { hwnd })
        }
    }

    /// Get the window handle for posting messages
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Get a thread-safe handle that can be sent to other threads
    pub fn handle(&self) -> MessageWindowHandle {
        MessageWindowHandle::new(self.hwnd)
    }
}

impl Drop for MessageWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
            println!("[MessageWindow] Message window destroyed");
        }
    }
}

/// Window procedure for message-only window
///
/// This function is called by Windows for every message sent to our message window.
/// During Phase 2, we log all messages for debugging. In later phases, this will
/// handle specific events like WM_USER_FOREGROUND_CHANGED, WM_USER_WINDOW_MOVED, etc.
unsafe extern "system" fn message_window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Increment message counter for all messages
    MESSAGE_COUNT.fetch_add(1, Ordering::Relaxed);

    match msg {
        WM_USER_TEST => {
            // Phase 2: Test message received
            println!(
                "[MessageWindow] WM_USER_TEST received (wparam: {}, lparam: {})",
                wparam, lparam
            );
            0 // Message handled
        }
        WM_DISPLAYCHANGE => {
            // Phase 3: Display configuration changed (resolution, hotplug, rotation, etc.)
            // Note: WM_DISPLAYCHANGE is sent BEFORE Windows finishes reconfiguring displays
            // Two-stage stabilization: 500ms (fast) + 5000ms (safety net)
            println!(
                "[MessageWindow] WM_DISPLAYCHANGE received - starting two-stage stabilization ({}ms + {}ms)",
                DISPLAY_STABILIZATION_DELAY_FIRST_MS, DISPLAY_STABILIZATION_DELAY_SECOND_MS
            );
            record_display_change();
            0 // Message handled
        }
        WM_USER_DISPLAY_CHANGED => {
            // Phase 3: Custom message for display change handling
            println!("[MessageWindow] WM_USER_DISPLAY_CHANGED received");
            0 // Message handled
        }
        // Future phases will add handlers here:
        // WM_USER_FOREGROUND_CHANGED => { ... }
        // WM_USER_WINDOW_MOVED => { ... }
        // WM_USER_CONFIG_CHANGED => { ... }
        _ => {
            // For Phase 2 debugging, log all other messages
            // This helps us understand what messages Windows sends to message-only windows
            // Comment out in later phases to reduce noise
            if msg >= winapi::um::winuser::WM_USER {
                // Only log custom messages (WM_USER and above)
                println!(
                    "[MessageWindow] Custom message 0x{:04X} (wparam: {}, lparam: {})",
                    msg, wparam, lparam
                );
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

/// Convert Rust string to null-terminated wide string for Windows API
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_window_handle_is_send() {
        // This test ensures MessageWindowHandle implements Send
        // If it doesn't, this won't compile
        fn assert_send<T: Send>() {}
        assert_send::<MessageWindowHandle>();
    }

    #[test]
    fn test_to_wstring() {
        let result = to_wstring("test");
        assert!(result.ends_with(&[0])); // Null-terminated
        assert!(result.len() > 1); // Has content + null
    }
}
