using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.Extensions.Logging;

namespace SpotlightDimmer.Core;

/// <summary>
/// JSON source generation context for AOT compatibility.
/// </summary>
[JsonSourceGenerationOptions(WriteIndented = true)]
[JsonSerializable(typeof(AppConfig))]
[JsonSerializable(typeof(OverlayConfig))]
[JsonSerializable(typeof(SystemConfig))]
[JsonSerializable(typeof(Profile))]
internal partial class AppConfigJsonContext : JsonSerializerContext
{
}

/// <summary>
/// Manages application configuration with file watching and hot-reload support.
/// Monitors a JSON configuration file and notifies subscribers when changes occur.
/// </summary>
public class ConfigurationManager : IDisposable
{
    private readonly string _configFilePath;
    private readonly FileSystemWatcher _watcher;
    private AppConfig _currentConfig;
    private readonly object _lock = new();
    private DateTime _lastReloadTime = DateTime.MinValue;
    private const int DebounceMilliseconds = 100; // Debounce rapid file changes
    private readonly ILogger<ConfigurationManager> _logger;

    /// <summary>
    /// Event fired when the configuration changes.
    /// Handlers receive the new configuration.
    /// </summary>
    public event Action<AppConfig>? ConfigurationChanged;

    /// <summary>
    /// Gets the current configuration.
    /// Thread-safe access to the configuration.
    /// </summary>
    public AppConfig Current
    {
        get
        {
            lock (_lock)
            {
                return _currentConfig;
            }
        }
    }

    /// <summary>
    /// Creates a new ConfigurationManager using the default configuration path.
    /// Default path: %AppData%\SpotlightDimmer\config.json
    /// </summary>
    /// <param name="logger">Logger instance for configuration operations.</param>
    public ConfigurationManager(ILogger<ConfigurationManager> logger) : this(GetDefaultConfigPath(), logger)
    {
    }

    /// <summary>
    /// Creates a new ConfigurationManager with a custom configuration file path.
    /// </summary>
    /// <param name="configFilePath">Path to the configuration JSON file.</param>
    /// <param name="logger">Logger instance for configuration operations.</param>
    public ConfigurationManager(string configFilePath, ILogger<ConfigurationManager> logger)
    {
        _logger = logger;
        _configFilePath = configFilePath;

        // Ensure the directory exists
        var directory = Path.GetDirectoryName(_configFilePath);
        if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
        {
            Directory.CreateDirectory(directory);
            _logger.LogInformation("Created config directory: {Directory}", directory);
        }

        // Load or create initial configuration
        _currentConfig = LoadOrCreateConfig();

        // Set up file watcher
        _watcher = new FileSystemWatcher
        {
            Path = directory!,
            Filter = Path.GetFileName(_configFilePath),
            NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.Size,
            EnableRaisingEvents = true
        };

        _watcher.Changed += OnConfigFileChanged;
        _watcher.Created += OnConfigFileChanged;

        _logger.LogDebug("Watching config file: {ConfigFilePath}", _configFilePath);
    }

    /// <summary>
    /// Gets the default configuration file path.
    /// Location: %AppData%\SpotlightDimmer\config.json
    /// </summary>
    public static string GetDefaultConfigPath()
    {
        var appDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        var appFolder = Path.Combine(appDataPath, "SpotlightDimmer");
        return Path.Combine(appFolder, "config.json");
    }

    /// <summary>
    /// Loads the configuration from disk, or creates a default one if it doesn't exist.
    /// </summary>
    private AppConfig LoadOrCreateConfig()
    {
        if (!File.Exists(_configFilePath))
        {
            _logger.LogInformation("Config file not found. Creating default config at: {ConfigFilePath}", _configFilePath);
            var defaultConfig = AppConfig.Default;
            SaveConfig(defaultConfig);
            return defaultConfig;
        }

        try
        {
            var json = File.ReadAllText(_configFilePath);
            var config = JsonSerializer.Deserialize(json, AppConfigJsonContext.Default.AppConfig);

            if (config == null)
            {
                _logger.LogWarning("Failed to parse config file. Using default configuration");
                return AppConfig.Default;
            }

            _logger.LogDebug("Loaded configuration from: {ConfigFilePath}", _configFilePath);
            return config;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error loading config. Using default configuration");
            return AppConfig.Default;
        }
    }

    /// <summary>
    /// Saves the configuration to disk.
    /// </summary>
    private void SaveConfig(AppConfig config)
    {
        try
        {
            var json = JsonSerializer.Serialize(config, AppConfigJsonContext.Default.AppConfig);
            File.WriteAllText(_configFilePath, json);
            _logger.LogDebug("Saved configuration to: {ConfigFilePath}", _configFilePath);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error saving config");
        }
    }

    /// <summary>
    /// Saves the current configuration to disk and updates the internal state.
    /// This method should be used when modifying the configuration from external code.
    /// </summary>
    /// <param name="config">The configuration to save.</param>
    public void SaveConfiguration(AppConfig config)
    {
        lock (_lock)
        {
            _currentConfig = config;
        }
        SaveConfig(config);
    }

    /// <summary>
    /// Handles file system change events for the configuration file.
    /// </summary>
    private void OnConfigFileChanged(object sender, FileSystemEventArgs e)
    {
        // Debounce: Ignore changes that happen too quickly
        // (FileSystemWatcher can fire multiple times for a single save)
        lock (_lock)
        {
            var now = DateTime.UtcNow;
            if ((now - _lastReloadTime).TotalMilliseconds < DebounceMilliseconds)
            {
                return;
            }
            _lastReloadTime = now;
        }

        // Small delay to ensure the file write is complete
        Thread.Sleep(50);

        ReloadConfiguration();
    }

    /// <summary>
    /// Reloads the configuration from disk and notifies subscribers if it changed.
    /// </summary>
    private void ReloadConfiguration()
    {
        try
        {
            if (!File.Exists(_configFilePath))
            {
                _logger.LogWarning("Config file was deleted. Using current configuration");
                return;
            }

            var json = File.ReadAllText(_configFilePath);
            var newConfig = JsonSerializer.Deserialize(json, AppConfigJsonContext.Default.AppConfig);

            if (newConfig == null)
            {
                _logger.LogWarning("Failed to parse updated config file. Keeping current configuration");
                return;
            }

            lock (_lock)
            {
                _currentConfig = newConfig;
            }

            // Log configuration details at debug level
            _logger.LogDebug("Configuration reloaded from file");
            _logger.LogDebug("  Mode: {Mode}", newConfig.Overlay.Mode);
            _logger.LogDebug("  Inactive: {InactiveColor} @ {InactiveOpacity}/255", newConfig.Overlay.InactiveColor, newConfig.Overlay.InactiveOpacity);
            _logger.LogDebug("  Active: {ActiveColor} @ {ActiveOpacity}/255", newConfig.Overlay.ActiveColor, newConfig.Overlay.ActiveOpacity);

            // Show current profile status
            if (!string.IsNullOrEmpty(newConfig.CurrentProfile))
            {
                bool matches = newConfig.DoesOverlayMatchProfile(newConfig.CurrentProfile);
                _logger.LogDebug("  Current profile: {CurrentProfile}{ProfileMatch}", newConfig.CurrentProfile, matches ? "" : " *");
            }

            // Notify subscribers
            ConfigurationChanged?.Invoke(newConfig);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error reloading config");
        }
    }

    /// <summary>
    /// Disposes resources used by the ConfigurationManager.
    /// </summary>
    public void Dispose()
    {
        _watcher.EnableRaisingEvents = false;
        _watcher.Changed -= OnConfigFileChanged;
        _watcher.Created -= OnConfigFileChanged;
        _watcher.Dispose();
    }
}
