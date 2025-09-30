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
    TrackPopupMenu, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, MF_STRING, TPM_BOTTOMALIGN,
    TPM_LEFTALIGN, WNDCLASSEXW, WM_COMMAND, WM_LBUTTONUP, WM_RBUTTONUP, WS_OVERLAPPEDWINDOW,
};

const WM_TRAYICON: u32 = winapi::um::winuser::WM_USER + 1;
const CMD_EXIT: u16 = 1001;

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

        exe_path.parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "Failed to get executable directory".to_string())
    }
}

/// System tray icon manager
pub struct TrayIcon {
    hwnd: HWND,
    hicon: winapi::shared::windef::HICON,
    exit_flag: Arc<AtomicBool>,
}

impl TrayIcon {
    /// Create a new system tray icon
    pub fn new(icon_path: &str, tooltip: &str, exit_flag: Arc<AtomicBool>) -> Result<Self, String> {
        unsafe {
            // Register window class for hidden message window
            let class_name = to_wstring("SpotlightDimmerTrayWindow");
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
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: ptr::null_mut(),
            };

            if RegisterClassExW(&wnd_class) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                // Class already registered is OK
                if err != 1410 {
                    return Err(format!("Failed to register tray window class: error {}", err));
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

            // Store exit flag pointer in window user data for access in window proc
            winapi::um::winuser::SetWindowLongPtrW(
                hwnd,
                winapi::um::winuser::GWLP_USERDATA,
                Arc::into_raw(exit_flag.clone()) as isize,
            );

            // Get absolute path to icon file (relative to executable)
            let exe_dir = get_exe_directory().map_err(|e| {
                DestroyWindow(hwnd);
                e
            })?;
            let icon_full_path = exe_dir.join(icon_path);

            if !icon_full_path.exists() {
                DestroyWindow(hwnd);
                return Err(format!(
                    "Icon file not found: {} (looked in: {})",
                    icon_path,
                    icon_full_path.display()
                ));
            }

            // Load icon from file using absolute path
            let icon_path_wide = to_wstring(&icon_full_path.to_string_lossy());
            let hicon = LoadImageW(
                ptr::null_mut(),
                icon_path_wide.as_ptr(),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            ) as winapi::shared::windef::HICON;

            if hicon.is_null() {
                DestroyWindow(hwnd);
                return Err(format!("Failed to load icon from: {}", icon_full_path.display()));
            }

            // Create tray icon
            let tooltip_wide = to_wstring(tooltip);
            let mut nid: NOTIFYICONDATAW = mem::zeroed();
            nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = hicon;

            // Copy tooltip (max 128 chars)
            let tooltip_len = tooltip_wide.len().min(127);
            ptr::copy_nonoverlapping(
                tooltip_wide.as_ptr(),
                nid.szTip.as_mut_ptr(),
                tooltip_len,
            );

            if Shell_NotifyIconW(NIM_ADD, &mut nid) == 0 {
                winapi::um::winuser::DestroyIcon(hicon);
                DestroyWindow(hwnd);
                return Err("Failed to add tray icon".to_string());
            }

            println!("[Tray] System tray icon created successfully");

            Ok(Self {
                hwnd,
                hicon,
                exit_flag,
            })
        }
    }

    /// Get the window handle for message processing
    pub fn hwnd(&self) -> HWND {
        self.hwnd
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
            winapi::um::winuser::DestroyIcon(self.hicon);
            DestroyWindow(self.hwnd);

            // Cleanup exit flag pointer stored in window user data
            let ptr = winapi::um::winuser::GetWindowLongPtrW(
                self.hwnd,
                winapi::um::winuser::GWLP_USERDATA,
            ) as *const AtomicBool;
            if !ptr.is_null() {
                drop(Arc::from_raw(ptr));
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
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            // Handle menu commands
            let cmd_id = (wparam & 0xFFFF) as u16;
            if cmd_id == CMD_EXIT {
                println!("[Tray] Exit command received");
                // Get exit flag from window user data
                let ptr = winapi::um::winuser::GetWindowLongPtrW(
                    hwnd,
                    winapi::um::winuser::GWLP_USERDATA,
                ) as *const AtomicBool;
                if !ptr.is_null() {
                    let exit_flag = Arc::from_raw(ptr);
                    exit_flag.store(true, Ordering::SeqCst);
                    // Don't drop - it's still owned by TrayIcon
                    mem::forget(exit_flag);
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