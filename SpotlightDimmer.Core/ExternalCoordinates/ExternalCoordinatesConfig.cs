namespace SpotlightDimmer.Core.ExternalCoordinates;

/// <summary>
/// Configuration for a single external coordinates provider.
/// </summary>
public class ExternalCoordinatesProviderConfig
{
    /// <summary>
    /// Path to the coordinates file. Supports environment variables like %AppData%.
    /// Default: %AppData%\SpotlightDimmer\external-pane.json
    /// </summary>
    public string FilePath { get; set; } = "%AppData%\\SpotlightDimmer\\external-pane.json";

    /// <summary>
    /// Regular expression pattern to match against window titles.
    /// When the focused window matches this pattern, external coordinates will be used.
    /// Default: "TMUX.*" (matches windows with TMUX in the title)
    /// </summary>
    public string WindowTitlePattern { get; set; } = "TMUX.*";

    /// <summary>
    /// Default terminal padding in pixels (top, left, bottom, right).
    /// Used when the coordinates file doesn't specify padding.
    /// </summary>
    public TerminalPaddingConfig TerminalPadding { get; set; } = new();

    /// <summary>
    /// Gets the expanded file path with environment variables resolved.
    /// </summary>
    public string GetExpandedFilePath()
    {
        return Environment.ExpandEnvironmentVariables(FilePath);
    }
}

/// <summary>
/// Terminal padding configuration in pixels.
/// </summary>
public class TerminalPaddingConfig
{
    /// <summary>
    /// Top padding in pixels (e.g., for title bar and tabs).
    /// Default: 32 (typical Windows Terminal tab bar height)
    /// </summary>
    public int Top { get; set; } = 32;

    /// <summary>
    /// Left padding in pixels.
    /// Default: 8
    /// </summary>
    public int Left { get; set; } = 8;

    /// <summary>
    /// Bottom padding in pixels.
    /// Default: 0
    /// </summary>
    public int Bottom { get; set; } = 0;

    /// <summary>
    /// Right padding in pixels.
    /// Default: 8
    /// </summary>
    public int Right { get; set; } = 8;

    /// <summary>
    /// Converts to the TerminalPadding struct.
    /// </summary>
    public TerminalPadding ToTerminalPadding()
    {
        return new TerminalPadding(Top, Left, Bottom, Right);
    }
}

/// <summary>
/// Configuration for external coordinates support.
/// </summary>
public class ExternalCoordinatesConfig
{
    /// <summary>
    /// Whether external coordinates support is enabled.
    /// Default: false
    /// </summary>
    public bool Enabled { get; set; } = false;

    /// <summary>
    /// List of external coordinates providers.
    /// </summary>
    public List<ExternalCoordinatesProviderConfig> Providers { get; set; } = new();
}
