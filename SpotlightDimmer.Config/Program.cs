using System.Runtime.InteropServices;

namespace SpotlightDimmer.Config;

static class Program
{
    private const string MUTEX_NAME = "SpotlightDimmer.Config.SingleInstance";
    private const string WINDOW_TITLE = "SpotlightDimmer Configuration";

    [DllImport("user32.dll")]
    private static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    private static extern bool IsIconic(IntPtr hWnd);

    private const int SW_RESTORE = 9;

    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    static void Main()
    {
        // Single instance check using Mutex
        using var mutex = new Mutex(true, MUTEX_NAME, out bool createdNew);

        if (!createdNew)
        {
            // Another instance is already running - try to find and focus it
            BringExistingInstanceToFront();
            return;
        }

        // To customize application configuration such as set high DPI settings or default font,
        // see https://aka.ms/applicationconfiguration.
        ApplicationConfiguration.Initialize();
        Application.Run(new ConfigForm());

        // Keep mutex alive until application exits
        GC.KeepAlive(mutex);
    }

    private static void BringExistingInstanceToFront()
    {
        // Find the window by title
        var processes = System.Diagnostics.Process.GetProcessesByName("SpotlightDimmer.Config");

        foreach (var process in processes)
        {
            if (process.MainWindowHandle != IntPtr.Zero)
            {
                var handle = process.MainWindowHandle;

                // If minimized, restore it
                if (IsIconic(handle))
                {
                    ShowWindow(handle, SW_RESTORE);
                }

                // Bring to foreground
                SetForegroundWindow(handle);
                return;
            }
        }
    }
}