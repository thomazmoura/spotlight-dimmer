# Tmux Integration Setup Guide

This guide walks you through enabling Spotlight Dimmer's tmux pane focusing feature, which dims inactive tmux panes when using Windows Terminal.

## Prerequisites

Before starting, ensure you have:

- ✅ **Windows Terminal** installed (from Microsoft Store or GitHub)
- ✅ **WSL** (Windows Subsystem for Linux) installed and configured
- ✅ **tmux** installed in WSL (version 2.1 or later)
- ✅ **Spotlight Dimmer** installed on Windows

## Quick Start (5 Minutes)

### Step 1: Enable Tmux Mode in Spotlight Dimmer

Open PowerShell or Command Prompt on Windows and run:

```powershell
spotlight-dimmer-config tmux-enable
```

You'll see output like:
```
✓ Tmux pane focusing ENABLED
  Inactive tmux panes will be dimmed when Windows Terminal is focused

  Setup required:
    1. Add this to your ~/.tmux.conf:
       set-hook -g pane-focus-in 'run-shell "tmux display -p \"#{pane_left},#{pane_top},#{pane_right},#{pane_bottom},#{window_width},#{window_height}\" > ~/.spotlight-dimmer/tmux-active-pane.txt"'
    2. Reload tmux config: tmux source-file ~/.tmux.conf
    3. Configure terminal geometry with: tmux-config command
```

### Step 2: Configure Tmux Hook

Open your tmux configuration file in WSL:

```bash
nano ~/.tmux.conf
```

Add this line at the end of the file:

```tmux
set-hook -g pane-focus-in 'run-shell "tmux display -p \"#{pane_left},#{pane_top},#{pane_right},#{pane_bottom},#{window_width},#{window_height}\" > ~/.spotlight-dimmer/tmux-active-pane.txt"'
```

**What this does:**
- Creates a global hook that runs every time you focus a pane
- Captures the pane boundaries (in character coordinates)
- Writes them to a file accessible from both WSL and Windows

Save the file (`Ctrl+O`, `Enter`, `Ctrl+X` in nano).

### Step 3: Create Shared Directory

Create the directory where tmux will write pane information:

```bash
mkdir -p ~/.spotlight-dimmer
```

This creates the directory in your WSL home, which is accessible from Windows at:
```
C:\Users\{YourUsername}\.spotlight-dimmer\
```

### Step 4: Reload Tmux Configuration

If you're already in a tmux session, reload the configuration:

```bash
tmux source-file ~/.tmux.conf
```

If tmux isn't running, the configuration will be loaded automatically when you start it.

### Step 5: Configure Terminal Geometry

Back in Windows PowerShell/Command Prompt, run the automatic configuration:

```powershell
# Option A: Automatic detection (Recommended)
spotlight-dimmer-config tmux-auto-config

# Option B: Automatic detection from specific profile
spotlight-dimmer-config tmux-auto-config "Ubuntu-22.04"

# Option C: Manual configuration (if auto-detection doesn't work)
spotlight-dimmer-config tmux-config 9 20 0 35
```

**Automatic detection output example:**
```
🔍 Auto-detecting Windows Terminal configuration...

📖 Found settings:
  Source: Defaults section
  Font: Cascadia Code at 12 pt
  Padding: left=8, top=8, right=8, bottom=8

📐 Calculating font metrics...
  Character width: 9 pixels
  Character height: 20 pixels

✅ Terminal geometry configured successfully!
```

## Verification

### Test the Setup

1. **Start a tmux session** in Windows Terminal:
   ```bash
   tmux new -s test
   ```

2. **Split the window** to create multiple panes:
   ```bash
   # Vertical split
   tmux split-window -h

   # Horizontal split
   tmux split-window -v
   ```

3. **Switch between panes** using `Ctrl+B` then arrow keys

4. **Observe the dimming**: Inactive panes should be dimmed automatically!

### Check File Updates

Verify the tmux hook is working:

```bash
# Watch the pane file update in real-time
watch -n 0.5 cat ~/.spotlight-dimmer/tmux-active-pane.txt
```

Switch between tmux panes and you should see the numbers change. Example output:
```
0,0,119,29,240,60
```
This means: pane spans columns 0-119, rows 0-29, in a 240x60 character window.

### Check Configuration Status

```powershell
spotlight-dimmer-config tmux-status
```

Output should show:
```
TMUX PANE FOCUSING:
  Status: ENABLED

TERMINAL GEOMETRY:
  Font: 9x20 pixels
  Padding: left=8, top=35

TMUX PANE FILE:
  Path: C:\Users\{YourUsername}\.spotlight-dimmer\tmux-active-pane.txt
```

## Troubleshooting

### Problem: Panes don't dim

**Check 1: Is tmux mode enabled?**
```powershell
spotlight-dimmer-config tmux-status
```
If disabled, run: `spotlight-dimmer-config tmux-enable`

**Check 2: Is the hook configured?**
```bash
tmux show-hooks | grep pane-focus-in
```
Should show your hook. If not, add it to `~/.tmux.conf` and reload.

**Check 3: Is the file being written?**
```bash
ls -la ~/.spotlight-dimmer/tmux-active-pane.txt
```
File should exist and timestamp should update when you switch panes.

**Check 4: Is Windows Terminal focused?**
The feature only works when Windows Terminal is the active application on Windows.

### Problem: Overlays are misaligned

**Solution 1: Reconfigure with auto-detection**
```powershell
spotlight-dimmer-config tmux-auto-config
```

**Solution 2: Manual adjustment**
If auto-detection gives wrong values, measure and configure manually:
```powershell
spotlight-dimmer-config tmux-config <font_width> <font_height> <padding_left> <padding_top>
```

**How to measure:**
1. Take a screenshot of Windows Terminal with text
2. Measure character width/height in pixels using an image editor
3. Measure padding from window edge to first character

### Problem: Hook not triggering

**Check tmux version:**
```bash
tmux -V
```
Requires tmux 2.1 or later for pane coordinate variables.

**Verify hook syntax:**
```bash
# List all hooks
tmux show-hooks -g

# Test hook manually
tmux display -p "#{pane_left},#{pane_top},#{pane_right},#{pane_bottom},#{window_width},#{window_height}"
```
Should output numbers like: `0,0,119,29,240,60`

### Problem: Works in windowed mode but not fullscreen

This should work in both modes. If it doesn't:

1. Check Windows Terminal is detected:
   ```powershell
   tasklist | findstr WindowsTerminal
   ```

2. Verify Spotlight Dimmer is running:
   ```powershell
   tasklist | findstr spotlight-dimmer
   ```

3. Check system tray for Spotlight Dimmer icon

## Advanced Configuration

### Multiple Profiles

If you use different Windows Terminal profiles with different fonts:

```powershell
# Configure for each profile
spotlight-dimmer-config tmux-auto-config "Profile Name 1"
spotlight-dimmer-config tmux-auto-config "Profile Name 2"
```

Note: Only the last configuration is saved. You'll need to reconfigure when switching profiles, or use the profile with the largest font size for safety.

### Custom Shared File Location

If you want to use a different location for the pane file:

1. Edit your Windows config: `%APPDATA%\spotlight-dimmer\config.toml`
2. Add or modify:
   ```toml
   tmux_pane_file_path = "C:\\path\\to\\your\\file.txt"
   ```
3. Update your tmux hook to write to the same location

### Disabling Temporarily

To temporarily disable without removing the tmux hook:

```powershell
spotlight-dimmer-config tmux-disable
```

Re-enable with:
```powershell
spotlight-dimmer-config tmux-enable
```

## Understanding the Integration

### How It Works

```
┌─────────────────────────────────────────────────┐
│ tmux (in WSL)                                   │
│  ├─ Detects pane focus change                   │
│  ├─ Runs hook: captures pane boundaries         │
│  └─ Writes to: ~/.spotlight-dimmer/...txt       │
└─────────────────┬───────────────────────────────┘
                  │
                  │ (File shared between WSL & Windows)
                  │
┌─────────────────▼───────────────────────────────┐
│ Spotlight Dimmer (on Windows)                   │
│  ├─ Watches file for changes (event-driven)     │
│  ├─ Reads pane coordinates (character units)    │
│  ├─ Converts to pixel coordinates               │
│  └─ Creates overlay windows over inactive panes │
└─────────────────────────────────────────────────┘
```

### Coordinate Translation

Tmux uses **character coordinates** (columns and rows), while Windows uses **pixel coordinates**. Spotlight Dimmer translates between them:

```
Pixel X = Padding Left + (Column × Font Width)
Pixel Y = Padding Top  + (Row × Font Height)
```

Example:
- Font: 9px wide, 20px tall
- Padding: left=0, top=35
- Pane at column 60, row 15
- Pixel position: X = 0 + (60 × 9) = 540px, Y = 35 + (15 × 20) = 335px

This is why accurate font metrics are crucial!

## Tips & Best Practices

### 1. Use Automatic Configuration

Manual configuration is error-prone. The auto-config command uses Windows GDI APIs to get exact font metrics:

```powershell
spotlight-dimmer-config tmux-auto-config
```

### 2. Keep Windows Terminal Updated

Newer versions have better settings consistency. Check for updates:
- Microsoft Store: Automatically updates
- GitHub releases: Check https://github.com/microsoft/terminal/releases

### 3. Use Consistent Font Settings

For best results:
- Use the same font across all profiles
- Avoid very small fonts (< 10pt) - harder to measure accurately
- Stick to monospace fonts (Consolas, Cascadia Code, Fira Code, etc.)

### 4. Test After System Updates

Windows updates can sometimes reset terminal settings. After major updates:
```powershell
spotlight-dimmer-config tmux-auto-config
```

### 5. Combine with Partial Dimming

For maximum focus, enable both tmux pane focusing and partial dimming:

```powershell
spotlight-dimmer-config enable
spotlight-dimmer-config partial-enable
spotlight-dimmer-config tmux-enable
```

This gives you:
- Dimmed inactive displays
- Highlighted active window with surrounding dimming
- Dimmed inactive tmux panes within the active window

## Getting Help

If you encounter issues:

1. **Check the main troubleshooting guide**: See AGENTS.md in the repository
2. **Enable debug output**: Watch the console output when running `spotlight-dimmer.exe` directly
3. **Report issues**: https://github.com/thomazmoura/spotlight-dimmer/issues

Include in your report:
- Windows Terminal version
- tmux version
- Output of `spotlight-dimmer-config status`
- Content of `~/.spotlight-dimmer/tmux-active-pane.txt`

## Uninstallation

To remove the tmux integration:

1. **Disable in Spotlight Dimmer:**
   ```powershell
   spotlight-dimmer-config tmux-disable
   ```

2. **Remove tmux hook:**
   ```bash
   nano ~/.tmux.conf
   # Delete the set-hook line
   tmux source-file ~/.tmux.conf
   ```

3. **Optional: Remove shared file:**
   ```bash
   rm -rf ~/.spotlight-dimmer
   ```

---

**Enjoy your focused tmux workflow! 🎯**

For more information about Spotlight Dimmer, visit: https://github.com/thomazmoura/spotlight-dimmer
