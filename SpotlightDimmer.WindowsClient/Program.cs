using System.Reflection;
using Microsoft.Extensions.Logging;
using SpotlightDimmer.Core;
using SpotlightDimmer.WindowsBindings;

// ========================================================================
// Logging Initialization
// ========================================================================

// Initialize file-based logging with default settings first
// This ensures we capture ALL log messages, including ConfigurationManager's initial load
var loggerFactory = LoggingConfiguration.Initialize(AppConfig.Default);
var logger = loggerFactory.CreateLogger<Program>();

// Get application version for schema URL generation
var appVersion = Assembly.GetExecutingAssembly()
    .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?
    .InformationalVersion ?? "0.0.0";

logger.LogInformation("SpotlightDimmer v{Version} starting...", appVersion);
logger.LogInformation("Logs directory: {LogsDirectory}", LoggingConfiguration.GetLogsDirectory());

// Configuration: Load from file with hot-reload support
// Now ConfigurationManager can log properly during initialization
var configManager = new ConfigurationManager(LoggingConfiguration.GetLogger<ConfigurationManager>(), appVersion);

// Reconfigure logging based on actual loaded settings (if different from defaults)
LoggingConfiguration.Reconfigure(configManager.Current);
logger.LogInformation("Logging configured: Level={LogLevel}, Enabled={Enabled}",
    configManager.Current.System.LogLevel, configManager.Current.System.EnableLogging);

// Create loggers for all components
var monitorManagerLogger = LoggingConfiguration.GetLogger<MonitorManager>();
var focusTrackerLogger = LoggingConfiguration.GetLogger<FocusTracker>();
var displayChangeMonitorLogger = LoggingConfiguration.GetLogger<DisplayChangeMonitor>();
var autoStartManagerLogger = LoggingConfiguration.GetLogger<AutoStartManager>();

// ========================================================================
// Core Layer - Pure calculation logic
// ========================================================================

var config = configManager.Current.ToOverlayConfig();

// Set up graceful shutdown
using var cts = new CancellationTokenSource();
var mainThreadId = WinApi.GetCurrentThreadId();

// ========================================================================
// WindowsBindings Layer - Platform-specific components
// ========================================================================

// Create AutoStartManager instance first (needed by system tray)
var autoStartManager = new AutoStartManager(autoStartManagerLogger);

// System Tray
var systemTray = new SystemTrayManager(
    Path.Combine(AppContext.BaseDirectory, "spotlight-dimmer-icon.ico"),
    Path.Combine(AppContext.BaseDirectory, "spotlight-dimmer-icon-paused.ico"),
    configManager.Current,
    autoStartManager);

var monitorManager = new MonitorManager(monitorManagerLogger);

if (monitorManager.Monitors.Count == 0)
{
    logger.LogError("No monitors detected. Exiting");
    return 1;
}

logger.LogInformation("Detected {Count} monitor(s)", monitorManager.Monitors.Count);

// Create renderer based on configuration
var renderer = CreateRenderer(configManager.Current.System.RendererBackend, logger);
var displayChangeMonitor = new DisplayChangeMonitor(displayChangeMonitorLogger);
logger.LogInformation("Display change monitor initialized");

// Create app state with pre-allocated overlay definitions
var displays = monitorManager.GetDisplayInfo();
var appState = new AppState(displays);

// Pre-create all overlay windows (6 per display, all initially hidden)
// Pre-allocate brushes for configured colors - zero allocations during updates
renderer.CreateOverlays(displays, config);

// Apply screen capture exclusion setting if configured (experimental feature)
if (configManager.Current.Overlay.ExcludeFromScreenCapture)
{
    var successCount = renderer.UpdateScreenCaptureExclusion(true);
    var totalWindows = displays.Length * 6; // 6 overlays per display
    if (successCount < totalWindows)
    {
        logger.LogWarning("Screen capture exclusion is EXPERIMENTAL and may not work on all systems");
        logger.LogWarning("Successfully applied to {SuccessCount}/{TotalCount} overlay windows", successCount, totalWindows);
    }
    else
    {
        logger.LogInformation("Screen capture exclusion enabled for all overlay windows");
    }
}

logger.LogInformation("Overlay configuration loaded");
logger.LogInformation("  Config file: {ConfigPath}", ConfigurationManager.GetDefaultConfigPath());
logger.LogInformation("  Mode: {Mode}", config.Mode);
logger.LogInformation("  Inactive: #{R:X2}{G:X2}{B:X2} @ {Opacity}/255 opacity",
    config.InactiveColor.R, config.InactiveColor.G, config.InactiveColor.B, config.InactiveOpacity);
logger.LogInformation("  Active: #{R:X2}{G:X2}{B:X2} @ {Opacity}/255 opacity",
    config.ActiveColor.R, config.ActiveColor.G, config.ActiveColor.B, config.ActiveOpacity);

// ========================================================================
// Wire Core and WindowsBindings together
// ========================================================================

// Cache displays and config to avoid allocations in hot path
// These are only updated when monitors change or config changes
var cachedDisplays = monitorManager.GetDisplayInfo();
var cachedConfig = configManager.Current.ToOverlayConfig();

// Track display count for detecting layout changes
// Windows fires WM_DISPLAYCHANGE before displays are actually reconfigured,
// so we need to retry checking the count at intervals
var currentDisplayCount = cachedDisplays.Length;

// Helper function to update overlays (zero allocations - uses cached values)
void UpdateOverlays(int displayIndex, Rectangle windowBounds)
{
    // Don't update overlays if paused
    if (systemTray.IsPaused)
        return;

    appState.Calculate(cachedDisplays, windowBounds, displayIndex, cachedConfig);
    renderer.UpdateOverlays(appState.DisplayStates);
}

// Create overlay update service wrapper
var overlayUpdateService = new OverlayUpdateServiceWrapper(UpdateOverlays);

// Create focus change handler with the update service
var focusChangeHandler = new FocusChangeHandler(overlayUpdateService);

// Create focus tracker with the handler
var focusTracker = new FocusTracker(monitorManager, focusChangeHandler, focusTrackerLogger);

// ========================================================================
// System Tray Event Handlers
// ========================================================================

// Handle pause/resume from system tray
systemTray.PauseStateChanged += (isPaused) =>
{
    if (isPaused)
    {
        logger.LogInformation("Paused - overlays hidden");
        renderer.HideAllOverlays();
    }
    else
    {
        logger.LogInformation("Resumed - overlays active");
        // Trigger an update to show overlays again
        if (focusTracker.HasFocus && focusTracker.CurrentWindowRect.HasValue)
        {
            UpdateOverlays(focusTracker.CurrentFocusedDisplayIndex, focusTracker.CurrentWindowRect.Value);
        }
    }
};

// Handle quit from system tray
systemTray.QuitRequested += () =>
{
    logger.LogInformation("Quit requested from system tray");
    cts.Cancel();
    WinApi.PostThreadMessage(mainThreadId, WinApi.WM_QUIT, IntPtr.Zero, IntPtr.Zero);
};

// Handle profile selection from system tray
systemTray.ProfileSelected += (profileName) =>
{
    logger.LogInformation("Applying profile: {ProfileName}", profileName);

    // Get current config
    var currentConfig = configManager.Current;

    // Apply the profile
    if (currentConfig.ApplyProfile(profileName))
    {
        // Save the updated configuration
        configManager.SaveConfiguration(currentConfig);

        // Update cached config
        cachedConfig = currentConfig.ToOverlayConfig();

        // Update brush colors for all windows
        renderer.UpdateBrushColors(cachedConfig);

        // Update system tray with new config
        systemTray.UpdateConfig(currentConfig);

        // Trigger overlay update if we have a focused window
        if (focusTracker.HasFocus && focusTracker.CurrentWindowRect.HasValue)
        {
            UpdateOverlays(focusTracker.CurrentFocusedDisplayIndex, focusTracker.CurrentWindowRect.Value);
        }

        logger.LogInformation("Profile '{ProfileName}' applied successfully", profileName);
    }
    else
    {
        logger.LogError("Failed to apply profile: {ProfileName}", profileName);
    }
};

// Handle open config app request from system tray
systemTray.OpenConfigAppRequested += () =>
{
    try
    {
        var appDirectory = AppContext.BaseDirectory;
        var configAppPath = Path.Combine(appDirectory, "SpotlightDimmer.Config.exe");

        logger.LogInformation("Launching config app: {ConfigAppPath}", configAppPath);

        var processStartInfo = new System.Diagnostics.ProcessStartInfo
        {
            FileName = configAppPath,
            UseShellExecute = true,
            WorkingDirectory = appDirectory
        };

        System.Diagnostics.Process.Start(processStartInfo);
    }
    catch (Exception ex)
    {
        logger.LogError(ex, "Failed to launch config app");
    }
};

// Handle open config file request from system tray
systemTray.OpenConfigFileRequested += () =>
{
    try
    {
        var configPath = ConfigurationManager.GetDefaultConfigPath();
        logger.LogInformation("Opening config file: {ConfigPath}", configPath);

        var processStartInfo = new System.Diagnostics.ProcessStartInfo
        {
            FileName = configPath,
            UseShellExecute = true
        };

        System.Diagnostics.Process.Start(processStartInfo);
    }
    catch (Exception ex)
    {
        logger.LogError(ex, "Failed to open config file");
    }
};

// Handle view logs folder request from system tray
systemTray.ViewLogsFolderRequested += () =>
{
    try
    {
        var logsDir = LoggingConfiguration.GetLogsDirectory();
        logger.LogInformation("Opening logs folder: {LogsDirectory}", logsDir);

        // Create directory if it doesn't exist
        if (!Directory.Exists(logsDir))
        {
            Directory.CreateDirectory(logsDir);
        }

        var processStartInfo = new System.Diagnostics.ProcessStartInfo
        {
            FileName = logsDir,
            UseShellExecute = true
        };

        System.Diagnostics.Process.Start(processStartInfo);
    }
    catch (Exception ex)
    {
        logger.LogError(ex, "Failed to open logs folder");
    }
};

// Handle view latest log request from system tray
systemTray.ViewLatestLogRequested += () =>
{
    try
    {
        // Check if logging is disabled and no logs exist
        if (!configManager.Current.System.EnableLogging && !LoggingConfiguration.DoLogFilesExist())
        {
            logger.LogWarning("View latest log requested but logging is disabled and no logs exist");
            WinApi.MessageBox(IntPtr.Zero,
                "Logging is currently disabled and no log files exist.\n\nEnable logging from the Diagnostics menu to start logging.",
                "No Logs Available",
                WinApi.MB_OK | WinApi.MB_ICONINFORMATION);
            return;
        }

        var latestLogFile = LoggingConfiguration.GetMostRecentLogFile();
        if (latestLogFile == null)
        {
            logger.LogWarning("No log files found");
            WinApi.MessageBox(IntPtr.Zero,
                "No log files found.",
                "No Logs Available",
                WinApi.MB_OK | WinApi.MB_ICONINFORMATION);
            return;
        }

        logger.LogInformation("Opening latest log file: {LogFile}", latestLogFile);

        var processStartInfo = new System.Diagnostics.ProcessStartInfo
        {
            FileName = latestLogFile,
            UseShellExecute = true
        };

        System.Diagnostics.Process.Start(processStartInfo);
    }
    catch (Exception ex)
    {
        logger.LogError(ex, "Failed to open latest log file");
    }
};

// Handle logging toggle from system tray
systemTray.LoggingToggled += (enableLogging) =>
{
    logger.LogInformation("Logging {State} via system tray", enableLogging ? "enabled" : "disabled");

    // Get current config
    var currentConfig = configManager.Current;
    currentConfig.System.EnableLogging = enableLogging;

    // Save the updated configuration
    configManager.SaveConfiguration(currentConfig);

    // Logging reconfiguration will happen automatically via ConfigurationChanged event
};

// Handle About request from system tray
systemTray.AboutRequested += () =>
{
    try
    {
        var assembly = typeof(Program).Assembly;
        var version = assembly.GetName().Version?.ToString() ?? "Unknown";

        var aboutMessage = $"SpotlightDimmer\n" +
                          $"Version {version}\n\n" +
                          $"Author: Thomaz Moura\n\n" +
                          $"Technology:\n" +
                          $".NET, WinForms and Windows APIs\n\n" +
                          $"GitHub:\n" +
                          $"github.com/thomazmoura/spotlight-dimmer";

        logger.LogInformation("Showing About dialog");
        WinApi.MessageBox(IntPtr.Zero,
            aboutMessage,
            "About SpotlightDimmer",
            WinApi.MB_OK | WinApi.MB_ICONINFORMATION);
    }
    catch (Exception ex)
    {
        logger.LogError(ex, "Failed to show About dialog");
    }
};

// Handle Visit GitHub request from system tray
systemTray.VisitGithubRequested += () =>
{
    try
    {
        var githubUrl = "https://github.com/thomazmoura/spotlight-dimmer";
        logger.LogInformation("Opening GitHub page: {Url}", githubUrl);

        WinApi.ShellExecute(IntPtr.Zero, "open", githubUrl, null, null, WinApi.SW_SHOW);
    }
    catch (Exception ex)
    {
        logger.LogError(ex, "Failed to open GitHub page");
    }
};

// ========================================================================
// Configuration and Focus Event Handlers
// ========================================================================

// Handle configuration changes - recalculate and render overlays
configManager.ConfigurationChanged += (newAppConfig) =>
{
    // Update cached config when configuration changes
    cachedConfig = newAppConfig.ToOverlayConfig();

    // Reconfigure logging if logging settings changed
    LoggingConfiguration.Reconfigure(newAppConfig);

    // Update system tray with new config (refreshes profile list)
    systemTray.UpdateConfig(newAppConfig);

    // Update brush colors for all windows
    renderer.UpdateBrushColors(cachedConfig);

    // Update screen capture exclusion setting (experimental feature)
    var successCount = renderer.UpdateScreenCaptureExclusion(newAppConfig.Overlay.ExcludeFromScreenCapture);
    var totalWindows = cachedDisplays.Length * 6; // 6 overlays per display
    if (newAppConfig.Overlay.ExcludeFromScreenCapture)
    {
        if (successCount < totalWindows)
        {
            logger.LogWarning("Screen capture exclusion enabled but only applied to {SuccessCount}/{TotalCount} windows (EXPERIMENTAL feature)", successCount, totalWindows);
        }
        else
        {
            logger.LogInformation("Screen capture exclusion enabled for all overlay windows");
        }
    }
    else
    {
        logger.LogInformation("Screen capture exclusion disabled");
    }

    // If we have a focused window, trigger an update with the new config
    if (focusTracker.HasFocus && focusTracker.CurrentWindowRect.HasValue)
    {
        UpdateOverlays(focusTracker.CurrentFocusedDisplayIndex, focusTracker.CurrentWindowRect.Value);
    }

    logger.LogInformation("Configuration reloaded - overlays updated");
};

// Handle display changes - recalculate and render overlays
focusTracker.FocusedDisplayChanged += (displayIndex, windowBounds) =>
{
    logger.LogDebug("Display {DisplayIndex} gained focus", displayIndex);
    UpdateOverlays(displayIndex, windowBounds);
};

// Handle window position/size changes - recalculate and render overlays
focusTracker.WindowPositionChanged += (displayIndex, windowBounds) =>
{
    // This event fires frequently during window movement
    // In FullScreen mode, we don't need to update (only display changes matter)
    // In Partial/PartialWithActive modes, we need to update on every movement
    // CRITICAL: Use cachedConfig to avoid allocating on every event!
    if (cachedConfig.Mode != DimmingMode.FullScreen)
    {
        UpdateOverlays(displayIndex, windowBounds);
    }
};

// Handle display configuration changes - event-driven with immediate + 2s safety check
displayChangeMonitor.CheckDisplaysRequested += () =>
{
    logger.LogInformation("Display change detected - resetting displays and overlays");

    // Refresh monitor list
    monitorManager.RefreshMonitors();
    var newDisplayCount = monitorManager.Monitors.Count;

    // Update tracked count
    var oldDisplayCount = currentDisplayCount;
    currentDisplayCount = newDisplayCount;

    // Check if we still have monitors
    if (newDisplayCount == 0)
    {
        logger.LogWarning("No monitors detected after display change");
        renderer.CleanupOverlays();
        return;
    }

    // Clean up old overlays
    renderer.CleanupOverlays();

    // Get updated display information
    var newDisplays = monitorManager.GetDisplayInfo();
    cachedDisplays = newDisplays;

    // Recreate app state with new display configuration
    appState = new AppState(newDisplays);

    // Recreate all overlays with current configuration
    renderer.CreateOverlays(newDisplays, cachedConfig);

    // Reapply screen capture exclusion if configured
    if (configManager.Current.Overlay.ExcludeFromScreenCapture)
    {
        renderer.UpdateScreenCaptureExclusion(true);
    }

    logger.LogInformation("Overlays recreated: {OldCount} -> {NewCount} display(s)", oldDisplayCount, newDisplays.Length);

    // Trigger an overlay update if we have a focused window
    if (focusTracker.HasFocus && focusTracker.CurrentWindowRect.HasValue)
    {
        UpdateOverlays(focusTracker.CurrentFocusedDisplayIndex, focusTracker.CurrentWindowRect.Value);
    }
};

// Start tracking
focusTracker.Start();

logger.LogInformation("SpotlightDimmer is running");
logger.LogInformation("Mode: {Mode}", config.Mode);
logger.LogInformation("System tray: Double-click to pause/resume, right-click for menu");

// GDI object monitoring for leak detection (debug mode only)
var initialGdiCount = 0;
var lastGdiCount = 0;
var gdiCheckTimer = System.Diagnostics.Stopwatch.StartNew();
if (configManager.Current.System.LogLevel == "Debug")
{
    var currentProcess = System.Diagnostics.Process.GetCurrentProcess();
    initialGdiCount = WinApi.GetGuiResources(currentProcess.Handle, WinApi.GR_GDIOBJECTS);
    lastGdiCount = initialGdiCount;
    logger.LogDebug("Initial GDI objects: {Count}", initialGdiCount);
}

// ========================================================================
// Windows Message Loop
// ========================================================================

try
{
    while (!cts.Token.IsCancellationRequested)
    {
        // Process Windows messages
        while (WinApi.GetMessage(out var msg, IntPtr.Zero, 0, 0))
        {
            WinApi.TranslateMessage(ref msg);
            WinApi.DispatchMessage(ref msg);

            // Check for cancellation periodically
            if (cts.Token.IsCancellationRequested)
                break;

            // Periodic GDI object monitoring (debug mode only, every 5 seconds)
            if (configManager.Current.System.LogLevel == "Debug" && gdiCheckTimer.Elapsed.TotalSeconds >= 5)
            {
                var currentProcess = System.Diagnostics.Process.GetCurrentProcess();
                var currentGdiCount = WinApi.GetGuiResources(currentProcess.Handle, WinApi.GR_GDIOBJECTS);
                if (currentGdiCount != lastGdiCount)
                {
                    var delta = currentGdiCount - initialGdiCount;
                    logger.LogDebug("GDI objects: {Count} ({Delta:+0;-0} from start)", currentGdiCount, delta);
                    lastGdiCount = currentGdiCount;
                }
                gdiCheckTimer.Restart();
            }
        }

        // If we exit the message loop, wait a bit before checking again
        if (!cts.Token.IsCancellationRequested)
        {
            Thread.Sleep(100);
        }
    }
}
catch (Exception ex)
{
    logger.LogCritical(ex, "Fatal error occurred");
    return 1;
}
finally
{
    // Clean up resources
    logger.LogInformation("Shutting down SpotlightDimmer");

    systemTray.Dispose();
    focusTracker.Dispose();
    displayChangeMonitor.Dispose();
    renderer.Dispose();
    configManager.Dispose();

    // Shutdown logging system
    LoggingConfiguration.Shutdown();
}

return 0;

// ========================================================================
// Renderer Factory
// ========================================================================

/// <summary>
/// Creates an overlay renderer based on the configured backend type.
/// Falls back to Legacy renderer if the requested backend is unavailable or fails.
/// </summary>
static IOverlayRenderer CreateRenderer(string rendererBackend, ILogger logger)
{
    try
    {
        return rendererBackend.ToLowerInvariant() switch
        {
            "updatelayeredwindow" => CreateRendererWithLogging<UpdateLayeredWindowRenderer>("UpdateLayeredWindow", logger),
            "legacy" => CreateRendererWithLogging<LegacyLayeredWindowRenderer>("Legacy", logger),
            _ => CreateRendererWithFallback(rendererBackend, logger)
        };
    }
    catch (Exception ex)
    {
        logger.LogWarning(ex, "Failed to create {Backend} renderer, falling back to Legacy", rendererBackend);
        return new LegacyLayeredWindowRenderer();
    }
}

static IOverlayRenderer CreateRendererWithLogging<T>(string name, ILogger logger) where T : IOverlayRenderer, new()
{
    logger.LogInformation("Using {Backend} renderer", name);
    return new T();
}

static IOverlayRenderer CreateRendererWithFallback(string unknownBackend, ILogger logger)
{
    logger.LogWarning("Unknown renderer backend '{Backend}', falling back to Legacy", unknownBackend);
    logger.LogInformation("Available renderers: Legacy, UpdateLayeredWindow");
    return new LegacyLayeredWindowRenderer();
}

// ========================================================================
// Helper Classes
// ========================================================================

/// <summary>
/// Simple wrapper that implements IOverlayUpdateService using a delegate.
/// </summary>
sealed class OverlayUpdateServiceWrapper : IOverlayUpdateService
{
    private readonly Action<int, Rectangle> _updateAction;

    public OverlayUpdateServiceWrapper(Action<int, Rectangle> updateAction)
    {
        _updateAction = updateAction ?? throw new ArgumentNullException(nameof(updateAction));
    }

    public void UpdateOverlays(int displayIndex, Rectangle windowBounds)
    {
        _updateAction(displayIndex, windowBounds);
    }
}
