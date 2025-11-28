# External Coordinates Guide

This guide explains how to use the External Coordinates feature to dim specific regions within terminal applications like Windows Terminal running tmux.

## Overview

The External Coordinates feature allows SpotlightDimmer to receive pane coordinates from external sources (like tmux) and use them to create precise overlays around the active pane instead of the entire window.

This is particularly useful when:
- Running tmux in Windows Terminal with multiple panes
- You want to dim inactive tmux panes while keeping the active pane visible
- You want integration with other terminal multiplexers or applications that can output pane coordinates

## How It Works

1. An external script (e.g., a tmux hook) writes pane coordinates to a JSON file
2. SpotlightDimmer watches this file using FileSystemWatcher (event-driven, no polling)
3. When a window title matches the configured pattern (e.g., contains "TMUX"), SpotlightDimmer uses the external coordinates instead of the window bounds
4. The overlays are calculated to dim everything except the active pane

## Configuration

### Enabling External Coordinates

Add the `ExternalCoordinates` section to your `config.json`:

```json
{
  "ExternalCoordinates": {
    "Enabled": true,
    "Providers": [
      {
        "Type": "File",
        "WindowTitlePattern": ".*TMUX.*",
        "FilePath": "%AppData%\\SpotlightDimmer\\external-pane.json",
        "TerminalPadding": {
          "Top": 32,
          "Left": 8,
          "Bottom": 0,
          "Right": 8
        }
      }
    ]
  }
}
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `Enabled` | Enable/disable external coordinates feature | `false` |
| `Providers` | List of coordinate providers | Empty list |

### Provider Options

| Option | Description | Default |
|--------|-------------|---------|
| `Type` | Provider type (currently only `File` is supported) | `File` |
| `WindowTitlePattern` | Regex pattern to match window titles | `TMUX.*` |
| `FilePath` | Path to the coordinates JSON file | `%AppData%\SpotlightDimmer\external-pane.json` |
| `TerminalPadding` | Padding to account for terminal UI elements | `{ Top: 32, Left: 8, Bottom: 0, Right: 8 }` |

### Terminal Padding

The terminal padding accounts for UI elements that occupy space in the terminal window:

- **Top** (32px): Space for the tab bar in Windows Terminal
- **Left/Right** (8px): Margins around the terminal content
- **Bottom** (0px): Usually no padding needed at the bottom

Adjust these values based on your terminal configuration and theme.

## Coordinate File Format

The external coordinates file must be a JSON file with the following structure:

```json
{
  "version": 1,
  "timestamp": 1699999999,
  "source": "tmux-pane-watcher",
  "windowTitlePattern": ".*TMUX.*",
  "pane": {
    "top": 1,
    "left": 0,
    "width": 80,
    "height": 24
  },
  "cell": {
    "width": 10,
    "height": 20
  },
  "padding": {
    "top": 32,
    "left": 8,
    "bottom": 0,
    "right": 8
  }
}
```

### Field Descriptions

| Field | Description |
|-------|-------------|
| `version` | Schema version (currently 1) |
| `timestamp` | Unix timestamp when coordinates were captured |
| `source` | Identifier for the source of coordinates |
| `windowTitlePattern` | Optional: Override the pattern from config |
| `pane.top` | Pane's top position in character cells |
| `pane.left` | Pane's left position in character cells |
| `pane.width` | Pane width in character cells |
| `pane.height` | Pane height in character cells |
| `cell.width` | Width of a single character cell in pixels |
| `cell.height` | Height of a single character cell in pixels |
| `padding` | Optional: Override padding from config |

## tmux Integration

### Getting tmux Format Variables

tmux provides several format variables that are useful for this integration:

| Variable | Description |
|----------|-------------|
| `#{pane_top}` | Top position of the pane in cells |
| `#{pane_left}` | Left position of the pane in cells |
| `#{pane_width}` | Width of the pane in cells |
| `#{pane_height}` | Height of the pane in cells |
| `#{window_cell_width}` | Width of a cell in pixels (tmux 3.4+) |
| `#{window_cell_height}` | Height of a cell in pixels (tmux 3.4+) |

### Example tmux Hook Script

Create a script to update the coordinates file when the active pane changes:

```bash
#!/bin/bash
# File: ~/.tmux/update-pane-coordinates.sh

# Get Windows AppData path from WSL
APPDATA=$(wslpath "$(cmd.exe /c 'echo %AppData%' 2>/dev/null | tr -d '\r')")
OUTPUT_FILE="$APPDATA/SpotlightDimmer/external-pane.json"

# Ensure directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Get pane coordinates from tmux
PANE_TOP=$(tmux display-message -p '#{pane_top}')
PANE_LEFT=$(tmux display-message -p '#{pane_left}')
PANE_WIDTH=$(tmux display-message -p '#{pane_width}')
PANE_HEIGHT=$(tmux display-message -p '#{pane_height}')

# Get cell dimensions (requires tmux 3.4+)
# If not available, you'll need to configure these manually
CELL_WIDTH=$(tmux display-message -p '#{window_cell_width}' 2>/dev/null || echo "10")
CELL_HEIGHT=$(tmux display-message -p '#{window_cell_height}' 2>/dev/null || echo "20")

# Get current timestamp
TIMESTAMP=$(date +%s)

# Write JSON file
cat > "$OUTPUT_FILE" << EOF
{
  "version": 1,
  "timestamp": $TIMESTAMP,
  "source": "tmux",
  "pane": {
    "top": $PANE_TOP,
    "left": $PANE_LEFT,
    "width": $PANE_WIDTH,
    "height": $PANE_HEIGHT
  },
  "cell": {
    "width": $CELL_WIDTH,
    "height": $CELL_HEIGHT
  }
}
EOF
```

### Setting Up tmux Hooks

Add to your `~/.tmux.conf`:

```bash
# Update SpotlightDimmer coordinates when pane changes
set-hook -g pane-focus-in 'run-shell "~/.tmux/update-pane-coordinates.sh"'
set-hook -g window-pane-changed 'run-shell "~/.tmux/update-pane-coordinates.sh"'
```

### Cell Dimensions for Older tmux Versions

If you're using tmux < 3.4, the `window_cell_width` and `window_cell_height` variables are not available. You'll need to:

1. Measure your cell dimensions manually:
   - Open Windows Terminal
   - Take a screenshot
   - Measure the width and height of a single character in pixels

2. Hardcode the values in your script:
   ```bash
   CELL_WIDTH=10  # Adjust based on your font
   CELL_HEIGHT=20 # Adjust based on your font
   ```

Common cell dimensions for popular fonts:
- **Cascadia Code** (11pt): ~10x20 px
- **Consolas** (12pt): ~8x16 px
- **JetBrains Mono** (11pt): ~10x20 px

## Troubleshooting

### Coordinates Not Being Applied

1. **Check if feature is enabled**: Ensure `ExternalCoordinates.Enabled` is `true` in config
2. **Check window title pattern**: The pattern must match the window title (use `.*TMUX.*` for Windows Terminal with tmux)
3. **Check file path**: Ensure the coordinates file exists and is being updated
4. **Enable debug logging**: Set `LogLevel` to `Debug` and check logs for external coordinate messages

### Overlays Appear in Wrong Position

1. **Check terminal padding**: The padding values might need adjustment for your terminal theme
2. **Check cell dimensions**: Ensure cell width/height match your terminal font
3. **Check pane coordinates**: Verify the pane coordinates in the JSON file are correct

### File Not Being Detected

1. **Check file path**: Use `%AppData%` or full path
2. **Check permissions**: Ensure SpotlightDimmer can read the file
3. **Check file format**: Ensure valid JSON with correct structure

## Performance Considerations

- The FileSystemWatcher is event-driven (no polling)
- Coordinate file should be updated only when the active pane changes
- JSON parsing is minimal and uses source generation for AOT compatibility
- Regex patterns are pre-compiled with timeout protection

## Limitations

- Currently only file-based providers are supported
- Cell dimensions must be provided (not auto-detected)
- Only one active pane can be highlighted at a time
- The feature requires the window title to match a pattern
