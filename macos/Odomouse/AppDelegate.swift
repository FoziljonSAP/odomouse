import AppKit
import ServiceManagement
import UserNotifications

/// Wires macOS (menu bar, windows, input, displays, notifications) to the
/// Rust core. It only decides *when*; the core decides *what* to count.
final class AppDelegate: NSObject, NSApplicationDelegate, NSPopoverDelegate, NSWindowDelegate, BridgeHost,
    UNUserNotificationCenterDelegate {
    private var core: Core!
    private var input: InputMonitor!

    private var statusItem: NSStatusItem?
    private let popover = NSPopover()
    private var popupPage: WebPage?
    private var popupTeardown: DispatchWorkItem?

    private var dashboardWindow: NSWindow?
    private var dashboardPage: WebPage?

    private var timer: Timer?
    private var lastTray: String?
    private var settings: [String: Any] = [:]
    /// A newer version found by the daily check (from the core).
    private var update: [String: Any]?
    private var updateBusy = false

    static let dataDir: URL = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        .appendingPathComponent("Odomouse", isDirectory: true)

    // MARK: - lifecycle

    func applicationDidFinishLaunching(_ notification: Notification) {
        guard let core = Core(dataDir: Self.dataDir) else {
            let alert = NSAlert()
            // the core (and its texts) did not load: pick the language here
            let os = Locale.preferredLanguages.first ?? ""
            let (uz, ru) = (os.hasPrefix("uz"), os.hasPrefix("ru"))
            alert.messageText = uz ? "Odomouse ishga tushmadi" : ru ? "Odomouse не запустился" : "Odomouse couldn't start"
            alert.informativeText = (uz ? "Ma'lumotlar papkasini ochib bo'lmadi: " : ru ? "Не удалось открыть папку данных: " : "Couldn't open the data folder: ")
                + Self.dataDir.path
            alert.runModal()
            NSApp.terminate(nil)
            return
        }
        self.core = core
        // "auto" language follows macOS until the user picks one
        _ = core.call("setSystemLanguage", JSON.string([Locale.preferredLanguages.first ?? "en"]))
        reloadSettings()
        reloadStrings()

        core.setDisplays(DisplayProbe.json())
        NotificationCenter.default.addObserver(self, selector: #selector(displaysChanged),
                                               name: NSApplication.didChangeScreenParametersNotification, object: nil)

        let ws = NSWorkspace.shared.notificationCenter
        ws.addObserver(self, selector: #selector(frontAppChanged(_:)),
                       name: NSWorkspace.didActivateApplicationNotification, object: nil)
        ws.addObserver(self, selector: #selector(willSleep),
                       name: NSWorkspace.willSleepNotification, object: nil)

        input = InputMonitor(core: core)
        input.onStateChange = { [weak self] _ in
            self?.refreshTray()
            self?.pushLive()
        }
        applySettings(broadcast: false)
        input.start()

        UNUserNotificationCenter.current().delegate = self
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }

        let t = Timer(timeInterval: 1, repeats: true) { [weak self] _ in self?.tick() }
        t.tolerance = 0.25 // lets macOS batch wake-ups: less energy use
        RunLoop.main.add(t, forMode: .common)
        timer = t
        refreshTray()
        // started by hand but the counter cannot be seen: open the window so the app is not invisible
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) { [weak self] in self?.checkStatusItemVisible(openWindow: true) }
    }

    func applicationWillTerminate(_ notification: Notification) {
        timer?.invalidate()
        input?.stop()
        core?.close()
    }

    /// Clicking the app in Finder / Launchpad while it runs.
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        openDashboard(tab: "stats")
        return true
    }

    /// odomouse://dashboard, odomouse://settings
    func application(_ application: NSApplication, open urls: [URL]) {
        guard let url = urls.first else { return }
        openDashboard(tab: url.host == "settings" ? "settings" : "stats")
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }

    // MARK: - timer

    private func tick() {
        let cursor = CGEvent(source: nil)?.location
        if let out = JSON.object(core.tick(cursor: cursor)) {
            if let tray = out["tray"] as? String { setTray(tray) }
            if let n = out["notify"] as? [String: Any], let title = n["title"] as? String, let body = n["body"] as? String {
                notify(title: title, body: body, id: n["kind"] as? String ?? "break")
            }
            if let feed = out["checkUpdate"] as? String { checkUpdate(feed: feed, manual: false, reply: nil) }
        }
        if popover.isShown { pushLive() }
    }

    @objc private func displaysChanged() {
        core.setDisplays(DisplayProbe.json())
        let displays = core.call("getDisplays")
        dashboardPage?.emit("displays", displays)
    }

    @objc private func frontAppChanged(_ note: Notification) {
        let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
        core.setApp(app?.localizedName)
    }

    @objc private func willSleep() {
        core.save()
    }

    private func notify(title: String, body: String, id: String = "break") {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        let request = UNNotificationRequest(identifier: id, content: content, trigger: nil)
        UNUserNotificationCenter.current().add(request, withCompletionHandler: nil)
    }

    // MARK: - notifications

    /// Show notifications even while a page of the app is in front.
    func userNotificationCenter(_ center: UNUserNotificationCenter, willPresent notification: UNNotification,
                                withCompletionHandler done: @escaping (UNNotificationPresentationOptions) -> Void) {
        done([.banner, .list])
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse,
                                withCompletionHandler done: @escaping () -> Void) {
        if response.notification.request.identifier == "update" { installUpdate() }
        done()
    }

    // MARK: - updates

    /// Download latest.json and let the core decide. `reply`: "Check now" in Settings.
    private func checkUpdate(feed: String, manual: Bool, reply: ((String) -> Void)?) {
        Updater.fetchText(feed) { [weak self] body, error in
            guard let self = self else { return }
            guard let body = body else {
                reply?("{\"error\":\(JSON.quote(error))}")
                return
            }
            let result = self.core.call("updateFeed", JSON.string([body, manual] as [Any]))
            if let r = JSON.object(result), r["error"] == nil {
                self.update = r["update"] as? [String: Any]
                if let n = r["notify"] as? [String: Any], let t = n["title"] as? String, let b = n["body"] as? String {
                    self.notify(title: t, body: b, id: "update")
                }
                self.reloadSettings()
                self.applySettings() // the pages show the new version
            }
            reply?(result)
        }
    }

    /// Download the new disk image, check it, then replace this app and
    /// restart. Where that is not possible, the image opens in Finder; if
    /// anything fails, the website opens.
    private func installUpdate() {
        guard let u = update, !updateBusy else { return }
        let page = URL(string: u["page"] as? String ?? "")
        let version = u["version"] as? String ?? ""
        guard let url = URL(string: u["url"] as? String ?? ""), url.scheme == "https", url.pathExtension == "dmg",
              let sha = u["sha256"] as? String, sha.count == 64 else {
            if let p = URL(string: u["url"] as? String ?? "") ?? page { NSWorkspace.shared.open(p) }
            return
        }
        updateBusy = true
        notify(title: "Odomouse \(version)", body: s("updateDownloading"), id: "updating")
        Updater.downloadImage(url, sha256: sha, version: version) { [weak self] dmg in
            guard let self = self else { return }
            self.updateBusy = false
            guard let dmg = dmg else {
                let alert = NSAlert()
                alert.messageText = self.s("updateFailed")
                alert.runModal()
                if let page = page { NSWorkspace.shared.open(page) }
                return
            }
            self.core.save()
            if Updater.canReplaceSelf && Updater.replaceSelf(with: dmg) {
                NSApp.terminate(nil)
            } else {
                NSWorkspace.shared.open(dmg)
            }
        }
    }

    @objc private func menuUpdate() { installUpdate() }

    // MARK: - settings

    private var currentTheme: String { settings["theme"] as? String ?? "system" }

    private func reloadSettings() {
        settings = JSON.object(core.call("getSettings")) ?? [:]
    }

    /// Menu and dialog texts in the current language (from the core).
    private var strings: [String: String] = [:]
    private var stringsLang = ""

    private func s(_ key: String) -> String { strings[key] ?? key }

    private func reloadStrings() {
        let all = JSON.object(core.call("getStrings")) ?? [:]
        strings = all.compactMapValues { $0 as? String }
        if strings["lang"] != stringsLang {
            stringsLang = strings["lang"] ?? ""
            NSApp.mainMenu = makeMainMenu()
        }
    }

    /// Apply settings to macOS and tell every open page.
    private func applySettings(broadcast: Bool = true) {
        reloadStrings()
        switch settings["theme"] as? String {
        case "light": NSApp.appearance = NSAppearance(named: .aqua)
        case "dark": NSApp.appearance = NSAppearance(named: .darkAqua)
        default: NSApp.appearance = nil
        }
        syncStatusItem()
        syncLaunchAtLogin()
        core.setApp(NSWorkspace.shared.frontmostApplication?.localizedName)
        refreshTray()
        if broadcast {
            let json = JSON.string(settings)
            popupPage?.emit("settings", json)
            dashboardPage?.emit("settings", json)
            pushLive()
        }
    }

    private func syncLaunchAtLogin() {
        guard #available(macOS 13.0, *) else { return }
        let want = settings["launchAtLogin"] as? Bool ?? false
        let service = SMAppService.mainApp
        do {
            if want && service.status != .enabled { try service.register() }
            if !want && service.status == .enabled { try service.unregister() }
        } catch {
            NSLog("Odomouse: launch at login: \(error.localizedDescription)")
        }
    }

    // MARK: - menu bar

    private func syncStatusItem() {
        if statusItem == nil {
            let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
            if let button = item.button {
                let image = NSImage(named: "TrayIcon")
                image?.isTemplate = true
                button.image = image
                button.imagePosition = .imageLeft
                button.font = NSFont.monospacedDigitSystemFont(ofSize: NSFont.systemFontSize, weight: .regular)
                button.target = self
                button.action = #selector(statusItemClicked(_:))
                button.sendAction(on: [.leftMouseUp, .rightMouseUp])
                button.toolTip = "Odomouse"
            }
            // keep its place, and undo a ⌘-drag out of the menu bar from an older run
            item.autosaveName = "OdomouseCounter"
            item.isVisible = true
            statusItem = item
            lastTray = nil
            if let window = item.button?.window {
                NotificationCenter.default.addObserver(self, selector: #selector(statusItemOcclusionChanged),
                                                       name: NSWindow.didChangeOcclusionStateNotification, object: window)
            }
        }
    }

    // MARK: - hidden counter
    // A full menu bar puts items behind the camera notch (or off the edge), and
    // then the app looks like it is not running at all. Notice it and say so.

    private var hiddenWarned = false

    private var statusItemVisible: Bool {
        guard let window = statusItem?.button?.window, window.screen != nil else { return false }
        return window.occlusionState.contains(.visible)
    }

    @objc private func statusItemOcclusionChanged() {
        // wait a moment: the menu bar briefly hides items while it rearranges
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) { [weak self] in self?.checkStatusItemVisible(openWindow: false) }
    }

    private func checkStatusItemVisible(openWindow: Bool) {
        guard statusItem != nil, !statusItemVisible, !hiddenWarned else { return }
        // a full-screen app hides the whole menu bar: that is not our problem
        if !NSMenu.menuBarVisible() { return }
        hiddenWarned = true
        notify(title: s("hiddenTitle"), body: s("hiddenBody"), id: "hidden")
        if openWindow { openDashboard(tab: "stats") }
    }

    private func refreshTray() {
        // without permission only "⚠": a long label is what pushes the item behind the notch
        let text = core.trayText()
        setTray(input?.running == false ? "⚠" : text)
        statusItem?.button?.toolTip = input?.running == false ? "Odomouse: \(text)" : "Odomouse"
    }

    private func setTray(_ text: String) {
        guard let button = statusItem?.button, text != lastTray else { return }
        lastTray = text
        button.title = text.isEmpty ? "" : " " + text
    }

    @objc private func statusItemClicked(_ sender: Any?) {
        let event = NSApp.currentEvent
        let rightClick = event?.type == .rightMouseUp || event?.modifierFlags.contains(.control) == true
        if rightClick { showStatusMenu() } else { togglePopover() }
    }

    private func showStatusMenu() {
        guard let item = statusItem else { return }
        let menu = NSMenu()
        if !input.running {
            // what the app sees about its permission: helps when it says "ruxsat kerak"
            let keys = s(input.keyboardAllowed ? "keysAllowed" : "keysDenied")
            let mouse = s(input.tapActive ? "mouseOn" : "mouseOff")
            let status = NSMenuItem(title: "\(keys) · \(mouse)", action: nil, keyEquivalent: "")
            status.isEnabled = false
            menu.addItem(status)
            menu.addItem(withTitle: s("grantKeyboard"), action: #selector(menuPermissions), keyEquivalent: "").target = self
        }
        if let v = update?["version"] as? String {
            menu.addItem(.separator())
            menu.addItem(withTitle: s("updateMenu").replacingOccurrences(of: "{v}", with: v),
                         action: #selector(menuUpdate), keyEquivalent: "").target = self
        }
        menu.addItem(.separator())
        menu.addItem(withTitle: s("stats"), action: #selector(menuStats), keyEquivalent: "").target = self
        menu.addItem(withTitle: s("settings"), action: #selector(menuSettings), keyEquivalent: "").target = self
        menu.addItem(.separator())
        menu.addItem(withTitle: s("quit"), action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        item.menu = menu
        item.button?.performClick(nil)
        item.menu = nil
    }

    @objc private func menuStats() { openDashboard(tab: "stats") }
    @objc private func menuPermissions() { openPermissions() }
    @objc private func menuSettings() { openDashboard(tab: "settings") }

    // MARK: - popover (popup.html)

    private func togglePopover() {
        guard let button = statusItem?.button else { return }
        if popover.isShown {
            popover.performClose(nil)
            return
        }
        popupTeardown?.cancel()
        if popupPage == nil {
            let size = NSSize(width: 360, height: 640)
            let page = WebPage(page: "popup.html", theme: currentTheme, host: self, size: size)
            let controller = NSViewController()
            controller.view = page.webView
            popover.contentViewController = controller
            popover.contentSize = size
            popupPage = page
        }
        popover.behavior = .transient
        popover.animates = false
        popover.delegate = self
        NSApp.activate(ignoringOtherApps: true)
        popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
        pushLive()
    }

    /// Keep the page a little in case it is reopened, then free its memory.
    func popoverDidClose(_ notification: Notification) {
        let work = DispatchWorkItem { [weak self] in
            guard let self = self, !self.popover.isShown else { return }
            self.popupPage?.close()
            self.popupPage = nil
            self.popover.contentViewController = nil
        }
        popupTeardown = work
        DispatchQueue.main.asyncAfter(deadline: .now() + 45, execute: work)
    }

    private func pushLive() {
        guard let page = popupPage, popover.isShown else { return }
        page.emit("live", core.call("getLive"))
    }

    // MARK: - dashboard window (dashboard.html)

    private func openDashboard(tab: String) {
        if popover.isShown { popover.performClose(nil) }
        if settings["seenDashboard"] as? Bool != true {
            _ = core.call("markDashboardSeen")
            reloadSettings()
            applySettings()
        }
        if let window = dashboardWindow {
            dashboardPage?.emit("tab", JSON.quote(tab))
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }
        let size = NSSize(width: 1040, height: 760)
        let page = WebPage(page: "dashboard.html", tab: tab, theme: currentTheme, host: self, size: size)
        let window = NSWindow(contentRect: NSRect(origin: .zero, size: size),
                              styleMask: [.titled, .closable, .miniaturizable, .resizable],
                              backing: .buffered, defer: false)
        window.title = "Odomouse"
        window.minSize = NSSize(width: 820, height: 600)
        window.isReleasedWhenClosed = false
        window.contentView = page.webView
        window.delegate = self
        window.center()
        window.setFrameAutosaveName("OdomouseDashboard")
        dashboardWindow = window
        dashboardPage = page
        NSApp.setActivationPolicy(.regular) // Dock icon + app menu while the window is open
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func windowWillClose(_ notification: Notification) {
        guard let window = notification.object as? NSWindow, window === dashboardWindow else { return }
        dashboardPage?.close()
        dashboardPage = nil
        dashboardWindow = nil
        DispatchQueue.main.async {
            window.contentView = nil
            NSApp.setActivationPolicy(.accessory)
        }
    }

    // MARK: - BridgeHost: window.odomouse.* from the pages

    func handle(method: String, args: [Any], from page: WebPage, reply: @escaping (String) -> Void) {
        let argsJSON = JSON.string(args)
        switch method {
        case "getLive", "getDashboard", "getWrapped", "getUpdate":
            reply(core.call(method, argsJSON))
        case "checkUpdatesNow":
            guard let feed = JSON.object(core.call("getUpdate"))?["feed"] as? String else {
                reply("{\"error\":\"no feed\"}")
                return
            }
            checkUpdate(feed: feed, manual: true, reply: reply)
        case "installUpdate":
            reply("null")
            installUpdate()
        case "updateSettings":
            let result = core.call(method, argsJSON)
            reloadSettings()
            applySettings()
            reply(result)
        case "openDashboard":
            reply("null")
            openDashboard(tab: args.first as? String == "settings" ? "settings" : "stats")
        case "exportCsv":
            exportCsv(reply)
        case "resetData":
            confirm(message: s("resetTitle"), detail: s("resetDetail"), action: s("resetAction")) { [weak self] ok in
                guard let self = self, ok else { reply("{\"ok\":false}"); return }
                reply(self.core.call("resetData"))
                self.refreshTray()
                self.pushLive()
            }
        case "clearApps":
            confirm(message: s("appsTitle"), detail: s("appsDetail"), action: s("appsAction")) { [weak self] ok in
                guard let self = self, ok else { reply("{\"ok\":false}"); return }
                reply(self.core.call("clearApps"))
            }
        case "saveWrapped":
            saveWrapped(dataURL: args.first as? String, period: args.count > 1 ? args[1] as? String : nil, reply: reply)
        case "copyWrapped":
            guard let data = pngData(args.first as? String), let image = NSImage(data: data) else {
                reply("{\"ok\":false}")
                return
            }
            NSPasteboard.general.clearContents()
            NSPasteboard.general.writeObjects([image])
            reply("{\"ok\":true}")
        case "openPermissions":
            reply("null")
            openPermissions()
        case "quit":
            reply("null")
            NSApp.terminate(nil)
        default:
            reply("{\"error\":\(JSON.quote("unknown method: " + method))}")
        }
    }

    /// The switch in System Settings may be on but belong to an older build
    /// (each ad-hoc signed build looks like a new app to macOS), so a stale
    /// entry is cleared and the app asks again before the pane opens.
    private func openPermissions() {
        if !input.keyboardAllowed { input.resetPermission() }
        let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")!
        NSWorkspace.shared.open(url)
    }

    // MARK: - files and dialogs

    private func confirm(message: String, detail: String, action: String, done: @escaping (Bool) -> Void) {
        let alert = NSAlert()
        alert.alertStyle = .warning
        alert.messageText = message
        alert.informativeText = detail
        alert.addButton(withTitle: s("cancel"))
        let destructive = alert.addButton(withTitle: action)
        destructive.hasDestructiveAction = true
        if let window = dashboardWindow {
            alert.beginSheetModal(for: window) { done($0 == .alertSecondButtonReturn) }
        } else {
            NSApp.activate(ignoringOtherApps: true)
            done(alert.runModal() == .alertSecondButtonReturn)
        }
    }

    private func runSavePanel(_ panel: NSSavePanel, done: @escaping (URL?) -> Void) {
        if let window = dashboardWindow {
            panel.beginSheetModal(for: window) { done($0 == .OK ? panel.url : nil) }
        } else {
            NSApp.activate(ignoringOtherApps: true)
            done(panel.runModal() == .OK ? panel.url : nil)
        }
    }

    private func exportCsv(_ reply: @escaping (String) -> Void) {
        guard let out = JSON.object(core.call("exportCsv")), let csv = out["csv"] as? String else {
            reply("{\"ok\":false}")
            return
        }
        let date = out["date"] as? String ?? "data"
        let panel = NSSavePanel()
        panel.title = s("saveCsv")
        panel.nameFieldStringValue = "odomouse-\(date).csv"
        panel.directoryURL = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first
        runSavePanel(panel) { url in
            guard let url = url else { reply("{\"ok\":false,\"canceled\":true}"); return }
            do {
                try csv.write(to: url, atomically: true, encoding: .utf8)
                reply(JSON.string(["ok": true, "filePath": url.path]))
            } catch {
                reply("{\"ok\":false}")
            }
        }
    }

    private func pngData(_ dataURL: String?) -> Data? {
        let prefix = "data:image/png;base64,"
        guard let s = dataURL, s.hasPrefix(prefix), s.count < 30_000_000 else { return nil }
        return Data(base64Encoded: String(s.dropFirst(prefix.count)))
    }

    private func saveWrapped(dataURL: String?, period: String?, reply: @escaping (String) -> Void) {
        guard let data = pngData(dataURL) else { reply("{\"ok\":false}"); return }
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        let panel = NSSavePanel()
        panel.title = s("saveWrapped")
        panel.nameFieldStringValue = "odomouse-wrapped-\(period ?? "week")-\(formatter.string(from: Date())).png"
        panel.directoryURL = FileManager.default.urls(for: .desktopDirectory, in: .userDomainMask).first
        runSavePanel(panel) { url in
            guard let url = url else { reply("{\"ok\":false,\"canceled\":true}"); return }
            do {
                try data.write(to: url, options: .atomic)
                NSWorkspace.shared.activateFileViewerSelecting([url])
                reply(JSON.string(["ok": true, "filePath": url.path]))
            } catch {
                reply("{\"ok\":false}")
            }
        }
    }

    // MARK: - main menu (needed for ⌘C/⌘V/⌘W while the dashboard is open)

    private func makeMainMenu() -> NSMenu {
        let main = NSMenu()

        let appItem = NSMenuItem()
        let appMenu = NSMenu()
        appMenu.addItem(withTitle: s("stats"), action: #selector(menuStats), keyEquivalent: "1").target = self
        appMenu.addItem(withTitle: s("settingsMenu"), action: #selector(menuSettings), keyEquivalent: ",").target = self
        appMenu.addItem(.separator())
        appMenu.addItem(withTitle: s("hide"), action: #selector(NSApplication.hide(_:)), keyEquivalent: "h")
        appMenu.addItem(withTitle: s("quit"), action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        appItem.submenu = appMenu
        main.addItem(appItem)

        let editItem = NSMenuItem()
        let edit = NSMenu(title: s("edit"))
        edit.addItem(withTitle: s("undo"), action: Selector(("undo:")), keyEquivalent: "z")
        let redo = edit.addItem(withTitle: s("redo"), action: Selector(("redo:")), keyEquivalent: "z")
        redo.keyEquivalentModifierMask = [.command, .shift]
        edit.addItem(.separator())
        edit.addItem(withTitle: s("cut"), action: #selector(NSText.cut(_:)), keyEquivalent: "x")
        edit.addItem(withTitle: s("copy"), action: #selector(NSText.copy(_:)), keyEquivalent: "c")
        edit.addItem(withTitle: s("paste"), action: #selector(NSText.paste(_:)), keyEquivalent: "v")
        edit.addItem(withTitle: s("selectAll"), action: #selector(NSText.selectAll(_:)), keyEquivalent: "a")
        editItem.submenu = edit
        main.addItem(editItem)

        let windowItem = NSMenuItem()
        let windowMenu = NSMenu(title: s("window"))
        windowMenu.addItem(withTitle: s("close"), action: #selector(NSWindow.performClose(_:)), keyEquivalent: "w")
        windowMenu.addItem(withTitle: s("minimize"), action: #selector(NSWindow.performMiniaturize(_:)), keyEquivalent: "m")
        windowItem.submenu = windowMenu
        main.addItem(windowItem)
        return main
    }
}
