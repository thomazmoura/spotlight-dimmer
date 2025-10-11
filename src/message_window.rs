// This entire module is Windows-only
// Message window infrastructure for event-driven Windows API communication
// Phase 2 of polling-to-events migration

use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, HWND_MESSAGE, WNDCLASSEXW,
};

// Custom message constants for event-driven architecture
// Phase 2: Testing infrastructure
pub const WM_USER_TEST: UINT = 0x0400; // WM_USER base

// Reserved for future phases (commented out until needed)
// Phase 4: Foreground window change events
// const WM_USER_FOREGROUND_CHANGED: UINT = 0x0401;

// Phase 5: Window location/size change events
// const WM_USER_WINDOW_MOVED: UINT = 0x0402;

// Phase 6: Configuration file change events
// const WM_USER_CONFIG_CHANGED: UINT = 0x0403;

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

/// Message-only window for receiving custom Windows messages
///
/// This window is created with HWND_MESSAGE as parent, making it a message-only window
/// that exists solely for inter-thread communication. It doesn't appear in the taskbar,
/// Alt+Tab list, or on screen. It's more efficient than a hidden window because it
/// doesn't participate in z-order, painting, or input processing.
///
/// Message-only windows are ideal for:
/// - Receiving custom messages from event hooks
/// - Inter-thread communication
/// - Timer messages
/// - System notifications
pub struct MessageWindow {
    hwnd: HWND,
}

impl MessageWindow {
    /// Create a new message-only window
    ///
    /// Returns a MessageWindow instance or an error string describing the failure.
    ///
    /// The window is created with HWND_MESSAGE as parent, which creates a message-only
    /// window that cannot be enumerated by EnumWindows or seen by users.
    pub fn new() -> Result<Self, String> {
        unsafe {
            // Register window class for message-only window
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

            // Create message-only window (HWND_MESSAGE as parent)
            let window_name = to_wstring("Spotlight Dimmer Message Window");
            let hwnd = CreateWindowExW(
                0,                    // dwExStyle
                class_name.as_ptr(),  // lpClassName
                window_name.as_ptr(), // lpWindowName
                0,                    // dwStyle (no WS_* flags needed for message-only)
                0,                    // x
                0,                    // y
                0,                    // width
                0,                    // height
                HWND_MESSAGE,         // hWndParent - KEY: creates message-only window
                ptr::null_mut(),      // hMenu
                hinstance,            // hInstance
                ptr::null_mut(),      // lpParam
            );

            if hwnd.is_null() {
                let err = winapi::um::errhandlingapi::GetLastError();
                return Err(format!("Failed to create message window: error {}", err));
            }

            println!(
                "[MessageWindow] Message-only window created successfully (HWND: {:?})",
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
