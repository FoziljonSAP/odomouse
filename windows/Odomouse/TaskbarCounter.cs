using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Drawing.Text;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using Microsoft.Win32;

namespace Odomouse
{
    /// <summary>
    /// Today's number on the taskbar, next to the clock, like the macOS menu
    /// bar text. A notification-area icon cannot show text, so this is a tiny
    /// always-on-top layered window placed over the taskbar. It can be dragged
    /// anywhere (the place is remembered), hides while a full-screen app is in
    /// front, and acts like the tray icon: click for the popup, right-click
    /// for the menu, double-click for the statistics.
    /// </summary>
    internal sealed class TaskbarCounter : Form
    {
        private const string Key = @"Software\Odomouse";
        private readonly Icon icon;
        private string text = "";
        private bool lightTaskbar;
        private Point? saved;            // a place the user dragged it to
        private Point downAt;
        private Point downLocation;
        private bool pressed, dragged;

        public event Action Clicked;
        public event Action DoubleClicked;
        public ContextMenuStrip TrayMenu { get; set; }

        public TaskbarCounter(Icon appIcon)
        {
            icon = appIcon;
            FormBorderStyle = FormBorderStyle.None;
            ShowInTaskbar = false;
            StartPosition = FormStartPosition.Manual;
            TopMost = true;
            saved = LoadPlace();
        }

        protected override CreateParams CreateParams
        {
            get
            {
                var cp = base.CreateParams;
                cp.ExStyle |= Win32.WS_EX_LAYERED | Win32.WS_EX_TOOLWINDOW | Win32.WS_EX_NOACTIVATE | Win32.WS_EX_TOPMOST;
                return cp;
            }
        }

        protected override bool ShowWithoutActivation => true;

        // ------------------------------------------------------------ text

        public void SetText(string value)
        {
            value = value ?? "";
            bool light = TaskbarIsLight();
            if (value == text && light == lightTaskbar && IsHandleCreated) return;
            text = value;
            lightTaskbar = light;
            Render();
        }

        private static bool TaskbarIsLight()
        {
            try
            {
                using (var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"))
                    return key?.GetValue("SystemUsesLightTheme") is int v && v == 1;
            }
            catch (Exception) { return false; }
        }

        /// <summary>Draws the pill into a per-pixel-alpha bitmap: smooth on any taskbar colour.</summary>
        private void Render()
        {
            if (!IsHandleCreated) CreateHandle();
            var tb = Taskbar();
            double scale = Win32.DpiAt(tb.Left + tb.Width / 2, tb.Top + tb.Height / 2) / 96.0;
            int h = (int)Math.Round(26 * scale);
            int pad = (int)Math.Round(9 * scale);
            int iconSize = (int)Math.Round(16 * scale);
            int gap = (int)Math.Round(6 * scale);
            using (var font = new Font("Segoe UI Semibold", (float)(13 * scale), FontStyle.Regular, GraphicsUnit.Pixel))
            {
                Size textSize;
                using (var probe = new Bitmap(1, 1))
                using (var g0 = Graphics.FromImage(probe))
                {
                    g0.TextRenderingHint = TextRenderingHint.AntiAliasGridFit;
                    var sz = g0.MeasureString(text.Length == 0 ? " " : text, font, PointF.Empty, StringFormat.GenericTypographic);
                    textSize = new Size((int)Math.Ceiling(sz.Width), (int)Math.Ceiling(sz.Height));
                }
                int w = pad + iconSize + (text.Length > 0 ? gap + textSize.Width : 0) + pad;
                using (var bmp = new Bitmap(w, h, PixelFormat.Format32bppArgb))
                {
                    using (var g = Graphics.FromImage(bmp))
                    {
                        g.Clear(Color.Transparent);
                        g.SmoothingMode = SmoothingMode.AntiAlias;
                        g.TextRenderingHint = TextRenderingHint.AntiAliasGridFit;
                        g.InterpolationMode = InterpolationMode.HighQualityBicubic;
                        var pill = lightTaskbar ? Color.FromArgb(150, 255, 255, 255) : Color.FromArgb(120, 60, 63, 70);
                        var ink = lightTaskbar ? Color.FromArgb(255, 26, 28, 33) : Color.FromArgb(255, 242, 241, 236);
                        using (var path = Rounded(new RectangleF(0.5f, 0.5f, w - 1, h - 1), (h - 1) / 2f))
                        using (var brush = new SolidBrush(pill))
                            g.FillPath(brush, path);
                        using (var small = IconAt(iconSize))
                            if (small != null) g.DrawIcon(small, new Rectangle(pad, (h - iconSize) / 2, iconSize, iconSize));
                        if (text.Length > 0)
                        {
                            using (var brush = new SolidBrush(ink))
                                g.DrawString(text, font, brush, pad + iconSize + gap, (h - textSize.Height) / 2f - (float)(scale * 0.5), StringFormat.GenericTypographic);
                        }
                    }
                    Size = new Size(w, h);
                    Location = Place(tb, w, h);
                    Win32.SetLayeredBitmap(Handle, bmp, Location);
                }
            }
        }

        /// <summary>The app icon at this size, picked from app.ico (sharp at 125-200% scale).</summary>
        private Icon IconAt(int size)
        {
            try
            {
                using (var s = System.Reflection.Assembly.GetExecutingAssembly().GetManifestResourceStream("app.ico"))
                    if (s != null) return new Icon(s, size, size);
            }
            catch (Exception) { }
            return icon == null ? null : new Icon(icon, size, size);
        }

        private static GraphicsPath Rounded(RectangleF r, float radius)
        {
            var p = new GraphicsPath();
            float d = radius * 2;
            p.AddArc(r.Left, r.Top, d, d, 180, 90);
            p.AddArc(r.Right - d, r.Top, d, d, 270, 90);
            p.AddArc(r.Right - d, r.Bottom - d, d, d, 0, 90);
            p.AddArc(r.Left, r.Bottom - d, d, d, 90, 90);
            p.CloseFigure();
            return p;
        }

        // ------------------------------------------------------- placement

        /// <summary>The taskbar of the primary screen, or a strip at the bottom if it cannot be found.</summary>
        private static Rectangle Taskbar()
        {
            IntPtr tray = Win32.FindWindow("Shell_TrayWnd", null);
            if (tray != IntPtr.Zero && Win32.GetWindowRect(tray, out Win32.RECT r))
                return Rectangle.FromLTRB(r.Left, r.Top, r.Right, r.Bottom);
            var b = Screen.PrimaryScreen.Bounds;
            return new Rectangle(b.Left, b.Bottom - 48, b.Width, 48);
        }

        private Point Place(Rectangle tb, int w, int h)
        {
            if (saved.HasValue && OnScreen(new Rectangle(saved.Value, new Size(w, h)))) return saved.Value;
            bool horizontal = tb.Width >= tb.Height;
            IntPtr tray = Win32.FindWindow("Shell_TrayWnd", null);
            IntPtr notify = tray == IntPtr.Zero ? IntPtr.Zero : Win32.FindWindowEx(tray, IntPtr.Zero, "TrayNotifyWnd", null);
            Rectangle n = Rectangle.Empty;
            if (notify != IntPtr.Zero && Win32.GetWindowRect(notify, out Win32.RECT nr))
                n = Rectangle.FromLTRB(nr.Left, nr.Top, nr.Right, nr.Bottom);
            int margin = h / 4;
            if (horizontal)
            {
                int right = n.Width > 0 ? n.Left - margin : tb.Right - h * 9;
                return new Point(right - w, tb.Top + (tb.Height - h) / 2);
            }
            int bottom = n.Height > 0 ? n.Top - margin : tb.Bottom - h * 8;
            return new Point(tb.Left + Math.Max(0, (tb.Width - w) / 2), bottom - h);
        }

        private static bool OnScreen(Rectangle r)
        {
            foreach (var s in Screen.AllScreens)
                if (s.Bounds.Contains(r.Left + r.Width / 2, r.Top + r.Height / 2)) return true;
            return false;
        }

        /// <summary>Call every second: back on top of the taskbar, hidden under full-screen apps.</summary>
        public void Keep(bool wanted)
        {
            bool show = wanted && !FullScreenInFront();
            if (!show)
            {
                if (Visible) Hide();
                return;
            }
            if (!Visible) { Render(); Show(); }
            // the taskbar raises itself when clicked: put the counter back above it
            Win32.SetWindowPos(Handle, Win32.HWND_TOPMOST, 0, 0, 0, 0,
                Win32.SWP_NOMOVE | Win32.SWP_NOSIZE | Win32.SWP_NOACTIVATE | Win32.SWP_SHOWWINDOW);
        }

        /// <summary>Taskbar moved or resized, displays changed: re-place unless the user put it somewhere.</summary>
        public void Relayout() { if (IsHandleCreated) Render(); }

        private static bool FullScreenInFront()
        {
            IntPtr fg = Win32.GetForegroundWindow();
            if (fg == IntPtr.Zero || fg == Win32.GetShellWindow()) return false;
            var cls = new System.Text.StringBuilder(64);
            Win32.GetClassName(fg, cls, cls.Capacity);
            string c = cls.ToString();
            if (c == "Progman" || c == "WorkerW" || c == "Shell_TrayWnd" || c == "Shell_SecondaryTrayWnd") return false;
            if (!Win32.GetWindowRect(fg, out Win32.RECT r)) return false;
            var s = Screen.FromHandle(fg).Bounds;
            return r.Left <= s.Left && r.Top <= s.Top && r.Right >= s.Right && r.Bottom >= s.Bottom;
        }

        public void ResetPlace()
        {
            saved = null;
            try { Registry.CurrentUser.CreateSubKey(Key)?.DeleteValue("CounterX", false); } catch (Exception) { }
            try { Registry.CurrentUser.CreateSubKey(Key)?.DeleteValue("CounterY", false); } catch (Exception) { }
            Render();
        }

        private static Point? LoadPlace()
        {
            try
            {
                using (var k = Registry.CurrentUser.OpenSubKey(Key))
                    if (k?.GetValue("CounterX") is int x && k.GetValue("CounterY") is int y) return new Point(x, y);
            }
            catch (Exception) { }
            return null;
        }

        private void SavePlace(Point p)
        {
            saved = p;
            try
            {
                using (var k = Registry.CurrentUser.CreateSubKey(Key))
                {
                    k?.SetValue("CounterX", p.X, RegistryValueKind.DWord);
                    k?.SetValue("CounterY", p.Y, RegistryValueKind.DWord);
                }
            }
            catch (Exception) { }
        }

        // ----------------------------------------------------------- mouse

        protected override void OnMouseDown(MouseEventArgs e)
        {
            base.OnMouseDown(e);
            if (e.Button != MouseButtons.Left) return;
            pressed = true;
            dragged = false;
            downAt = Cursor.Position;
            downLocation = Location;
        }

        protected override void OnMouseMove(MouseEventArgs e)
        {
            base.OnMouseMove(e);
            if (!pressed) return;
            var p = Cursor.Position;
            int dx = p.X - downAt.X, dy = p.Y - downAt.Y;
            if (!dragged && Math.Abs(dx) + Math.Abs(dy) < SystemInformation.DragSize.Width) return;
            dragged = true;
            Location = new Point(downLocation.X + dx, downLocation.Y + dy);
        }

        protected override void OnMouseUp(MouseEventArgs e)
        {
            base.OnMouseUp(e);
            if (e.Button == MouseButtons.Right)
            {
                TrayMenu?.Show(Cursor.Position);
                return;
            }
            if (!pressed) return;
            pressed = false;
            if (dragged) SavePlace(Location);
            else Clicked?.Invoke();
        }

        protected override void OnMouseDoubleClick(MouseEventArgs e)
        {
            base.OnMouseDoubleClick(e);
            if (e.Button == MouseButtons.Left) DoubleClicked?.Invoke();
        }
    }
}
