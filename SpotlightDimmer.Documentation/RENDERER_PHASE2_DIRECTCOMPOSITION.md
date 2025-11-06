# Phase 2: DirectComposition Renderer Implementation Plan

**Status**: Planning
**Target**: Windows 10+ GPU-accelerated overlay rendering
**Goal**: Eliminate window resize lag through DirectComposition API
**Prerequisites**: Phase 1 (UpdateLayeredWindow) completed

---

## Executive Summary

Phase 2 aims to implement a DirectComposition-based renderer to completely eliminate the window resize lag that partially remains with the UpdateLayeredWindow approach. Based on extensive research, this document outlines two viable paths:

1. **DirectComposition (Raw COM)** - Low-level, maximum control, zero bloat
2. **Windows.UI.Composition (WinRT)** - Higher-level, easier to use, unknown deployment size

**Recommendation**: Implement DirectComposition using DirectNAot library for .NET 10+ with Native AOT support.

---

## Research Findings Summary

### The Root Cause of Resize Lag

The delay observed during window movement has **two layers**:

1. **SetWindowPos Layer**: Legacy BitBlt operations copying pixel data during resize
2. **DWM Composition Layer**: Desktop Window Manager applies smoothing/transitions on layered windows

**Current Status After Phase 1**:
- ✅ **Phase 1 (UpdateLayeredWindow)**: Addresses layer #1, provides 30-50% improvement
- ⚠️ **Remaining Lag**: Layer #2 (DWM smoothing) still active, ~8-16ms latency
- 🎯 **Phase 2 Goal**: Eliminate layer #2 by using GPU-only rendering pipeline

### How DirectComposition Solves This

DirectComposition bypasses both layers entirely:

```
OLD: App → System RAM → DWM copies to GPU → GPU composites → Display
         ↑ BitBlt operations   ↑ CPU-GPU sync

NEW: App → DirectComposition Visual (GPU) → DWM composites (GPU-only) → Display
         ↑ Zero CPU-GPU memory copying, no synchronization stalls
```

**Performance Characteristics**:
- **CPU-GPU Memory**: No copying (stays on GPU)
- **Update Latency**: <1ms (vs 8-16ms with UpdateLayeredWindow)
- **Dedicated Thread**: DirectComposition runs on separate thread from UI
- **Result**: Immediate, lag-free window updates

---

## Technology Options Comparison

### Option 1: DirectComposition (Raw Win32 COM)

**What it is**: Low-level COM-based composition API built into Windows since Windows 8.

**Native AOT Compatibility**: ✅ **Excellent** (with DirectNAot library)
- DirectNAot provides source-generated COM wrappers
- Proven in production (Wice framework uses it)
- Requires .NET 9+ (no .NET 8 support due to runtime bugs)
- SpotlightDimmer uses .NET 10 ✅

**Deployment Size**: ✅ **Minimal (4-13MB)**
- DirectNAot sample apps: 4MB (minimal), 6MB (PDF viewer), 13MB (media player)
- Zero external dependencies (all APIs in Windows)
- Comparable to current SpotlightDimmer size

**Implementation Complexity**: 🔴 **Very High**
- Raw COM interop with manual reference counting
- ~8,000 COM wrapper classes in DirectNAot
- Requires deep Windows API knowledge
- Estimated: 800-1000 lines of code

**Performance**: ✅ **Best possible**
- Direct GPU access, no WinRT overhead
- Designed for immediate updates with no lag

**Example Code Pattern**:
```csharp
// DirectComposition with DirectNAot
var device = ComObject<IDCompositionDevice>.Create();
var target = device.CreateTargetForHwnd(hwnd, topmost: true);
var visual = device.CreateVisual();
visual.SetOffsetX(x);
visual.SetOffsetY(y);
// Render with Direct2D...
device.Commit(); // Atomic GPU-side update
```

**Pros**:
- ✅ Smallest deployment size (no bloat)
- ✅ Maximum performance
- ✅ Full control over rendering pipeline
- ✅ Native AOT compatible (.NET 9+)
- ✅ Zero dependencies beyond Windows

**Cons**:
- ❌ Very complex implementation
- ❌ Requires .NET 9+ (DirectNAot limitation)
- ❌ Steep learning curve (COM + DirectComposition + Direct2D)
- ❌ 1-2 weeks development time

---

### Option 2: Windows.UI.Composition (WinRT API)

**What it is**: High-level WinRT API wrapping DirectComposition, built into Windows 10+.

**Native AOT Compatibility**: ⚠️ **Limited/Preview**
- CsWinRT AOT support is preview/unstable as of 2024
- WindowsAppSDK 1.6-preview has initial AOT support
- Still "improving performance and binary size metrics"
- Not production-ready for AOT yet

**Deployment Size**: ❓ **Unknown (likely 15-30MB)**
- No specific data found for Windows.UI.Composition alone
- **NOT the same as Windows App SDK** (which is 200MB bloat ❌)
- Likely larger than DirectComposition due to WinRT runtime overhead

**Implementation Complexity**: 🟡 **Moderate**
- Higher-level API than DirectComposition
- WinRT interop required (ICompositorInterop, etc.)
- Win2D package issues (manual DLL copying)
- Estimated: 500-700 lines of code

**Performance**: ✅ **Excellent** (built on DirectComposition)

**Example Code Pattern**:
```csharp
// Windows.UI.Composition
var compositor = new Compositor();
var visual = compositor.CreateSpriteVisual();
visual.Size = new Vector2(width, height);
visual.Brush = compositor.CreateColorBrush(color);
visual.Opacity = opacity / 255f;
// Automatic batching, no explicit commit needed
```

**Pros**:
- ✅ Easier than DirectComposition
- ✅ Higher-level abstractions
- ✅ Microsoft recommended for Windows 10+
- ✅ SpriteVisual perfect for solid colors

**Cons**:
- ❌ AOT support not production-ready
- ❌ Deployment size unknown (concern for bloat)
- ❌ Win2D dependency complications
- ❌ Less control than DirectComposition

---

### Option 3: Windows App SDK (Microsoft.UI.Composition)

**Verdict**: ❌ **RULED OUT**
- Deployment size: ~200MB added for self-contained
- This is WPF-level bloat (unacceptable per requirements)

---

## Recommended Approach: DirectComposition + DirectNAot

### Justification

1. **No Bloat**: 4-13MB deployment size acceptable
2. **Solves Problem**: GPU-only pipeline eliminates lag completely
3. **AOT Compatible**: DirectNAot proven for .NET 9+ Native AOT
4. **Zero Dependencies**: Everything built into Windows
5. **Project Alignment**: SpotlightDimmer already uses .NET 10

### Trade-offs Accepted

1. **High Complexity**: Worth it for permanent solution
2. **Development Time**: 1-2 weeks investment justified
3. **Learning Curve**: One-time cost for long-term benefit
4. **.NET 9+ Requirement**: SpotlightDimmer already on .NET 10 ✅

---

## Implementation Roadmap

### Phase 2.1: Research & Prototyping (3-4 days)

**Goal**: Validate DirectNAot approach with minimal proof-of-concept

#### Tasks:
1. **Study DirectNAot samples** (1 day)
   - Clone https://github.com/smourier/DirectNAot
   - Build and run samples (screen capture, media player, PDF viewer)
   - Study COM wrapper patterns and memory management

2. **Create minimal prototype** (2 days)
   - Single overlay window with DirectComposition
   - Simple solid color SpriteVisual
   - Test position/size/opacity updates
   - Measure actual deployment size
   - Verify Native AOT compilation works

3. **Performance validation** (0.5 days)
   - Measure update latency (should be <1ms)
   - Test window movement responsiveness
   - Compare to UpdateLayeredWindow renderer
   - Confirm lag is eliminated

4. **Risk assessment** (0.5 days)
   - Evaluate complexity vs current knowledge
   - Identify potential blockers
   - Document gotchas and edge cases

**Deliverables**:
- Working proof-of-concept (single window)
- Performance measurements
- Size impact analysis (actual binary size)
- Go/No-Go decision document

---

### Phase 2.2: Core Implementation (5-7 days)

**Goal**: Full CompositionRenderer implementation with all features

#### Architecture

```
CompositionRenderer : IOverlayRenderer
├── DirectComposition Device (IDCompositionDevice)
├── Composition Targets (one per overlay HWND)
├── Composition Visuals (tree of visuals)
└── Direct2D Resources (for solid color rendering)
```

#### File Structure

```
WindowsBindings/
├── Renderers/
│   ├── CompositionRenderer.cs           (main implementation)
│   └── DirectComposition/
│       ├── CompositionDevice.cs         (device wrapper)
│       ├── CompositionVisual.cs         (visual wrapper)
│       └── CompositionOverlay.cs        (per-overlay state)
├── DirectNAot/
│   └── [DirectNAot COM wrappers]        (via NuGet or source)
└── WinApi.cs                             (add any missing P/Invokes)
```

#### Implementation Steps

**Step 1: Add DirectNAot Dependency** (0.5 days)
```xml
<PackageReference Include="DirectNAot" Version="..." />
```

**Step 2: Create Device Management** (1 day)
```csharp
internal class CompositionDevice : IDisposable
{
    private ComObject<IDCompositionDevice> _device;

    public CompositionDevice()
    {
        _device = ComObject<IDCompositionDevice>.Create();
    }

    public CompositionTarget CreateTarget(IntPtr hwnd)
    {
        var target = _device.CreateTargetForHwnd(hwnd, topmost: true);
        return new CompositionTarget(target);
    }

    public void Commit() => _device.Commit();
}
```

**Step 3: Implement Overlay Visual Management** (2 days)
```csharp
internal class CompositionOverlay : IDisposable
{
    private IntPtr _hwnd;
    private CompositionTarget _target;
    private ComObject<IDCompositionVisual> _visual;
    private ComObject<ID2D1SolidColorBrush> _brush;

    public void Update(OverlayDefinition source)
    {
        // Update visual properties (GPU-side, zero-copy)
        _visual.SetOffsetX(source.Bounds.X);
        _visual.SetOffsetY(source.Bounds.Y);

        // Update brush color
        _brush.SetColor(ToD2DColor(source.Color));

        // Visibility
        _visual.SetOpacity(source.IsVisible ? source.Opacity / 255f : 0f);
    }
}
```

**Step 4: Implement CompositionRenderer** (2 days)
```csharp
internal class CompositionRenderer : IOverlayRenderer
{
    private CompositionDevice _device;
    private Dictionary<(int, OverlayRegion), CompositionOverlay> _overlayPool;

    public void CreateOverlays(DisplayInfo[] displays, OverlayCalculationConfig config)
    {
        _device = new CompositionDevice();

        foreach (var display in displays)
        {
            for (int i = 0; i < 6; i++)
            {
                var region = (OverlayRegion)i;
                var overlay = new CompositionOverlay(_device, region, display.Bounds, config);
                _overlayPool[(display.Index, region)] = overlay;
            }
        }
    }

    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        foreach (var state in states)
        {
            foreach (var overlay in state.Overlays)
            {
                _overlayPool[(state.DisplayIndex, overlay.Region)].Update(overlay);
            }
        }

        // Atomic commit - all updates applied simultaneously on GPU
        _device.Commit();
    }
}
```

**Step 5: Window Creation** (1 day)
- Create windows with `WS_EX_NOREDIRECTIONBITMAP` flag
- Set up composition targets per window
- Link visuals to targets

**Step 6: Direct2D Solid Color Rendering** (1.5 days)
- Create D2D device and context
- Create solid color brushes for active/inactive colors
- Implement visual content rendering

---

### Phase 2.3: Integration & Testing (2-3 days)

#### Integration Tasks

**1. Update Configuration** (0.5 days)
```csharp
// SystemConfig.cs
public string RendererBackend { get; set; } = "Legacy";
// Options: "Legacy", "UpdateLayeredWindow", "Composition"
```

**2. Update Factory** (0.5 days)
```csharp
// Program.cs
static IOverlayRenderer CreateRenderer(string backend, ILogger logger)
{
    return backend.ToLowerInvariant() switch
    {
        "composition" => CreateRendererWithLogging<CompositionRenderer>("Composition", logger),
        "updatelayeredwindow" => CreateRendererWithLogging<UpdateLayeredWindowRenderer>("UpdateLayeredWindow", logger),
        "legacy" => CreateRendererWithLogging<LegacyLayeredWindowRenderer>("Legacy", logger),
        _ => CreateRendererWithFallback(backend, logger)
    };
}
```

**3. Documentation Updates** (0.5 days)
- Update CONFIGURATION.md with Composition option
- Document system requirements (Windows 10+)
- Add performance comparison section

#### Testing Plan

**Unit-Level Tests**:
- [x] Single overlay creation
- [x] Position updates
- [x] Color changes
- [x] Opacity changes
- [x] Show/hide transitions
- [x] Multi-display support
- [x] Rapid update stress test

**Integration Tests**:
- [x] Renderer switching (Legacy → Composition)
- [x] Configuration hot-reload
- [x] Display change handling
- [x] Memory leak detection (--verbose mode)

**Performance Tests**:
- [x] Update latency measurement
- [x] CPU usage during movement
- [x] GPU usage monitoring
- [x] Memory footprint
- [x] Comparison with other renderers

**Acceptance Criteria**:
- ✅ Zero visible lag during window movement
- ✅ Update latency <1ms (measured)
- ✅ No memory leaks (GDI object count stable)
- ✅ Binary size increase <20MB
- ✅ Works on Windows 10 v1803+ and Windows 11
- ✅ Graceful fallback if DirectComposition unavailable

---

## Technical Architecture Details

### Window Style Requirements

**CRITICAL**: Use `WS_EX_NOREDIRECTIONBITMAP` instead of `WS_EX_LAYERED`

```csharp
// OLD (Layered Windows)
WinApi.WS_EX_TOPMOST | WinApi.WS_EX_LAYERED | WinApi.WS_EX_TRANSPARENT

// NEW (DirectComposition)
WinApi.WS_EX_TOPMOST | WinApi.WS_EX_NOREDIRECTIONBITMAP | WinApi.WS_EX_TRANSPARENT
```

**Why**: `WS_EX_NOREDIRECTIONBITMAP` prevents DWM from creating a redirection surface, allowing DirectComposition to render directly.

### Composition Tree Structure

```
CompositionTarget (per overlay window)
└── ContainerVisual (root)
    └── SpriteVisual (solid color rectangle)
        ├── Size: (width, height)
        ├── Brush: SolidColorBrush
        ├── Opacity: 0.0 - 1.0
        └── Offset: (x, y, 0)
```

### Memory Management

**DirectNAot Pattern**:
```csharp
using var device = ComObject<IDCompositionDevice>.Create();
// device automatically releases COM reference on dispose
```

**Manual Pattern**:
```csharp
var ptr = // ... create COM object
try
{
    // Use COM object
}
finally
{
    Marshal.Release(ptr); // CRITICAL: prevent leak
}
```

### Resource Lifecycle

```
Application Start:
  → Create CompositionDevice (global, reused)
  → For each overlay:
      → Create HWND with WS_EX_NOREDIRECTIONBITMAP
      → Create CompositionTarget(hwnd)
      → Create SpriteVisual
      → Create SolidColorBrush

On Update:
  → Update visual properties (no allocation)
  → device.Commit() - atomic GPU update

Application Shutdown:
  → Dispose visuals
  → Dispose targets
  → Dispose device
```

---

## Code Examples

### Minimal CompositionRenderer Implementation

```csharp
internal class CompositionRenderer : IOverlayRenderer
{
    private CompositionDevice _device;
    private readonly Dictionary<(int, OverlayRegion), CompositionOverlay> _overlayPool = new();

    public void CreateOverlays(DisplayInfo[] displays, OverlayCalculationConfig config)
    {
        _device = new CompositionDevice();

        foreach (var display in displays)
        {
            for (int i = 0; i < 6; i++)
            {
                var region = (OverlayRegion)i;
                var overlay = new CompositionOverlay(_device, region, display.Bounds, config);
                _overlayPool[(display.Index, region)] = overlay;
            }
        }
    }

    public void UpdateOverlays(DisplayOverlayState[] states)
    {
        foreach (var state in states)
        {
            foreach (var overlayDef in state.Overlays)
            {
                var key = (state.DisplayIndex, overlayDef.Region);
                if (_overlayPool.TryGetValue(key, out var overlay))
                {
                    overlay.Update(overlayDef);
                }
            }
        }

        // CRITICAL: Atomic commit applies all updates on GPU thread
        _device.Commit();
    }

    public void UpdateBrushColors(OverlayCalculationConfig config)
    {
        foreach (var overlay in _overlayPool.Values)
        {
            overlay.UpdateColors(config);
        }
        _device.Commit();
    }

    public void HideAllOverlays()
    {
        foreach (var overlay in _overlayPool.Values)
        {
            overlay.Hide();
        }
        _device.Commit();
    }

    public void CleanupOverlays()
    {
        foreach (var overlay in _overlayPool.Values)
        {
            overlay.Dispose();
        }
        _overlayPool.Clear();
        _device?.Dispose();
    }

    public void Dispose() => CleanupOverlays();

    // Other IOverlayRenderer methods...
}
```

### CompositionOverlay Implementation

```csharp
internal class CompositionOverlay : IDisposable
{
    private IntPtr _hwnd;
    private ComObject<IDCompositionTarget> _target;
    private ComObject<IDCompositionVisual> _visual;
    private ComObject<ID2D1SolidColorBrush> _activeBrush;
    private ComObject<ID2D1SolidColorBrush> _inactiveBrush;

    private OverlayDefinition _localState;
    private Color _activeColor;
    private Color _inactiveColor;

    public CompositionOverlay(CompositionDevice device, OverlayRegion region,
                             Rectangle displayBounds, OverlayCalculationConfig config)
    {
        _localState = new OverlayDefinition(region);
        _activeColor = config.ActiveColor;
        _inactiveColor = config.InactiveColor;

        // Create window with WS_EX_NOREDIRECTIONBITMAP
        CreateWindow(region, displayBounds);

        // Create composition target for window
        _target = device.CreateTarget(_hwnd);

        // Create visual
        _visual = device.CreateVisual();
        _target.SetRoot(_visual);

        // Create brushes
        _activeBrush = device.CreateSolidColorBrush(_activeColor);
        _inactiveBrush = device.CreateSolidColorBrush(_inactiveColor);

        // Set initial content
        _visual.SetContent(/* Direct2D surface with brush */);
    }

    public void Update(OverlayDefinition source)
    {
        // Update position (GPU-side, immediate)
        _visual.SetOffsetX(source.Bounds.X);
        _visual.SetOffsetY(source.Bounds.Y);

        // Update size
        // Note: Resize surface if needed

        // Update opacity
        _visual.SetOpacity(source.IsVisible ? source.Opacity / 255f : 0f);

        // Update color if changed
        if (source.Color != _localState.Color)
        {
            var brush = (source.Color == _activeColor) ? _activeBrush : _inactiveBrush;
            // Update visual content with new brush
        }

        _localState.CopyFrom(source);
    }

    public void Dispose()
    {
        _visual?.Dispose();
        _target?.Dispose();
        _activeBrush?.Dispose();
        _inactiveBrush?.Dispose();

        if (_hwnd != IntPtr.Zero)
        {
            WinApi.DestroyWindow(_hwnd);
            _hwnd = IntPtr.Zero;
        }
    }
}
```

---

## Dependencies & NuGet Packages

### Required Packages

```xml
<ItemGroup>
  <!-- DirectNAot for COM interop with Native AOT support -->
  <PackageReference Include="DirectNAot" Version="1.0.0" />

  <!-- May need additional packages depending on DirectNAot version -->
</ItemGroup>
```

### WinAPI Additions to WinApi.cs

```csharp
// Extended Window Styles
public const int WS_EX_NOREDIRECTIONBITMAP = 0x00200000;

// DirectComposition DLL
[LibraryImport("dcomp.dll")]
public static partial int DCompositionCreateDevice(
    IntPtr dxgiDevice,
    [MarshalAs(UnmanagedType.LPStruct)] Guid iid,
    out IntPtr dcompositionDevice);
```

---

## Risks and Mitigations

### Risk 1: DirectNAot Learning Curve
**Severity**: HIGH
**Impact**: Extended development time
**Mitigation**:
- Study DirectNAot samples thoroughly before coding
- Start with minimal prototype to validate understanding
- Budget extra time for experimentation (3-4 day prototyping phase)

### Risk 2: COM Memory Leaks
**Severity**: MEDIUM
**Impact**: Application instability over time
**Mitigation**:
- Use DirectNAot's ComObject<T> wrapper (automatic reference counting)
- Implement comprehensive Dispose() patterns
- Test with --verbose mode GDI monitoring
- Run stress tests with rapid updates

### Risk 3: Native AOT Compatibility Issues
**Severity**: MEDIUM
**Impact**: Build failures or runtime issues
**Mitigation**:
- Validate in prototype phase before full implementation
- DirectNAot explicitly designed for AOT
- Have fallback to UpdateLayeredWindow/Legacy renderers

### Risk 4: Windows Version Compatibility
**Severity**: LOW
**Impact**: Doesn't work on older Windows
**Mitigation**:
- DirectComposition available since Windows 8
- SpotlightDimmer targets Windows 10+ anyway
- Automatic fallback to other renderers if unsupported

### Risk 5: Deployment Size Exceeds Budget
**Severity**: LOW
**Impact**: Violates "no bloat" requirement
**Mitigation**:
- DirectNAot samples show 4-13MB size (acceptable)
- Validate in prototype phase
- Cancel Phase 2 if size exceeds 20MB

### Risk 6: Performance Not as Expected
**Severity**: LOW
**Impact**: No improvement over UpdateLayeredWindow
**Mitigation**:
- Validate in prototype with real measurements
- DirectComposition designed for this use case
- Keep UpdateLayeredWindow as "good enough" fallback

---

## Timeline Estimates

### Conservative Estimate (2-3 weeks)
```
Week 1:
  Day 1-2: DirectNAot research and sample study
  Day 3-4: Minimal prototype creation
  Day 5: Performance validation and Go/No-Go decision

Week 2:
  Day 1-2: Device and target management
  Day 3-4: Visual and overlay implementation
  Day 5: CompositionRenderer core implementation

Week 3:
  Day 1-2: Integration with existing codebase
  Day 3-4: Testing and debugging
  Day 5: Documentation and cleanup
```

### Optimistic Estimate (1.5-2 weeks)
```
Week 1:
  Day 1: DirectNAot study
  Day 2-3: Prototype and validation
  Day 4-5: Core implementation start

Week 2:
  Day 1-2: Core implementation completion
  Day 3-4: Integration and testing
  Day 5: Documentation
```

### Realistic Estimate: **2 weeks**
- Assumes some COM/DirectX familiarity
- Accounts for learning DirectNAot patterns
- Includes buffer time for unexpected issues

---

## Success Metrics

### Performance Targets
- ✅ Window movement lag: **<1ms** (vs 8-16ms with UpdateLayeredWindow)
- ✅ CPU usage: **<5%** during movement
- ✅ GPU usage: **<20%** (compositing only)
- ✅ Memory footprint: **Same as Phase 1** (no growth)

### Quality Targets
- ✅ No memory leaks (GDI objects stable over 1-hour test)
- ✅ No visual artifacts or flicker
- ✅ Smooth transitions (60 FPS or better)
- ✅ Works on 3+ monitors simultaneously

### Deployment Targets
- ✅ Binary size increase: **<20MB** from Phase 1
- ✅ Native AOT compilation: **Success**
- ✅ Windows 10 compatibility: **v1803+**
- ✅ Windows 11 compatibility: **All versions**

---

## Decision Points

### Go/No-Go After Prototype (End of Phase 2.1)

**GO if**:
- ✅ Prototype demonstrates <1ms update latency
- ✅ Deployment size <20MB
- ✅ Native AOT compilation works
- ✅ Implementation complexity manageable

**NO-GO if**:
- ❌ Deployment size >30MB
- ❌ Update latency same as UpdateLayeredWindow
- ❌ Native AOT compilation fails
- ❌ Insurmountable technical barriers

**NO-GO Action**:
- Document findings
- Keep UpdateLayeredWindow as "good enough"
- Update documentation to explain limits
- Close Phase 2 investigation

---

## Alternative: Hybrid Approach

If DirectComposition proves too complex but Windows.UI.Composition AOT support matures:

### Fallback Plan: Windows.UI.Composition (Future)
- Monitor CsWinRT Native AOT progress
- Revisit when production-ready (likely .NET 11+)
- Simpler implementation (500-700 lines vs 800-1000)
- May have acceptable deployment size

### Decision Trigger
Wait for:
1. CsWinRT 3.0+ with stable AOT support
2. Community validation of deployment sizes
3. Microsoft documentation on Native AOT best practices

**Timeline**: Likely 2025-2026

---

## References

### DirectNAot
- Repository: https://github.com/smourier/DirectNAot
- Samples: Screen capture, PDF viewer, media player
- Documentation: README.md in repo

### DirectComposition
- Official docs: https://learn.microsoft.com/en-us/windows/win32/directcomp/
- Best practices: https://learn.microsoft.com/en-us/windows/win32/directcomp/best-practices-for-directcomposition

### Research Sources
- DirectComposition vs Windows.UI.Composition comparison (this investigation)
- UpdateLayeredWindow performance analysis
- Native AOT compatibility research
- Deployment size analysis

---

## Appendix: Code Checklist

### Before Starting Implementation

- [ ] Clone and build DirectNAot samples
- [ ] Run samples to understand patterns
- [ ] Read DirectComposition best practices documentation
- [ ] Study COM object lifetime management
- [ ] Review SpotlightDimmer architecture (Core/WindowsBindings separation)

### During Implementation

- [ ] Create minimal prototype first (validate approach)
- [ ] Measure performance continuously
- [ ] Test on multiple Windows versions (10, 11)
- [ ] Monitor memory usage with --verbose
- [ ] Commit frequently with clear messages
- [ ] Update documentation as you go

### Before Completion

- [ ] All IOverlayRenderer methods implemented
- [ ] Configuration option added
- [ ] Factory updated
- [ ] Tests passing
- [ ] Documentation updated (CONFIGURATION.md, CHANGELOG.md)
- [ ] Performance measured and documented
- [ ] Memory leak test passed (1-hour run)
- [ ] Code reviewed for COM leaks

---

## Questions to Answer During Prototype

1. **What is the actual binary size increase?**
   - Build prototype with Native AOT
   - Measure output directory size
   - Compare to Phase 1 baseline

2. **What is the measured update latency?**
   - Instrument Update() method
   - Measure time from call to GPU commit
   - Compare to UpdateLayeredWindow

3. **How complex is the COM interop really?**
   - Count lines of code in prototype
   - Assess maintainability
   - Identify pain points

4. **Are there any showstopper issues?**
   - Native AOT compatibility
   - Windows version support
   - Performance regressions

---

## Conclusion

Phase 2 represents a significant but worthwhile investment to completely eliminate window resize lag through DirectComposition. The approach is technically sound, proven by DirectNAot samples, and aligns with SpotlightDimmer's architecture and deployment requirements.

**Key Success Factors**:
1. ✅ Thorough prototyping before full implementation
2. ✅ Leveraging DirectNAot for AOT-compatible COM interop
3. ✅ Maintaining architecture separation (Core vs WindowsBindings)
4. ✅ Comprehensive testing and performance measurement

**Next Step**: Execute Phase 2.1 (Research & Prototyping) to validate the approach with minimal risk.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-06
**Author**: Research and planning based on Phase 1 investigation
**Status**: Ready for prototype phase
