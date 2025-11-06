# DirectComposition ComWrappers Implementation Plan

**Status**: Planning
**Goal**: Migrate DirectComposition renderer from built-in COM ([ComImport]) to ComWrappers API for proper Native AOT support
**Context**: Built-in COM interop is disabled by default in Native AOT. Microsoft recommends using ComWrappers API as the AOT-friendly alternative.

---

## Executive Summary

This document outlines the migration strategy for the DirectComposition renderer from built-in COM interop to the ComWrappers API. We have two viable approaches:

1. **Use DirectNAot NuGet Package** (Recommended) - Leverage existing production-ready implementation
2. **Custom GeneratedComInterface Implementation** - Manual implementation using .NET 8+ source generator

**Recommendation**: Start with DirectNAot package approach for fastest path to working AOT support, with fallback to custom implementation if package integration proves problematic.

---

## Problem Statement

### Current Implementation Issues

The current `CompositionRenderer` implementation uses:
- `[ComImport]` attribute for interface definitions (`DirectCompositionApi.cs:49, 74, 89`)
- `Marshal.GetObjectForIUnknown()` for COM pointer conversion (`CompositionRenderer.cs:59, 151, 264, 276, 311`)
- `Marshal.ReleaseComObject()` for cleanup (`CompositionRenderer.cs:66, 158, 284, 289, 319`)
- `Marshal.Release()` for raw pointer cleanup

These APIs rely on built-in COM support which is **disabled by default in Native AOT** to reduce binary size.

### Why BuiltInComInteropSupport Didn't Work

While `<BuiltInComInteropSupport>true</BuiltInComInteropSupport>` is supposed to re-enable built-in COM, it may fail due to:
- Trimming removing required COM infrastructure
- IL linker aggressively removing "unused" COM code
- Runtime code generation being unavailable in AOT
- Platform-specific AOT limitations

---

## Approach 1: DirectNAot Package (Recommended)

### Overview

DirectNAot is a production-ready NuGet package specifically designed for DirectComposition and other Windows APIs with Native AOT support.

**Package**: `DirectNAot` version 1.3.2
**Repository**: https://github.com/smourier/DirectNAot
**Features**: DirectComposition, DXGI, WIC, DirectX 9-12, Direct2D, Direct Write, Media Foundation, etc.
**AOT Compatibility**: Designed explicitly for .NET 9+ Native AOT using ComWrappers source generator

### Advantages

✅ **Production-Ready**: Battle-tested implementation used in real applications
✅ **Comprehensive**: Includes all DirectComposition interfaces we need
✅ **Maintained**: Active project with regular updates
✅ **Zero Implementation**: Just add package reference and refactor usage
✅ **Small Size**: No dependencies, minimal overhead
✅ **Proven AOT**: Explicitly designed for Native AOT scenarios

### Disadvantages

❌ **External Dependency**: Adds NuGet package dependency to project
❌ **API Changes**: May have different API surface than our current implementation
❌ **Learning Curve**: Need to understand DirectNAot's API patterns
❌ **Version Lock-in**: Need to track package updates

### Implementation Steps

#### Step 1: Add Package Reference

**File**: `SpotlightDimmer.WindowsClient/SpotlightDimmer.WindowsClient.csproj`

```xml
<ItemGroup>
  <PackageReference Include="DirectNAot" Version="1.3.2" />
  <PackageReference Include="Serilog" Version="4.3.0" />
  <PackageReference Include="Serilog.Extensions.Logging" Version="9.0.2" />
  <PackageReference Include="Serilog.Sinks.File" Version="7.0.0" />
</ItemGroup>
```

#### Step 2: Remove BuiltInComInteropSupport

Remove the `<BuiltInComInteropSupport>true</BuiltInComInteropSupport>` line from project file since DirectNAot doesn't need it.

#### Step 3: Refactor DirectCompositionApi.cs

**Current Implementation**: Raw P/Invoke + [ComImport] interfaces
**New Implementation**: Use DirectNAot's types

Expected API surface from DirectNAot:
- `DirectN.DComposition` namespace
- `IDCompositionDevice` interface
- `IDCompositionTarget` interface
- `IDCompositionVisual` interface
- `DCompositionCreateDevice()` method
- ComWrappers-based object creation

**Action Items**:
1. Explore DirectNAot package API documentation
2. Identify exact type names and namespaces
3. Map our current usage to DirectNAot equivalents
4. Create adapter layer if API differences are significant

#### Step 4: Refactor CompositionRenderer.cs

Replace all `Marshal.GetObjectForIUnknown()` calls with DirectNAot's object creation patterns.

**Before**:
```csharp
var device = (IDCompositionDevice)Marshal.GetObjectForIUnknown(_devicePtr);
try
{
    device.Commit();
}
finally
{
    Marshal.ReleaseComObject(device);
}
```

**After** (expected pattern):
```csharp
// DirectNAot likely provides wrapper objects that handle lifetime automatically
using var device = ComObject<IDCompositionDevice>.FromPointer(_devicePtr);
device.Commit();
// Automatic cleanup via IDisposable
```

#### Step 5: Update Device Creation

**Current**:
```csharp
_devicePtr = DirectCompositionApi.CreateDevice();
```

**Expected with DirectNAot**:
```csharp
// May return managed wrapper directly
_device = DirectNAot.DComposition.CreateDevice();
// Or use COM wrappers
var cw = new StrategyBasedComWrappers();
_device = cw.CreateDCompositionDevice();
```

#### Step 6: Testing

1. Build with Native AOT: `dotnet publish -c Release -r win-x64`
2. Verify no COM-related errors during compilation
3. Test DirectComposition renderer functionality:
   - Overlay creation
   - Position updates
   - Color/opacity changes
   - Multi-monitor support
   - Memory leak testing (--verbose mode)

---

## Approach 2: Custom GeneratedComInterface Implementation

### Overview

Manually implement DirectComposition interfaces using .NET 8+ ComWrappers source generator with `[GeneratedComInterface]` attribute.

### Advantages

✅ **No External Dependencies**: Keeps codebase self-contained
✅ **Full Control**: Complete control over API surface
✅ **Minimal Surface**: Only implement interfaces we actually use
✅ **Learning Opportunity**: Deep understanding of COM interop

### Disadvantages

❌ **Implementation Time**: Requires manual interface definition and testing
❌ **Maintenance Burden**: Need to maintain COM interface definitions
❌ **Error-Prone**: Easy to get vtable ordering or signatures wrong
❌ **Limited Scope**: Only covers DirectComposition (no reuse for future needs)

### Implementation Steps

#### Step 1: Define COM Interfaces with GeneratedComInterface

**File**: `SpotlightDimmer.WindowsClient/WindowsBindings/DirectCompositionInterfaces.cs` (new file)

```csharp
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// DirectComposition device interface.
/// AOT-compatible via ComWrappers source generator.
/// </summary>
[GeneratedComInterface]
[Guid("C37EA93A-E7AA-450D-B16F-9746CB0407F3")]
internal partial interface IDCompositionDevice
{
    // Note: IUnknown methods (QueryInterface, AddRef, Release) are handled by generator

    void Commit();
    void WaitForCommitCompletion();
    void GetFrameStatistics(IntPtr statistics);

    [PreserveSig]
    int CreateTargetForHwnd(IntPtr hwnd, bool topmost, out IntPtr target);

    [PreserveSig]
    int CreateVisual(out IntPtr visual);

    [PreserveSig]
    int CreateSurface(int width, int height, int pixelFormat, int alphaMode, out IntPtr surface);
}

[GeneratedComInterface]
[Guid("EACDD04C-117E-4E17-88F4-D1B12B0E3D89")]
internal partial interface IDCompositionTarget
{
    [PreserveSig]
    int SetRoot(IntPtr visual);
}

[GeneratedComInterface]
[Guid("4D93059D-097B-4651-9A60-F0F25116E2F3")]
internal partial interface IDCompositionVisual
{
    [PreserveSig]
    int SetOffsetX(float offsetX);

    [PreserveSig]
    int SetOffsetY(float offsetY);

    [PreserveSig]
    int SetTransform(IntPtr transform);

    [PreserveSig]
    int SetTransformParent(IntPtr visual);

    [PreserveSig]
    int SetEffect(IntPtr effect);

    [PreserveSig]
    int SetBitmapInterpolationMode(int interpolationMode);

    [PreserveSig]
    int SetBorderMode(int borderMode);

    [PreserveSig]
    int SetClip(IntPtr clip);

    [PreserveSig]
    int SetContent(IntPtr content);

    [PreserveSig]
    int AddVisual(IntPtr visual, bool insertAbove, IntPtr referenceVisual);

    [PreserveSig]
    int RemoveVisual(IntPtr visual);

    [PreserveSig]
    int RemoveAllVisuals();

    [PreserveSig]
    int SetCompositeMode(int compositeMode);
}
```

**Key Points**:
- `[GeneratedComInterface]` triggers source generator
- `[Guid]` specifies COM interface ID
- Interfaces must be `partial` and `internal` or `public`
- `[PreserveSig]` prevents HRESULT-to-exception conversion
- IUnknown methods (QueryInterface, AddRef, Release) are auto-generated

#### Step 2: Create ComWrappers Helper

**File**: `SpotlightDimmer.WindowsClient/WindowsBindings/DirectCompositionWrappers.cs` (new file)

```csharp
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// ComWrappers-based helper for DirectComposition object creation.
/// Provides AOT-compatible COM interop.
/// </summary>
internal static class DirectCompositionWrappers
{
    private static readonly StrategyBasedComWrappers s_comWrappers = new();

    /// <summary>
    /// Creates a DirectComposition device.
    /// </summary>
    public static IDCompositionDevice CreateDevice()
    {
        var iid = typeof(IDCompositionDevice).GUID;
        int hr = DCompositionCreateDevice(IntPtr.Zero, iid, out var devicePtr);

        if (hr < 0)
        {
            throw new COMException($"Failed to create DirectComposition device. HRESULT: 0x{hr:X8}", hr);
        }

        // Convert native COM pointer to managed interface using ComWrappers
        var device = (IDCompositionDevice)s_comWrappers.GetOrCreateObjectForComInstance(
            devicePtr,
            CreateObjectFlags.UniqueInstance);

        // Release the initial reference since ComWrappers now owns it
        Marshal.Release(devicePtr);

        return device;
    }

    /// <summary>
    /// Wraps a raw COM pointer as a managed interface.
    /// </summary>
    public static T WrapComPointer<T>(IntPtr ptr) where T : class
    {
        if (ptr == IntPtr.Zero)
            throw new ArgumentNullException(nameof(ptr));

        return (T)s_comWrappers.GetOrCreateObjectForComInstance(ptr, CreateObjectFlags.None);
    }

    [DllImport("dcomp.dll", PreserveSig = true)]
    private static extern int DCompositionCreateDevice(
        IntPtr dxgiDevice,
        [In] Guid iid,
        out IntPtr dcompositionDevice);
}
```

#### Step 3: Refactor CompositionRenderer to Use New API

**Changes to `CompositionRenderer.cs`**:

**Before**:
```csharp
private IntPtr _devicePtr = IntPtr.Zero;

public void CreateOverlays(...)
{
    _devicePtr = DirectCompositionApi.CreateDevice();
    // ...
}

private void CommitChanges()
{
    if (_devicePtr == IntPtr.Zero)
        return;

    var device = (IDCompositionDevice)Marshal.GetObjectForIUnknown(_devicePtr);
    try
    {
        device.Commit();
    }
    finally
    {
        Marshal.ReleaseComObject(device);
    }
}
```

**After**:
```csharp
private IDCompositionDevice? _device = null;

public void CreateOverlays(...)
{
    _device = DirectCompositionWrappers.CreateDevice();
    // ...
}

private void CommitChanges()
{
    _device?.Commit();
    // ComWrappers handles lifetime automatically
}

public void Dispose()
{
    CleanupOverlays();
    // Let garbage collector handle _device cleanup
    // ComWrappers will release COM references appropriately
}
```

**Changes to `CompositionOverlay` class**:

**Before**:
```csharp
private IntPtr _targetPtr = IntPtr.Zero;
private IntPtr _visualPtr = IntPtr.Zero;

private void CreateCompositionResources()
{
    var device = (IDCompositionDevice)Marshal.GetObjectForIUnknown(_devicePtr);
    try
    {
        int hr = device.CreateTargetForHwnd(_hwnd, true, out _targetPtr);
        CheckHResult(hr, "CreateTargetForHwnd");

        hr = device.CreateVisual(out _visualPtr);
        CheckHResult(hr, "CreateVisual");

        var target = (IDCompositionTarget)Marshal.GetObjectForIUnknown(_targetPtr);
        try
        {
            hr = target.SetRoot(_visualPtr);
            CheckHResult(hr, "SetRoot");
        }
        finally
        {
            Marshal.ReleaseComObject(target);
        }
    }
    finally
    {
        Marshal.ReleaseComObject(device);
    }
}

public void Update(OverlayDefinition source)
{
    var visual = (IDCompositionVisual)Marshal.GetObjectForIUnknown(_visualPtr);
    try
    {
        visual.SetOffsetX((float)source.Bounds.X);
        visual.SetOffsetY((float)source.Bounds.Y);
    }
    finally
    {
        Marshal.ReleaseComObject(visual);
    }
    // ...
}
```

**After**:
```csharp
private IDCompositionTarget? _target = null;
private IDCompositionVisual? _visual = null;

private void CreateCompositionResources()
{
    int hr = _device.CreateTargetForHwnd(_hwnd, true, out var targetPtr);
    CheckHResult(hr, "CreateTargetForHwnd");
    _target = DirectCompositionWrappers.WrapComPointer<IDCompositionTarget>(targetPtr);

    hr = _device.CreateVisual(out var visualPtr);
    CheckHResult(hr, "CreateVisual");
    _visual = DirectCompositionWrappers.WrapComPointer<IDCompositionVisual>(visualPtr);

    hr = _target.SetRoot(visualPtr);
    CheckHResult(hr, "SetRoot");
}

public void Update(OverlayDefinition source)
{
    if (_visual == null)
        return;

    _visual.SetOffsetX((float)source.Bounds.X);
    _visual.SetOffsetY((float)source.Bounds.Y);
    // ...
}

public void Dispose()
{
    // Let ComWrappers/GC handle cleanup
    _visual = null;
    _target = null;

    if (_hwnd != IntPtr.Zero)
    {
        WinApi.DestroyWindow(_hwnd);
        _hwnd = IntPtr.Zero;
    }
}
```

#### Step 4: Remove Old DirectCompositionApi.cs

Delete `WindowsBindings/DirectCompositionApi.cs` as it's replaced by the new ComWrappers-based implementation.

#### Step 5: Update Project File

Remove `<BuiltInComInteropSupport>true</BuiltInComInteropSupport>` since we're no longer using built-in COM.

---

## Comparison: DirectNAot vs. Custom Implementation

| Aspect | DirectNAot Package | Custom GeneratedComInterface |
|--------|-------------------|------------------------------|
| **Implementation Time** | 2-4 hours (integration + testing) | 1-2 days (implementation + debugging) |
| **Code Maintenance** | Low (package updates) | High (manual maintenance) |
| **AOT Compatibility** | Proven (production-ready) | Theoretical (needs validation) |
| **Dependencies** | +1 NuGet package | Zero external |
| **Binary Size** | Unknown (need to measure) | Minimal (only what we use) |
| **Risk Level** | Low (battle-tested) | Medium (custom implementation) |
| **Future Reusability** | High (includes other APIs) | None (DirectComposition only) |
| **Learning Curve** | Low (use existing API) | High (understand COM internals) |

---

## Recommended Implementation Path

### Phase 1: DirectNAot Package Attempt (Preferred)

1. Add DirectNAot package reference
2. Explore package API surface (inspect types, methods, patterns)
3. Create small proof-of-concept:
   - Create single DirectComposition device
   - Create one visual
   - Set position
   - Commit changes
4. Measure binary size impact (compare to current baseline)
5. **Decision Point**:
   - ✅ If POC works and size is acceptable → Proceed with full integration
   - ❌ If POC fails or size unacceptable → Fall back to Phase 2

### Phase 2: Custom GeneratedComInterface (Fallback)

1. Implement interface definitions with `[GeneratedComInterface]`
2. Create ComWrappers helper class
3. Build small test program to validate vtables are correct
4. Refactor `CompositionRenderer` step-by-step
5. Test thoroughly with all renderer features

---

## Testing Strategy

### Compilation Testing

1. **Build with AOT**: `dotnet publish -c Release -r win-x64`
2. **Verify Success**: Check for COM-related errors (IL3052, IL3053, etc.)
3. **Size Measurement**: Compare binary size before/after
4. **Trimming Warnings**: Review ILLink warnings for COM-related issues

### Functional Testing

1. **Basic Functionality**:
   - Overlay creation (6 per display)
   - Initial positioning
   - Color/opacity rendering
   - Show/hide functionality

2. **Dynamic Updates**:
   - Window movement tracking
   - Position updates (<1ms latency)
   - Size changes
   - Color changes
   - Multi-monitor scenarios

3. **Edge Cases**:
   - Display configuration changes
   - Zero-dimension windows
   - Rapid focus changes
   - Configuration hot-reload

4. **Memory/Resource Testing**:
   - Run with `--verbose` flag
   - Monitor GDI object count (should remain stable)
   - 1-hour soak test with active window switching
   - Check for COM reference leaks

### Performance Validation

1. **Update Latency**: Should remain <1ms for position updates
2. **CPU Usage**: Should be <5% during window movement
3. **Startup Time**: Measure impact on application startup
4. **Memory Footprint**: Compare before/after implementation

---

## Rollback Plan

If ComWrappers implementation fails or proves too problematic:

### Option 1: Revert to Legacy/UpdateLayeredWindow Renderers

- Remove DirectComposition renderer entirely
- Document limitation in CONFIGURATION.md
- Set default to UpdateLayeredWindow renderer
- Accept 8-16ms latency as "good enough"

### Option 2: Non-AOT DirectComposition Build

- Create separate build configuration without AOT
- Use `<BuiltInComInteropSupport>true` for that configuration
- Provide both builds to users (AOT vs. DirectComposition)
- Document trade-offs clearly

### Option 3: Hybrid Approach

- Use UpdateLayeredWindow for AOT builds
- Use DirectComposition for non-AOT builds
- Runtime detection and selection
- Transparent to user (automatic fallback)

---

## Success Criteria

### Must Have

✅ DirectComposition renderer compiles with Native AOT
✅ No runtime COM exceptions
✅ Overlay position updates work correctly
✅ Color/opacity changes apply properly
✅ No memory leaks (stable GDI object count)
✅ Multi-monitor support functional

### Should Have

✅ Binary size increase <15MB from baseline
✅ Update latency remains <1ms
✅ Implementation time <1 week
✅ Code maintainability acceptable

### Nice to Have

✅ Zero external dependencies (custom implementation)
✅ Reusable for future Windows API needs (DirectNAot package)
✅ Clear documentation for future maintenance

---

## File Changes Summary

### New Files (DirectNAot Approach)

- None (just package reference)

### New Files (Custom Approach)

- `WindowsBindings/DirectCompositionInterfaces.cs` - Interface definitions
- `WindowsBindings/DirectCompositionWrappers.cs` - ComWrappers helper

### Modified Files (Both Approaches)

- `SpotlightDimmer.WindowsClient.csproj` - Remove BuiltInComInteropSupport, optionally add DirectNAot package
- `WindowsBindings/CompositionRenderer.cs` - Refactor to use new API patterns
- `CHANGELOG.md` - Document the change

### Deleted Files (Both Approaches)

- `WindowsBindings/DirectCompositionApi.cs` - Replaced by new implementation

---

## Implementation Checklist

### Pre-Implementation

- [ ] Review this plan thoroughly
- [ ] Understand ComWrappers API basics
- [ ] Decide on DirectNAot vs. Custom approach
- [ ] Set up test environment for AOT compilation

### DirectNAot Approach

- [ ] Add DirectNAot package reference
- [ ] Explore package API (inspect types in IDE)
- [ ] Create proof-of-concept test
- [ ] Measure binary size impact
- [ ] If successful, proceed with full integration
- [ ] If unsuccessful, switch to Custom approach

### Custom Approach

- [ ] Implement DirectCompositionInterfaces.cs
- [ ] Implement DirectCompositionWrappers.cs
- [ ] Build and verify source generation works
- [ ] Create small test program
- [ ] Validate vtable correctness
- [ ] Refactor CompositionRenderer
- [ ] Test thoroughly

### Post-Implementation

- [ ] Full functional testing
- [ ] Memory leak testing (1-hour run)
- [ ] Performance validation
- [ ] Multi-monitor testing
- [ ] Update CHANGELOG.md
- [ ] Document any API changes
- [ ] Commit and push changes

---

## Questions to Answer During Implementation

1. **DirectNAot Package**:
   - What are the exact type names and namespaces?
   - Does it provide managed wrappers or raw pointers?
   - How does object lifetime work?
   - What's the binary size impact?

2. **Custom Implementation**:
   - Does source generator correctly handle all interface methods?
   - Are vtable orderings correct for all interfaces?
   - How do we properly handle object lifetime with ComWrappers?
   - Do we need explicit disposal or rely on GC?

3. **General**:
   - What's the actual compilation error without fixes?
   - Are there any performance regressions?
   - How does this affect debugging experience?

---

## References

### Documentation

- **ComWrappers Source Generation**: https://learn.microsoft.com/en-us/dotnet/standard/native-interop/comwrappers-source-generation
- **Using ComWrappers API**: https://learn.microsoft.com/en-us/dotnet/standard/native-interop/tutorial-comwrappers
- **DirectNAot Repository**: https://github.com/smourier/DirectNAot
- **DirectComposition API**: https://learn.microsoft.com/en-us/windows/win32/directcomp/

### Code Samples

- **Generated COM Sample**: https://learn.microsoft.com/en-us/samples/dotnet/samples/generated-comwrappers/
- **DirectNAot Examples**: https://github.com/smourier/DirectNAot (check samples folder)

---

**Document Version**: 1.0
**Created**: 2025-11-06
**Status**: Ready for implementation
**Next Step**: Review plan and decide on DirectNAot vs. Custom approach
