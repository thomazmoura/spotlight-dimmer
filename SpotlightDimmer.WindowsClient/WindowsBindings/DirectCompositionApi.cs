using System.Runtime.InteropServices;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Direct P/Invoke declarations for DirectComposition COM APIs.
/// DirectComposition is available on Windows 8+ for GPU-accelerated compositing.
/// These declarations enable Native AOT compatible DirectComposition usage.
/// Uses raw COM pointers without managed wrappers for AOT compatibility.
/// </summary>
internal static partial class DirectCompositionApi
{
    // DirectComposition Device Creation
    [DllImport("dcomp.dll", PreserveSig = true)]
    private static extern int DCompositionCreateDevice(
        IntPtr dxgiDevice,
        [In, MarshalAs(UnmanagedType.LPStruct)] Guid iid,
        out IntPtr dcompositionDevice);

    /// <summary>
    /// Creates a DirectComposition device for the calling application.
    /// Returns raw COM pointer that must be manually released with Marshal.Release().
    /// </summary>
    public static IntPtr CreateDevice()
    {
        var iid = typeof(IDCompositionDevice).GUID;
        int hr = DCompositionCreateDevice(IntPtr.Zero, iid, out var device);

        if (hr < 0)
        {
            throw new COMException($"Failed to create DirectComposition device. HRESULT: 0x{hr:X8}", hr);
        }

        return device;
    }

    /// <summary>
    /// Safely calls a COM interface method and wraps HRESULT errors.
    /// </summary>
    public static void CheckHResult(int hr, string operation)
    {
        if (hr < 0)
        {
            throw new COMException($"DirectComposition operation '{operation}' failed. HRESULT: 0x{hr:X8}", hr);
        }
    }

    // COM Interface GUIDs
    [ComImport]
    [Guid("C37EA93A-E7AA-450D-B16F-9746CB0407F3")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IDCompositionDevice
    {
        // IUnknown methods
        void QueryInterface([In] ref Guid riid, out IntPtr ppvObject);
        uint AddRef();
        uint Release();

        // IDCompositionDevice methods
        void Commit();
        void WaitForCommitCompletion();
        void GetFrameStatistics(IntPtr statistics);

        [PreserveSig]
        int CreateTargetForHwnd(IntPtr hwnd, [MarshalAs(UnmanagedType.Bool)] bool topmost, out IntPtr target);

        [PreserveSig]
        int CreateVisual(out IntPtr visual);

        [PreserveSig]
        int CreateSurface(int width, int height, int pixelFormat, int alphaMode, out IntPtr surface);
    }

    [ComImport]
    [Guid("EACDD04C-117E-4E17-88F4-D1B12B0E3D89")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IDCompositionTarget
    {
        // IUnknown methods
        void QueryInterface([In] ref Guid riid, out IntPtr ppvObject);
        uint AddRef();
        uint Release();

        // IDCompositionTarget methods
        [PreserveSig]
        int SetRoot(IntPtr visual);
    }

    [ComImport]
    [Guid("4D93059D-097B-4651-9A60-F0F25116E2F3")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IDCompositionVisual
    {
        // IUnknown methods
        void QueryInterface([In] ref Guid riid, out IntPtr ppvObject);
        uint AddRef();
        uint Release();

        // IDCompositionVisual methods
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
        int AddVisual(IntPtr visual, [MarshalAs(UnmanagedType.Bool)] bool insertAbove, IntPtr referenceVisual);

        [PreserveSig]
        int RemoveVisual(IntPtr visual);

        [PreserveSig]
        int RemoveAllVisuals();

        [PreserveSig]
        int SetCompositeMode(int compositeMode);
    }
}
