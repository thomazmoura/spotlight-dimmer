using Microsoft.Extensions.Logging;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Tracks the focused pane inside configured terminals so the spotlight can
/// shrink from the whole window to a single pane. Windows analog of the Linux
/// daemon's integration state machine (daemon/src/integrations/mod.rs).
///
/// Three providers are supported (config AppIntegrations[].Provider):
/// - "windows-terminal": the focused pane control is located via MSAA
///   accessibility. A per-process EVENT_OBJECT_FOCUS hook fires on pane
///   switches (which never change the foreground window), and the pane's
///   IAccessible is cached so the drag hot path costs one vtable call.
///   tmux running inside a WT tab is sub-resolved over the pane's rect.
/// - "wezterm": the focused pane is queried from the `wezterm cli` (same
///   strategy as Linux); tmux reports join deterministically via the
///   WEZTERM_PANE hint.
/// - "tmux": generic terminal running tmux full-window; the tmux cell grid
///   is mapped over the window's content area.
///
/// All state mutation happens on the main thread: WinEvent hooks are
/// registered there (OUTOFCONTEXT callbacks arrive via the message loop),
/// pipe reports are marshalled through WM_APP_PANE_REPORT, and async wezterm
/// results are marshalled through a private window message.
/// </summary>
internal class TerminalPaneTracker : IDisposable
{
    /// <summary>Debounce for title-change re-evaluation (mirrors the Linux daemon's TITLE_DEBOUNCE_MS).</summary>
    private const uint TitleDebounceMs = 150;

    private const nint TitleDebounceTimerId = 1;

    /// <summary>Private message: an async wezterm CLI query completed.</summary>
    private const uint WM_APP_WEZTERM_RESULT = WinApi.WM_TRAYICON + 3; // 0x8003

    /// <summary>Minimum plausible pane control size; smaller focus targets (buttons, tabs) are ignored.</summary>
    private const int MinPaneWidth = 80;
    private const int MinPaneHeight = 40;

    private readonly ILogger<TerminalPaneTracker> _logger;
    private readonly PaneTrackerPipeServer _pipeServer;
    private readonly WeztermCliClient _weztermClient;
    private readonly PaneTrackerState _paneState = new();

    // Must keep references to prevent garbage collection (same pattern as FocusTracker)
    private readonly WinApi.WinEventDelegate _hookDelegate;
    private readonly WinApi.WndProc _wndProcDelegate;

    private AppConfig _config;
    private AppIntegration? _matched;
    private string? _lastProcessName;
    private IntPtr _terminalHwnd;
    private uint _terminalPid;
    private uint _terminalThreadId;
    private IntPtr _focusHook;
    private IntPtr _nameHook;
    private IntPtr _messageWindow;
    private MsaaInterop.AccessibleTarget _focusedPane;
    private int _generation;
    private volatile WeztermResultBox? _pendingWezterm;
    private WeztermActivePane? _weztermPane;

    private sealed record WeztermResultBox(int Generation, WeztermActivePane? Pane);

    /// <summary>
    /// Fired on the main thread whenever the resolved pane state may have
    /// changed (new report, pane focus change, wezterm query completion, ...).
    /// The subscriber should re-run the overlay update.
    /// </summary>
    public event Action? PaneStateChanged;

    public TerminalPaneTracker(
        AppConfig initialConfig,
        PaneTrackerPipeServer pipeServer,
        WeztermCliClient weztermClient,
        ILogger<TerminalPaneTracker> logger)
    {
        _config = initialConfig;
        _pipeServer = pipeServer;
        _weztermClient = weztermClient;
        _logger = logger;
        _hookDelegate = OnWinEvent;
        _wndProcDelegate = MessageWindowProc;
    }

    /// <summary>
    /// Creates the message window and starts consuming pipe reports.
    /// Must be called on the main thread.
    /// </summary>
    public void Start()
    {
        if (_messageWindow != IntPtr.Zero)
            return;

        var wc = new WinApi.WNDCLASSEX
        {
            cbSize = System.Runtime.InteropServices.Marshal.SizeOf<WinApi.WNDCLASSEX>(),
            lpfnWndProc = System.Runtime.InteropServices.Marshal.GetFunctionPointerForDelegate(_wndProcDelegate),
            hInstance = WinApi.GetModuleHandle(null),
            lpszClassName = "SpotlightDimmer_PaneTracker_MessageWindow"
        };

        if (WinApi.RegisterClassEx(wc) == 0)
        {
            throw new InvalidOperationException("Failed to register pane tracker message window class");
        }

        _messageWindow = WinApi.CreateWindowEx(
            0,
            wc.lpszClassName,
            "SpotlightDimmer Pane Tracker Message Window",
            0,
            0, 0, 0, 0,
            WinApi.HWND_MESSAGE,
            IntPtr.Zero,
            wc.hInstance,
            IntPtr.Zero);

        if (_messageWindow == IntPtr.Zero)
        {
            throw new InvalidOperationException("Failed to create pane tracker message window");
        }

        _pipeServer.Start(_messageWindow);
        _logger.LogDebug("[PANE] Terminal pane tracker started");
    }

    /// <summary>
    /// Notifies the tracker that the foreground window changed. Re-matches the
    /// process against AppIntegrations and arms/disarms the per-process hooks.
    /// Must be called on the main thread.
    /// </summary>
    public void SetFocusedWindow(IntPtr hwnd, uint pid, string? processName)
    {
        _lastProcessName = processName;
        var match = _config.MatchIntegration(processName);

        if (match is null)
        {
            if (_matched is not null)
            {
                DisarmHooks();
                _matched = null;
                _weztermPane = null;
                _generation++;
                PaneStateChanged?.Invoke();
            }
            return;
        }

        var pidChanged = pid != _terminalPid || _focusHook == IntPtr.Zero && _nameHook == IntPtr.Zero;
        _matched = match;
        _terminalHwnd = hwnd;
        _terminalThreadId = WinApi.GetWindowThreadProcessId(hwnd, out _);

        if (pidChanged)
        {
            DisarmHooks();
            _terminalPid = pid;
            ArmHooks(match.Provider);
        }

        if (IsProvider(match, "windows-terminal"))
        {
            // Hooks may have been armed after the terminal's focus event fired
            // (app switch, startup): probe the currently focused control.
            RefreshFocusedPaneFromThread();
        }
        else if (IsProvider(match, "wezterm"))
        {
            SpawnWeztermQuery();
        }

        _logger.LogDebug("[PANE] Integration matched: {Process} provider={Provider}", processName, match.Provider);
        PaneStateChanged?.Invoke();
    }

    /// <summary>
    /// Applies a configuration change: re-matches the focused process and
    /// re-arms hooks as needed. Stored pane reports are kept (they are
    /// independent of configuration). Must be called on the main thread.
    /// </summary>
    public void OnConfigChanged(AppConfig config)
    {
        _config = config;
        SetFocusedWindow(_terminalHwnd, _terminalPid, _lastProcessName);
    }

    /// <summary>
    /// Resolves the inner (pane) rect for the focused window, or returns false
    /// to use the whole window. Called from the overlay update hot path: pure
    /// struct math over cached state plus at most one COM vtable call - no
    /// allocations.
    /// </summary>
    public bool TryResolveInnerRect(in Core.Rectangle windowFrame, out Core.Rectangle inner)
    {
        inner = default;
        var match = _matched;
        if (match is null)
            return false;

        if (IsProvider(match, "windows-terminal"))
            return TryResolveWindowsTerminal(match, windowFrame, out inner);

        if (IsProvider(match, "wezterm"))
            return TryResolveWezterm(match, windowFrame, out inner);

        // Generic "tmux" provider: cell grid over the window's content area
        return TryResolveCells(windowFrame, ShrinkByContentOffsets(windowFrame, match), null, out inner);
    }

    private bool TryResolveWindowsTerminal(AppIntegration match, in Core.Rectangle frame, out Core.Rectangle inner)
    {
        inner = default;

        if (MsaaInterop.TryGetLocation(_focusedPane, out var paneRect) &&
            paneRect.IntersectsWith(frame) &&
            paneRect.Width >= MinPaneWidth && paneRect.Height >= MinPaneHeight)
        {
            // tmux sub-resolution over the focused WT pane's content
            if (TryResolveCells(frame, ShrinkByContentOffsets(paneRect, match), null, out inner))
                return true;

            // Native WT pane spotlight
            var clamped = PaneGeometry.PaneRect(frame, 0, 0, 0, 0,
                new Core.Rectangle(paneRect.X - frame.X, paneRect.Y - frame.Y, paneRect.Width, paneRect.Height));
            if (clamped is not null)
            {
                inner = clamped.Value;
                return true;
            }

            return false;
        }

        // No usable pane control (accessibility failed): still allow tmux
        // sub-resolution over the whole window content (single-pane WT).
        return TryResolveCells(frame, ShrinkByContentOffsets(frame, match), null, out inner);
    }

    private bool TryResolveWezterm(AppIntegration match, in Core.Rectangle frame, out Core.Rectangle inner)
    {
        inner = default;

        if (_weztermPane is { } pane)
        {
            var hasPixelSize = pane.Width > 0 && pane.Height > 0;
            var content = hasPixelSize
                ? new Core.Rectangle(
                    frame.X + match.ContentOffsetX + pane.RelX,
                    frame.Y + match.ContentOffsetY + pane.RelY,
                    pane.Width,
                    pane.Height)
                : ShrinkByContentOffsets(frame, match);

            // tmux sub-resolution joined deterministically via WEZTERM_PANE
            if (TryResolveCells(frame, content, pane.PaneId, out inner))
                return true;

            // Native wezterm pane spotlight
            if (hasPixelSize)
            {
                var clamped = PaneGeometry.PaneRect(frame, 0, 0, 0, 0,
                    new Core.Rectangle(content.X - frame.X, content.Y - frame.Y, content.Width, content.Height));
                if (clamped is not null)
                {
                    inner = clamped.Value;
                    return true;
                }
            }

            return false;
        }

        // No CLI result (yet): allow tmux sub-resolution over the window content
        return TryResolveCells(frame, ShrinkByContentOffsets(frame, match), null, out inner);
    }

    private bool TryResolveCells(in Core.Rectangle frame, in Core.Rectangle content, string? weztermPaneHint, out Core.Rectangle inner)
    {
        inner = default;

        if (!_paneState.TrySelectReport(weztermPaneHint, out var report))
            return false;

        var rect = PaneGeometry.CellPaneRect(frame, content, report);
        if (rect is null)
            return false;

        inner = rect.Value;
        return true;
    }

    /// <summary>
    /// Shrinks a rect by the configured content offsets to approximate the
    /// terminal's cell grid bounding box: ContentOffsetX is applied to the
    /// left, right and bottom edges (uniform padding), ContentOffsetY to the
    /// top edge (padding plus tab bar). A few pixels of error is not visually
    /// noticeable in a dimming overlay, and results are always clamped to the
    /// window frame downstream.
    /// </summary>
    private static Core.Rectangle ShrinkByContentOffsets(in Core.Rectangle rect, AppIntegration match)
    {
        var left = rect.X + match.ContentOffsetX;
        var top = rect.Y + match.ContentOffsetY;
        var right = rect.Right - match.ContentOffsetX;
        var bottom = rect.Bottom - match.ContentOffsetX;

        if (right <= left || bottom <= top)
            return rect;

        return Core.Rectangle.FromLTRB(left, top, right, bottom);
    }

    private static bool IsProvider(AppIntegration match, string provider) =>
        string.Equals(match.Provider, provider, StringComparison.OrdinalIgnoreCase);

    // ====================================================================
    // Hooks and event handling
    // ====================================================================

    private void ArmHooks(string provider)
    {
        // EVENT_OBJECT_NAMECHANGE (title changes) re-evaluates the integration
        // for every provider: tmux sets the terminal title, so attach/detach
        // and pane switches show up as title changes.
        _nameHook = WinApi.SetWinEventHook(
            WinApi.EVENT_OBJECT_NAMECHANGE,
            WinApi.EVENT_OBJECT_NAMECHANGE,
            IntPtr.Zero,
            _hookDelegate,
            _terminalPid,
            0,
            WinApi.WINEVENT_OUTOFCONTEXT);

        // EVENT_OBJECT_FOCUS fires on pane switches within the terminal window
        // (which never change the foreground window). Only needed for the
        // accessibility-based Windows Terminal provider.
        if (string.Equals(provider, "windows-terminal", StringComparison.OrdinalIgnoreCase))
        {
            _focusHook = WinApi.SetWinEventHook(
                WinApi.EVENT_OBJECT_FOCUS,
                WinApi.EVENT_OBJECT_FOCUS,
                IntPtr.Zero,
                _hookDelegate,
                _terminalPid,
                0,
                WinApi.WINEVENT_OUTOFCONTEXT);
        }

        _logger.LogDebug("[PANE] Hooks armed for pid {Pid} (focus={HasFocusHook}, name={HasNameHook})",
            _terminalPid, _focusHook != IntPtr.Zero, _nameHook != IntPtr.Zero);
    }

    private void DisarmHooks()
    {
        if (_focusHook != IntPtr.Zero)
        {
            WinApi.UnhookWinEvent(_focusHook);
            _focusHook = IntPtr.Zero;
        }

        if (_nameHook != IntPtr.Zero)
        {
            WinApi.UnhookWinEvent(_nameHook);
            _nameHook = IntPtr.Zero;
        }

        if (_messageWindow != IntPtr.Zero)
        {
            WinApi.KillTimer(_messageWindow, TitleDebounceTimerId);
        }

        MsaaInterop.Release(ref _focusedPane);
        _terminalPid = 0;
    }

    private void OnWinEvent(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime)
    {
        try
        {
            if (eventType == WinApi.EVENT_OBJECT_FOCUS)
            {
                OnTerminalFocusEvent(hwnd, idObject, idChild);
            }
            else if (eventType == WinApi.EVENT_OBJECT_NAMECHANGE && idObject == WinApi.OBJID_WINDOW)
            {
                // Debounced re-evaluation; a burst of title changes (shell
                // startup, tmux status updates) coalesces into one requery.
                WinApi.SetTimer(_messageWindow, TitleDebounceTimerId, TitleDebounceMs, IntPtr.Zero);
            }
        }
        catch (Exception ex)
        {
            _logger.LogDebug("[PANE] WinEvent handler error: {Message}", ex.Message);
        }
    }

    private void OnTerminalFocusEvent(IntPtr hwnd, int idObject, int idChild)
    {
        if (!MsaaInterop.TryFromEvent(hwnd, idObject, idChild, out var target))
            return;

        if (MsaaInterop.TryGetLocation(in target, out var rect))
        {
            if (!IsPlausiblePaneRect(in rect))
            {
                // Small chrome element (tab header, button): keep the current pane.
                MsaaInterop.Release(ref target);
                return;
            }
        }
        else
        {
            // Freshly created panes (split-pane) report their location only a
            // moment later (observed DISP_E_MEMBERNOTFOUND right after a
            // split). Cache the target anyway - the fallback path covers the
            // gap - and re-probe after the debounce interval.
            WinApi.SetTimer(_messageWindow, TitleDebounceTimerId, TitleDebounceMs, IntPtr.Zero);
        }

        MsaaInterop.Release(ref _focusedPane);
        _focusedPane = target;
        _logger.LogDebug("[PANE] Focused pane control updated (hwnd={Hwnd:X}, idObject={IdObject})", hwnd, idObject);
        PaneStateChanged?.Invoke();
    }

    private void RefreshFocusedPaneFromThread()
    {
        if (_terminalThreadId == 0)
            return;

        if (MsaaInterop.TryFromFocusedWindow(_terminalThreadId, out var target))
        {
            if (IsPlausiblePane(in target))
            {
                MsaaInterop.Release(ref _focusedPane);
                _focusedPane = target;
                _logger.LogDebug("[PANE] Focused pane control probed from thread {ThreadId}", _terminalThreadId);
                return;
            }

            MsaaInterop.Release(ref target);
        }
    }

    /// <summary>
    /// Sanity-checks a focus target before caching it as "the pane": it must
    /// report a plausible on-screen rect inside the terminal window. Filters
    /// out tab headers, buttons and other small chrome elements.
    /// </summary>
    private bool IsPlausiblePane(in MsaaInterop.AccessibleTarget target)
    {
        return MsaaInterop.TryGetLocation(in target, out var rect) && IsPlausiblePaneRect(in rect);
    }

    private bool IsPlausiblePaneRect(in Core.Rectangle rect)
    {
        if (rect.Width < MinPaneWidth || rect.Height < MinPaneHeight)
            return false;

        if (_terminalHwnd != IntPtr.Zero &&
            WinApi.GetExtendedWindowRect(_terminalHwnd, out var windowRect))
        {
            return rect.IntersectsWith(WinApi.ToRectangle(windowRect));
        }

        return true;
    }

    private void SpawnWeztermQuery()
    {
        var generation = ++_generation;
        _ = QueryWeztermAsync(generation);
    }

    private async Task QueryWeztermAsync(int generation)
    {
        try
        {
            var pane = await _weztermClient.QueryActivePaneAsync().ConfigureAwait(false);
            _pendingWezterm = new WeztermResultBox(generation, pane);
            WinApi.PostMessage(_messageWindow, WM_APP_WEZTERM_RESULT, IntPtr.Zero, IntPtr.Zero);
        }
        catch (Exception ex)
        {
            _logger.LogDebug("[PANE] wezterm query failed: {Message}", ex.Message);
        }
    }

    private IntPtr MessageWindowProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        if (msg == WinApi.WM_APP_PANE_REPORT)
        {
            var changed = false;
            var now = Environment.TickCount64;
            while (_pipeServer.TryDequeue(out var report))
            {
                changed |= _paneState.Apply(report, now);
            }

            if (changed && _matched is not null)
            {
                PaneStateChanged?.Invoke();
            }

            return IntPtr.Zero;
        }

        if (msg == WinApi.WM_TIMER && wParam == TitleDebounceTimerId)
        {
            WinApi.KillTimer(hWnd, TitleDebounceTimerId);
            OnTitleChangedDebounced();
            return IntPtr.Zero;
        }

        if (msg == WM_APP_WEZTERM_RESULT)
        {
            var pending = _pendingWezterm;
            if (pending is not null && pending.Generation == _generation)
            {
                _weztermPane = pending.Pane;
                _logger.LogDebug("[PANE] wezterm focused pane resolved: {PaneId}", pending.Pane?.PaneId ?? "none");
                PaneStateChanged?.Invoke();
            }

            return IntPtr.Zero;
        }

        return WinApi.DefWindowProc(hWnd, msg, wParam, lParam);
    }

    private void OnTitleChangedDebounced()
    {
        var match = _matched;
        if (match is null)
            return;

        if (IsProvider(match, "wezterm"))
        {
            SpawnWeztermQuery();
            return;
        }

        if (IsProvider(match, "windows-terminal"))
        {
            // Pane layout may have changed without a focus event (zoom, close);
            // re-probe the focused control so the cached rect stays honest.
            RefreshFocusedPaneFromThread();
        }

        PaneStateChanged?.Invoke();
    }

    public void Dispose()
    {
        DisarmHooks();

        if (_messageWindow != IntPtr.Zero)
        {
            WinApi.DestroyWindow(_messageWindow);
            _messageWindow = IntPtr.Zero;
        }

        _logger.LogDebug("[PANE] Terminal pane tracker stopped");
    }
}
