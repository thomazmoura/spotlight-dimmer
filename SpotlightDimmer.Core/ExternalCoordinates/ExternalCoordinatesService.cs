using System.Text.RegularExpressions;

namespace SpotlightDimmer.Core.ExternalCoordinates;

/// <summary>
/// Manages external coordinates providers and determines when to apply external bounds.
/// This service coordinates between multiple providers and the focus tracking system.
/// </summary>
public class ExternalCoordinatesService : IDisposable
{
    private readonly List<IExternalCoordinatesProvider> _providers = new();
    private readonly Dictionary<IExternalCoordinatesProvider, Regex> _compiledPatterns = new();
    private bool _isStarted;

    /// <summary>
    /// Fired when any provider's coordinates change.
    /// </summary>
    public event Action<ExternalPaneCoordinates>? CoordinatesChanged;

    /// <summary>
    /// Gets whether the service has any active providers.
    /// </summary>
    public bool HasProviders => _providers.Count > 0;

    /// <summary>
    /// Registers an external coordinates provider.
    /// </summary>
    /// <param name="provider">The provider to register.</param>
    public void RegisterProvider(IExternalCoordinatesProvider provider)
    {
        if (provider == null) throw new ArgumentNullException(nameof(provider));

        _providers.Add(provider);

        // Pre-compile the regex pattern for performance
        if (!string.IsNullOrEmpty(provider.WindowTitlePattern))
        {
            try
            {
                _compiledPatterns[provider] = new Regex(
                    provider.WindowTitlePattern,
                    RegexOptions.Compiled | RegexOptions.IgnoreCase,
                    TimeSpan.FromMilliseconds(100)); // Timeout to prevent ReDoS
            }
            catch (ArgumentException)
            {
                // Invalid regex pattern - provider won't match any windows
                _compiledPatterns[provider] = new Regex("^$"); // Never matches
            }
        }

        // Subscribe to coordinate changes
        provider.CoordinatesChanged += OnProviderCoordinatesChanged;

        // If service is already started, start the provider
        if (_isStarted)
        {
            provider.Start();
        }
    }

    /// <summary>
    /// Starts all registered providers.
    /// </summary>
    public void Start()
    {
        if (_isStarted) return;

        _isStarted = true;
        foreach (var provider in _providers)
        {
            provider.Start();
        }
    }

    /// <summary>
    /// Stops all registered providers.
    /// </summary>
    public void Stop()
    {
        if (!_isStarted) return;

        _isStarted = false;
        foreach (var provider in _providers)
        {
            provider.Stop();
        }
    }

    /// <summary>
    /// Tries to get external bounds for a window with the given title.
    /// If external coordinates are available and match the window title pattern,
    /// the pixel bounds are calculated and returned.
    /// </summary>
    /// <param name="windowTitle">The title of the focused window.</param>
    /// <param name="windowBounds">The current bounds of the window.</param>
    /// <param name="externalBounds">The calculated external bounds, if available.</param>
    /// <returns>True if external bounds were found and should be used.</returns>
    public bool TryGetExternalBounds(string? windowTitle, Rectangle windowBounds, out Rectangle externalBounds)
    {
        externalBounds = default;

        if (string.IsNullOrEmpty(windowTitle))
        {
            return false;
        }

        foreach (var provider in _providers)
        {
            // Check if window title matches the provider's pattern
            if (!_compiledPatterns.TryGetValue(provider, out var regex))
            {
                continue;
            }

            try
            {
                if (!regex.IsMatch(windowTitle))
                {
                    continue;
                }
            }
            catch (RegexMatchTimeoutException)
            {
                continue; // Skip if regex times out
            }

            // Get current coordinates from the provider
            var coordinates = provider.CurrentCoordinates;
            if (coordinates == null || !coordinates.IsValid)
            {
                continue;
            }

            // Convert to pixel bounds
            externalBounds = coordinates.ToPixelBounds(windowBounds, provider.DefaultPadding);
            return true;
        }

        return false;
    }

    private void OnProviderCoordinatesChanged(ExternalPaneCoordinates coordinates)
    {
        CoordinatesChanged?.Invoke(coordinates);
    }

    /// <summary>
    /// Disposes all registered providers.
    /// </summary>
    public void Dispose()
    {
        Stop();

        foreach (var provider in _providers)
        {
            provider.CoordinatesChanged -= OnProviderCoordinatesChanged;
            provider.Dispose();
        }

        _providers.Clear();
        _compiledPatterns.Clear();
    }
}
