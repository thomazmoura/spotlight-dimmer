# WinUI3 Runtime Initialization Fix

## Problem

When running the WinUI3 renderer, the application crashed with:

```
System.Runtime.InteropServices.COMException:
'Classe não registrada (0x80040154 (REGDB_E_CLASSNOTREG))'
```

## Root Cause

**Unpackaged WinUI3 applications** require explicit runtime initialization. Unlike packaged MSIX apps where COM registration happens automatically during app installation, unpackaged apps must manually bootstrap the Windows App SDK runtime.

The error occurs when trying to create WinUI3 objects (like `new Window()`) before initializing the dispatcher queue that manages WinUI3's threading model.

## Solution Implemented

### 1. DispatcherQueue Initialization

Added initialization code to create a `DispatcherQueueController` on the current thread:

```csharp
[DllImport("CoreMessaging.dll")]
private static extern int CreateDispatcherQueueController(
    DispatcherQueueOptions options,
    out IntPtr dispatcherQueueController);

private static void InitializeWinUI()
{
    var options = new DispatcherQueueOptions
    {
        dwSize = Marshal.SizeOf<DispatcherQueueOptions>(),
        threadType = 2, // DQTYPE_THREAD_CURRENT (use existing thread)
        apartmentType = 2  // DQTAT_COM_STA (COM Single-Threaded Apartment)
    };

    int hr = CreateDispatcherQueueController(options, out IntPtr controller);
    // ... error handling
}
```

### 2. One-Time Initialization

The initialization happens once when the first `WinUI3Renderer` is created:

```csharp
private static bool _winUIInitialized = false;

public WinUI3Renderer()
{
    if (!_winUIInitialized)
    {
        InitializeWinUI();
        _winUIInitialized = true;
    }
}
```

## Technical Details

### What `CreateDispatcherQueueController` Does

1. **Registers WinUI3 COM Objects**: Makes WinUI3's COM components available on the current thread
2. **Creates Dispatcher Queue**: Sets up the message queue for WinUI3's threading model
3. **Integrates with Win32**: Works with existing `GetMessage`/`DispatchMessage` loop
4. **No Separate Thread**: Uses `DQTYPE_THREAD_CURRENT` to avoid creating a new thread

### Thread Type Options

| Value | Constant | Description |
|-------|----------|-------------|
| 1 | `DQTYPE_THREAD_DEDICATED` | Create new thread with message loop |
| 2 | `DQTYPE_THREAD_CURRENT` | Use current thread (our choice) |

We use `DQTYPE_THREAD_CURRENT` because `Program.cs` already has a Win32 message loop:

```csharp
// Existing message loop in Program.cs
while (WinApi.GetMessage(out var msg, IntPtr.Zero, 0, 0))
{
    WinApi.TranslateMessage(ref msg);
    WinApi.DispatchMessage(ref msg);
}
```

### Apartment Type Options

| Value | Constant | Description |
|-------|----------|-------------|
| 1 | `DQTAT_COM_NONE` | No COM initialization |
| 2 | `DQTAT_COM_ASTA` | COM Application STA |
| 2 | `DQTAT_COM_STA` | COM Single-Threaded Apartment (our choice) |

WinUI3 requires STA threading for XAML interop.

## Why Not Use Application.Start()?

The typical WinUI3 initialization pattern is:

```csharp
Application.Start((p) => { /* app code */ });
```

However, this creates its **own message loop** and blocks, which conflicts with SpotlightDimmer's existing Win32 message pump. The `DispatcherQueueController` approach integrates WinUI3 with an existing message loop.

## Runtime Deployment

For unpackaged apps, the Windows App SDK runtime must be deployed alongside the exe. Our project configuration ensures this:

```xml
<PackageReference Include="Microsoft.WindowsAppSDK" Version="1.7.241209001">
  <ExcludeAssets>buildTransitive</ExcludeAssets>
  <!-- Include runtime and native DLLs -->
  <IncludeAssets>compile;runtime;native;contentfiles;analyzers;build</IncludeAssets>
</PackageReference>
```

This copies the required DLLs to the output folder:
- `Microsoft.ui.xaml.dll`
- `Microsoft.WindowsAppRuntime.dll`
- `Microsoft.WindowsAppRuntime.Bootstrap.dll`
- And other WinUI3 runtime components

## Testing the Fix

After building, run the application:

```powershell
cd SpotlightDimmer.WindowsClient\bin\x64\Release\net10.0-windows10.0.19041.0\win-x64
.\SpotlightDimmer.exe
```

Set the config to use WinUI3:
```json
{
  "System": {
    "RendererBackend": "WinUI3"
  }
}
```

## Expected Behavior

✅ **Application starts without COM errors**
✅ **WinUI3 windows are created successfully**
✅ **Overlays render using WinUI3/XAML**
✅ **Dispatcher queue integrates with existing message loop**

## Comparing Initialization Approaches

| Approach | Message Loop | Use Case | Our Choice |
|----------|--------------|----------|------------|
| `Application.Start()` | Creates own | Standalone WinUI3 app | ❌ No |
| `DispatcherQueueController` | Uses existing | Win32 app with WinUI3 content | ✅ Yes |
| XAML Islands | Uses existing | Win32 app with embedded XAML | Alternative |

## References

- [Windows App SDK Deployment Guide](https://learn.microsoft.com/windows/apps/windows-app-sdk/deploy-unpackaged-apps)
- [DispatcherQueue API](https://learn.microsoft.com/windows/windows-app-sdk/api/winrt/microsoft.ui.dispatching.dispatcherqueue)
- [Unpackaged App Architecture](https://learn.microsoft.com/windows/apps/windows-app-sdk/migrate-to-windows-app-sdk/guides/unpackaged)

## Summary

The fix enables WinUI3 to work in an unpackaged desktop application by:
1. Manually initializing the DispatcherQueue via P/Invoke
2. Using `DQTYPE_THREAD_CURRENT` to integrate with existing message loop
3. Ensuring Windows App SDK runtime is deployed with the exe

This allows performance comparison between lightweight Win32 renderers and the full WinUI3/XAML stack! 🎯
