using DirectN;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// DirectComposition API wrapper using DirectNAot for Native AOT compatibility.
/// DirectComposition is available on Windows 8+ for GPU-accelerated compositing.
/// Uses ComWrappers-based DirectNAot package instead of built-in COM interop.
/// </summary>
internal static partial class DirectCompositionApi
{
    /// <summary>
    /// Creates a DirectComposition device for the calling application.
    /// Returns a ComObject wrapper that must be disposed.
    /// </summary>
    public static ComObject<IDCompositionDevice> CreateDevice()
    {
        // DirectNAot provides the DCompositionCreateDevice function
        var hr = Functions.DCompositionCreateDevice(null, out var device);

        if (hr.IsError)
        {
            throw new System.Runtime.InteropServices.COMException(
                $"Failed to create DirectComposition device. HRESULT: {hr}", hr);
        }

        return device!;
    }

    /// <summary>
    /// Safely calls a COM interface method and wraps HRESULT errors.
    /// </summary>
    public static void CheckHResult(HRESULT hr, string operation)
    {
        if (hr.IsError)
        {
            throw new System.Runtime.InteropServices.COMException(
                $"DirectComposition operation '{operation}' failed. HRESULT: {hr}", hr);
        }
    }
}
