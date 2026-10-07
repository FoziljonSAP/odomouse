using System;
using System.Runtime.InteropServices;

namespace Odomouse
{
    /// <summary>
    /// Low-level keyboard and mouse hooks. They only observe (every event is
    /// passed on unchanged) and need no special permission on Windows. The
    /// callbacks run on the UI thread's message loop and return immediately.
    /// </summary>
    internal sealed class InputHooks : IDisposable
    {
        private readonly Core core;
        private readonly Win32.HookProc keyboardProc;
        private readonly Win32.HookProc mouseProc;
        private IntPtr keyboardHook = IntPtr.Zero;
        private IntPtr mouseHook = IntPtr.Zero;

        public bool Running => keyboardHook != IntPtr.Zero && mouseHook != IntPtr.Zero;

        public InputHooks(Core core)
        {
            this.core = core;
            // keep the delegates in fields so the GC never collects them
            keyboardProc = OnKeyboard;
            mouseProc = OnMouse;
        }

        public bool Start()
        {
            if (Running) return true;
            IntPtr module = Win32.GetModuleHandle(null);
            if (keyboardHook == IntPtr.Zero) keyboardHook = Win32.SetWindowsHookEx(Win32.WH_KEYBOARD_LL, keyboardProc, module, 0);
            if (mouseHook == IntPtr.Zero) mouseHook = Win32.SetWindowsHookEx(Win32.WH_MOUSE_LL, mouseProc, module, 0);
            core.SetHooksRunning(Running);
            return Running;
        }

        public void Dispose()
        {
            if (keyboardHook != IntPtr.Zero) Win32.UnhookWindowsHookEx(keyboardHook);
            if (mouseHook != IntPtr.Zero) Win32.UnhookWindowsHookEx(mouseHook);
            keyboardHook = mouseHook = IntPtr.Zero;
        }

        private static uint Mods()
        {
            uint m = 0;
            if (Win32.KeyHeld(Win32.VK_CONTROL)) m |= 1;
            if (Win32.KeyHeld(Win32.VK_MENU)) m |= 2;
            if (Win32.KeyHeld(Win32.VK_SHIFT)) m |= 4;
            if (Win32.KeyHeld(Win32.VK_LWIN) || Win32.KeyHeld(Win32.VK_RWIN)) m |= 8;
            return m;
        }

        // KBDLLHOOKSTRUCT: vkCode 0, scanCode 4, flags 8, time 12
        private IntPtr OnKeyboard(int nCode, IntPtr wParam, IntPtr lParam)
        {
            if (nCode >= 0)
            {
                try
                {
                    int msg = wParam.ToInt32();
                    int flags = Marshal.ReadInt32(lParam, 8);
                    if ((flags & Win32.LLKHF_INJECTED) == 0)
                    {
                        uint scan = (uint)Marshal.ReadInt32(lParam, 4);
                        uint vc = Core.VcFromScan(scan, (flags & Win32.LLKHF_EXTENDED) != 0);
                        if (vc != 0)
                        {
                            if (msg == Win32.WM_KEYDOWN || msg == Win32.WM_SYSKEYDOWN) core.KeyDown(vc, Mods());
                            else if (msg == Win32.WM_KEYUP || msg == Win32.WM_SYSKEYUP) core.KeyUp(vc);
                        }
                    }
                }
                catch (Exception) { /* never break the user's keyboard */ }
            }
            return Win32.CallNextHookEx(keyboardHook, nCode, wParam, lParam);
        }

        // MSLLHOOKSTRUCT: pt.x 0, pt.y 4, mouseData 8, flags 12
        private IntPtr OnMouse(int nCode, IntPtr wParam, IntPtr lParam)
        {
            if (nCode >= 0)
            {
                try
                {
                    int msg = wParam.ToInt32();
                    int flags = Marshal.ReadInt32(lParam, 12);
                    if ((flags & Win32.LLMHF_INJECTED) == 0)
                    {
                        int x = Marshal.ReadInt32(lParam, 0);
                        int y = Marshal.ReadInt32(lParam, 4);
                        switch (msg)
                        {
                            case Win32.WM_MOUSEMOVE: core.MouseMove(x, y); break;
                            case Win32.WM_LBUTTONDOWN: core.MouseDown(1); break;
                            case Win32.WM_RBUTTONDOWN: core.MouseDown(2); break;
                            case Win32.WM_MBUTTONDOWN: core.MouseDown(3); break;
                            case Win32.WM_XBUTTONDOWN: core.MouseDown(4); break;
                            case Win32.WM_MOUSEWHEEL:
                            case Win32.WM_MOUSEHWHEEL:
                                int delta = (short)((Marshal.ReadInt32(lParam, 8) >> 16) & 0xFFFF);
                                core.Wheel(x, y, WheelPixels(delta, x, y));
                                break;
                        }
                    }
                }
                catch (Exception) { }
            }
            return Win32.CallNextHookEx(mouseHook, nCode, wParam, lParam);
        }

        /// <summary>
        /// One wheel notch (120) scrolls "lines" lines (3 by default); a line
        /// of text is about 20 px at 100 % scaling.
        /// </summary>
        private static double WheelPixels(int delta, int x, int y)
        {
            int lines = System.Windows.Forms.SystemInformation.MouseWheelScrollLines;
            if (lines <= 0 || lines > 100) lines = 3; // -1 means "one page"
            double scale = Win32.DpiAt(x, y) / 96.0;
            return Math.Abs(delta) / 120.0 * lines * 20.0 * scale;
        }
    }
}
