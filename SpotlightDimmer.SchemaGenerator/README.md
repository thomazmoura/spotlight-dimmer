# SpotlightDimmer.SchemaGenerator

A console tool that automatically generates `config.schema.json` from the C# `AppConfig` class using reflection.

## Purpose

This tool ensures the JSON schema file stays synchronized with the C# configuration types. Instead of manually maintaining the schema file, we use **NJsonSchema** to generate it automatically from the source code.

## Benefits

✅ **Single Source of Truth** - C# classes drive the schema
✅ **Zero Manual Sync** - Schema updates automatically when code changes
✅ **Type Safety** - Schema always matches actual configuration structure
✅ **Maintainability** - No risk of schema drift

## How It Works

1. **Reflection**: NJsonSchema analyzes the `AppConfig` class and its properties
2. **Schema Generation**: Converts C# types to JSON Schema draft-07 format
3. **Customization**: Adds additional metadata, descriptions, and validation rules
4. **Output**: Writes formatted JSON schema to `config.schema.json`

## Usage

### Via PowerShell Script (Recommended)

```powershell
.\SpotlightDimmer.Scripts\Generate-Schema.ps1
```

This script:
- Builds and runs the generator
- Outputs to `config.schema.json` in the repository root
- Shows file size and confirmation

### Direct Execution

```bash
cd SpotlightDimmer.SchemaGenerator
dotnet run
```

### Custom Output Path

```bash
dotnet run --project SpotlightDimmer.SchemaGenerator -- /path/to/output.json
```

## When to Regenerate

Run the schema generator whenever you:
- ✅ Add new properties to `AppConfig`, `OverlayConfig`, `SystemConfig`, or `Profile`
- ✅ Change property types or constraints
- ✅ Add new enum values to `DimmingMode` or `LogLevel`
- ✅ Modify default values
- ✅ Update property descriptions in XML doc comments

## Implementation Details

### Dependencies

- **NJsonSchema** (v11.1.0+) - JSON Schema generation library
- **SpotlightDimmer.Core** - Contains the `AppConfig` class

### Customization

The generator adds custom metadata beyond what NJsonSchema infers:

- **Hex Color Validation**: Regex pattern `^#[0-9A-Fa-f]{6}$` for color properties
- **Opacity Ranges**: Min/max constraints (0-255)
- **Enhanced Descriptions**: User-friendly explanations for each property
- **Enum Descriptions**: Detailed explanations for dimming modes and log levels

See `Program.cs:CustomizeSchema()` for implementation details.

### Schema Output

The generated schema includes:
- Full type definitions for all configuration objects
- Enum constraints with descriptions
- Pattern validation (hex colors)
- Range validation (opacity 0-255)
- Default values
- Required properties
- IntelliSense-friendly descriptions

## Integration

### Manual Workflow

1. Modify `AppConfig` class in `SpotlightDimmer.Core`
2. Run `.\SpotlightDimmer.Scripts\Generate-Schema.ps1`
3. Review changes to `config.schema.json`
4. Commit both files together

### Future: Build Integration (Optional)

The schema generator could be integrated into the build process:

```xml
<Target Name="GenerateSchema" BeforeTargets="Build">
  <Exec Command="dotnet run --project $(MSBuildThisFileDirectory)SpotlightDimmer.SchemaGenerator" />
</Target>
```

**Note**: Currently manual to avoid slowing down every build. Run the script when configuration changes occur.

## Troubleshooting

### Generator Fails to Run

- Ensure .NET 10 SDK is installed
- Verify `SpotlightDimmer.Core.csproj` exists
- Check that NJsonSchema package restored correctly

### Schema Missing Properties

- Ensure properties are `public` with getters
- Check that properties have correct `[JsonPropertyName]` attributes (if using custom names)
- Verify the type is serializable by System.Text.Json

### Schema Validation Not Working in VS Code

- Ensure `$schema` property references the correct path
- Check that the schema file is valid JSON
- Reload VS Code window if schema changes don't appear

## Example Output

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SpotlightDimmer Configuration",
  "type": "object",
  "properties": {
    "Overlay": {
      "type": "object",
      "properties": {
        "Mode": {
          "type": "string",
          "enum": ["FullScreen", "Partial", "PartialWithActive"],
          "description": "The dimming mode controlling overlay behavior"
        },
        ...
      }
    },
    ...
  }
}
```

## References

- [NJsonSchema Documentation](https://github.com/RicoSuter/NJsonSchema)
- [JSON Schema Specification](https://json-schema.org/)
- [VS Code JSON Schema Support](https://code.visualstudio.com/docs/languages/json#_json-schemas-and-settings)
