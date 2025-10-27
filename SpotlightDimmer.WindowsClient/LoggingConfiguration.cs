using Microsoft.Extensions.Logging;
using Serilog;
using Serilog.Events;
using SpotlightDimmer.Core;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Manages Serilog configuration for file-based logging.
/// Provides methods to initialize, configure, and reconfigure logging based on AppConfig settings.
/// </summary>
public static class LoggingConfiguration
{
    private static ILoggerFactory? _loggerFactory;
    private static string? _currentLogPath;

    /// <summary>
    /// Gets the logs directory path.
    /// Location: %AppData%\SpotlightDimmer\logs\
    /// </summary>
    public static string GetLogsDirectory()
    {
        var appDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        return Path.Combine(appDataPath, "SpotlightDimmer", "logs");
    }

    /// <summary>
    /// Gets the log file path for the current date.
    /// Format: spotlight-YYYY-MM-DD.log
    /// </summary>
    public static string GetLogFilePath()
    {
        var logsDir = GetLogsDirectory();
        var fileName = $"spotlight-{DateTime.Now:yyyy-MM-dd}.log";
        return Path.Combine(logsDir, fileName);
    }

    /// <summary>
    /// Initializes Serilog logging based on the provided AppConfig.
    /// Creates the logs directory and configures file output with rolling intervals and retention.
    /// </summary>
    /// <param name="config">The application configuration containing logging settings.</param>
    /// <returns>An ILoggerFactory for creating typed loggers.</returns>
    public static ILoggerFactory Initialize(AppConfig config)
    {
        var logsDir = GetLogsDirectory();
        var logPath = Path.Combine(logsDir, "spotlight-.log");
        _currentLogPath = logPath;

        // Ensure logs directory exists
        if (!Directory.Exists(logsDir))
        {
            Directory.CreateDirectory(logsDir);
        }

        // Clean up old log files based on retention policy
        CleanupOldLogs(logsDir, config.System.LogRetentionDays);

        // Configure Serilog
        var loggerConfig = new LoggerConfiguration();

        if (config.System.EnableLogging)
        {
            var minLevel = ParseLogLevel(config.System.LogLevel);

            loggerConfig
                .MinimumLevel.Is(minLevel)
                .WriteTo.File(
                    logPath,
                    rollingInterval: RollingInterval.Day,
                    retainedFileCountLimit: config.System.LogRetentionDays,
                    outputTemplate: "{Timestamp:yyyy-MM-dd HH:mm:ss.fff} [{Level:u3}] {Message:lj}{NewLine}{Exception}");
        }
        else
        {
            // Logging disabled - configure silent logger
            loggerConfig.MinimumLevel.Fatal();
        }

        Log.Logger = loggerConfig.CreateLogger();

        // Create logger factory
        _loggerFactory = LoggerFactory.Create(builder =>
        {
            builder.AddSerilog(Log.Logger, dispose: false);
        });

        return _loggerFactory;
    }

    /// <summary>
    /// Reconfigures Serilog logging based on updated configuration.
    /// This method allows hot-reload of logging settings without restarting the application.
    /// </summary>
    /// <param name="config">The updated application configuration.</param>
    public static void Reconfigure(AppConfig config)
    {
        // Dispose existing logger factory
        _loggerFactory?.Dispose();

        // Close existing Serilog logger
        Log.CloseAndFlush();

        // Reinitialize with new configuration
        _loggerFactory = Initialize(config);
    }

    /// <summary>
    /// Gets a typed logger for the specified type.
    /// </summary>
    /// <typeparam name="T">The type to create a logger for.</typeparam>
    /// <returns>An ILogger instance for the specified type.</returns>
    public static ILogger<T> GetLogger<T>()
    {
        if (_loggerFactory == null)
        {
            throw new InvalidOperationException("LoggingConfiguration has not been initialized. Call Initialize() first.");
        }

        return _loggerFactory.CreateLogger<T>();
    }

    /// <summary>
    /// Shuts down the logging system and flushes any pending log entries.
    /// </summary>
    public static void Shutdown()
    {
        _loggerFactory?.Dispose();
        Log.CloseAndFlush();
    }

    /// <summary>
    /// Parses the log level string from configuration to a Serilog LogEventLevel.
    /// </summary>
    /// <param name="logLevel">The log level string: "Error", "Information", or "Debug".</param>
    /// <returns>The corresponding LogEventLevel.</returns>
    private static LogEventLevel ParseLogLevel(string logLevel)
    {
        return logLevel?.ToLowerInvariant() switch
        {
            "error" => LogEventLevel.Error,
            "information" => LogEventLevel.Information,
            "debug" => LogEventLevel.Debug,
            _ => LogEventLevel.Information
        };
    }

    /// <summary>
    /// Cleans up log files older than the retention period.
    /// </summary>
    /// <param name="logsDirectory">The directory containing log files.</param>
    /// <param name="retentionDays">Number of days to retain logs.</param>
    private static void CleanupOldLogs(string logsDirectory, int retentionDays)
    {
        try
        {
            if (!Directory.Exists(logsDirectory))
                return;

            var cutoffDate = DateTime.Now.AddDays(-retentionDays);
            var logFiles = Directory.GetFiles(logsDirectory, "spotlight-*.log");

            foreach (var logFile in logFiles)
            {
                var fileInfo = new FileInfo(logFile);
                if (fileInfo.LastWriteTime < cutoffDate)
                {
                    try
                    {
                        File.Delete(logFile);
                    }
                    catch
                    {
                        // Ignore errors deleting individual files
                    }
                }
            }
        }
        catch
        {
            // Ignore cleanup errors - not critical
        }
    }

    /// <summary>
    /// Checks if any log files exist in the logs directory.
    /// </summary>
    /// <returns>True if log files exist, false otherwise.</returns>
    public static bool DoLogFilesExist()
    {
        var logsDir = GetLogsDirectory();
        if (!Directory.Exists(logsDir))
            return false;

        var logFiles = Directory.GetFiles(logsDir, "spotlight-*.log");
        return logFiles.Length > 0;
    }

    /// <summary>
    /// Gets the most recent log file path.
    /// </summary>
    /// <returns>The path to the most recent log file, or null if no logs exist.</returns>
    public static string? GetMostRecentLogFile()
    {
        var logsDir = GetLogsDirectory();
        if (!Directory.Exists(logsDir))
            return null;

        var logFiles = Directory.GetFiles(logsDir, "spotlight-*.log")
            .Select(f => new FileInfo(f))
            .OrderByDescending(f => f.LastWriteTime)
            .FirstOrDefault();

        return logFiles?.FullName;
    }
}
