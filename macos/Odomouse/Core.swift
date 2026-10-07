import Foundation
import CoreGraphics

/// Thin Swift wrapper over the Rust core (core/include/odomouse_core.h).
/// Every rule about what to count lives in the core; this only forwards.
final class Core {
    private var handle: OpaquePointer?

    static var tzOffset: Int32 { Int32(TimeZone.current.secondsFromGMT()) }

    init?(dataDir: URL) {
        try? FileManager.default.createDirectory(at: dataDir, withIntermediateDirectories: true)
        guard let h = mk_open(dataDir.path, Core.tzOffset) else { return nil }
        handle = h
    }

    deinit { close() }

    /// Saves and releases the core. Safe to call twice.
    func close() {
        if let h = handle {
            handle = nil
            mk_close(h)
        }
    }

    private func take(_ p: UnsafeMutablePointer<CChar>?) -> String {
        guard let p = p else { return "null" }
        defer { mk_string_free(p) }
        return String(cString: p)
    }

    // MARK: input (called on the main thread by InputMonitor)

    func mouseMove(_ p: CGPoint) { mk_mouse_move(handle, Double(p.x), Double(p.y)) }
    func mouseDown(_ button: UInt32) { mk_mouse_down(handle, button) }
    func wheel(_ p: CGPoint, points: Double) { mk_wheel(handle, Double(p.x), Double(p.y), points) }
    func keyDown(_ vc: UInt32, mods: UInt32) { mk_key_down(handle, vc, mods) }
    func keyUp(_ vc: UInt32) { mk_key_up(handle, vc) }

    // MARK: state

    func setDisplays(_ json: String) { mk_set_displays(handle, json) }
    func setHooksRunning(_ running: Bool) { mk_set_hooks_running(handle, running) }

    func setApp(_ name: String?) {
        if let name = name {
            mk_set_app(handle, name)
        } else {
            mk_set_app(handle, nil)
        }
    }

    /// {"tray": "...", "notify": null | {"title", "body"}}
    func tick(cursor: CGPoint?) -> String {
        take(mk_tick(handle, Core.tzOffset, cursor != nil, Double(cursor?.x ?? 0), Double(cursor?.y ?? 0)))
    }

    /// Text next to the menu bar icon ("" = icon only).
    func trayText() -> String { take(mk_tray_text(handle)) }

    /// UI bridge call; `argsJSON` is a JSON array.
    func call(_ method: String, _ argsJSON: String = "[]") -> String {
        take(mk_call(handle, method, argsJSON))
    }

    @discardableResult
    func save() -> Bool { mk_save(handle) }
}

// MARK: - JSON helpers

enum JSON {
    static func object(_ text: String) -> [String: Any]? {
        guard let data = text.data(using: .utf8) else { return nil }
        return (try? JSONSerialization.jsonObject(with: data, options: [.fragmentsAllowed])) as? [String: Any]
    }

    /// Encodes arrays / dictionaries; anything else becomes "null".
    static func string(_ value: Any) -> String {
        guard JSONSerialization.isValidJSONObject(value),
              let data = try? JSONSerialization.data(withJSONObject: value, options: []),
              let s = String(data: data, encoding: .utf8) else { return "null" }
        return s
    }

    /// A Swift string as a JSON string literal ("tab" -> "\"tab\"").
    static func quote(_ s: String) -> String {
        let wrapped = string([s])
        return String(wrapped.dropFirst().dropLast())
    }
}
