using System.Runtime.InteropServices;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Manages the system tray icon with pause/resume and quit functionality.
/// Survives explorer.exe restarts.
/// </summary>
internal class SystemTrayManager : IDisposable
{
    private const string WINDOW_CLASS_NAME = "SpotlightDimmerTrayWindow";
    private const uint TRAY_ICON_ID = 1;

    // Menu item IDs
    private const uint MENU_PAUSE_RESUME = 1001;
    private const uint MENU_AUTOSTART = 1002;
    private const uint MENU_QUIT = 1003;
    private const uint MENU_OPEN_CONFIG = 1004;

    // Profile menu IDs (2000-2999)
    private const uint MENU_PROFILE_START = 2000;
    private const uint MENU_PROFILE_END = 2999;

    private static readonly WinApi.WndProc _wndProcDelegate = WndProc;
    private static SystemTrayManager? _instance;

    private IntPtr _hwnd = IntPtr.Zero;
    private IntPtr _activeIcon = IntPtr.Zero;
    private IntPtr _pausedIcon = IntPtr.Zero;
    private bool _isPaused = false;
    private string _activeIconPath;
    private string _pausedIconPath;
    private AppConfig _currentConfig;

    // Events
    public event Action<bool>? PauseStateChanged;
    public event Action? QuitRequested;
    public event Action<string>? ProfileSelected;
    public event Action? OpenConfigRequested;

    public bool IsPaused => _isPaused;

    public SystemTrayManager(string activeIconPath, string pausedIconPath, AppConfig config)
    {
        _activeIconPath = Path.GetFullPath(activeIconPath);
        _pausedIconPath = Path.GetFullPath(pausedIconPath);
        _currentConfig = config;
        _instance = this;

        RegisterWindowClass();
        CreateMessageWindow();
        LoadIcons();
        AddTrayIcon();
    }

    /// <summary>
    /// Updates the current configuration (used to refresh profile list state).
    /// </summary>
    public void UpdateConfig(AppConfig config)
    {
        _currentConfig = config;
    }

    /// <summary>
    /// Toggles pause/resume state.
    /// </summary>
    public void TogglePause()
    {
        _isPaused = !_isPaused;
        UpdateTrayIcon();
        PauseStateChanged?.Invoke(_isPaused);
    }

    /// <summary>
    /// Sets the pause state explicitly.
    /// </summary>
    public void SetPauseState(bool paused)
    {
        if (_isPaused != paused)
        {
            _isPaused = paused;
            UpdateTrayIcon();
            PauseStateChanged?.Invoke(_isPaused);
        }
    }

    /// <summary>
    /// Toggles auto-start at login.
    /// </summary>
    private void ToggleAutoStart()
    {
        bool isEnabled = AutoStartManager.IsEnabled();
        bool success = isEnabled ? AutoStartManager.Disable() : AutoStartManager.Enable();

        if (success)
        {
            Console.WriteLine($"[System Tray] Auto-start {(isEnabled ? "disabled" : "enabled")}");
        }
        else
        {
            Console.WriteLine("[System Tray] Failed to toggle auto-start");
        }
    }

    private void RegisterWindowClass()
    {
        var wndClass = new WinApi.WNDCLASSEX
        {
            lpfnWndProc = Marshal.GetFunctionPointerForDelegate(_wndProcDelegate),
            hInstance = WinApi.GetModuleHandle(null),
            lpszClassName = WINDOW_CLASS_NAME
        };

        var atom = WinApi.RegisterClassEx(wndClass);
        if (atom == 0)
        {
            throw new InvalidOperationException($"Failed to register tray window class. Error: {Marshal.GetLastWin32Error()}");
        }
    }

    private void CreateMessageWindow()
    {
        _hwnd = WinApi.CreateWindowEx(
            0,
            WINDOW_CLASS_NAME,
            "SpotlightDimmer Tray",
            0,
            0, 0, 0, 0,
            IntPtr.Zero,
            IntPtr.Zero,
            WinApi.GetModuleHandle(null),
            IntPtr.Zero);

        if (_hwnd == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to create tray message window. Error: {Marshal.GetLastWin32Error()}");
        }
    }

    private void LoadIcons()
    {
        // Load active icon
        _activeIcon = WinApi.LoadImage(
            IntPtr.Zero,
            _activeIconPath,
            WinApi.IMAGE_ICON,
            0, 0,
            WinApi.LR_LOADFROMFILE);

        if (_activeIcon == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to load active icon from {_activeIconPath}");
        }

        // Load paused icon
        _pausedIcon = WinApi.LoadImage(
            IntPtr.Zero,
            _pausedIconPath,
            WinApi.IMAGE_ICON,
            0, 0,
            WinApi.LR_LOADFROMFILE);

        if (_pausedIcon == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to load paused icon from {_pausedIconPath}");
        }
    }

    private void AddTrayIcon()
    {
        var nid = new WinApi.NOTIFYICONDATA
        {
            cbSize = Marshal.SizeOf<WinApi.NOTIFYICONDATA>(),
            hWnd = _hwnd,
            uID = TRAY_ICON_ID,
            uFlags = WinApi.NIF_ICON | WinApi.NIF_MESSAGE | WinApi.NIF_TIP,
            uCallbackMessage = WinApi.WM_TRAYICON,
            hIcon = _activeIcon,
            szTip = "SpotlightDimmer - Active"
        };

        if (!WinApi.Shell_NotifyIcon(WinApi.NIM_ADD, ref nid))
        {
            throw new InvalidOperationException("Failed to add tray icon");
        }

        // Set to version 4 for modern behavior
        nid.uVersion = WinApi.NOTIFYICON_VERSION_4;
        WinApi.Shell_NotifyIcon(WinApi.NIM_SETVERSION, ref nid);
    }

    private void UpdateTrayIcon()
    {
        var nid = new WinApi.NOTIFYICONDATA
        {
            cbSize = Marshal.SizeOf<WinApi.NOTIFYICONDATA>(),
            hWnd = _hwnd,
            uID = TRAY_ICON_ID,
            uFlags = WinApi.NIF_ICON | WinApi.NIF_TIP,
            hIcon = _isPaused ? _pausedIcon : _activeIcon,
            szTip = _isPaused ? "SpotlightDimmer - Paused" : "SpotlightDimmer - Active"
        };

        WinApi.Shell_NotifyIcon(WinApi.NIM_MODIFY, ref nid);
    }

    private void RemoveTrayIcon()
    {
        var nid = new WinApi.NOTIFYICONDATA
        {
            cbSize = Marshal.SizeOf<WinApi.NOTIFYICONDATA>(),
            hWnd = _hwnd,
            uID = TRAY_ICON_ID
        };

        WinApi.Shell_NotifyIcon(WinApi.NIM_DELETE, ref nid);
    }

    private void ShowContextMenu()
    {
        // Try to get the tray icon position
        var identifier = new WinApi.NOTIFYICONIDENTIFIER
        {
            cbSize = Marshal.SizeOf<WinApi.NOTIFYICONIDENTIFIER>(),
            hWnd = _hwnd,
            uID = TRAY_ICON_ID,
            guidItem = Guid.Empty
        };

        WinApi.POINT pt;
        int result = WinApi.Shell_NotifyIconGetRect(ref identifier, out var iconRect);

        if (result == 0) // S_OK
        {
            // Successfully got icon position - use center-bottom of icon
            pt.X = iconRect.Left + (iconRect.Right - iconRect.Left) / 2;
            pt.Y = iconRect.Bottom;
        }
        else
        {
            // Fallback: use current cursor position
            WinApi.GetCursorPos(out pt);
        }

        // Create popup menu
        var hMenu = WinApi.CreatePopupMenu();
        if (hMenu == IntPtr.Zero)
            return;

        try
        {
            // Add menu items
            string pauseResumeText = _isPaused ? "Resume" : "Pause";
            WinApi.AppendMenu(hMenu, WinApi.MF_STRING, MENU_PAUSE_RESUME, pauseResumeText);
            WinApi.AppendMenu(hMenu, WinApi.MF_SEPARATOR, 0, string.Empty);

            // Add Profiles submenu
            var hProfilesMenu = WinApi.CreatePopupMenu();
            if (hProfilesMenu != IntPtr.Zero)
            {
                // Add profile items
                for (int i = 0; i < _currentConfig.Profiles.Count; i++)
                {
                    var profile = _currentConfig.Profiles[i];
                    uint profileId = MENU_PROFILE_START + (uint)i;

                    // Check if this is the current profile
                    bool isCurrentProfile = profile.Name == _currentConfig.CurrentProfile;

                    // Check if current profile has been modified
                    bool isModified = isCurrentProfile && !_currentConfig.DoesOverlayMatchProfile(profile.Name);

                    // Build menu text
                    string profileText = profile.Name;
                    if (isModified)
                        profileText += " *";

                    // Add checkbox if this is the current profile
                    uint profileFlags = WinApi.MF_STRING | (isCurrentProfile ? WinApi.MF_CHECKED : WinApi.MF_UNCHECKED);
                    WinApi.AppendMenu(hProfilesMenu, profileFlags, profileId, profileText);
                }

                // Add Profiles submenu to main menu
                WinApi.AppendMenu(hMenu, WinApi.MF_POPUP, (uint)hProfilesMenu, "Profiles");
                WinApi.AppendMenu(hMenu, WinApi.MF_SEPARATOR, 0, string.Empty);
            }

            // Add "Start at Login" menu item with checkbox
            bool isAutoStartEnabled = AutoStartManager.IsEnabled();
            uint autoStartFlags = WinApi.MF_STRING | (isAutoStartEnabled ? WinApi.MF_CHECKED : WinApi.MF_UNCHECKED);
            WinApi.AppendMenu(hMenu, autoStartFlags, MENU_AUTOSTART, "Start at Login");

            // Add "Open Config File" menu item
            WinApi.AppendMenu(hMenu, WinApi.MF_STRING, MENU_OPEN_CONFIG, "Open Config File");

            WinApi.AppendMenu(hMenu, WinApi.MF_SEPARATOR, 0, string.Empty);
            WinApi.AppendMenu(hMenu, WinApi.MF_STRING, MENU_QUIT, "Quit");

            // Required for proper menu behavior
            WinApi.SetForegroundWindow(_hwnd);

            // Show menu and get selected item
            var cmd = WinApi.TrackPopupMenu(
                hMenu,
                WinApi.TPM_BOTTOMALIGN | WinApi.TPM_LEFTALIGN | WinApi.TPM_RETURNCMD,
                pt.X,
                pt.Y,
                0,
                _hwnd,
                IntPtr.Zero);

            // Handle menu selection
            if (cmd == MENU_PAUSE_RESUME)
            {
                TogglePause();
            }
            else if (cmd == MENU_AUTOSTART)
            {
                ToggleAutoStart();
            }
            else if (cmd == MENU_OPEN_CONFIG)
            {
                OpenConfigRequested?.Invoke();
            }
            else if (cmd >= MENU_PROFILE_START && cmd <= MENU_PROFILE_END)
            {
                // Handle profile selection
                int profileIndex = (int)(cmd - MENU_PROFILE_START);
                if (profileIndex >= 0 && profileIndex < _currentConfig.Profiles.Count)
                {
                    string profileName = _currentConfig.Profiles[profileIndex].Name;
                    ProfileSelected?.Invoke(profileName);
                }
            }
            else if (cmd == MENU_QUIT)
            {
                QuitRequested?.Invoke();
            }
        }
        finally
        {
            WinApi.DestroyMenu(hMenu);
        }
    }

    private static IntPtr WndProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        if (_instance == null)
            return WinApi.DefWindowProc(hWnd, msg, wParam, lParam);

        // Handle TaskbarCreated message (explorer.exe restart)
        if (msg == WinApi.WM_TASKBARCREATED)
        {
            // Re-add the tray icon
            _instance.AddTrayIcon();
            return IntPtr.Zero;
        }

        if (msg == WinApi.WM_TRAYICON)
        {
            uint mouseMsg = (uint)(lParam.ToInt64() & 0xFFFF);

            switch (mouseMsg)
            {
                case WinApi.WM_LBUTTONDBLCLK:
                    // Double-click toggles pause/resume
                    _instance.TogglePause();
                    break;

                case WinApi.WM_RBUTTONUP:
                    // Right-click shows context menu
                    _instance.ShowContextMenu();
                    break;

                case WinApi.NIN_SELECT:
                    // This is sent for mouse selection (single click) - ignore it
                    // We handle double-click separately
                    break;

                case WinApi.NIN_KEYSELECT:
                    // Keyboard activation - check which key was pressed
                    // GetAsyncKeyState returns:
                    //   High-order bit (0x8000): Key is currently down
                    //   Low-order bit (0x0001): Key was pressed after previous call
                    // We check BOTH bits because the key might be released quickly

                    short enterState = WinApi.GetAsyncKeyState(WinApi.VK_RETURN);
                    short spaceState = WinApi.GetAsyncKeyState(WinApi.VK_SPACE);

                    // Check if Enter key is/was pressed (check both bits)
                    if ((enterState & 0x8001) != 0)
                    {
                        // Enter key - open context menu
                        _instance.ShowContextMenu();
                    }
                    // Check if Space key is/was pressed (check both bits)
                    else if ((spaceState & 0x8001) != 0)
                    {
                        // Space key - toggle pause/resume
                        _instance.TogglePause();
                    }
                    break;

                case WinApi.WM_CONTEXTMENU:
                    // Context menu key (Apps key or Shift+F10) shows context menu
                    _instance.ShowContextMenu();
                    break;
            }

            return IntPtr.Zero;
        }

        return WinApi.DefWindowProc(hWnd, msg, wParam, lParam);
    }

    public void Dispose()
    {
        RemoveTrayIcon();

        if (_activeIcon != IntPtr.Zero)
        {
            WinApi.DestroyIcon(_activeIcon);
            _activeIcon = IntPtr.Zero;
        }

        if (_pausedIcon != IntPtr.Zero)
        {
            WinApi.DestroyIcon(_pausedIcon);
            _pausedIcon = IntPtr.Zero;
        }

        if (_hwnd != IntPtr.Zero)
        {
            WinApi.DestroyWindow(_hwnd);
            _hwnd = IntPtr.Zero;
        }

        _instance = null;
    }
}
