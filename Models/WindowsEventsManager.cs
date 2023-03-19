﻿
using SpotlightDimmer.Models;
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;

namespace SpotlightDimmer
{
    public class WindowsEventsManager: IDisposable
    {
        private const uint EVENT_SYSTEM_FOREGROUND = 0x0003;

        private readonly string[] _ignoredWindows;
        private readonly DimmerState _state;

        private readonly IntPtr _windowsFocusHook;
        private readonly IntPtr _windowsResizedHook;
        private readonly WinEventDelegate _winEventDelegate;

        // Methods to get focus events
        private delegate void WinEventDelegate(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime);

        [DllImport("user32.dll")]
        private static extern IntPtr SetWinEventHook(uint eventMin, uint eventMax, IntPtr hmodWinEventProc, WinEventDelegate lpfnWinEventProc, uint idProcess, uint idThread, uint dwFlags);
        [DllImport("user32.dll")]
        private static extern bool UnhookWinEvent(IntPtr hWinEventHook);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        private static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetWindowRect(IntPtr hWnd, ref RECT lpRect);

        // Methods to get resize events
        private const uint WINEVENT_OUTOFCONTEXT = 0x0000; // Events are ASYNC
        private const uint EVENT_OBJECT_LOCATIONCHANGE = 0x800B;

        [DllImport("user32.dll")]
        private static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll")]
        private static extern IntPtr GetFocus();


        public WindowsEventsManager(DimmerState state)
        {
            _state = state;
            _ignoredWindows = new string[] {};
            _winEventDelegate = new WinEventDelegate(WinEventProc);
            _windowsFocusHook = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, _winEventDelegate, 0, 0, 0);
            _windowsResizedHook = SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE, IntPtr.Zero, _winEventDelegate, 0, 0, WINEVENT_OUTOFCONTEXT);
        }
        

        private void WinEventProc(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime)
        {
            const int nChars = 256;
            var stringBuilder = new StringBuilder(nChars);

            // get the title of the window
            if (GetWindowText(hwnd, stringBuilder, nChars) > 0)
            {
                var title = stringBuilder.ToString();

                if (_ignoredWindows.Contains(title) || !HasTextFocus(hwnd))
                    return;

                var rect = new RECT();
                GetWindowRect(hwnd, ref rect);
                _state.ActiveWindowInfo = new ActiveWindowInfo(title, rect);

                //var inactiveScreens = GetNonIntersectingScreens(rect, -20);
                var activeScreen = GetIntersectingScreen(rect, -20);
                _state.FocusedScreen = activeScreen;
            }

        }

        public static List<Screen> GetNonIntersectingScreens(RECT rect, int sensitivity)
        {
            var nonIntersectingScreens = new List<Screen>();

            foreach (var screen in Screen.AllScreens)
            {
                System.Drawing.Rectangle screenBounds = screen.Bounds;

                // Expand the screen bounds by the sensitivity amount
                screenBounds.Inflate(sensitivity, sensitivity);

                // Check if the expanded screen intersects with the given rectangle
                if (screenBounds.IntersectsWith(System.Drawing.Rectangle.FromLTRB(rect.left, rect.top, rect.right, rect.bottom)))
                {
                    // Screen intersects with the given rectangle
                    continue;
                }

                // Screen does not intersect with the given rectangle
                nonIntersectingScreens.Add(screen);
            }

            return nonIntersectingScreens;
        }

        public static Screen GetIntersectingScreen(RECT rect, int sensitivity)
        {
            foreach (var screen in Screen.AllScreens)
            {
                System.Drawing.Rectangle screenBounds = screen.Bounds;

                // Expand the screen bounds by the sensitivity amount
                screenBounds.Inflate(sensitivity, sensitivity);

                // Check if the expanded screen intersects with the given rectangle
                if (screenBounds.IntersectsWith(System.Drawing.Rectangle.FromLTRB(rect.left, rect.top, rect.right, rect.bottom)))
                {
                    return screen;
                }
            }

            return Screen.PrimaryScreen;
        }

        private bool HasTextFocus(IntPtr windowHandle)
        {
            // Check if the window is in the foreground
            if (windowHandle == GetForegroundWindow())
            {
                return true;
            }

            return false;
        }

        public void Dispose()
        {
            UnhookWinEvent(_windowsFocusHook);
            UnhookWinEvent(_windowsResizedHook);
        }
    }
}
