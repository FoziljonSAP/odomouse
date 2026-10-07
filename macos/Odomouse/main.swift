import AppKit

// Menu bar app: no Dock icon until the dashboard window opens (LSUIElement).
let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
