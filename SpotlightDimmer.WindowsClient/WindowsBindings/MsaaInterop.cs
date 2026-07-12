using System.Runtime.InteropServices;

namespace SpotlightDimmer.WindowsBindings;

/// <summary>
/// Minimal, Native AOT-safe MSAA (IAccessible) interop used to locate the
/// focused terminal pane control inside Windows Terminal.
///
/// Built-in COM interop is unavailable under Native AOT, so the IAccessible
/// methods are invoked through raw vtable function pointers. Only the slots
/// needed are bound: AddRef/Release/QueryInterface (IUnknown), accLocation,
/// get_accFocus and get_accRole (IAccessible).
///
/// ABI note: IDL-by-value VARIANT parameters (24 bytes) are passed via a
/// hidden pointer on both x64 and arm64, so every by-value VARIANT in a
/// vtable signature appears as VARIANT* here. x86 is not supported (the
/// project only ships win-x64 and win-arm64).
/// </summary>
internal static unsafe partial class MsaaInterop
{
    public const uint OBJID_CLIENT = 0xFFFFFFFC;
    public const int CHILDID_SELF = 0;

    private const ushort VT_I4 = 3;
    private const ushort VT_DISPATCH = 9;

    // IAccessible vtable slots (IUnknown 0-2, IDispatch 3-6, IAccessible 7+)
    private const int SlotQueryInterface = 0;
    private const int SlotAddRef = 1;
    private const int SlotRelease = 2;
    private const int SlotGetAccRole = 13;
    private const int SlotGetAccFocus = 18;
    private const int SlotAccLocation = 22;

    private static readonly Guid IID_IAccessible = new("618736E0-3C3D-11CF-810C-00AA00389B71");

    /// <summary>
    /// Blittable VARIANT layout (8-byte header + 16-byte data area on 64-bit).
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct VARIANT
    {
        public ushort vt;
        public ushort wReserved1;
        public ushort wReserved2;
        public ushort wReserved3;
        public IntPtr data1;
        public IntPtr data2;
    }

    /// <summary>
    /// A cached reference to an accessible object (an AddRef'd IAccessible
    /// pointer plus the MSAA child id it refers to).
    /// </summary>
    public struct AccessibleTarget
    {
        public IntPtr Accessible;
        public int ChildId;

        public readonly bool IsValid => Accessible != IntPtr.Zero;
    }

    [LibraryImport("oleacc.dll")]
    private static partial int AccessibleObjectFromEvent(IntPtr hwnd, uint dwId, uint dwChildId, out IntPtr ppacc, out VARIANT pvarChild);

    [LibraryImport("oleacc.dll")]
    private static partial int AccessibleObjectFromWindow(IntPtr hwnd, uint dwId, ref Guid riid, out IntPtr ppvObject);

    [LibraryImport("oleaut32.dll")]
    private static partial void VariantClear(VARIANT* pvarg);

    private static IntPtr* Vtable(IntPtr comObject) => *(IntPtr**)comObject;

    /// <summary>
    /// Resolves the accessible object a WinEvent refers to (the focused pane
    /// control for EVENT_OBJECT_FOCUS). The returned target owns a COM
    /// reference; callers must eventually call <see cref="Release"/>.
    /// </summary>
    public static bool TryFromEvent(IntPtr hwnd, int idObject, int idChild, out AccessibleTarget target)
    {
        target = default;

        try
        {
            var hr = AccessibleObjectFromEvent(hwnd, (uint)idObject, (uint)idChild, out var acc, out var varChild);
            if (hr != 0 || acc == IntPtr.Zero)
                return false;

            var childId = CHILDID_SELF;
            if (varChild.vt == VT_I4)
            {
                childId = (int)varChild.data1;
            }
            else if (varChild.vt != 0)
            {
                // Defensive: release any unexpected reference the variant holds
                VariantClear(&varChild);
            }
            target = new AccessibleTarget { Accessible = acc, ChildId = childId };
            return true;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Resolves the accessible object that currently has keyboard focus in the
    /// given thread by following the MSAA accFocus chain from the focused
    /// window's client object. Used when hooks are armed after the focus event
    /// already fired (e.g. the terminal was already foreground at startup).
    /// The returned target owns a COM reference; callers must call <see cref="Release"/>.
    /// </summary>
    public static bool TryFromFocusedWindow(uint threadId, out AccessibleTarget target)
    {
        target = default;

        try
        {
            var info = new WinApi.GUITHREADINFO { cbSize = Marshal.SizeOf<WinApi.GUITHREADINFO>() };
            if (!WinApi.GetGUIThreadInfo(threadId, ref info) || info.hwndFocus == IntPtr.Zero)
                return false;

            var iid = IID_IAccessible;
            var hr = AccessibleObjectFromWindow(info.hwndFocus, OBJID_CLIENT, ref iid, out var acc);
            if (hr != 0 || acc == IntPtr.Zero)
                return false;

            // Follow the accFocus chain to the innermost focused object.
            // Bounded to keep a cyclic tree from hanging the main thread.
            var childId = CHILDID_SELF;
            for (var depth = 0; depth < 16; depth++)
            {
                VARIANT focus = default;
                var getFocus = (delegate* unmanaged[Stdcall]<IntPtr, VARIANT*, int>)Vtable(acc)[SlotGetAccFocus];
                if (getFocus(acc, &focus) != 0)
                    break;

                if (focus.vt == VT_DISPATCH && focus.data1 != IntPtr.Zero)
                {
                    // Focus is a child accessible object: descend into it.
                    var dispatch = focus.data1;
                    var queryInterface = (delegate* unmanaged[Stdcall]<IntPtr, Guid*, IntPtr*, int>)Vtable(dispatch)[SlotQueryInterface];
                    IntPtr childAcc = IntPtr.Zero;
                    var iidLocal = IID_IAccessible;
                    var qiResult = queryInterface(dispatch, &iidLocal, &childAcc);
                    VariantClear(&focus); // releases the IDispatch reference

                    if (qiResult != 0 || childAcc == IntPtr.Zero)
                        break;

                    ReleasePointer(acc);
                    acc = childAcc;
                    childId = CHILDID_SELF;
                    continue;
                }

                if (focus.vt == VT_I4)
                {
                    var focusedChild = (int)focus.data1;
                    if (focusedChild != CHILDID_SELF)
                        childId = focusedChild;
                    break;
                }

                // VT_EMPTY or anything else: current object is the focus.
                VariantClear(&focus);
                break;
            }

            target = new AccessibleTarget { Accessible = acc, ChildId = childId };
            return true;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Gets the screen-space location of the target via IAccessible.accLocation.
    /// Coordinates are in this process's coordinate space (consistent with
    /// GetWindowRect). Safe to call repeatedly on a cached target; returns
    /// false when the underlying object is gone.
    /// </summary>
    public static bool TryGetLocation(in AccessibleTarget target, out Core.Rectangle rect)
    {
        rect = default;
        if (!target.IsValid)
            return false;

        try
        {
            int left, top, width, height;
            var varChild = new VARIANT { vt = VT_I4, data1 = target.ChildId };
            var accLocation = (delegate* unmanaged[Stdcall]<IntPtr, int*, int*, int*, int*, VARIANT*, int>)Vtable(target.Accessible)[SlotAccLocation];
            if (accLocation(target.Accessible, &left, &top, &width, &height, &varChild) != 0)
                return false;

            if (width <= 0 || height <= 0)
                return false;

            rect = new Core.Rectangle(left, top, width, height);
            return true;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Gets the MSAA role of the target (ROLE_SYSTEM_* constant), or 0 on failure.
    /// Currently used for diagnostics only.
    /// </summary>
    public static int GetRole(in AccessibleTarget target)
    {
        if (!target.IsValid)
            return 0;

        try
        {
            VARIANT varChild = new() { vt = VT_I4, data1 = target.ChildId };
            VARIANT role = default;
            var getRole = (delegate* unmanaged[Stdcall]<IntPtr, VARIANT*, VARIANT*, int>)Vtable(target.Accessible)[SlotGetAccRole];
            if (getRole(target.Accessible, &varChild, &role) != 0)
                return 0;

            var result = role.vt == VT_I4 ? (int)role.data1 : 0;
            VariantClear(&role);
            return result;
        }
        catch
        {
            return 0;
        }
    }

    /// <summary>
    /// Returns a target holding an additional COM reference to the same
    /// accessible object. The clone must be released independently of the
    /// original via <see cref="Release"/>.
    /// </summary>
    public static AccessibleTarget Clone(in AccessibleTarget target)
    {
        if (!target.IsValid)
            return default;

        try
        {
            var addRef = (delegate* unmanaged[Stdcall]<IntPtr, uint>)Vtable(target.Accessible)[SlotAddRef];
            addRef(target.Accessible);
            return new AccessibleTarget { Accessible = target.Accessible, ChildId = target.ChildId };
        }
        catch
        {
            return default;
        }
    }

    /// <summary>
    /// Releases the COM reference held by the target and invalidates it.
    /// </summary>
    public static void Release(ref AccessibleTarget target)
    {
        if (target.Accessible != IntPtr.Zero)
        {
            ReleasePointer(target.Accessible);
            target = default;
        }
    }

    private static void ReleasePointer(IntPtr comObject)
    {
        try
        {
            var release = (delegate* unmanaged[Stdcall]<IntPtr, uint>)Vtable(comObject)[SlotRelease];
            release(comObject);
        }
        catch
        {
            // Releasing a dead pointer must never crash the app
        }
    }
}
