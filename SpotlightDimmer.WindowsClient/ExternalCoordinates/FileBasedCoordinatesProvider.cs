using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.Extensions.Logging;
using SpotlightDimmer.Core.ExternalCoordinates;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// JSON model for deserializing external pane coordinates from file.
/// </summary>
internal class ExternalPaneCoordinatesJson
{
    [JsonPropertyName("version")]
    public int Version { get; set; } = 1;

    [JsonPropertyName("timestamp")]
    public long Timestamp { get; set; }

    [JsonPropertyName("source")]
    public string Source { get; set; } = string.Empty;

    [JsonPropertyName("windowTitlePattern")]
    public string? WindowTitlePattern { get; set; }

    [JsonPropertyName("pane")]
    public PaneJson? Pane { get; set; }

    [JsonPropertyName("cell")]
    public CellJson? Cell { get; set; }

    [JsonPropertyName("padding")]
    public PaddingJson? Padding { get; set; }
}

internal class PaneJson
{
    [JsonPropertyName("top")]
    public int Top { get; set; }

    [JsonPropertyName("left")]
    public int Left { get; set; }

    [JsonPropertyName("width")]
    public int Width { get; set; }

    [JsonPropertyName("height")]
    public int Height { get; set; }
}

internal class CellJson
{
    [JsonPropertyName("width")]
    public int Width { get; set; }

    [JsonPropertyName("height")]
    public int Height { get; set; }
}

internal class PaddingJson
{
    [JsonPropertyName("top")]
    public int Top { get; set; }

    [JsonPropertyName("left")]
    public int Left { get; set; }

    [JsonPropertyName("bottom")]
    public int Bottom { get; set; }

    [JsonPropertyName("right")]
    public int Right { get; set; }
}

/// <summary>
/// JSON source generation context for AOT compatibility.
/// </summary>
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(ExternalPaneCoordinatesJson))]
internal partial class ExternalCoordinatesJsonContext : JsonSerializerContext
{
}

/// <summary>
/// File-based external coordinates provider using FileSystemWatcher.
/// Watches a JSON file for coordinate updates, similar to ConfigurationManager pattern.
/// </summary>
public class FileBasedCoordinatesProvider : IExternalCoordinatesProvider
{
    private readonly string _filePath;
    private readonly string _windowTitlePattern;
    private readonly TerminalPadding _defaultPadding;
    private readonly ILogger<FileBasedCoordinatesProvider> _logger;
    private FileSystemWatcher? _watcher;
    private ExternalPaneCoordinates? _currentCoordinates;
    private readonly object _lock = new();
    private DateTime _lastReloadTime = DateTime.MinValue;
    private const int DebounceMilliseconds = 50; // Debounce rapid file changes

    /// <inheritdoc/>
    public ExternalPaneCoordinates? CurrentCoordinates
    {
        get
        {
            lock (_lock)
            {
                return _currentCoordinates;
            }
        }
    }

    /// <inheritdoc/>
    public string WindowTitlePattern => _windowTitlePattern;

    /// <inheritdoc/>
    public TerminalPadding DefaultPadding => _defaultPadding;

    /// <inheritdoc/>
    public event Action<ExternalPaneCoordinates>? CoordinatesChanged;

    /// <summary>
    /// Creates a new FileBasedCoordinatesProvider.
    /// </summary>
    /// <param name="config">Provider configuration.</param>
    /// <param name="logger">Logger instance.</param>
    public FileBasedCoordinatesProvider(
        ExternalCoordinatesProviderConfig config,
        ILogger<FileBasedCoordinatesProvider> logger)
    {
        _filePath = config.GetExpandedFilePath();
        _windowTitlePattern = config.WindowTitlePattern;
        _defaultPadding = config.TerminalPadding.ToTerminalPadding();
        _logger = logger;
    }

    /// <inheritdoc/>
    public void Start()
    {
        if (_watcher != null)
            return;

        var directory = Path.GetDirectoryName(_filePath);
        var fileName = Path.GetFileName(_filePath);

        if (string.IsNullOrEmpty(directory))
        {
            _logger.LogWarning("Invalid external coordinates file path: {FilePath}", _filePath);
            return;
        }

        // Ensure the directory exists
        if (!Directory.Exists(directory))
        {
            try
            {
                Directory.CreateDirectory(directory);
                _logger.LogInformation("Created external coordinates directory: {Directory}", directory);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "Failed to create external coordinates directory: {Directory}", directory);
                return;
            }
        }

        // Load existing file if present
        LoadCoordinates();

        // Set up file watcher
        _watcher = new FileSystemWatcher
        {
            Path = directory,
            Filter = fileName,
            NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.Size | NotifyFilters.CreationTime,
            EnableRaisingEvents = true
        };

        _watcher.Changed += OnFileChanged;
        _watcher.Created += OnFileChanged;

        _logger.LogInformation("Watching external coordinates file: {FilePath}", _filePath);
        _logger.LogInformation("  Window title pattern: {Pattern}", _windowTitlePattern);
        _logger.LogInformation("  Terminal padding: Top={Top}, Left={Left}, Bottom={Bottom}, Right={Right}",
            _defaultPadding.Top, _defaultPadding.Left, _defaultPadding.Bottom, _defaultPadding.Right);
    }

    /// <inheritdoc/>
    public void Stop()
    {
        if (_watcher == null)
            return;

        _watcher.EnableRaisingEvents = false;
        _watcher.Changed -= OnFileChanged;
        _watcher.Created -= OnFileChanged;
        _watcher.Dispose();
        _watcher = null;

        _logger.LogDebug("Stopped watching external coordinates file");
    }

    private void OnFileChanged(object sender, FileSystemEventArgs e)
    {
        // Debounce: Ignore changes that happen too quickly
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
        Thread.Sleep(20);

        LoadCoordinates();
    }

    private void LoadCoordinates()
    {
        if (!File.Exists(_filePath))
        {
            _logger.LogDebug("External coordinates file not found: {FilePath}", _filePath);
            return;
        }

        try
        {
            var json = File.ReadAllText(_filePath);
            var jsonModel = JsonSerializer.Deserialize(json, ExternalCoordinatesJsonContext.Default.ExternalPaneCoordinatesJson);

            if (jsonModel == null)
            {
                _logger.LogWarning("Failed to parse external coordinates file");
                return;
            }

            var coordinates = ConvertToCoordinates(jsonModel);

            lock (_lock)
            {
                _currentCoordinates = coordinates;
            }

            if (coordinates.IsValid)
            {
                _logger.LogDebug("Loaded external coordinates: Pane({Left},{Top}) {Width}x{Height} chars, Cell {CellWidth}x{CellHeight}px",
                    coordinates.Pane.Left, coordinates.Pane.Top,
                    coordinates.Pane.Width, coordinates.Pane.Height,
                    coordinates.Cell.Width, coordinates.Cell.Height);

                CoordinatesChanged?.Invoke(coordinates);
            }
            else
            {
                _logger.LogWarning("External coordinates are invalid (zero dimensions)");
            }
        }
        catch (JsonException ex)
        {
            _logger.LogWarning(ex, "Failed to parse external coordinates JSON");
        }
        catch (IOException ex)
        {
            _logger.LogWarning(ex, "Failed to read external coordinates file");
        }
    }

    private ExternalPaneCoordinates ConvertToCoordinates(ExternalPaneCoordinatesJson json)
    {
        var coordinates = new ExternalPaneCoordinates
        {
            Version = json.Version,
            Timestamp = json.Timestamp,
            Source = json.Source,
            WindowTitlePattern = json.WindowTitlePattern ?? _windowTitlePattern
        };

        if (json.Pane != null)
        {
            coordinates.Pane = new CharacterCoordinates(
                json.Pane.Top,
                json.Pane.Left,
                json.Pane.Width,
                json.Pane.Height);
        }

        if (json.Cell != null)
        {
            coordinates.Cell = new CellDimensions(json.Cell.Width, json.Cell.Height);
        }

        if (json.Padding != null)
        {
            coordinates.Padding = new TerminalPadding(
                json.Padding.Top,
                json.Padding.Left,
                json.Padding.Bottom,
                json.Padding.Right);
        }

        return coordinates;
    }

    /// <inheritdoc/>
    public void Dispose()
    {
        Stop();
    }
}
