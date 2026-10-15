import AppKit
import CryptoKit

/// New versions: downloads latest.json (a public file on the website; nothing
/// about the user is sent) and, when the user agrees, the new disk image.
/// The image is only used if its SHA-256 matches latest.json.
enum Updater {
    private static func request(_ url: URL) -> URLRequest {
        var r = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 60)
        r.setValue("Odomouse", forHTTPHeaderField: "User-Agent")
        return r
    }

    private static func ok(_ response: URLResponse?) -> Bool {
        guard let http = response as? HTTPURLResponse else { return false }
        return (200..<300).contains(http.statusCode)
    }

    /// latest.json as text, on the main thread (nil + error text on failure).
    static func fetchText(_ feed: String, done: @escaping (String?, String) -> Void) {
        guard let url = URL(string: feed), url.scheme == "https" else { done(nil, "https only"); return }
        URLSession.shared.dataTask(with: request(url)) { data, response, error in
            let text = ok(response) ? data.flatMap { String(data: $0, encoding: .utf8) } : nil
            DispatchQueue.main.async { done(text, error?.localizedDescription ?? "download failed") }
        }.resume()
    }

    /// The disk image, saved and checked. Calls back on the main thread.
    static func downloadImage(_ url: URL, sha256: String, version: String, done: @escaping (URL?) -> Void) {
        let safe = version.filter { $0.isLetter || $0.isNumber || $0 == "." || $0 == "-" }
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("Odomouse-update", isDirectory: true)
        let dest = dir.appendingPathComponent("Odomouse-\(safe.isEmpty ? "new" : safe).dmg")
        URLSession.shared.downloadTask(with: request(url)) { tmp, response, _ in
            var result: URL?
            if let tmp = tmp, ok(response) {
                try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
                try? FileManager.default.removeItem(at: dest)
                if (try? FileManager.default.moveItem(at: tmp, to: dest)) != nil,
                   let data = try? Data(contentsOf: dest, options: .mappedIfSafe) {
                    let hex = SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
                    if hex == sha256.lowercased() { result = dest } else { try? FileManager.default.removeItem(at: dest) }
                }
            }
            DispatchQueue.main.async { done(result) }
        }.resume()
    }

    /// Can this copy replace itself? Not when it runs from the disk image,
    /// from a translocated (quarantined) place or from a folder the user
    /// cannot write to: then the disk image is opened for a manual update.
    static var canReplaceSelf: Bool {
        let app = Bundle.main.bundleURL
        let parent = app.deletingLastPathComponent().path
        return app.pathExtension == "app"
            && !app.path.contains("/AppTranslocation/")
            && !app.path.hasPrefix("/Volumes/")
            && FileManager.default.isWritableFile(atPath: parent)
    }

    /// Once this process has exited: mount the image, copy the new app next to
    /// the old one, swap them (the old one comes back if anything fails) and
    /// start it. Runs as a separate shell process so this app can quit first.
    static func replaceSelf(with dmg: URL) -> Bool {
        let script = """
        pid="$1"; dmg="$2"; app="$3"
        i=0; while kill -0 "$pid" 2>/dev/null && [ $i -lt 150 ]; do sleep 0.2; i=$((i+1)); done
        mnt=$(mktemp -d /tmp/odomouse-update.XXXXXX) || { open "$app"; exit 1; }
        if hdiutil attach -nobrowse -readonly -noautoopen -mountpoint "$mnt" "$dmg" >/dev/null 2>&1; then
          if [ -d "$mnt/Odomouse.app" ]; then
            rm -rf "$app.new" "$app.old"
            if ditto "$mnt/Odomouse.app" "$app.new"; then
              if mv "$app" "$app.old"; then
                if mv "$app.new" "$app"; then rm -rf "$app.old"; else mv "$app.old" "$app"; fi
              fi
            fi
            rm -rf "$app.new"
          fi
          hdiutil detach "$mnt" -quiet || hdiutil detach "$mnt" -force -quiet
        fi
        rmdir "$mnt" 2>/dev/null
        rm -f "$dmg"
        open "$app"
        """
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "/bin/sh")
        p.arguments = ["-c", script, "odomouse-update",
                       String(ProcessInfo.processInfo.processIdentifier), dmg.path, Bundle.main.bundleURL.path]
        do { try p.run() } catch { return false }
        return true
    }
}
