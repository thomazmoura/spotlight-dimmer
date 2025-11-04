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
            var settings = new JsonSchemaGeneratorSettings
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
