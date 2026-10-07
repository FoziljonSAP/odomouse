using System;
using System.Collections.Generic;
using System.Web.Script.Serialization;

namespace Odomouse
{
    /// <summary>
    /// Monitors in physical pixels (the process is per-monitor DPI aware, so
    /// cursor positions use the same space), with the physical size Windows
    /// reports from EDID. The core checks that size and falls back to an
    /// estimate from the monitor's DPI setting when it looks wrong.
    /// </summary>
    internal static class Displays
    {
        public static string Json()
        {
            var list = new List<Dictionary<string, object>>();
            Win32.MonitorEnumProc proc = (IntPtr hMonitor, IntPtr hdc, ref Win32.RECT rect, IntPtr data) =>
            {
                var info = new Win32.MONITORINFOEX { cbSize = System.Runtime.InteropServices.Marshal.SizeOf(typeof(Win32.MONITORINFOEX)) };
                if (!Win32.GetMonitorInfo(hMonitor, ref info)) return true;
                var r = info.rcMonitor;
                double widthMm = 0, heightMm = 0;
                IntPtr dc = Win32.CreateDC("DISPLAY", info.szDevice, null, IntPtr.Zero);
                if (dc != IntPtr.Zero)
                {
                    widthMm = Win32.GetDeviceCaps(dc, Win32.HORZSIZE);
                    heightMm = Win32.GetDeviceCaps(dc, Win32.VERTSIZE);
                    Win32.DeleteDC(dc);
                }
                uint dpi = Win32.MonitorDpi(hMonitor);
                list.Add(new Dictionary<string, object>
                {
                    ["id"] = info.szDevice,
                    ["label"] = FriendlyName(info.szDevice),
                    ["internal"] = false,
                    ["scaleFactor"] = dpi / 96.0,
                    ["bounds"] = new Dictionary<string, object>
                    {
                        ["x"] = r.Left, ["y"] = r.Top, ["width"] = r.Right - r.Left, ["height"] = r.Bottom - r.Top,
                    },
                    ["widthMm"] = widthMm,
                    ["heightMm"] = heightMm,
                    // Windows picks the default scaling so that 96 * scale is close to the real pixel density.
                    ["estimatePpi"] = (double)dpi,
                });
                return true;
            };
            Win32.EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, proc, IntPtr.Zero);
            GC.KeepAlive(proc);
            return new JavaScriptSerializer().Serialize(list);
        }

        private static string FriendlyName(string device)
        {
            var dd = new Win32.DISPLAY_DEVICE { cb = System.Runtime.InteropServices.Marshal.SizeOf(typeof(Win32.DISPLAY_DEVICE)) };
            if (Win32.EnumDisplayDevices(device, 0, ref dd, 0) && !string.IsNullOrWhiteSpace(dd.DeviceString)
                && dd.DeviceString.IndexOf("Generic", StringComparison.OrdinalIgnoreCase) < 0)
                return dd.DeviceString.Trim();
            return "Monitor " + device.Replace(@"\\.\DISPLAY", "");
        }
    }
}
