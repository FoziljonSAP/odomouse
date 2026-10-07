using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Web.Script.Serialization;
using System.Windows.Forms;
using Microsoft.Web.WebView2.Core;
using Microsoft.Web.WebView2.WinForms;

namespace Odomouse
{
    /// <summary>Answers window.odomouse.* calls coming from a page.</summary>
    internal interface IBridgeHost
    {
        void Handle(string method, object[] args, WebWindow from, Action<string> reply);
        string Theme { get; }
        bool IsDark { get; }
        void WebViewFailed(Exception error);
    }

    /// <summary>
    /// A window showing one page of the shared web UI (src/ui) in WebView2.
    /// It is created only while it is needed, so the app stays light when idle.
    /// </summary>
    internal sealed class WebWindow : Form
    {
        private const string Host = "odomouse.app";
        private static readonly JavaScriptSerializer Json = new JavaScriptSerializer { MaxJsonLength = int.MaxValue };
        private readonly IBridgeHost host;
        private readonly string page;
        private readonly string query;
        private WebView2 web;
        private bool ready;
        private readonly List<string> queuedScripts = new List<string>();

        public string PageName => page;

        public WebWindow(IBridgeHost host, string page, string query, bool popup)
        {
            this.host = host;
            this.page = page;
            this.query = query ?? "";
            Text = "Odomouse";
            Icon = TrayApp.AppIcon;
            BackColor = host.IsDark ? Color.FromArgb(0x22, 0x25, 0x2a) : Color.FromArgb(0xf8, 0xf9, 0xfb);
            StartPosition = FormStartPosition.Manual;
            if (popup)
            {
                FormBorderStyle = FormBorderStyle.None;
                ShowInTaskbar = false;
                TopMost = true;
            }
            web = new WebView2 { Dock = DockStyle.Fill };
            try { web.DefaultBackgroundColor = BackColor; } catch (Exception) { }
            Controls.Add(web);
            HandleCreated += (s, e) => Win32.StyleWindow(Handle, host.IsDark, true);
            Load += async (s, e) => await InitAsync();
        }

        public static string WebDataDir => Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Odomouse", "WebView2");

        private async System.Threading.Tasks.Task InitAsync()
        {
            try
            {
                var env = await CoreWebView2Environment.CreateAsync(null, WebDataDir);
                await web.EnsureCoreWebView2Async(env);
            }
            catch (Exception ex)
            {
                host.WebViewFailed(ex);
                Close();
                return;
            }
            var cw = web.CoreWebView2;
            cw.Settings.AreDevToolsEnabled = false;
            cw.Settings.IsStatusBarEnabled = false;
            cw.Settings.AreDefaultContextMenusEnabled = false;
            cw.Settings.IsZoomControlEnabled = false;
            cw.Settings.IsPasswordAutosaveEnabled = false;
            cw.Settings.IsGeneralAutofillEnabled = false;
            string root = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "web");
            cw.SetVirtualHostNameToFolderMapping(Host, root, CoreWebView2HostResourceAccessKind.Allow);
            await cw.AddScriptToExecuteOnDocumentCreatedAsync(
                "window.ODOMOUSE_PLATFORM = 'windows'; window.ODOMOUSE_THEME = " + Json.Serialize(host.Theme) + ";");
            cw.WebMessageReceived += OnMessage;
            cw.NavigationStarting += (s, e) =>
            {
                if (!e.Uri.StartsWith("https://" + Host + "/", StringComparison.OrdinalIgnoreCase)) e.Cancel = true;
            };
            cw.NewWindowRequested += (s, e) => e.Handled = true;
            cw.NavigationCompleted += (s, e) =>
            {
                ready = true;
                foreach (var script in queuedScripts) cw.ExecuteScriptAsync(script);
                queuedScripts.Clear();
            };
            cw.Navigate("https://" + Host + "/ui/" + page + (query.Length > 0 ? "?" + query : ""));
        }

        private void Run(string script)
        {
            if (IsDisposed || web == null) return;
            if (!ready || web.CoreWebView2 == null) { queuedScripts.Add(script); return; }
            web.CoreWebView2.ExecuteScriptAsync(script);
        }

        /// <summary>window.__odomouseEmit(name, payload); json must be valid JSON.</summary>
        public void Emit(string name, string json)
        {
            if (!ready) return; // the page asks for fresh data when it loads anyway
            Run("window.__odomouseEmit && window.__odomouseEmit(" + Json.Serialize(name) + ", " + json + ")");
        }

        public void ApplyTheme()
        {
            BackColor = host.IsDark ? Color.FromArgb(0x22, 0x25, 0x2a) : Color.FromArgb(0xf8, 0xf9, 0xfb);
            if (IsHandleCreated) Win32.StyleWindow(Handle, host.IsDark, true);
        }

        private void OnMessage(object sender, CoreWebView2WebMessageReceivedEventArgs e)
        {
            Dictionary<string, object> msg;
            try { msg = Json.DeserializeObject(e.WebMessageAsJson) as Dictionary<string, object>; }
            catch (Exception) { return; }
            if (msg == null || !msg.TryGetValue("id", out object idObj) || !msg.TryGetValue("method", out object methodObj)) return;
            long id = Convert.ToInt64(idObj);
            string method = methodObj as string ?? "";
            object[] args = msg.TryGetValue("args", out object a) && a is object[] arr ? arr : new object[0];
            host.Handle(method, args, this, json => Run("window.__odomouseReply(" + id + ", " + json + ")"));
        }

        protected override void Dispose(bool disposing)
        {
            if (disposing && web != null)
            {
                web.Dispose(); // ends the WebView2 processes for this window
                web = null;
            }
            base.Dispose(disposing);
        }
    }
}
