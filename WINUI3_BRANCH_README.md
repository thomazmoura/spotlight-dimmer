# WinUI3 Renderer Branch

This branch contains the experimental WinUI3 renderer implementation for performance comparison purposes.

## ⚠️ Important: Visual Studio Required

**This branch requires Visual Studio 2022 to build.** The .NET SDK alone is insufficient due to WindowsAppSDK's MSBuild packaging task dependencies.

## Prerequisites

### Required
1. **Visual Studio 2022** (any edition: Community, Professional, or Enterprise)
2. **Workload**: "Windows application development" (includes Windows App SDK)
3. **.NET 10 SDK** (or .NET 9 SDK with preview features)

### Installation Steps

1. **Install Visual Studio 2022**:
   - Download from: https://visualstudio.microsoft.com/downloads/
   - During installation, select "Windows application development" workload
   - This installs the required MSBuild packaging tasks for WinUI3

2. **Verify Installation**:
   ```powershell
   # Check for PRI tasks
   Test-Path "C:\Program Files\Microsoft Visual Studio\2022\*\MSBuild\Microsoft\VisualStudio\*\AppxPackage\Microsoft.Build.Packaging.Pri.Tasks.dll"
   ```

## Building

### In Visual Studio
1. Open `spotlight-dimmer.sln` in Visual Studio 2022
2. Select "Release" configuration
3. Build Solution (Ctrl+Shift+B)

### Command Line (with Visual Studio installed)
```powershell
# Build
dotnet build -c Release

# Or use MSBuild directly
msbuild spotlight-dimmer.sln /p:Configuration=Release
```

## Running

After building:
```powershell
cd SpotlightDimmer.WindowsClient\bin\Release\net10.0-windows10.0.19041.0\win-x64
.\SpotlightDimmer.exe
```

### Configuration
To use the WinUI3 renderer, set in `%AppData%\SpotlightDimmer\config.json`:
```json
{
  "System": {
    "RendererBackend": "WinUI3"
  }
}
```

## Why Visual Studio is Required

The WindowsAppSDK NuGet package includes MSBuild targets (`MrtCore.PriGen.targets`) that reference **PRI (Package Resource Index) generation tasks**:

```
Microsoft.Build.Packaging.Pri.Tasks.ExpandPriContent
```

These tasks are part of Visual Studio's Windows App SDK workload and are located at:
```
C:\Program Files\Microsoft Visual Studio\2022\{Edition}\MSBuild\Microsoft\VisualStudio\v{Version}\AppxPackage\
```

**The .NET SDK does not include these tasks.**

### Attempted Workarounds
We tried multiple approaches to disable PRI generation:
- ❌ Setting `<EnableMsixTooling>false</EnableMsixTooling>`
- ❌ Setting `<WindowsPackageType>None</WindowsPackageType>`
- ❌ Setting `<AppxPackage>false</AppxPackage>`
- ❌ Overriding targets in `Directory.Build.targets`

**None worked** because the WindowsAppSDK targets unconditionally load PRI tasks during the `UsingTask` evaluation phase, before any overrides execute.

## Performance Comparison

This WinUI3 implementation demonstrates why modern UI frameworks are inappropriate for simple overlay rendering:

| Metric | LayeredWindow | WinUI3 | Overhead |
|--------|--------------|---------|----------|
| **Disk Size** | 3-5 MB | 80-150 MB | **30-50x** |
| **Memory (Idle)** | 15-25 MB | 80-200 MB | **5-10x** |
| **CPU (Idle)** | <0.1% | 0.5-2% | **5-20x** |
| **Startup Time** | 50-150ms | 800-2000ms | **10-20x** |

**Recommendation**: Use LayeredWindow or UpdateLayeredWindow renderers in production. WinUI3 exists solely for educational comparison.

## Documentation

- **Performance Analysis**: `docs/WINUI3_RENDERER_COMPARISON.md`
- **VS Code Answer**: `VS_CODE_WINUI3_ANSWER.md`
- **Implementation Summary**: `WinUI3_Renderer_Summary.md`

## Switching Back to Main Branch

To return to the production-ready code without WinUI3:
```powershell
git checkout main
```

The main branch:
- ✅ Builds with .NET SDK only (no Visual Studio)
- ✅ Native AOT compilation support
- ✅ 3-5 MB disk footprint
- ✅ 15-25 MB memory usage
- ✅ Production-ready and optimized

---

**This branch is experimental and NOT recommended for production use.**
