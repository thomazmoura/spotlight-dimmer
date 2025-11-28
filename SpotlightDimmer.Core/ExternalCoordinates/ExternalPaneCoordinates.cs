namespace SpotlightDimmer.Core.ExternalCoordinates;

/// <summary>
/// Represents the pane position in character coordinates (as reported by tmux or similar tools).
/// </summary>
public readonly record struct CharacterCoordinates(int Top, int Left, int Width, int Height);

/// <summary>
/// Represents the cell (character) dimensions in pixels.
/// </summary>
public readonly record struct CellDimensions(int Width, int Height);

/// <summary>
/// Represents padding around the terminal content area in pixels.
/// This accounts for title bars, tab bars, borders, etc.
/// </summary>
public readonly record struct TerminalPadding(int Top, int Left, int Bottom, int Right);

/// <summary>
/// Represents external pane coordinates received from an external source like tmux.
/// Contains all information needed to convert character-based coordinates to pixel coordinates.
/// </summary>
public class ExternalPaneCoordinates
{
    /// <summary>
    /// Version of the coordinate format (for future compatibility).
    /// </summary>
    public int Version { get; set; } = 1;

    /// <summary>
    /// Unix timestamp when the coordinates were captured.
    /// </summary>
    public long Timestamp { get; set; }

    /// <summary>
    /// Source of the coordinates (e.g., "tmux", "vim", etc.).
    /// </summary>
    public string Source { get; set; } = string.Empty;

    /// <summary>
    /// Pattern to match against window title to identify the target window.
    /// If empty, coordinates apply to any focused window.
    /// </summary>
    public string WindowTitlePattern { get; set; } = string.Empty;

    /// <summary>
    /// Pane position and size in character coordinates.
    /// </summary>
    public CharacterCoordinates Pane { get; set; }

    /// <summary>
    /// Cell (character) dimensions in pixels.
    /// </summary>
    public CellDimensions Cell { get; set; }

    /// <summary>
    /// Optional terminal padding (if not provided, uses configuration defaults).
    /// </summary>
    public TerminalPadding? Padding { get; set; }

    /// <summary>
    /// Checks if the coordinates are valid (have non-zero cell dimensions).
    /// </summary>
    public bool IsValid => Cell.Width > 0 && Cell.Height > 0 && Pane.Width > 0 && Pane.Height > 0;

    /// <summary>
    /// Converts the character-based pane coordinates to pixel-based window bounds.
    /// </summary>
    /// <param name="windowBounds">The bounds of the terminal window containing the pane.</param>
    /// <param name="defaultPadding">Default padding to use if not specified in coordinates.</param>
    /// <returns>The pixel bounds of the pane within the screen.</returns>
    public Rectangle ToPixelBounds(Rectangle windowBounds, TerminalPadding defaultPadding)
    {
        if (!IsValid)
        {
            return windowBounds; // Fallback to window bounds if invalid
        }

        var padding = Padding ?? defaultPadding;

        // Calculate content area (window bounds minus padding)
        int contentLeft = windowBounds.Left + padding.Left;
        int contentTop = windowBounds.Top + padding.Top;

        // Convert character coordinates to pixel coordinates
        int panePixelLeft = contentLeft + (Pane.Left * Cell.Width);
        int panePixelTop = contentTop + (Pane.Top * Cell.Height);
        int panePixelWidth = Pane.Width * Cell.Width;
        int panePixelHeight = Pane.Height * Cell.Height;

        return new Rectangle(panePixelLeft, panePixelTop, panePixelWidth, panePixelHeight);
    }
}
