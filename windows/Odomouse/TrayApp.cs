using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Globalization;
using System.IO;
using System.Reflection;
using System.Text;
using System.Threading;
using System.Web.Script.Serialization;
using System.Windows.Forms;
using Microsoft.Win32;

namespace Odomouse
{
    /// <summary>
    /// The tray app: wires Windows (hooks, tray icon, windows, notifications)
    /// to the Rust core. Like the macOS app it only decides *when*; the core
    /// decides *what* to count.
    /// </summary>
    internal sealed class TrayApp : ApplicationContext, IBridgeHost
    {
        private static readonly JavaScriptSerializer Json = new JavaScriptSerializer { MaxJsonLength = int.MaxValue };
        private static Icon appIcon;
        public static Icon AppIcon => appIcon ?? (appIcon = LoadIcon());

        private readonly Core core;
        private readonly InputHooks hooks;
        private readonly NotifyIcon tray;
        private readonly System.Windows.Forms.Timer timer;
        private readonly System.Windows.Forms.Timer popupReaper;
        private readonly Control invoker;
        private readonly Win32.WinEventProc foregroundProc;
        private IntPtr foregroundHook = IntPtr.Zero;
        private readonly Dictionary<uint, string> appNames = new Dictionary<uint, string>();
        private RegisteredWaitHandle showWait, quitWait;

        private WebWindow popup;
        private WebWindow dashboard;
        private DateTime popupHiddenAt = DateTime.MinValue;
        private Dictionary<string, object> settings = new Dictionary<string, object>();
        private string lastTip;
        private bool webViewWarned;

        public static string DataDir => Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Odomouse");

        public TrayApp(bool background, EventWaitHandle showEvent, EventWaitHandle quitEvent)
        {
            invoker = new Control();
            invoker.CreateControl();
            _ = invoker.Handle;

            core = new Core(DataDir);
            // "auto" language follows Windows until the user picks one
            core.Call("setSystemLanguage", Json.Serialize(new[] { CultureInfo.CurrentUICulture.Name }));
            ReloadSettings();
            core.SetDisplays(Displays.Json());
            SystemEvents.DisplaySettingsChanged += OnDisplaysChanged;
            SystemEvents.PowerModeChanged += OnPowerModeChanged;

            hooks = new InputHooks(core);
            hooks.Start();

            foregroundProc = OnForeground;
            foregroundHook = Win32.SetWinEventHook(Win32.EVENT_SYSTEM_FOREGROUND, Win32.EVENT_SYSTEM_FOREGROUND,
                IntPtr.Zero, foregroundProc, 0, 0, Win32.WINEVENT_OUTOFCONTEXT);

            var menu = new ContextMenuStrip();
            statsItem = menu.Items.Add("", null, (s, e) => OpenDashboard("stats"));
            settingsItem = menu.Items.Add("", null, (s, e) => OpenDashboard("settings"));
            menu.Items.Add(new ToolStripSeparator());
            quitItem = menu.Items.Add("", null, (s, e) => ExitThread());
            tray = new NotifyIcon { Icon = AppIcon, Text = "Odomouse", ContextMenuStrip = menu, Visible = true };
            tray.MouseClick += (s, e) => { if (e.Button == MouseButtons.Left) TogglePopup(); };

            timer = new System.Windows.Forms.Timer { Interval = 1000 };
            timer.Tick += (s, e) => Tick();
            timer.Start();

            popupReaper = new System.Windows.Forms.Timer { Interval = 45000 };
            popupReaper.Tick += (s, e) => DropPopup();

            // a second launch (Start menu, desktop shortcut) opens the dashboard here
            showWait = ThreadPool.RegisterWaitForSingleObject(showEvent,
                (state, timedOut) => invoker.BeginInvoke((Action)(() => OpenDashboard("stats"))), null, -1, false);
            // "Odomouse.exe --quit" (the installer, before an update or uninstall)
            quitWait = ThreadPool.RegisterWaitForSingleObject(quitEvent,
                (state, timedOut) => invoker.BeginInvoke((Action)ExitThread), null, -1, true);

            ApplySettings(false);
            Tick();
            // Windows 11 puts new tray icons under ^: say where the counter is until the guide is done
            if (!Flag("onboardingDone", false))
                tray.ShowBalloonTip(10000, S("trayHintTitle"), S("trayHintBody"), ToolTipIcon.Info);
            if (!background) OpenDashboard("stats");
        }

        private static Icon LoadIcon()
        {
            using (var s = Assembly.GetExecutingAssembly().GetManifestResourceStream("app.ico"))
                return s != null ? new Icon(s, SystemInformation.SmallIconSize) : SystemIcons.Application;
        }

        // ------------------------------------------------------------ timer

        private void Tick()
        {
            if (!hooks.Running && hooks.Start()) PushLive();
            bool hasCursor = Win32.GetCursorPos(out Win32.POINT p);
            var result = Json.DeserializeObject(core.Tick(hasCursor, p.X, p.Y)) as Dictionary<string, object>;
            if (result != null)
            {
                SetTip(result.TryGetValue("tray", out object t) ? t as string : null);
                if (result.TryGetValue("notify", out object n) && n is Dictionary<string, object> note)
                    tray.ShowBalloonTip(15000, note["title"] as string, note["body"] as string, ToolTipIcon.Info);
            }
            if (popup != null && popup.Visible) PushLive();
        }

        private void SetTip(string text)
        {
            string tip = string.IsNullOrEmpty(text) ? "Odomouse" : "Odomouse: " + text;
            if (tip.Length > 63) tip = tip.Substring(0, 63); // NotifyIcon limit on .NET Framework
            if (tip == lastTip) return;
            lastTip = tip;
            tray.Text = tip;
        }

        private void PushLive()
        {
            if (popup != null && popup.Visible) popup.Emit("live", core.Call("getLive"));
        }

        private void OnDisplaysChanged(object sender, EventArgs e)
        {
            invoker.BeginInvoke((Action)(() =>
            {
                core.SetDisplays(Displays.Json());
                dashboard?.Emit("displays", core.Call("getDisplays"));
            }));
        }

        private void OnPowerModeChanged(object sender, PowerModeChangedEventArgs e)
        {
            if (e.Mode == PowerModes.Suspend) core.Save();
        }

        private void OnForeground(IntPtr hook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint thread, uint time)
        {
            core.SetApp(AppName(hwnd));
        }

        /// <summary>"Google Chrome" (file description) or the process name.</summary>
        private string AppName(IntPtr hwnd)
        {
            Win32.GetWindowThreadProcessId(hwnd, out uint pid);
            if (pid == 0) return null;
            if (appNames.TryGetValue(pid, out string cached)) return cached;
            string name = null;
            try
            {
                using (var p = Process.GetProcessById((int)pid))
                {
                    try { name = p.MainModule.FileVersionInfo.FileDescription; } catch (Exception) { }
                    if (string.IsNullOrWhiteSpace(name)) name = p.ProcessName;
                }
            }
            catch (Exception) { }
            if (appNames.Count > 500) appNames.Clear();
            appNames[pid] = name;
            return name;
        }

        // --------------------------------------------------------- settings

        public string Theme => settings.TryGetValue("theme", out object t) && t is string s ? s : "system";

        public bool IsDark
        {
            get
            {
                if (Theme == "dark") return true;
                if (Theme == "light") return false;
                try
                {
                    using (var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"))
                        return key?.GetValue("AppsUseLightTheme") is int v && v == 0;
                }
                catch (Exception) { return false; }
            }
        }

        private bool Flag(string key, bool fallback) =>
            settings.TryGetValue(key, out object v) && v is bool b ? b : fallback;

        private void ReloadSettings()
        {
            settings = Json.DeserializeObject(core.Call("getSettings")) as Dictionary<string, object>
                       ?? new Dictionary<string, object>();
        }

        // menu and dialog texts in the current language (from the core)
        private Dictionary<string, object> strings = new Dictionary<string, object>();
        private ToolStripItem statsItem, settingsItem, quitItem;

        private string S(string key) => strings.TryGetValue(key, out object v) && v is string str ? str : key;

        private void ReloadStrings()
        {
            strings = Json.DeserializeObject(core.Call("getStrings")) as Dictionary<string, object>
                      ?? new Dictionary<string, object>();
            statsItem.Text = S("stats");
            settingsItem.Text = S("settings");
            quitItem.Text = S("quit");
        }

        private void ApplySettings(bool broadcast = true)
        {
            ReloadStrings();
            SyncLaunchAtLogin(Flag("launchAtLogin", false));
            popup?.ApplyTheme();
            dashboard?.ApplyTheme();
            SetTip(core.TrayText());
            if (!broadcast) return;
            string json = Json.Serialize(settings);
            popup?.Emit("settings", json);
            dashboard?.Emit("settings", json);
            PushLive();
        }

        private static void SyncLaunchAtLogin(bool want)
        {
            try
            {
                using (var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Run", true))
                {
                    if (key == null) return;
                    if (key.GetValue("Mishka Tracker") != null) key.DeleteValue("Mishka Tracker", false); // the old name
                    if (want) key.SetValue("Odomouse", "\"" + Application.ExecutablePath + "\" --background");
                    else if (key.GetValue("Odomouse") != null) key.DeleteValue("Odomouse", false);
                }
            }
            catch (Exception) { }
        }

        // ----------------------------------------------------------- windows

        private void TogglePopup()
        {
            if (popup != null && popup.Visible) { popup.Hide(); return; }
            // clicking the icon first deactivates (hides) the popup: don't reopen it at once
            if ((DateTime.Now - popupHiddenAt).TotalMilliseconds < 300) return;
            popupReaper.Stop();
            if (popup == null || popup.IsDisposed)
            {
                popup = new WebWindow(this, "popup.html", null, true);
                popup.Deactivate += (s, e) => { popup.Hide(); popupHiddenAt = DateTime.Now; popupReaper.Start(); };
            }
            PlacePopup(popup);
            popup.Show();
            popup.Activate();
            PushLive();
        }

        private void DropPopup()
        {
            popupReaper.Stop();
            if (popup != null && !popup.Visible)
            {
                popup.Dispose();
                popup = null;
            }
        }

        private static void PlacePopup(Form f)
        {
            Win32.GetCursorPos(out Win32.POINT p);
            var screen = Screen.FromPoint(new Point(p.X, p.Y));
            Rectangle wa = screen.WorkingArea, b = screen.Bounds;
            double scale = Win32.DpiAt(p.X, p.Y) / 96.0;
            int w = (int)(360 * scale);
            int h = Math.Min((int)(640 * scale), wa.Height - 16);
            int x = Math.Max(wa.Left + 8, Math.Min(p.X - w / 2, wa.Right - w - 8));
            int y = wa.Top > b.Top ? wa.Top + 8 : wa.Bottom - h - 8; // taskbar on top or bottom
            f.Bounds = new Rectangle(x, y, w, h);
        }

        private void OpenDashboard(string tab)
        {
            if (popup != null && popup.Visible) popup.Hide();
            if (!Flag("seenDashboard", false))
            {
                core.Call("markDashboardSeen");
                ReloadSettings();
                ApplySettings();
            }
            if (dashboard != null && !dashboard.IsDisposed)
            {
                dashboard.Emit("tab", Json.Serialize(tab));
                if (dashboard.WindowState == FormWindowState.Minimized) dashboard.WindowState = FormWindowState.Normal;
                dashboard.Show();
                dashboard.Activate();
                return;
            }
            Win32.GetCursorPos(out Win32.POINT p);
            var wa = Screen.FromPoint(new Point(p.X, p.Y)).WorkingArea;
            double scale = Win32.DpiAt(p.X, p.Y) / 96.0;
            int w = Math.Min((int)(1040 * scale), wa.Width), h = Math.Min((int)(760 * scale), wa.Height);
            dashboard = new WebWindow(this, "dashboard.html", "tab=" + tab, false)
            {
                Bounds = new Rectangle(wa.Left + (wa.Width - w) / 2, wa.Top + (wa.Height - h) / 2, w, h),
                MinimumSize = new Size((int)(820 * scale), (int)(600 * scale)),
            };
            dashboard.FormClosed += (s, e) => { var d = dashboard; dashboard = null; d?.Dispose(); };
            dashboard.Show();
            dashboard.Activate();
        }

        public void WebViewFailed(Exception error)
        {
            if (webViewWarned) return;
            webViewWarned = true;
            var r = MessageBox.Show(
                S("webview2Missing") + "\n\n(" + error.Message + ")",
                "Odomouse", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
            if (r == DialogResult.Yes)
                Process.Start(new ProcessStartInfo("https://developer.microsoft.com/microsoft-edge/webview2/") { UseShellExecute = true });
        }

        // ------------------------------------------- window.odomouse.* bridge

        public void Handle(string method, object[] args, WebWindow from, Action<string> reply)
        {
            string argsJson = Json.Serialize(args);
            string first = args.Length > 0 ? args[0] as string : null;
            switch (method)
            {
                case "getLive":
                case "getDashboard":
                case "getWrapped":
                    reply(core.Call(method, argsJson));
                    break;
                case "updateSettings":
                    {
                        string result = core.Call(method, argsJson);
                        ReloadSettings();
                        ApplySettings();
                        reply(result);
                        break;
                    }
                case "openDashboard":
                    reply("null");
                    OpenDashboard(first == "settings" ? "settings" : "stats");
                    break;
                case "exportCsv":
                    reply(ExportCsv(from));
                    break;
                case "resetData":
                    if (Confirm(from, S("resetTitle"), S("resetDetail")))
                    {
                        reply(core.Call("resetData"));
                        SetTip(core.TrayText());
                        PushLive();
                    }
                    else reply("{\"ok\":false}");
                    break;
                case "clearApps":
                    reply(Confirm(from, S("appsTitle"), S("appsDetail"))
                        ? core.Call("clearApps") : "{\"ok\":false}");
                    break;
                case "saveWrapped":
                    reply(SaveWrapped(from, first, args.Length > 1 ? args[1] as string : null));
                    break;
                case "copyWrapped":
                    {
                        byte[] png = PngData(first);
                        if (png == null) { reply("{\"ok\":false}"); break; }
                        using (var ms = new MemoryStream(png))
                        using (var img = Image.FromStream(ms))
                            Clipboard.SetImage(img);
                        reply("{\"ok\":true}");
                        break;
                    }
                case "openPermissions":
                    reply("null");
                    bool ok = hooks.Running || hooks.Start();
                    MessageBox.Show(from,
                        ok ? S("noPermissionNeededWindows") : S("hooksFailedWindows"),
                        "Odomouse", MessageBoxButtons.OK, ok ? MessageBoxIcon.Information : MessageBoxIcon.Warning);
                    break;
                case "quit":
                    reply("null");
                    ExitThread();
                    break;
                default:
                    reply("{\"error\":" + Json.Serialize("unknown method: " + method) + "}");
                    break;
            }
        }

        private static bool Confirm(IWin32Window owner, string message, string detail) =>
            MessageBox.Show(owner, message + "\n\n" + detail, "Odomouse", MessageBoxButtons.YesNo,
                MessageBoxIcon.Warning, MessageBoxDefaultButton.Button2) == DialogResult.Yes;

        private string ExportCsv(IWin32Window owner)
        {
            var result = Json.DeserializeObject(core.Call("exportCsv")) as Dictionary<string, object>;
            if (result == null || !(result["csv"] is string csv)) return "{\"ok\":false}";
            using (var dlg = new SaveFileDialog
            {
                Title = S("saveCsv"),
                Filter = "CSV|*.csv",
                FileName = "odomouse-" + (result["date"] as string ?? "data") + ".csv",
                InitialDirectory = Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments),
            })
            {
                if (dlg.ShowDialog(owner) != DialogResult.OK) return "{\"ok\":false,\"canceled\":true}";
                try
                {
                    File.WriteAllText(dlg.FileName, csv, new UTF8Encoding(false));
                    return Json.Serialize(new Dictionary<string, object> { ["ok"] = true, ["filePath"] = dlg.FileName });
                }
                catch (Exception) { return "{\"ok\":false}"; }
            }
        }

        private static byte[] PngData(string dataUrl)
        {
            const string prefix = "data:image/png;base64,";
            if (dataUrl == null || !dataUrl.StartsWith(prefix) || dataUrl.Length > 30000000) return null;
            try { return Convert.FromBase64String(dataUrl.Substring(prefix.Length)); }
            catch (FormatException) { return null; }
        }

        private string SaveWrapped(IWin32Window owner, string dataUrl, string period)
        {
            byte[] png = PngData(dataUrl);
            if (png == null) return "{\"ok\":false}";
            using (var dlg = new SaveFileDialog
            {
                Title = S("saveWrapped"),
                Filter = "PNG|*.png",
                FileName = "odomouse-wrapped-" + (period ?? "week") + "-" + DateTime.Now.ToString("yyyy-MM-dd") + ".png",
                InitialDirectory = Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory),
            })
            {
                if (dlg.ShowDialog(owner) != DialogResult.OK) return "{\"ok\":false,\"canceled\":true}";
                try
                {
                    File.WriteAllBytes(dlg.FileName, png);
                    Process.Start("explorer.exe", "/select,\"" + dlg.FileName + "\"");
                    return Json.Serialize(new Dictionary<string, object> { ["ok"] = true, ["filePath"] = dlg.FileName });
                }
                catch (Exception) { return "{\"ok\":false}"; }
            }
        }

        // ------------------------------------------------------------- exit

        protected override void ExitThreadCore()
        {
            timer.Stop();
            popupReaper.Stop();
            showWait?.Unregister(null);
            quitWait?.Unregister(null);
            SystemEvents.DisplaySettingsChanged -= OnDisplaysChanged;
            SystemEvents.PowerModeChanged -= OnPowerModeChanged;
            if (foregroundHook != IntPtr.Zero) Win32.UnhookWinEvent(foregroundHook);
            hooks.Dispose();
            popup?.Dispose();
            dashboard?.Dispose();
            tray.Visible = false;
            tray.Dispose();
            core.Dispose(); // saves today
            base.ExitThreadCore();
        }
    }
}
