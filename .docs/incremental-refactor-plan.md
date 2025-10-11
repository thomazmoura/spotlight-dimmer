# Incremental Refactor Plan: Polling to Event-Driven

**Date:** 2025-10-10
**Status:** Planning Phase
**Related:** See `polling-to-events-migration-report.md` for detailed analysis

---

## Lessons Learned from Initial Attempt

### What Went Wrong

1. **Attempted too much at once**: Tried to add event infrastructure (message window, callbacks, global state) while keeping the old polling loop
2. **Threading safety issues**: `HWND` is a raw pointer (`*mut HWND__`) that doesn't implement `Send`, causing `lazy_static` compilation errors
3. **Code conflicts**: Had both old and new systems partially implemented, causing conflicts

### Key Insights

- **HWND threading**: Windows handles are thread-affine and require careful handling in Rust's ownership system
- **Atomic operations**: Need to carefully marshal data between Windows callback threads and main thread
- **State management**: Current polling loop has complex state (last_window_rect, is_dragging, etc.) that can't be easily shared

---

## Revised Incremental Strategy

### Principle: "Keep it working at every step"

Each phase should:
1. ✅ Compile without errors
2. ✅ Run without crashes
3. ✅ Maintain all existing functionality
4. ✅ Add measurable improvement OR preparation for next step
5. ✅ Be independently testable

---

## Phase 0: Baseline & Cleanup ✅ COMPLETED

**Goal:** Establish clean baseline with working code

**Changes:**
- ✅ Removed partial event infrastructure that was causing compilation errors
- ✅ Kept `process_windows_messages()` helper function for tray icon
- ✅ Code compiles and runs correctly
- ✅ All existing functionality preserved

**Files Modified:**
- `src/main_new.rs`: Commented out event hooks, removed unused imports

---

## Phase 1: Message Loop Optimization (NEXT)

**Goal:** Improve message processing efficiency without changing architecture

**Estimated Effort:** 1-2 hours
**Risk:** Low

### Changes

1. **Add adaptive sleep**
   - Current: Fixed 100ms sleep
   - New: Variable sleep based on activity (50ms active, 200ms idle)
   - Benefit: Slightly better responsiveness when active, lower CPU when idle

2. **Optimize message processing**
   - Current: `PeekMessageW` processes all pending messages
   - Keep: Same approach, but add timing measurement
   - Add: Debug logging to understand message frequency

3. **Document findings**
   - Measure: Message processing frequency
   - Measure: CPU usage in different scenarios
   - Output: Baseline metrics for comparison

### Implementation Steps

1. Add message processing counter and timer
2. Log message frequency every 10 seconds
3. Implement adaptive sleep based on activity
4. Test and measure CPU improvement

### Success Criteria

- ✅ Code compiles and runs
- ✅ No functionality regression
- ✅ Baseline metrics documented
- ✅ Slight CPU improvement (5-10%)

---

## Phase 2: Message Window Infrastructure

**Goal:** Add message-only window for receiving custom messages

**Estimated Effort:** 3-4 hours
**Risk:** Medium

### Pre-requisites

- Phase 1 completed and tested
- Understanding of HWND threading model

### Changes

1. **Create thread-safe HWND wrapper**
   ```rust
   struct MessageWindowHandle(usize);  // Store HWND as usize for Send
   unsafe impl Send for MessageWindowHandle {}

   impl MessageWindowHandle {
       fn as_hwnd(&self) -> HWND {
           self.0 as HWND
       }
   }
   ```

2. **Create message-only window**
   - Register window class
   - Create window with `HWND_MESSAGE`
   - Store handle in thread-safe wrapper

3. **Add custom message constants**
   ```rust
   const WM_USER_TEST: UINT = 0x0400;
   ```

4. **Test message delivery**
   - Post test message from main loop
   - Verify it's received in window procedure
   - Ensure no deadlocks or crashes

### Implementation Steps

1. Create `MessageWindowHandle` wrapper struct
2. Implement `create_message_window()` function
3. Add window procedure with logging
4. Test message posting and receipt
5. Verify no memory leaks

### Success Criteria

- ✅ Message window creates successfully
- ✅ Can post and receive custom messages
- ✅ No threading issues or deadlocks
- ✅ Window properly cleaned up on exit

---

## Phase 3: WM_DISPLAYCHANGE Event Hook

**Goal:** Replace display count polling with event-based detection

**Estimated Effort:** 2-3 hours
**Risk:** Low (simplest event)

### Pre-requisites

- Phase 2 completed (message window working)

### Changes

1. **Handle WM_DISPLAYCHANGE in window procedure**
   ```rust
   WM_DISPLAYCHANGE => {
       println!("[Event] Display change detected");
       // Set flag or post custom message
       0
   }
   ```

2. **Remove display count polling from main loop**
   - Keep the handling code
   - Just remove the periodic check
   - React to events instead

3. **Test hotplug scenarios**
   - Connect/disconnect monitor
   - Change resolution
   - Rotate display

### Implementation Steps

1. Add `WM_DISPLAYCHANGE` handler to window procedure
2. Keep existing display handling code
3. Add flag to track when display changed
4. Check flag in main loop instead of polling
5. Test thoroughly with monitor hotplug

### Success Criteria

- ✅ Instant detection of display changes (no 100ms delay)
- ✅ Overlays recreate correctly
- ✅ No polling for display count
- ✅ Measurable latency improvement

---

## Phase 4: EVENT_SYSTEM_FOREGROUND Hook

**Goal:** Replace foreground window polling with event hook

**Estimated Effort:** 4-6 hours
**Risk:** Medium (callback thread complexity)

### Pre-requisites

- Phase 3 completed and stable
- Understanding of `SetWinEventHook` API

### Changes

1. **Add event hook callback**
   ```rust
   unsafe extern "system" fn foreground_callback(...) {
       // Must use PostMessageW to marshal to main thread
       // Cannot directly access Rust state here
       PostMessageW(global_hwnd, WM_USER_FOREGROUND_CHANGED, 0, 0);
   }
   ```

2. **Install hook**
   ```rust
   SetWinEventHook(
       EVENT_SYSTEM_FOREGROUND,
       EVENT_SYSTEM_FOREGROUND,
       ptr::null_mut(),
       Some(foreground_callback),
       0, 0,
       WINEVENT_OUTOFCONTEXT,
   );
   ```

3. **Handle custom message**
   - Get foreground window when message received
   - Run existing focus change logic
   - Remove `GetForegroundWindow` polling

### Implementation Steps

1. Create callback function with logging only
2. Install hook and verify callback fires
3. Post custom message from callback
4. Handle message in main thread
5. Move focus change logic to message handler
6. Remove polling code
7. Extensive testing

### Success Criteria

- ✅ Instant focus change detection (<1ms vs 0-100ms)
- ✅ No polling for active window
- ✅ No crashes or threading issues
- ✅ Correct behavior with rapid window switching

---

## Phase 5: EVENT_OBJECT_LOCATIONCHANGE Hook

**Goal:** Replace window rect polling with event hook

**Estimated Effort:** 6-8 hours
**Risk:** High (complex drag detection logic)

### Pre-requisites

- Phase 4 completed and stable
- Decision on drag detection approach

### Challenges

1. **Drag detection complexity**
   - Current: Polling-based heuristic (rapid changes <150ms = dragging)
   - Event-based: Receives constant stream during drag
   - Need: Debouncing or rate limiting

2. **Event frequency**
   - During drag: Hundreds of events per second
   - Risk: Message queue flooding
   - Solution: Coalesce or rate-limit events

### Proposed Approach

**Option A: Debouncing with timer**
```rust
static LAST_MOVE_EVENT: Atomic<Instant> = ...;
const DEBOUNCE_MS: u64 = 16;  // ~60 FPS

fn location_callback() {
    let now = Instant::now();
    let last = LAST_MOVE_EVENT.load();
    if now.duration_since(last) > DEBOUNCE_MS {
        PostMessageW(hwnd, WM_USER_WINDOW_MOVED, 0, 0);
        LAST_MOVE_EVENT.store(now);
    }
}
```

**Option B: Message coalescing**
- Only post message if no `WM_USER_WINDOW_MOVED` already queued
- Use `PeekMessage` with `PM_NOREMOVE` to check
- Prevents queue flooding

**Option C: Remove drag detection**
- Simplify: Always update overlays
- Accept: Potential flicker during drag
- Test: May be acceptable with fast updates

### Implementation Steps

1. Install `EVENT_OBJECT_LOCATIONCHANGE` hook
2. Add callback with logging only
3. Measure event frequency during drag
4. Implement chosen approach (start with Option B)
5. Test with various window sizes and drag speeds
6. Remove rect polling code
7. Extensive testing

### Success Criteria

- ✅ Real-time partial overlay updates
- ✅ No significant flicker during drag
- ✅ Acceptable CPU usage (<5% during drag)
- ✅ No message queue flooding
- ✅ Works with all window types

---

## Phase 6: Config File Watching (Optional)

**Goal:** Replace 2-second config polling with file watcher

**Estimated Effort:** 4-6 hours
**Risk:** Medium (file watcher complexity)

### Decision Point

**Option A: Keep polling**
- Current 2-second interval is acceptable
- Config changes are rare
- Low risk, no complexity

**Option B: Add file watcher**
- Use `FindFirstChangeNotification`
- Integrate with `MsgWaitForMultipleObjects`
- Immediate config reload

### Recommendation

**Defer to later** - 2-second polling is acceptable and not a performance issue. Focus on the user-visible improvements first (focus changes, window movement, display hotplug).

---

## Phase 7: Full Event Loop Migration

**Goal:** Replace polling loop with blocking `GetMessageW`

**Estimated Effort:** 3-4 hours
**Risk:** Medium

### Pre-requisites

- Phases 3-5 completed (all events working)
- No remaining polling dependencies

### Changes

Replace:
```rust
loop {
    process_windows_messages();
    thread::sleep(Duration::from_millis(100));
    // ... polling checks ...
}
```

With:
```rust
loop {
    let mut msg: MSG = zeroed();
    let result = GetMessageW(&mut msg, null_mut(), 0, 0);

    if result <= 0 {
        break;  // WM_QUIT or error
    }

    TranslateMessage(&msg);
    DispatchMessageW(&msg);

    // No sleep needed - blocks until message arrives
}
```

### Success Criteria

- ✅ Zero CPU usage when idle (no wake-ups)
- ✅ Instant response to all events
- ✅ Clean shutdown via WM_QUIT
- ✅ No polling loops remain

---

## Testing Strategy

### For Each Phase

1. **Unit Testing**
   - Compile without warnings
   - No memory leaks (test with long runs)
   - Proper resource cleanup

2. **Integration Testing**
   - All existing features work
   - New event-driven behavior correct
   - Edge cases handled

3. **Performance Testing**
   - CPU usage measurement
   - Response latency measurement
   - Memory usage tracking

4. **Stress Testing**
   - Rapid window switching
   - Fast dragging
   - Monitor hotplug scenarios
   - Config file changes

### Acceptance Criteria (Final)

- ✅ CPU usage: ~0% idle (vs current ~0.3%)
- ✅ Focus change latency: <1ms (vs 0-100ms)
- ✅ Display hotplug: Instant (vs 0-100ms)
- ✅ Window move: <16ms (vs 0-100ms)
- ✅ No functionality regression
- ✅ No crashes or memory leaks
- ✅ Clean shutdown

---

## Rollback Strategy

Each phase should be in a separate Git commit with:
- Clear commit message describing changes
- Before/after metrics
- Known issues (if any)

If a phase introduces issues:
1. `git revert` the commit
2. Document what went wrong
3. Revise approach
4. Try again

---

## Current Status

- ✅ Phase 0: Baseline established
- ⏸️ Phase 1: Ready to start (message loop optimization)
- ⏳ Phase 2-7: Waiting for Phase 1 completion

---

## Next Steps

1. Complete Phase 1 (message loop optimization)
2. Document baseline metrics
3. Begin Phase 2 (message window infrastructure)

---

## Notes

- This plan is iterative - adjust based on learnings
- Don't skip phases even if tempting
- Test thoroughly at each step
- Document findings and metrics
- Keep migration report up-to-date
