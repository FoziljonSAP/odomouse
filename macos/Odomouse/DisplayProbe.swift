import AppKit
import CoreGraphics

/// Displays in the same global coordinate space as CGEvent.location
/// (points, origin at the top-left of the main display), with the physical
/// size macOS reads from each display's EDID.
enum DisplayProbe {
    static func json() -> String {
        var ids = [CGDirectDisplayID](repeating: 0, count: 16)
        var count: UInt32 = 0
        guard CGGetActiveDisplayList(UInt32(ids.count), &ids, &count) == .success else { return "[]" }
        var out: [[String: Any]] = []
        for id in ids.prefix(Int(count)) {
            // a mirrored copy shows the same pixels: count the display once
            if CGDisplayMirrorsDisplay(id) != 0 { continue }
            let bounds = CGDisplayBounds(id)
            let size = CGDisplaySizeOrZero(id)
            let screen = NSScreen.screens.first {
                ($0.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value == id
            }
            var d: [String: Any] = [
                "id": String(id),
                "internal": CGDisplayIsBuiltin(id) != 0,
                "scaleFactor": Double(screen?.backingScaleFactor ?? 1),
                "bounds": [
                    "x": Double(bounds.origin.x), "y": Double(bounds.origin.y),
                    "width": Double(bounds.width), "height": Double(bounds.height),
                ],
                "widthMm": Double(size.width),
                "heightMm": Double(size.height),
            ]
            if let name = screen?.localizedName { d["label"] = name }
            out.append(d)
        }
        return JSON.string(out)
    }

    private static func CGDisplaySizeOrZero(_ id: CGDirectDisplayID) -> CGSize {
        let s = CGDisplayScreenSize(id)
        return s.width.isFinite && s.height.isFinite ? s : .zero
    }
}
