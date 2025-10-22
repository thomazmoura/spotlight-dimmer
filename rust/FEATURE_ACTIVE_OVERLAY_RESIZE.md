# Active Overlay Resizing Feature

## Overview
This feature enhances the behavior of active overlays when partial dimming is enabled. Previously, when both active and inactive overlays were visible, the borders around the focused window would get darker because both overlays overlapped at the window edges. This implementation fixes that issue by making the active overlay intelligently resize to match the focused window.

## Implementation Details

### Problem
- When partial dimming and active overlays were both enabled, the active overlay covered the entire display
- The inactive/partial overlays covered the empty areas around the window
- At the window borders, both overlays overlapped, causing double darkening
- This made the borders darker than the inactive areas, which was visually incorrect

### Solution
The active overlay now intelligently adapts its size based on the window state:

1. **Windowed Mode** (window is not maximized/fullscreen):
   - Active overlay resizes to **exactly match the window size and position**
   - Active overlay edges touch the inactive/partial overlay edges with no overlap
   - Eliminates the border darkening issue

2. **Maximized/Fullscreen Mode**:
   - Active overlay covers the **entire display** (original behavior)
   - No partial overlays are created (window fills the screen)

3. **During Drag Operations**:
   - Active overlay temporarily returns to **full screen mode**
   - This provides smooth performance during window dragging
   - After drag ends (window stable for 200ms), active overlay resizes to final window position

### Technical Implementation

#### New Functions Added

##### `platform/mod.rs` & `platform/windows.rs`
- **`is_window_maximized(window_handle: u64) -> Result<bool, String>`**
  - Checks if a window is maximized using Windows `IsZoomed()` API
  - Also detects fullscreen windows by comparing window bounds to monitor bounds
  - Uses 10-pixel tolerance for edge detection

##### `overlay.rs`
- **`resize_active_overlay(display_id: &str, window_rect: RECT) -> Result<(), String>`**
  - Resizes active overlay to match the exact window rectangle
  - Uses `SetWindowPos()` with `SWP_NOZORDER | SWP_NOACTIVATE` flags
  - Maintains overlay layering and doesn't steal focus

- **`restore_active_overlay_full_size(display_id: &str, display: &DisplayInfo) -> Result<(), String>`**
  - Restores active overlay to full display size
  - Used during drag operations and for maximized windows
  - Ensures smooth transitions between windowed and fullscreen states

#### Main Loop Changes (`main_new.rs`)

The partial dimming logic was enhanced to:
1. Check if the window is maximized/fullscreen
2. If **windowed** and **active overlays enabled**:
   - Resize active overlay to match window size
   - Creates perfect alignment with partial overlays
3. If **maximized/fullscreen** and **active overlays enabled**:
   - Restore active overlay to full display size
4. During **drag operations**:
   - Temporarily restore active overlay to full screen
   - Clear partial overlays for performance
5. After **drag ends**:
   - Recreate partial overlays
   - Resize active overlay to match final window position (if windowed)

### Behavior Summary

| Window State | Active Overlay Size | Partial Overlays | Result |
|-------------|-------------------|------------------|---------|
| Windowed | Exact window size | Around window edges | No border overlap ✅ |
| Maximized | Full display | None | Full coverage ✅ |
| Fullscreen | Full display | None | Full coverage ✅ |
| Dragging | Full display (temp) | Hidden (temp) | Smooth performance ✅ |

## User Experience

### Before This Feature
- Double-darkened borders around windows when both overlay types enabled
- Borders darker than inactive areas
- Visually distracting and incorrect

### After This Feature
- **Perfect edge alignment** between active and inactive overlays
- **No overlap** = consistent darkening across all overlays
- **Smooth transitions** during window drag operations
- **Automatic adaptation** to window state changes (maximize/restore)

## Configuration Requirements

This feature **automatically activates** when:
- ✅ Partial dimming is enabled (`is_partial_dimming_enabled = true`)
- ✅ Active overlays are enabled (`is_active_overlay_enabled = true`)

No additional configuration needed!

## Performance Impact

- **Minimal**: Only resizes active overlay when window state changes
- **Efficient**: Uses native Windows `SetWindowPos()` API
- **Optimized**: Full-screen mode during drag for smooth performance
- **Smart**: Only checks window state when partial dimming is active

## Code Changes Summary

### Files Modified
1. **`src/platform/mod.rs`** - Added `is_window_maximized()` to WindowManager trait
2. **`src/platform/windows.rs`** - Implemented window maximization detection
3. **`src/overlay.rs`** - Added overlay resize/restore functions
4. **`src/main_new.rs`** - Enhanced partial dimming logic with active overlay resizing
5. **`CHANGELOG.md`** - Documented feature with bilingual entry

### Lines of Code
- Platform layer: ~40 lines (maximization detection)
- Overlay manager: ~70 lines (resize/restore functions)
- Main loop: ~30 lines modified (integration logic)
- **Total: ~140 lines** of new/modified code

## Testing Recommendations

1. **Enable both features**:
   ```
   spotlight-dimmer-config enable-partial-dimming
   spotlight-dimmer-config enable-active-overlay
   spotlight-dimmer-config set-active-overlay-color 255 255 255 0.2
   ```

2. **Test scenarios**:
   - Open a windowed application → Active overlay should match window size
   - Maximize the window → Active overlay should fill display
   - Restore to windowed → Active overlay should resize to window
   - Drag the window → Active overlay should temporarily go full screen
   - Release drag → Active overlay should resize to final position

3. **Visual validation**:
   - Check border areas - should NOT be darker than inactive areas
   - Active overlay should perfectly align with partial overlay edges
   - No visible gaps or overlaps at window borders

## Future Enhancements

Possible improvements:
- Animation/transition effects during resize
- Configurable tolerance for edge detection
- Support for non-rectangular window shapes
- Per-application overlay size preferences

## Conclusion

This feature significantly improves the visual quality when using both active overlays and partial dimming together. By eliminating the border darkening issue, users now get a consistent, professional-looking dimming experience that properly highlights the focused window without visual artifacts.
