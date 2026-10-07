using System;
using System.Linq;
using System.Threading;
using System.Windows.Forms;

namespace Odomouse
{
    internal static class Program
    {
        [STAThread]
        private static int Main(string[] args)
        {
            using (var mutex = new Mutex(true, @"Local\Odomouse", out bool first))
            using (var show = new EventWaitHandle(false, EventResetMode.AutoReset, @"Local\Odomouse.Show"))
            using (var quit = new EventWaitHandle(false, EventResetMode.AutoReset, @"Local\Odomouse.Quit"))
            {
                if (args.Contains("--quit"))
                {
                    // used by the installer: ask a running copy to save and exit
                    if (first) return 0;
                    quit.Set();
                    try { if (mutex.WaitOne(8000)) mutex.ReleaseMutex(); }
                    catch (AbandonedMutexException) { }
                    return 0;
                }
                if (!first)
                {
                    show.Set(); // already running: ask that copy to open its window
                    return 0;
                }
                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                try
                {
                    Core.LoadNative();
                    using (var app = new TrayApp(args.Contains("--background"), show, quit))
                        Application.Run(app);
                }
                catch (Exception ex)
                {
                    // the core (and its texts) may not have loaded: pick the language here
                    string os = System.Globalization.CultureInfo.CurrentUICulture.TwoLetterISOLanguageName;
                    string title = os == "uz" ? "Odomouse ishga tushmadi" : os == "ru" ? "Odomouse не запустился" : "Odomouse couldn't start";
                    MessageBox.Show(title + ":\n\n" + ex.Message, "Odomouse",
                        MessageBoxButtons.OK, MessageBoxIcon.Error);
                    return 1;
                }
                GC.KeepAlive(mutex);
                return 0;
            }
        }
    }
}
