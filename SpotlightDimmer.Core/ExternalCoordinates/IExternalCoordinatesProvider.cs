namespace SpotlightDimmer.Core.ExternalCoordinates;

/// <summary>
/// Interface for providers that supply external pane coordinates.
/// Implementations watch external sources (files, sockets, etc.) for coordinate updates.
/// </summary>
public interface IExternalCoordinatesProvider : IDisposable
{
    /// <summary>
    /// Gets the current external pane coordinates, if available.
    /// Returns null if no coordinates are available or if the provider is not active.
    /// </summary>
    ExternalPaneCoordinates? CurrentCoordinates { get; }

    /// <summary>
    /// Gets the window title pattern for this provider.
    /// Used to determine when to apply external coordinates.
    /// </summary>
    string WindowTitlePattern { get; }

    /// <summary>
    /// Gets the default terminal padding configuration.
    /// </summary>
    TerminalPadding DefaultPadding { get; }

    /// <summary>
    /// Fired when coordinates are updated from the external source.
    /// </summary>
    event Action<ExternalPaneCoordinates>? CoordinatesChanged;

    /// <summary>
    /// Starts watching for coordinate updates.
    /// </summary>
    void Start();

    /// <summary>
    /// Stops watching for coordinate updates.
    /// </summary>
    void Stop();
}
