# WinUI3 Renderer Performance Testing Guide

This document describes how to build and test the experimental WinUI3 renderer to measure its resource usage compared to the standard renderers.

## Building the WinUI3 Renderer

### Standard Build (LayeredWindow/UpdateLayeredWindow - AOT Enabled)
```powershell
# Clean build
dotnet clean

# Build standard version
dotnet build -c Release

# Publish with Native AOT
dotnet publish -c Release -r win-x64

# Output location:
# bin/Release/net10.0-windows/win-x64/publish/
```

### WinUI3 Build (AOT Disabled)
```powershell
# Clean build
dotnet clean

# Build with WinUI3 renderer enabled
dotnet build -c Release /p:UseWinUI3Renderer=true

# Publish (no AOT, includes WindowsAppSDK)
dotnet publish -c Release -r win-x64 /p:UseWinUI3Renderer=true

# Output location:
# bin/Release/net10.0-windows10.0.19041.0/win-x64/publish/
```

## Configuration

To use the WinUI3 renderer, update your `config.json`:

```json
{
  "System": {
    "RendererBackend": "WinUI3"
  }
}
```

## Performance Metrics to Measure

### 1. Disk Space
```powershell
# Measure publish output size
$standardPath = ".\SpotlightDimmer.WindowsClient\bin\Release\net10.0-windows\win-x64\publish"
$winui3Path = ".\SpotlightDimmer.WindowsClient\bin\Release\net10.0-windows10.0.19041.0\win-x64\publish"

# Standard build size
Get-ChildItem -Path $standardPath -Recurse | Measure-Object -Property Length -Sum | Select-Object @{Name="SizeMB";Expression={[math]::Round($_.Sum / 1MB, 2)}}

# WinUI3 build size
Get-ChildItem -Path $winui3Path -Recurse | Measure-Object -Property Length -Sum | Select-Object @{Name="SizeMB";Expression={[math]::Round($_.Sum / 1MB, 2)}}
```

### 2. Memory Usage
```powershell
# Run the application and monitor memory
# Standard renderer:
$process = Get-Process SpotlightDimmer
$process | Select-Object Name, @{Name="MemoryMB";Expression={[math]::Round($_.WorkingSet64 / 1MB, 2)}}

# Check memory every 5 seconds for 60 seconds
1..12 | ForEach-Object {
    Start-Sleep -Seconds 5
    $p = Get-Process SpotlightDimmer -ErrorAction SilentlyContinue
    if ($p) {
        [PSCustomObject]@{
            Time = (Get-Date).ToString("HH:mm:ss")
            MemoryMB = [math]::Round($p.WorkingSet64 / 1MB, 2)
            PrivateMemoryMB = [math]::Round($p.PrivateMemorySize64 / 1MB, 2)
        }
    }
}
```

### 3. CPU Usage
Enable Debug logging in config.json and monitor:
- GDI object count
- CPU usage via Task Manager or Performance Monitor
- Window update frequency during drag operations

### 4. Startup Time
```powershell
# Measure startup time
Measure-Command {
    Start-Process ".\SpotlightDimmer.exe"
    Start-Sleep -Seconds 2  # Wait for initialization
    Stop-Process -Name SpotlightDimmer
}
```

## Expected Results

### Standard Renderers (LayeredWindow/UpdateLayeredWindow)
- **Disk Space**: ~3-5 MB (with AOT)
- **Memory Usage**: ~15-30 MB
- **CPU Usage**: Very low (< 1% idle, < 5% during window drag)
- **Startup Time**: < 100ms

### WinUI3 Renderer (EXPERIMENTAL)
- **Disk Space**: ~60-120 MB (WindowsAppSDK + XAML runtime)
- **Memory Usage**: ~80-200 MB (XAML runtime + composition)
- **CPU Usage**: Moderate (1-3% idle, 5-15% during window drag)
- **Startup Time**: 500-1500ms

## Notes

- WinUI3 renderer is **NOT recommended for production use**
- This renderer exists solely for performance comparison testing
- The standard renderers are far more efficient for this use case
- WinUI3 adds significant overhead that provides no benefit for simple colored overlays
