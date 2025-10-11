# Polling to Event-Driven Architecture Migration Report

**Date:** 2025-10-10
**Current Architecture:** 100ms polling loop (`thread::sleep` + `PeekMessageW`)
**Target Architecture:** Event-driven with `GetMessageW` + Windows event hooks

---

## Executive Summary

The current implementation uses a 100ms polling loop that continuously checks for:
1. Window focus changes
2. Display configuration changes
3. Window position/size changes
4. Configuration file modifications
5. Pause state changes
6. Exit requests

This report analyzes each dependency, ranks migration feasibility, and provides event-driven alternatives.

---

## Polling Dependencies Analysis

### 1. **Tray Icon Message Processing**
**Current Implementation:** Lines 128-129 (`process_windows_messages()`)
```rust
// Process Windows messages (for tray icon events)
process_windows_messages();
```

**Why it needs polling:** Uses `PeekMessageW` in non-blocking mode to handle tray icon clicks/events

**Event-driven alternative:** Replace `PeekMessageW` with `GetMessageW` (blocking)
```rust
// Blocking message loop - only wakes when events occur
GetMessageW(&mut msg, null_mut(), 0, 0);
TranslateMessage(&msg);
DispatchMessageW(&msg);
```

**Feasibility:** ⭐⭐⭐⭐⭐ **EASIEST** (5/5)
- Simple API swap
- No functional changes required
- Direct performance improvement
- Already integrated with tray icon infrastructure

**Risk:** Low - Standard Windows API pattern

---

### 2. **Exit Flag Monitoring**
**Current Implementation:** Lines 131-135
```rust
// Check if exit was requested via tray icon
if exit_flag.load(Ordering::SeqCst) {
    println!("[Main] Exit requested, shutting down...");
    break;
}
```

**Why it needs polling:** Atomic flag checked every loop iteration

**Event-driven alternative:** Handle `WM_QUIT` message in message loop
```rust
match msg.message {
    WM_QUIT => break,  // Exit cleanly
    _ => DispatchMessageW(&msg),
}
```

**Feasibility:** ⭐⭐⭐⭐⭐ **EASIEST** (5/5)
- Standard Windows shutdown pattern
- Tray icon already posts `WM_QUIT` or can be modified to do so
- No threading complexity

**Risk:** Low - Well-established pattern

---

### 3. **Pause State Changes**
**Current Implementation:** Lines 137-157
```rust
let current_paused = pause_flag.load(Ordering::SeqCst);
if current_paused != last_paused {
    // Hide/show overlays based on pause state
    // Update tray icon
}
```

**Why it needs polling:** Atomic flag checked every 100ms to detect tray icon pause/resume

**Event-driven alternative:** Custom window message `WM_USER_PAUSE_TOGGLED`
```rust
// In tray icon callback when user double-clicks:
PostMessageW(message_window, WM_USER_PAUSE_TOGGLED, 0, 0);

// In message loop:
case WM_USER_PAUSE_TOGGLED:
    handle_pause_toggle();
```

**Feasibility:** ⭐⭐⭐⭐ **EASY** (4/5)
- Requires tray icon modification to post message
- Clean separation of concerns
- Immediate response (no 100ms delay)

**Risk:** Low - Simple message posting

---

### 4. **Foreground Window Changes**
**Current Implementation:** Lines 367-411
```rust
match window_manager.get_active_window() {
    Ok(active_window) => {
        let window_changed = Some(active_window.handle) != last_window_handle;
        let display_changed = last_display_id.as_ref() != Some(&active_window.display_id);
        // Update overlays
    }
}
```

**Why it needs polling:** Calls `GetForegroundWindow()` every 100ms to detect focus changes

**Event-driven alternative:** `SetWinEventHook` with `EVENT_SYSTEM_FOREGROUND`
```rust
unsafe {
    SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND,      // Event to monitor
        EVENT_SYSTEM_FOREGROUND,      // Same event (single event)
        ptr::null_mut(),              // No DLL
        Some(foreground_hook_callback), // Callback function
        0,                            // All processes
        0,                            // All threads
        WINEVENT_OUTOFCONTEXT,        // Async callback
    );
}

unsafe extern "system" fn foreground_hook_callback(...) {
    // Post WM_USER_FOREGROUND_CHANGED to message window
}
```

**Feasibility:** ⭐⭐⭐⭐ **EASY** (4/5)
- Well-documented Windows API
- Already partially implemented in current codebase (lines 106-124)
- Instant notification (0ms latency vs 0-100ms polling delay)

**Risk:** Medium - Callback runs on different thread, needs proper message posting

---

### 5. **Display Configuration Changes**
**Current Implementation:** Lines 322-365
```rust
if let Ok(current_display_count) = display_manager.get_display_count() {
    if current_display_count != last_display_count {
        // Recreate overlays for new display configuration
    }
}
```

**Why it needs polling:** Calls `EnumDisplayMonitors` every 100ms to detect hotplug

**Event-driven alternative:** `WM_DISPLAYCHANGE` message (already handled!)
```rust
// Message loop automatically receives WM_DISPLAYCHANGE when:
// - Monitor added/removed
// - Resolution changed
// - Display orientation changed

case WM_DISPLAYCHANGE:
    handle_display_changed();  // Already implemented (line 181-186)
```

**Feasibility:** ⭐⭐⭐⭐⭐ **EASIEST** (5/5)
- **Already implemented in codebase!** (lines 181-186)
- No code changes needed
- Built-in Windows message

**Risk:** None - Standard Windows API

---

### 6. **Window Movement/Resize (Partial Dimming)**
**Current Implementation:** Lines 414-563 (complex drag detection logic)
```rust
if let Ok(current_rect) = window_manager.get_window_rect(active_window.handle) {
    let rect_changed = /* compare with last_rect */;
    if rect_changed {
        // Detect if dragging (< 150ms between changes)
        // Hide overlays during drag
        // Recreate after 200ms stability
    }
}
```

**Why it needs polling:**
- Detects window position changes via `DwmGetWindowAttribute` every 100ms
- Implements drag detection heuristic (rapid changes = dragging)
- Detects drag end (200ms without changes)

**Event-driven alternative:** `SetWinEventHook` with `EVENT_OBJECT_LOCATIONCHANGE`
```rust
SetWinEventHook(
    EVENT_OBJECT_LOCATIONCHANGE,  // Window moved/resized
    EVENT_OBJECT_LOCATIONCHANGE,
    ptr::null_mut(),
    Some(location_hook_callback),  // Already implemented (lines 127-145)
    0, 0, WINEVENT_OUTOFCONTEXT,
);
```

**Challenge:** Drag detection logic needs replacement
- **Polling approach:** Infers dragging from rapid rect changes (< 150ms apart)
- **Event approach:** `EVENT_OBJECT_LOCATIONCHANGE` fires constantly during drag

**Solutions:**
1. **Debouncing:** Use timer to delay overlay updates until events stop
2. **WM_ENTERSIZEMOVE/WM_EXITSIZEMOVE:** Detect drag start/end (requires subclassing target window - complex)
3. **Rate limiting:** Only update overlays every Nth event
4. **Remove drag detection:** Always update overlays (may cause flicker)

**Feasibility:** ⭐⭐⭐ **MEDIUM** (3/5)
- Hook infrastructure already exists
- Drag detection replacement is complex
- May need to sacrifice some UX polish

**Risk:** Medium-High - Could introduce flicker or performance issues

---

### 7. **Configuration File Watching**
**Current Implementation:** Lines 168-305 (60+ lines of complex logic)
```rust
// Check for config file changes every 2 seconds (20 iterations of 100ms)
if loop_counter % 20 == 0 {
    if let Some((new_config, new_modified_time)) = Config::reload_if_changed(last_config_modified) {
        // Complex logic to determine what changed
        // Recreate overlays with new colors
        // Enable/disable features
    }
}
```

**Why it needs polling:** Checks file modification time every 2 seconds

**Event-driven alternatives:**

**Option A: `FindFirstChangeNotification` + `WaitForMultipleObjects`**
```rust
let watch_handle = FindFirstChangeNotification(
    config_dir,
    FALSE,  // Don't watch subdirectories
    FILE_NOTIFY_CHANGE_LAST_WRITE,
);

// In message loop: Use MsgWaitForMultipleObjects instead of GetMessageW
MsgWaitForMultipleObjects(
    &[watch_handle],  // Also wait on file watcher
    INFINITE,
    QS_ALLINPUT,
);

if WaitForSingleObject(watch_handle, 0) == WAIT_OBJECT_0 {
    handle_config_changed();
    FindNextChangeNotification(watch_handle);
}
```

**Option B: `ReadDirectoryChangesW` (more complex, more control)**
**Option C: Keep 2-second polling (acceptable tradeoff)**

**Feasibility:** ⭐⭐ **HARD** (2/5)
- `FindFirstChangeNotification` is straightforward but requires `MsgWaitForMultipleObjects`
- Needs integration with message loop
- Config reload logic is already complex (60+ lines)
- 2-second delay is acceptable - not performance-critical

**Risk:** Medium - File watching can be finicky, especially with editors that use temp files

---

## Migration Roadmap (Ranked by Feasibility)

### Phase 1: Foundation (Low-Hanging Fruit)
**Estimated effort:** 2-4 hours

1. **✅ Display Hotplug Detection** (Feasibility: 5/5)
   - Already implemented via `WM_DISPLAYCHANGE`
   - Just needs polling code removal

2. **✅ Tray Icon Messages** (Feasibility: 5/5)
   - Replace `PeekMessageW` with `GetMessageW`
   - Instant CPU usage improvement

3. **✅ Exit Flag** (Feasibility: 5/5)
   - Handle `WM_QUIT` in message loop
   - Remove atomic flag polling

---

### Phase 2: Core Functionality (Medium Effort)
**Estimated effort:** 4-8 hours

4. **✅ Pause State** (Feasibility: 4/5)
   - Add `WM_USER_PAUSE_TOGGLED` message
   - Modify tray icon to post message
   - Remove atomic flag polling

5. **✅ Foreground Window Changes** (Feasibility: 4/5)
   - Enable `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)`
   - Already partially implemented
   - Remove `GetForegroundWindow` polling

---

### Phase 3: Complex Features (Higher Risk)
**Estimated effort:** 6-12 hours

6. **⚠️ Window Movement/Resize** (Feasibility: 3/5)
   - Enable `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE)`
   - **Decision required:** How to handle drag detection?
     - Option A: Debouncing (delay updates)
     - Option B: Rate limiting (every Nth event)
     - Option C: Remove drag optimization (accept flicker)
   - Test extensively with partial dimming enabled

7. **⚠️ Config File Watching** (Feasibility: 2/5)
   - **Recommendation:** Keep 2-second polling as acceptable compromise
   - Alternative: Implement `FindFirstChangeNotification` if needed
   - Not performance-critical (only checks every 2 seconds)

---

## Recommended Incremental Implementation Strategy

### Step 1: Preserve Existing Behavior
- Add event hooks **alongside** existing polling
- Both systems run in parallel
- Log which system triggers first
- Validate event hooks work correctly

### Step 2: Gradually Remove Polling
- Remove one polled feature at a time
- Test thoroughly after each removal
- Keep config file polling until last (lowest priority)

### Step 3: Optimize Message Loop
- Convert `PeekMessageW` to `GetMessageW`
- Remove all `thread::sleep` calls
- Measure CPU usage improvement

---

## Expected Performance Improvements

| Metric | Current (Polling) | Event-Driven | Improvement |
|--------|-------------------|--------------|-------------|
| CPU usage (idle) | ~0.3% (10 wake-ups/sec) | ~0% (0 wake-ups) | **100% reduction** |
| Focus change latency | 0-100ms | <1ms | **up to 100x faster** |
| Display hotplug latency | 0-100ms | Instant | **Instant** |
| Window move latency | 0-100ms | <10ms | **~10x faster** |
| Battery impact | Constant wake-ups | Event-driven | **Significant** |

---

## Risks and Mitigation

### Risk 1: Event Hook Callback Threading
- **Issue:** Callbacks run on different thread
- **Mitigation:** Use `PostMessageW` to marshal to main thread (already implemented)

### Risk 2: Drag Detection Complexity
- **Issue:** Polling-based heuristic hard to replicate
- **Mitigation:** Start with simple debouncing, iterate based on user feedback

### Risk 3: Message Loop Starvation
- **Issue:** Too many events could flood message queue
- **Mitigation:** Use `WM_USER_WINDOW_MOVED` coalescing (only post if not already queued)

### Risk 4: Config File Watcher Reliability
- **Issue:** Some editors use temp files, causing multiple notifications
- **Mitigation:** Keep 2-second polling OR add debouncing to file watcher

---

## Conclusion

**Recommended approach:** **Incremental migration starting with Phase 1 & 2**

- **Phase 1** provides immediate CPU savings with minimal risk
- **Phase 2** delivers most user-visible improvements (responsiveness)
- **Phase 3** can be deferred or partially implemented based on testing

**Total estimated effort:** 12-24 hours for complete migration
**Quickwin effort:** 2-4 hours for Phase 1 (60% of the benefit)

---

## Next Steps

1. ✅ **Create this migration report**
2. ⏭️ Implement Phase 1 (display hotplug + message loop)
3. ⏭️ Test CPU usage and responsiveness improvements
4. ⏭️ Implement Phase 2 (foreground changes + pause state)
5. ⏭️ Evaluate Phase 3 based on user feedback
