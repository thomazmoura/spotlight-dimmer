using System.Text;
using System.Text.Json;

namespace SpotlightDimmer.Core;

/// <summary>
/// Handles automatic injection and updating of JSON schema references in configuration files.
/// Manipulates raw JSON to preserve formatting and comments while adding/updating the $schema property.
/// </summary>
public static class SchemaInjector
{
    private const string SchemaPropertyName = "$schema";
    private const string SchemaUrlTemplate = "https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v{0}/config.schema.json";

    /// <summary>
    /// Gets the schema URL for a specific version.
    /// </summary>
    /// <param name="version">The version string (e.g., "0.8.5").</param>
    /// <returns>The versioned schema URL.</returns>
    public static string GetSchemaUrl(string version)
    {
        return string.Format(SchemaUrlTemplate, version);
    }

    /// <summary>
    /// Checks if JSON content contains a $schema property.
    /// </summary>
    /// <param name="jsonContent">The JSON content to check.</param>
    /// <returns>True if $schema is present, false otherwise.</returns>
    public static bool HasSchema(string jsonContent)
    {
        try
        {
            using var doc = JsonDocument.Parse(jsonContent);
            return doc.RootElement.TryGetProperty(SchemaPropertyName, out _);
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Gets the current $schema URL from JSON content.
    /// </summary>
    /// <param name="jsonContent">The JSON content to read.</param>
    /// <returns>The schema URL if present, null otherwise.</returns>
    public static string? GetCurrentSchemaUrl(string jsonContent)
    {
        try
        {
            using var doc = JsonDocument.Parse(jsonContent);
            if (doc.RootElement.TryGetProperty(SchemaPropertyName, out var schemaProp))
            {
                return schemaProp.GetString();
            }
            return null;
        }
        catch
        {
            return null;
        }
    }

    /// <summary>
    /// Injects or updates the $schema property in JSON content.
    /// Preserves JSON formatting by inserting the property as the first line after the opening brace.
    /// </summary>
    /// <param name="jsonContent">The original JSON content.</param>
    /// <param name="version">The version to use for the schema URL.</param>
    /// <returns>The updated JSON content with $schema property.</returns>
    public static string InjectOrUpdateSchema(string jsonContent, string version)
    {
        var schemaUrl = GetSchemaUrl(version);

        // If schema already exists with the correct URL, no changes needed
        var currentUrl = GetCurrentSchemaUrl(jsonContent);
        if (currentUrl == schemaUrl)
        {
            return jsonContent;
        }

        // Parse and rebuild with $schema as first property
        try
        {
            using var doc = JsonDocument.Parse(jsonContent);
            var root = doc.RootElement;

            // Use StringBuilder for efficient string construction
            var sb = new StringBuilder();
            sb.AppendLine("{");
            sb.Append("  \"");
            sb.Append(SchemaPropertyName);
            sb.Append("\": \"");
            sb.Append(schemaUrl);
            sb.Append('"');

            // Check if there are other properties to add
            bool hasOtherProperties = false;
            foreach (var property in root.EnumerateObject())
            {
                if (property.Name != SchemaPropertyName)
                {
                    hasOtherProperties = true;
                    break;
                }
            }

            if (hasOtherProperties)
            {
                sb.AppendLine(",");

                // Add all other properties
                bool isFirst = true;
                foreach (var property in root.EnumerateObject())
                {
                    if (property.Name == SchemaPropertyName)
                        continue; // Skip existing $schema property

                    if (!isFirst)
                    {
                        sb.AppendLine(",");
                    }

                    // Serialize the property
                    sb.Append("  \"");
                    sb.Append(property.Name);
                    sb.Append("\": ");
                    sb.Append(JsonSerializer.Serialize(property.Value, new JsonSerializerOptions { WriteIndented = false }));

                    isFirst = false;
                }
                sb.AppendLine();
            }
            else
            {
                sb.AppendLine();
            }

            sb.AppendLine("}");
            return sb.ToString();
        }
        catch
        {
            // If parsing fails, return original content
            return jsonContent;
        }
    }

    /// <summary>
    /// Determines if the schema URL needs to be updated based on config and app versions.
    /// </summary>
    /// <param name="configVersion">The version stored in the configuration file.</param>
    /// <param name="appVersion">The current application version.</param>
    /// <returns>True if the schema should be updated, false otherwise.</returns>
    public static bool ShouldUpdateSchema(string? configVersion, string appVersion)
    {
        // Always update if config has no version
        if (string.IsNullOrEmpty(configVersion))
            return true;

        // Update if versions don't match
        return configVersion != appVersion;
    }
}
