using System;
using System.IO;
using System.Runtime.InteropServices;

namespace Odomouse
{
    /// <summary>
    /// The Rust core (core/include/odomouse_core.h) through P/Invoke. All the
    /// counting rules live there; this class only forwards calls.
    /// </summary>
    internal sealed class Core : IDisposable
    {
        private const string Lib = "odomouse_core";
        private IntPtr handle;

        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr mk_open([MarshalAs(UnmanagedType.LPUTF8Str)] string dataDir, int tzOffsetS);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_close(IntPtr h);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_string_free(IntPtr s);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_mouse_move(IntPtr h, double x, double y);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_mouse_down(IntPtr h, uint button);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_wheel(IntPtr h, double x, double y, double points);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_key_down(IntPtr h, uint vc, uint mods);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_key_up(IntPtr h, uint vc);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern uint mk_vc_from_windows(uint scanCode, [MarshalAs(UnmanagedType.I1)] bool extended);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_set_displays(IntPtr h, [MarshalAs(UnmanagedType.LPUTF8Str)] string json);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_set_hooks_running(IntPtr h, [MarshalAs(UnmanagedType.I1)] bool running);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern void mk_set_app(IntPtr h, [MarshalAs(UnmanagedType.LPUTF8Str)] string name);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr mk_tick(IntPtr h, int tzOffsetS, [MarshalAs(UnmanagedType.I1)] bool hasCursor, double x, double y);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr mk_call(IntPtr h, [MarshalAs(UnmanagedType.LPUTF8Str)] string method, [MarshalAs(UnmanagedType.LPUTF8Str)] string argsJson);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr mk_tray_text(IntPtr h);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
        [return: MarshalAs(UnmanagedType.I1)]
        private static extern bool mk_save(IntPtr h);

        /// <summary>Load native\x64 or native\x86 before the first P/Invoke.</summary>
        public static void LoadNative()
        {
            string arch = Environment.Is64BitProcess ? "x64" : "x86";
            string path = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "native", arch, "odomouse_core.dll");
            if (Win32.LoadLibrary(path) == IntPtr.Zero)
                throw new DllNotFoundException("odomouse_core.dll topilmadi: " + path);
        }

        public static int TzOffset => (int)TimeZoneInfo.Local.GetUtcOffset(DateTime.Now).TotalSeconds;

        public Core(string dataDir)
        {
            Directory.CreateDirectory(dataDir);
            handle = mk_open(dataDir, TzOffset);
            if (handle == IntPtr.Zero) throw new IOException("Ma'lumotlar papkasini ochib bo'lmadi: " + dataDir);
        }

        public void Dispose()
        {
            if (handle != IntPtr.Zero)
            {
                mk_close(handle);
                handle = IntPtr.Zero;
            }
        }

        private static string Take(IntPtr p)
        {
            if (p == IntPtr.Zero) return "null";
            try { return Win32.Utf8(p); }
            finally { mk_string_free(p); }
        }

        public void MouseMove(double x, double y) => mk_mouse_move(handle, x, y);
        public void MouseDown(uint button) => mk_mouse_down(handle, button);
        public void Wheel(double x, double y, double points) => mk_wheel(handle, x, y, points);
        public void KeyDown(uint vc, uint mods) => mk_key_down(handle, vc, mods);
        public void KeyUp(uint vc) => mk_key_up(handle, vc);
        public static uint VcFromScan(uint scan, bool extended) => mk_vc_from_windows(scan, extended);

        public void SetDisplays(string json) => mk_set_displays(handle, json);
        public void SetHooksRunning(bool running) => mk_set_hooks_running(handle, running);
        public void SetApp(string name) => mk_set_app(handle, name);

        /// <summary>{"tray": "...", "notify": null | {"title", "body"}}</summary>
        public string Tick(bool hasCursor, double x, double y) => Take(mk_tick(handle, TzOffset, hasCursor, x, y));

        public string Call(string method, string argsJson = "[]") => Take(mk_call(handle, method, argsJson));

        public string TrayText() => Take(mk_tray_text(handle));

        public bool Save() => mk_save(handle);
    }
}
