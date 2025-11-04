# SpotlightDimmer Configuration

SpotlightDimmer supports **dynamic configuration** with real-time hot-reloading. Changes to the configuration file are automatically detected and applied without restarting the application.

## Configuration File Location

The configuration file is stored at:
- **Windows**: `%AppData%\SpotlightDimmer\config.json`
  - Typically: `C:\Users\<YourUsername>\AppData\Roaming\SpotlightDimmer\config.json`

This location works consistently whether you run the application:
- As a debug build from Visual Studio
- As a release build
- As an installed application (via installer)

## Automatic Creation

If the configuration file doesn't exist when SpotlightDimmer starts, it will be **automatically created** with default values.

## IntelliSense and Autocomplete (VS Code)

SpotlightDimmer includes a **JSON schema file** (`config.schema.json`) that provides IntelliSense, autocomplete, and inline documentation when editing the configuration file in VS Code.

### Automatic Schema Injection

**Good news**: SpotlightDimmer **automatically** adds the `$schema` property to your configuration file on first run! You don't need to do anything manually.

When the application starts, it:
- ✅ Detects if `$schema` is missing and adds it automatically
- ✅ Uses **version-specific schema URLs** (e.g., `v0.8.5`) matching your app version
- ✅ Updates the schema URL when you upgrade to a newer version
- ✅ Ensures you always get autocomplete for the features available in your version

### Version-Aware Schema URLs

The schema URL is version-specific:
```
https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v0.8.5/config.schema.json
```

**Why version-specific?**
- **Older versions** point to schemas matching their feature set (no confusion from newer properties)
- **Automatic upgrades** update the schema URL when you install a new version
- **Backwards compatibility** ensures old config files work with their original schema

### Manual Schema Configuration (Optional)

If you prefer to manually control the schema reference:

1. **Use versioned GitHub URL** (recommended):
   ```json
   {
     "$schema": "https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v0.8.5/config.schema.json",
     "ConfigVersion": "0.8.5",
     ...
   }
   ```

2. **Use relative path** if you have the repository cloned locally:
   ```json
   {
     "$schema": "./config.schema.json",
     ...
   }
   ```

### Benefits of Schema Validation

With the schema enabled, you get:
- ✅ **Autocomplete** - Press `Ctrl+Space` to see available properties
- ✅ **Validation** - Real-time error highlighting for invalid values
- ✅ **Documentation** - Hover over properties to see descriptions
- ✅ **Enum suggestions** - Dropdown for `Mode`, `LogLevel`, etc.
- ✅ **Type checking** - Ensures colors are valid hex codes, opacity is 0-255, etc.

## Configuration Options

### Example Configuration

```json
{
  "$schema": "https://raw.githubusercontent.com/thomazmoura/spotlight-dimmer/v0.8.5/config.schema.json",
  "ConfigVersion": "0.8.5",
  "Overlay": {
    "Mode": "PartialWithActive",
    "InactiveColor": "#000000",
    "InactiveOpacity": 153,
    "ActiveColor": "#000000",
    "ActiveOpacity": 102,
    "ExcludeFromScreenCapture": false
  },
  "System": {
    "EnableLogging": true,
    "LogLevel": "Information",
    "LogRetentionDays": 7
  },
  "Profiles": [
    {
      "Name": "Light Mode",
      "Mode": "Partial",
      "InactiveColor": "#000000",
      "InactiveOpacity": 128,
      "ActiveColor": "#000000",
      "ActiveOpacity": 102
    }
  ],
  "CurrentProfile": null
}
```

### Settings Explained

#### `Mode` (string)
Controls the dimming behavior. Available modes:

- **`"FullScreen"`**: Dims entire inactive displays
  - Only the display with the focused window remains bright
  - Window position doesn't affect dimming
  - Most efficient mode (minimal updates)

- **`"Partial"`**: Dims inactive display regions
  - Creates a "spotlight" around the active window
  - Tracks window position and size
  - Updates as you move/resize windows

- **`"PartialWithActive"`**: Partial mode + subtle active window highlight
  - Like Partial mode, but adds a lighter overlay on the active window
  - Creates a more pronounced spotlight effect

**Default**: `"FullScreen"`

#### `InactiveColor` (string)
The color of the dimming overlay for inactive areas.

- Format: Hex color code (e.g., `"#000000"`, `"#1A1A1A"`)
- **Default**: `"#000000"` (black)

**Examples**:
- `"#000000"` - Pure black (standard dimming)
- `"#1A1A1A"` - Dark gray (softer dimming)
- `"#001122"` - Very dark blue tint

#### `InactiveOpacity` (integer)
How opaque/dark the inactive overlay should be.

- Range: `0` (fully transparent) to `255` (fully opaque)
- **Default**: `153` (~60% opacity)

**Examples**:
- `102` - 40% opacity (subtle dimming)
- `153` - 60% opacity (moderate dimming) ✓ Default
- `204` - 80% opacity (strong dimming)

#### `ActiveColor` (string)
The color of the overlay on the active window (used only in `PartialWithActive` mode).

- Format: Hex color code
- **Default**: `"#000000"` (black)

#### `ActiveOpacity` (integer)
Opacity for the active window overlay (used only in `PartialWithActive` mode).

- Range: `0` to `255`
- Should be **less than** `InactiveOpacity` to create a spotlight effect
- **Default**: `102` (~40% opacity)

## Hot-Reload Behavior

When you edit and save the configuration file:

1. **Automatic Detection**: SpotlightDimmer detects the file change within milliseconds
2. **Configuration Reload**: The new settings are loaded and validated
3. **Immediate Application**: Overlays are recalculated and updated instantly
4. **Console Feedback**: You'll see a message confirming the reload

**Example console output**:
```
[Config] Configuration reloaded from file
[Config]   Mode: PartialWithActive
[Config]   Inactive: #000000 @ 153/255
[Config]   Active: #000000 @ 102/255
[Config] Overlays updated with new configuration
```

## Configuration Examples

### Example 1: Subtle Full-Screen Dimming
```json
{
  "Mode": "FullScreen",
  "InactiveColor": "#000000",
  "InactiveOpacity": 102,
  "ActiveColor": "#000000",
  "ActiveOpacity": 102
}
```
Perfect for multi-monitor setups where you want a gentle reminder of which display is active.

### Example 2: Strong Spotlight with Warm Tint
```json
{
  "Mode": "PartialWithActive",
  "InactiveColor": "#0A0806",
  "InactiveOpacity": 204,
  "ActiveColor": "#0A0806",
  "ActiveOpacity": 127
}
```
Creates a dramatic spotlight effect with a subtle warm tint.

### Example 3: Blue-Tinted Night Mode
```json
{
  "Mode": "FullScreen",
  "InactiveColor": "#001122",
  "InactiveOpacity": 153,
  "ActiveColor": "#000000",
  "ActiveOpacity": 102
}
```
Dims inactive displays with a dark blue tint, easier on the eyes at night.

## Tips for Experimenting

1. **Keep SpotlightDimmer Running**: Leave the application running while you edit the config
2. **Use a Text Editor**: Any text editor works (Notepad, VS Code, etc.)
3. **Save Often**: Changes apply immediately on save
4. **Watch the Console**: SpotlightDimmer logs configuration changes
5. **Invalid Values**: If you enter invalid values, the app will log an error and keep the previous valid configuration

## Finding Your Configuration File

If you're not sure where the config file is located, SpotlightDimmer prints the path when it starts:

```
Overlay Configuration:
  Config file: C:\Users\<YourUsername>\AppData\Roaming\SpotlightDimmer\config.json
  Mode: FullScreen
  ...
```

## Troubleshooting

### Changes Not Applying
- Ensure the JSON is valid (check for syntax errors)
- Check the console for error messages
- Try stopping and restarting SpotlightDimmer

### File Not Found
The configuration file is created automatically on first run. If it's missing:
1. Start SpotlightDimmer
2. It will create a default config file
3. Edit the newly created file

### Invalid Mode Names
Mode names are case-insensitive. These are equivalent:
- `"FullScreen"`, `"fullscreen"`, `"FULLSCREEN"`
- `"Partial"`, `"partial"`
- `"PartialWithActive"`, `"partialwithactive"`

If an invalid mode is specified, it defaults to `FullScreen`.
