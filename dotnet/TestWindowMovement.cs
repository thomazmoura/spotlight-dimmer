using System.Runtime.InteropServices;

namespace SpotlightDimmer;

/// <summary>
/// Test program to verify EVENT_OBJECT_LOCATIONCHANGE detects same-monitor movements
/// </summary>
internal class TestWindowMovement
{
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
        public override string ToString() => $"({Left},{Top})-({Right},{Bottom}) [{Right-Left}x{Bottom-Top}]";
    }

    [DllImport("user32.dll")]
    private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    private static void WinEventCallback(IntPtr hook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint thread, uint time)
    {
        if (eventType == WinApi.EVENT_OBJECT_LOCATIONCHANGE && idObject == WinApi.OBJID_WINDOW)
        {
            var foreground = WinApi.GetForegroundWindow();
            if (hwnd == foreground)
            {
                if (GetWindowRect(hwnd, out var rect))
                {
                    Console.WriteLine($"[LOCATION] Window moved/resized: {rect}");
                }
            }
        }
    }

    public static void Run()
    {
        Console.WriteLine("=== Testing EVENT_OBJECT_LOCATIONCHANGE ===");
        Console.WriteLine("Instructions:");
        Console.WriteLine("1. Focus any window (like browser or notepad)");
        Console.WriteLine("2. Try Win+Left, Win+Right, Win+Up, Win+Down");
        Console.WriteLine("3. Try dragging the window");
        Console.WriteLine("4. Watch for [LOCATION] events");
        Console.WriteLine("Press Ctrl+C to exit\n");

        var callback = new WinApi.WinEventDelegate(WinEventCallback);

        var hook = WinApi.SetWinEventHook(
            WinApi.EVENT_OBJECT_LOCATIONCHANGE,
            WinApi.EVENT_OBJECT_LOCATIONCHANGE,
            IntPtr.Zero,
            callback,
            0,
            0,
            WinApi.WINEVENT_OUTOFCONTEXT);

        if (hook == IntPtr.Zero)
        {
            Console.WriteLine("Failed to set hook!");
            return;
        }

        Console.WriteLine("Hook installed. Waiting for window movements...\n");

        // Message loop
        while (WinApi.GetMessage(out var msg, IntPtr.Zero, 0, 0))
        {
            WinApi.TranslateMessage(ref msg);
            WinApi.DispatchMessage(ref msg);
        }

        WinApi.UnhookWinEvent(hook);
    }
}
