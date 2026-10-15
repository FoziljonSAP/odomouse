using System;
using System.IO;
using System.Linq;
using System.Net;
using System.Security.Cryptography;
using System.Text;
using System.Threading.Tasks;

namespace Odomouse
{
    /// <summary>
    /// Downloads for the update check: latest.json (a public file on the
    /// website, nothing is sent) and, when the user agrees, the new installer,
    /// which is only run if its SHA-256 matches the one in latest.json.
    /// </summary>
    internal static class Updater
    {
        static Updater()
        {
            // .NET Framework 4.8 on older Windows may not offer TLS 1.2 by default
            ServicePointManager.SecurityProtocol |= SecurityProtocolType.Tls12;
        }

        private static WebClient Client()
        {
            var wc = new WebClient { Encoding = Encoding.UTF8 };
            wc.Headers[HttpRequestHeader.UserAgent] = "Odomouse";
            wc.Headers[HttpRequestHeader.CacheControl] = "no-cache";
            return wc;
        }

        public static async Task<string> FetchText(string url)
        {
            if (!url.StartsWith("https://", StringComparison.OrdinalIgnoreCase)) throw new InvalidOperationException("https only");
            using (var wc = Client())
                return await wc.DownloadStringTaskAsync(new Uri(url));
        }

        /// <summary>Saves the installer under %TEMP%\Odomouse and checks it. Returns its path.</summary>
        public static async Task<string> DownloadInstaller(string url, string sha256, string version)
        {
            if (!url.StartsWith("https://", StringComparison.OrdinalIgnoreCase)) throw new InvalidOperationException("https only");
            if (string.IsNullOrEmpty(sha256) || sha256.Length != 64) throw new InvalidOperationException("no checksum");
            string safe = new string((version ?? "").Where(c => char.IsLetterOrDigit(c) || c == '.' || c == '-').ToArray());
            string dir = Path.Combine(Path.GetTempPath(), "Odomouse");
            Directory.CreateDirectory(dir);
            string file = Path.Combine(dir, "Odomouse-Setup-" + (safe.Length > 0 ? safe : "new") + ".exe");
            string part = file + ".part";
            if (File.Exists(part)) File.Delete(part);
            using (var wc = Client())
                await wc.DownloadFileTaskAsync(new Uri(url), part);
            string actual;
            using (var sha = SHA256.Create())
            using (var s = File.OpenRead(part))
                actual = string.Concat(sha.ComputeHash(s).Select(b => b.ToString("x2")));
            if (!string.Equals(actual, sha256, StringComparison.OrdinalIgnoreCase))
            {
                File.Delete(part);
                throw new InvalidDataException("checksum mismatch");
            }
            if (File.Exists(file)) File.Delete(file);
            File.Move(part, file);
            return file;
        }
    }
}
