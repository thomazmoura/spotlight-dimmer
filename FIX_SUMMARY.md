# Fix Summary for v0.4.0 Profile System Issues

## Problem Description

Version 0.4.0 introduced a profile management system that allows users to save and switch between different overlay configurations. However, this feature had critical bugs that broke the overlay system:

1. **Active overlays stopped working** after loading a profile from the system tray
2. **Overlays appeared on wrong displays** after profile switches
3. **Only partial overlays continued to work** while full-display overlays broke
4. **Individual config changes worked** but `set-profile` command caused broken overlays
5. **Tray menu profile switching** had the same issues as the CLI `set-profile` command

## Root Cause Analysis

### Issue #1: Config Reload Logic Used `else if` Chains (CRITICAL)

**Location**: `src/main_new.rs` lines 227-283

**Problem**: The config reload code used `else if` statements to handle different types of changes:
```rust
if old_dimming_enabled != new_config.is_dimming_enabled {
    // Handle enable/disable
} else if color_changed {
    // Handle color change - NEVER REACHED if enabled state changed!
}
```

**Impact**: When a profile changed BOTH the enabled state AND the color (which most profiles do), only the first condition would execute. This caused overlays to be created with OLD colors from the OverlayManager's internal state.

**Example Scenario**:
- Current state: dimming disabled, color red
- Load profile: dimming enabled, color blue
- Result: Creates overlays with RED (old color), not blue!

### Issue #2: OverlayManager Colors Not Updated When Disabled (MAJOR)

**Location**: `src/main_new.rs` config reload section

**Problem**: When overlays were disabled, color changes weren't updating the OverlayManager's internal color storage.

**Impact**: 
1. User loads profile with new colors while overlays are disabled
2. Colors don't update in OverlayManager
3. User enables overlays later
4. Overlays appear with OLD colors

### Issue #3: Missing Visibility Update After Overlay Recreation (CRITICAL)

**Location**: `src/main_new.rs` lines 237-295

**Problem**: When overlays were recreated via `set_inactive_color()` or `set_active_color()`, the new overlays were created with `SW_SHOW` (shown on all displays), but their visibility was NOT updated based on the current active display.

**Impact**: After loading a profile from the tray:
- ALL inactive overlays were shown (including on the active display!)
- ALL active overlays were shown (including on inactive displays!)
- This created a "broken" appearance where overlays covered everything

**Why Partial Overlays Still Worked**: Partial overlays are managed separately and are only created when the active window bounds change, so they weren't affected by this bug.

### Issue #3b: Visibility Update Skipped When `last_display_id` Was None (CRITICAL)

**Location**: `src/main_new.rs` lines 250-254, 284-288

**Problem**: The visibility update after recreating overlays was conditional:
```rust
if let Some(ref display_id) = last_display_id {
    manager.update_visibility(display_id);
}
```

If `last_display_id` was `None` (which happens at startup before any window has been tracked, or when profile is loaded very early), the visibility update would be silently skipped!

**Impact**: 
- When using `set-profile` command shortly after app starts, overlays would appear on ALL displays
- Individual commands worked because they triggered separate config reloads, each getting a chance to detect the active window
- Tray menu profile switching had the same issue - if done before first window detection, overlays covered everything

**Why Individual Changes Worked**: When making changes individually (e.g., `disable` then `color`), each change triggered a separate config file modification. By the time the second change was detected, the main loop had likely already detected an active window and cached its display_id. But when using `set-profile`, ALL changes happened in one file write, often before the first window detection.

### Issue #4: Missing Partial Dimming State Change Handling (MODERATE)

**Location**: `src/main_new.rs` config reload section

**Problem**: Config reload logic didn't handle `is_partial_dimming_enabled` state changes.

**Impact**: When a profile disabled partial dimming, the partial overlays weren't cleared, causing visual artifacts.

### Issue #5: Default Profiles Not Added to Existing Configs (MINOR)

**Location**: `src/config.rs` `load()` method

**Problem**: When loading an old config file (pre-v0.4.0), the default profiles weren't automatically added.

**Impact**: Users upgrading from v0.3.0 wouldn't see the default "light-mode" and "dark-mode" profiles in the tray menu.

## Fixes Applied

### Fix #1: Replaced `else if` Chains with Independent Checks

**File**: `src/main_new.rs`

**Changes**:
```rust
// Pre-calculate all change flags
let dimming_enabled_changed = old_dimming_enabled != new_config.is_dimming_enabled;
let inactive_color_changed = /* compare all color components */;
let active_overlay_enabled_changed = /* ... */;
let active_color_changed = /* ... */;
let partial_dimming_changed = /* ... */;

// Handle ALL changes, not just the first one
if dimming_enabled_changed || inactive_color_changed {
    // Process both changes together
}

// Independent check for active overlays
if active_overlay_enabled_changed || active_color_changed {
    // Process both changes together
}

// Independent check for partial dimming
if partial_dimming_changed {
    // Handle state change
}
```

**Benefit**: Multiple simultaneous changes are now properly handled.

### Fix #2: Update Colors Even When Overlays Are Disabled

**File**: `src/main_new.rs` + `src/overlay.rs`

**Changes**:
- Added `update_inactive_color_only()` and `update_active_color_only()` methods to OverlayManager
- When overlays are disabled but colors change, update the internal color storage
- This ensures colors are correct when overlays are enabled later

```rust
if !new_config.is_dimming_enabled {
    if dimming_enabled_changed {
        manager.close_inactive();
    }
    // NEW: Update color even when disabled
    if inactive_color_changed {
        manager.update_inactive_color_only(new_config.overlay_color.clone());
    }
}
```

### Fix #3: Update Visibility After Recreating Overlays + Fallback to Active Window Detection

**File**: `src/main_new.rs`

**Changes**: After recreating overlays, immediately update their visibility based on the current active display. **Crucially**, if we don't have a cached `display_id`, query the window manager for the current active window:

```rust
if let Err(e) = manager.set_inactive_color(new_config.overlay_color.clone(), &current_displays) {
    eprintln!("[Main] Failed to update inactive overlays: {}", e);
} else {
    // Update visibility based on current active display
    // If we don't have a cached display_id, get the current active window
    if let Some(ref display_id) = last_display_id {
        manager.update_visibility(display_id);
    } else if let Ok(active_window) = window_manager.get_active_window() {
        manager.update_visibility(&active_window.display_id);
        last_display_id = Some(active_window.display_id.clone());
    }
}
```

**How It Works**:
1. `set_inactive_color()` recreates all inactive overlays with new color
2. All overlays are initially shown (SW_SHOW) on all displays
3. Check if we have a cached `display_id`
4. **If not**, query the window manager to get the current active window and its display
5. Call `update_visibility()` with the display ID (cached or freshly queried)
6. `update_visibility()` hides overlays on the active display, shows on others
7. Result: Correct visibility from the start, even at app startup

**Why This Fixes The Set-Profile Issue**: Even when `set-profile` is called immediately after app start (before any window has been tracked), the fix will query the active window on-demand and update visibility correctly. No more overlays covering the active display!

### Fix #4: Handle Partial Dimming State Changes

**File**: `src/main_new.rs`

**Changes**: Added detection and handling for partial dimming state changes:

```rust
if partial_dimming_changed {
    if !new_config.is_partial_dimming_enabled {
        println!("[Main] Partial dimming disabled via config change");
        let mut manager = overlay_manager.lock().unwrap();
        manager.clear_all_partial_overlays();
        last_window_rect = None;
        is_dragging = false;
    }
}
```

### Fix #5: Auto-Add Default Profiles on Load

**File**: `src/config.rs`

**Changes**:
```rust
pub fn load() -> Self {
    // ... load config from file ...
    match toml::from_str::<Config>(&content) {
        Ok(mut config) => {
            // NEW: Add default profiles if profiles HashMap is empty
            if config.profiles.is_empty() {
                config.add_default_profiles();
            }
            return config;
        }
        // ...
    }
}

// NEW: Helper method to add default profiles
fn add_default_profiles(&mut self) {
    self.profiles.insert("light-mode".to_string(), /* ... */);
    self.profiles.insert("dark-mode".to_string(), /* ... */);
}
```

## Testing Scenarios

### ✅ Test Case 1: Profile Switching with Overlays Enabled
1. Start app with overlays enabled on inactive displays
2. Right-click tray icon and select a different profile
3. **Expected**: Overlays update immediately with new color and settings
4. **Expected**: Active display remains clear, inactive displays are dimmed
5. **Expected**: No overlays appear on the active display

### ✅ Test Case 2: Profile Switching with Overlays Disabled
1. Start app with `is_dimming_enabled = false`
2. Switch to profile with different color
3. Enable overlays via config or another profile
4. **Expected**: New overlays use the profile's color, not the old color

### ✅ Test Case 3: Combined State Changes
1. Load profile that changes BOTH color AND enabled state
2. **Expected**: Both changes are applied correctly
3. **Expected**: Overlays appear with the new color

### ✅ Test Case 4: Partial Dimming State Changes
1. Enable partial dimming
2. Load profile that disables partial dimming
3. **Expected**: Partial overlays are cleared immediately
4. **Expected**: Only full-display overlays remain

### ✅ Test Case 5: Upgrade from v0.3.0
1. Use old config file without profiles field
2. Run v0.4.0 with fix
3. Right-click tray icon
4. **Expected**: Default profiles "light-mode" and "dark-mode" appear in menu

### ✅ Test Case 6: Multiple Profile Switches
1. Switch between profiles multiple times quickly
2. **Expected**: Each switch properly updates overlays
3. **Expected**: No "stuck" or "ghost" overlays
4. **Expected**: Correct visibility maintained throughout

## Files Modified

| File | Lines Changed | Purpose |
|------|--------------|---------|
| `src/main_new.rs` | ~113 | Fixed config reload logic, added visibility updates with fallback |
| `src/overlay.rs` | +10 | Added color update methods without overlay recreation |
| `src/config.rs` | +44 | Added default profile auto-loading for old configs |
| `Cargo.lock` | 2 | Version bump to 0.4.0 |

**Total**: 169 lines added/modified across 4 files

## Backward Compatibility

✅ **Fully backward compatible**
- Old config files work without modification
- Default profiles automatically added to old configs
- No breaking API changes
- Config file format remains compatible

## Performance Impact

✅ **No performance degradation**
- Color updates only happen on config changes (infrequent)
- Visibility updates are O(n) where n = number of displays (typically 1-3)
- Active window query is fast (single Win API call)
- Active window query only happens when `last_display_id` is None (rare - mostly at startup)
- Profile loading happens via tray menu or CLI (user-initiated, not frequent)

## Future Recommendations

1. **Reduce Config Check Interval**: Currently checks every 2 seconds (20 * 100ms). Consider reducing to 1 second for faster profile switching response.

2. **Add Profile Change Notification**: Instead of polling the config file, use file system watchers or inter-process communication for immediate updates.

3. **Persist Default Profiles**: Currently default profiles are added to memory but not saved to disk. Consider saving them on first load.

4. **Add Profile Validation**: Validate profile data before applying to catch corrupted configs.

5. **Add Tray Menu Indicators**: Show which profile is currently active in the tray menu with a checkmark.

## Conclusion

The fixes address all critical issues with the profile management system in v0.4.0:
- ✅ Overlays now work correctly after profile switches
- ✅ Visibility is properly maintained on correct displays
- ✅ Color changes are applied immediately and correctly
- ✅ Combined state changes are handled properly
- ✅ Default profiles available for all users

The profile management feature now works as originally intended, allowing users to quickly switch between different overlay configurations without any visual glitches or broken behavior.
