use super::{ActiveWindowInfo, DisplayInfo, DisplayManager, WindowManager};
use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::shared::windef::{HDC, HMONITOR, HWND, LPRECT, RECT};
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::psapi::{GetModuleBaseNameW, GetProcessImageFileNameW};
use winapi::um::winuser::{
    EnumDisplayMonitors, GetForegroundWindow, GetMonitorInfoW, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};

pub struct WindowsDisplayManager;
pub struct WindowsWindowManager;

impl DisplayManager for WindowsDisplayManager {
    fn get_displays(&self) -> Result<Vec<DisplayInfo>, String> {
        let mut displays = Vec::new();

        unsafe {
            extern "system" fn enum_proc(
                hmonitor: HMONITOR,
                _hdc: HDC,
                _rect: LPRECT,
                lparam: isize,
            ) -> i32 {
                unsafe {
                    let displays = &mut *(lparam as *mut Vec<DisplayInfo>);

                    let mut monitor_info: MONITORINFO = mem::zeroed();
                    monitor_info.cbSize = mem::size_of::<MONITORINFO>() as u32;

                    if GetMonitorInfoW(hmonitor, &mut monitor_info) != 0 {
                        let display = DisplayInfo {
                            id: format!("{:p}", hmonitor),
                            name: format!("Display {}", displays.len() + 1),
                            x: monitor_info.rcMonitor.left,
                            y: monitor_info.rcMonitor.top,
                            width: monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                            height: monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
                            is_primary: monitor_info.dwFlags & 1 != 0, // MONITORINFOF_PRIMARY
                        };
                        displays.push(display);
                    }
                }
                1 // Continue enumeration
            }

            EnumDisplayMonitors(
                ptr::null_mut(),
                ptr::null_mut(),
                Some(enum_proc),
                &mut displays as *mut _ as isize,
            );
        }

        if displays.is_empty() {
            Err("No displays found".to_string())
        } else {
            Ok(displays)
        }
    }

    fn get_display_count(&self) -> Result<usize, String> {
        let mut count = 0usize;

        unsafe {
            extern "system" fn count_proc(
                _hmonitor: HMONITOR,
                _hdc: HDC,
                _rect: LPRECT,
                lparam: isize,
            ) -> i32 {
                unsafe {
                    let count = &mut *(lparam as *mut usize);
                    *count += 1;
                }
                1 // Continue enumeration
            }

            EnumDisplayMonitors(
                ptr::null_mut(),
                ptr::null_mut(),
                Some(count_proc),
                &mut count as *mut _ as isize,
            );
        }

        Ok(count)
    }

    fn get_primary_display(&self) -> Result<DisplayInfo, String> {
        let displays = self.get_displays()?;
        displays
            .into_iter()
            .find(|d| d.is_primary)
            .ok_or_else(|| "No primary display found".to_string())
    }
}

impl WindowManager for WindowsWindowManager {
    fn get_active_window(&self) -> Result<ActiveWindowInfo, String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return Err("No active window found".to_string());
            }

            // Get window title
            let mut title_buffer = [0u16; 512];
            let title_len =
                GetWindowTextW(hwnd, title_buffer.as_mut_ptr(), title_buffer.len() as i32);
            let window_title = if title_len > 0 {
                String::from_utf16_lossy(&title_buffer[..title_len as usize])
            } else {
                "Unknown Window".to_string()
            };

            // Get process name
            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, &mut process_id);

            let process_name =
                get_process_name(process_id).unwrap_or_else(|_| "Unknown Process".to_string());

            // Get display information
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let display_id = format!("{:p}", hmonitor);

            Ok(ActiveWindowInfo {
                handle: hwnd as u64,
                display_id,
                process_name,
                window_title,
            })
        }
    }

    fn get_window_display(&self, window_handle: u64) -> Result<DisplayInfo, String> {
        unsafe {
            let hwnd = window_handle as HWND;
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

            let mut monitor_info: MONITORINFO = mem::zeroed();
            monitor_info.cbSize = mem::size_of::<MONITORINFO>() as u32;

            if GetMonitorInfoW(hmonitor, &mut monitor_info) != 0 {
                Ok(DisplayInfo {
                    id: format!("{:p}", hmonitor),
                    name: "Current Display".to_string(),
                    x: monitor_info.rcMonitor.left,
                    y: monitor_info.rcMonitor.top,
                    width: monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                    height: monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
                    is_primary: monitor_info.dwFlags & 1 != 0,
                })
            } else {
                Err("Failed to get window display info".to_string())
            }
        }
    }

    fn get_window_rect(&self, window_handle: u64) -> Result<RECT, String> {
        get_window_rect(window_handle)
    }

    fn is_window_maximized(&self, window_handle: u64) -> Result<bool, String> {
        is_window_maximized(window_handle)
    }
}

fn get_process_name(process_id: u32) -> Result<String, String> {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

    unsafe {
        let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, process_id);
        if process_handle.is_null() {
            return Err("Failed to open process".to_string());
        }

        let mut buffer = [0u16; 512];
        let result = GetModuleBaseNameW(
            process_handle,
            ptr::null_mut(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        );

        CloseHandle(process_handle);

        if result > 0 {
            let name = String::from_utf16_lossy(&buffer[..result as usize]);
            Ok(name)
        } else {
            Err("Failed to get process name".to_string())
        }
    }
}

// Helper function to get window rectangle (excludes drop shadow)
pub fn get_window_rect(window_handle: u64) -> Result<RECT, String> {
    unsafe {
        let hwnd = window_handle as HWND;

        // Try to get extended frame bounds (excludes drop shadow)
        // This requires dwmapi.dll
        use winapi::um::dwmapi::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};

        let mut rect: RECT = mem::zeroed();
        let result = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut _,
            mem::size_of::<RECT>() as u32,
        );

        if result == 0 {
            // Success - got extended frame bounds (without shadow)
            Ok(rect)
        } else {
            // Fallback to GetWindowRect if DWM call fails
            if GetWindowRect(hwnd, &mut rect) != 0 {
                Ok(rect)
            } else {
                Err("Failed to get window rectangle".to_string())
            }
        }
    }
}

// Helper function to check if a window is maximized or fullscreen
pub fn is_window_maximized(window_handle: u64) -> Result<bool, String> {
    unsafe {
        use winapi::um::winuser::IsZoomed;

        let hwnd = window_handle as HWND;
        let is_zoomed = IsZoomed(hwnd) != 0;

        // Also check if window fills its monitor (fullscreen check)
        let window_rect = get_window_rect(window_handle)?;
        let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

        let mut monitor_info: MONITORINFO = mem::zeroed();
        monitor_info.cbSize = mem::size_of::<MONITORINFO>() as u32;

        if GetMonitorInfoW(hmonitor, &mut monitor_info) != 0 {
            let monitor_rect = monitor_info.rcMonitor;

            // Check if window covers entire monitor (with some tolerance for taskbar/borders)
            let fills_monitor = (window_rect.left - monitor_rect.left).abs() <= 10
                && (window_rect.top - monitor_rect.top).abs() <= 10
                && (window_rect.right - monitor_rect.right).abs() <= 10
                && (window_rect.bottom - monitor_rect.bottom).abs() <= 10;

            Ok(is_zoomed || fills_monitor)
        } else {
            // If we can't get monitor info, just use IsZoomed
            Ok(is_zoomed)
        }
    }
}
