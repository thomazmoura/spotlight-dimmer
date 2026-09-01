using NJsonSchema;
using NJsonSchema.Generation;
using SpotlightDimmer.Core;
using System.Text.Json;

namespace SpotlightDimmer.SchemaGenerator;

/// <summary>
/// Console application that generates config.schema.json from AppConfig class.
/// This ensures the JSON schema stays in sync with the C# configuration types.
/// </summary>
class Program
{
    static async Task<int> Main(string[] args)
    {
        try
        {
            Console.WriteLine("SpotlightDimmer JSON Schema Generator");
            Console.WriteLine("=====================================\n");

            // Determine output path (default to repository root)
            string outputPath = args.Length > 0
                ? args[0]
                : Path.Combine(GetRepositoryRoot(), "config.schema.json");

            Console.WriteLine($"Generating schema from: {typeof(AppConfig).FullName}");
            Console.WriteLine($"Output path: {outputPath}\n");

            // Configure schema generation settings
            var settings = new SystemTextJsonSchemaGeneratorSettings
            {
                DefaultReferenceTypeNullHandling = ReferenceTypeNullHandling.NotNull,
                GenerateAbstractProperties = false,
                SerializerOptions = new System.Text.Json.JsonSerializerOptions
                {
                    PropertyNamingPolicy = null, // Use PascalCase (matching C# property names)
                    WriteIndented = true
                }
            };

            // Generate schema from AppConfig type
            var schema = JsonSchema.FromType<AppConfig>(settings);

            // Customize schema metadata
            schema.Title = "SpotlightDimmer Configuration";
            schema.Description = "Configuration schema for SpotlightDimmer overlay settings, system options, and profiles";
            schema.Id = "https://github.com/thomazmoura/spotlight-dimmer/config.schema.json";

            // Add additional descriptions for enum values
            CustomizeSchema(schema);

            // Serialize to JSON with indentation
            string schemaJson = schema.ToJson();

            // Write to file
            await File.WriteAllTextAsync(outputPath, schemaJson);

            Console.WriteLine("✓ Schema generated successfully!");
            Console.WriteLine($"✓ Written to: {outputPath}");
            Console.WriteLine($"✓ Size: {new FileInfo(outputPath).Length} bytes\n");

            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"ERROR: {ex.Message}");
            Console.Error.WriteLine(ex.StackTrace);
            return 1;
        }
    }

    /// <summary>
    /// Customizes the generated schema with additional metadata and descriptions.
    /// </summary>
    private static void CustomizeSchema(JsonSchema schema)
    {
        // Add enum descriptions for DimmingMode
        if (schema.Definitions.TryGetValue("DimmingMode", out var dimmingModeSchema))
        {
            dimmingModeSchema.Description = "Defines the different dimming modes for overlay calculation";
        }

        // Customize Overlay properties
        if (schema.Properties.TryGetValue("Overlay", out var overlayProperty))
        {
            var overlaySchema = overlayProperty.ActualSchema;

            if (overlaySchema.Properties.TryGetValue("Mode", out var modeProperty))
            {
                modeProperty.Description = "The dimming mode controlling overlay behavior";
            }

            if (overlaySchema.Properties.TryGetValue("InactiveColor", out var inactiveColorProp))
            {
                inactiveColorProp.Description = "Inactive overlay color in hex format (e.g., '#000000' for black, '#1A1A1A' for dark gray)";
                inactiveColorProp.Pattern = "^#[0-9A-Fa-f]{6}$";
            }

            if (overlaySchema.Properties.TryGetValue("InactiveOpacity", out var inactiveOpacityProp))
            {
                inactiveOpacityProp.Description = "Inactive overlay opacity (0 = fully transparent, 255 = fully opaque). Recommended: 153 (~60% opacity)";
            }

            if (overlaySchema.Properties.TryGetValue("ActiveColor", out var activeColorProp))
            {
                activeColorProp.Description = "Active overlay color in hex format (used only in PartialWithActive mode)";
                activeColorProp.Pattern = "^#[0-9A-Fa-f]{6}$";
            }

            if (overlaySchema.Properties.TryGetValue("ActiveOpacity", out var activeOpacityProp))
            {
                activeOpacityProp.Description = "Active overlay opacity (used only in PartialWithActive mode). Should be less than InactiveOpacity for spotlight effect. Recommended: 102 (~40% opacity)";
            }

            if (overlaySchema.Properties.TryGetValue("ExcludeFromScreenCapture", out var excludeCaptureProp))
            {
                excludeCaptureProp.Description = "EXPERIMENTAL: Exclude overlay windows from screen captures/screenshots. May not work on all systems due to Windows API limitations with layered windows.";
            }
        }

        // Customize System properties
        if (schema.Properties.TryGetValue("System", out var systemProperty))
        {
            var systemSchema = systemProperty.ActualSchema;

            if (systemSchema.Properties.TryGetValue("EnableLogging", out var enableLoggingProp))
            {
                enableLoggingProp.Description = "Enable file-based logging to %AppData%\\SpotlightDimmer\\logs";
            }

            if (systemSchema.Properties.TryGetValue("LogLevel", out var logLevelProp))
            {
                logLevelProp.Description = "Log level for file output";
            }

            if (systemSchema.Properties.TryGetValue("LogRetentionDays", out var retentionProp))
            {
                retentionProp.Description = "Number of days to retain log files. Older logs are automatically deleted.";
            }
        }

        // Customize Profiles array
        if (schema.Properties.TryGetValue("Profiles", out var profilesProperty))
        {
            profilesProperty.Description = "List of saved profiles for quick overlay configuration switching";

            var profilesSchema = profilesProperty.ActualSchema;
            if (profilesSchema.Item != null)
            {
                var profileSchema = profilesSchema.Item.ActualSchema;
                profileSchema.Description = "A saved overlay configuration preset";

                if (profileSchema.Properties.TryGetValue("Name", out var nameProp))
                {
                    nameProp.Description = "The name of the profile (e.g., 'Light Mode', 'Dark Mode', 'Night Mode')";
                }
            }
        }

        // Customize CurrentProfile
        if (schema.Properties.TryGetValue("CurrentProfile", out var currentProfileProp))
        {
            currentProfileProp.Description = "The name of the currently active profile, or null if using custom settings";
        }

        // Customize AppIntegrations array
        if (schema.Properties.TryGetValue("AppIntegrations", out var appIntegrationsProperty))
        {
            appIntegrationsProperty.Description = "Per-application integrations that let the spotlight target an inner region of the focused window (e.g. a terminal pane) instead of the whole window";

            var appIntegrationsSchema = appIntegrationsProperty.ActualSchema;
            if (appIntegrationsSchema.Item != null)
            {
                var integrationSchema = appIntegrationsSchema.Item.ActualSchema;
                integrationSchema.Description = "A per-application integration, matched against the focused window's process name on Windows and its WM_CLASS on Linux";

                if (integrationSchema.Properties.TryGetValue("ProcessName", out var processNameProp))
                {
                    processNameProp.Description = "Process executable name to match, including extension (e.g. 'WindowsTerminal.exe'). Case-insensitive.";
                }

                if (integrationSchema.Properties.TryGetValue("Provider", out var providerProp))
                {
                    providerProp.Description = "Integration provider: 'windows-terminal' (focused WT pane via accessibility, with optional tmux sub-resolution), 'wezterm' (focused WezTerm pane via the wezterm CLI, with optional tmux sub-resolution), or 'tmux' (generic terminal running tmux full-window). Default: 'tmux'";
                }

                if (integrationSchema.Properties.TryGetValue("ContentOffsetX", out var offsetXProp))
                {
                    offsetXProp.Description = "Pixels from the matched content area's left edge to the terminal cell grid (window padding). Default: 0";
                }

                if (integrationSchema.Properties.TryGetValue("ContentOffsetY", out var offsetYProp))
                {
                    offsetYProp.Description = "Pixels from the matched content area's top edge to the terminal cell grid (padding plus tab bar height, if any). Default: 0";
                }

                if (integrationSchema.Properties.TryGetValue("WmClass", out var wmClassProp))
                {
                    wmClassProp.Description = "Linux only. Window WM_CLASS to match, case-sensitively (e.g. 'org.wezfurlong.wezterm'). The Linux counterpart of ProcessName; ignored by the Windows client.";
                }

                if (integrationSchema.Properties.TryGetValue("TtySource", out var ttySourceProp))
                {
                    ttySourceProp.Description = "Linux only. How the focused tmux pane's tty is discovered: 'title' reads it from the window title (published by tmux via set-titles-string), anything else - 'wezterm' by convention - queries the WezTerm CLI. Default: 'wezterm'. Ignored by the Windows client.";
                }
            }
        }
    }

    /// <summary>
    /// Gets the repository root directory (assumes this tool is in a subdirectory).
    /// </summary>
    private static string GetRepositoryRoot()
    {
        string currentDir = Directory.GetCurrentDirectory();

        // Walk up until we find a directory containing .git or spotlight-dimmer.sln
        DirectoryInfo? dir = new DirectoryInfo(currentDir);
        while (dir != null)
        {
            if (Directory.Exists(Path.Combine(dir.FullName, ".git")) ||
                File.Exists(Path.Combine(dir.FullName, "spotlight-dimmer.sln")))
            {
                return dir.FullName;
            }
            dir = dir.Parent;
        }

        // Fallback: assume we're in a project subdirectory
        return Path.Combine(currentDir, "..");
    }
}
