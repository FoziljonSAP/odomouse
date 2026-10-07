import AppKit
import ApplicationServices
import CoreGraphics

/// Global input via a listen-only CGEventTap. It never sees *what* is
/// typed in a meaningful way (only key positions are counted by the core)
/// and cannot change events. Needs the "Input Monitoring" permission.
final class InputMonitor {
    private let core: Core
    private var tap: CFMachPort?
    private var source: CFRunLoopSource?
    private(set) var running = false
    var onStateChange: ((Bool) -> Void)?

    // Device-dependent modifier bits (IOKit NX_DEVICE*KEYMASK) per left/right key.
    private static let modifierBits: [Int64: UInt64] = [
        0x37: 0x08, 0x36: 0x10, // command L / R
        0x38: 0x02, 0x3C: 0x04, // shift L / R
        0x3B: 0x01, 0x3E: 0x2000, // control L / R
        0x3A: 0x20, 0x3D: 0x40, // option L / R
    ]
    private static let capsLock: Int64 = 0x39

    init(core: Core) {
        self.core = core
    }

    static var hasPermission: Bool { CGPreflightListenEventAccess() }

    /// Input Monitoring granted. Mouse events reach the tap without it, but
    /// macOS silently drops every key event, so "running" needs both.
    private(set) var keyboardAllowed = false
    private var watchTimer: Timer?

    func start() {
        keyboardAllowed = CGPreflightListenEventAccess()
        if !keyboardAllowed {
            _ = CGRequestListenEventAccess() // adds the app to the list (and asks, the first time)
            keyboardAllowed = CGPreflightListenEventAccess()
        }
        createTap()
        scheduleWatch()
    }

    func stop() {
        watchTimer?.invalidate()
        watchTimer = nil
        destroyTap()
        setRunning(false)
    }

    /// Notices the permission being granted (or taken back) in System Settings
    /// and rebuilds the tap, so keys start counting without a restart.
    /// Often while waiting for it, rarely once it is there.
    private func scheduleWatch() {
        watchTimer?.invalidate()
        let interval: TimeInterval = keyboardAllowed && tap != nil ? 30 : 2
        let timer = Timer(timeInterval: interval, repeats: true) { [weak self] _ in self?.checkPermission() }
        timer.tolerance = interval / 4
        RunLoop.main.add(timer, forMode: .common)
        watchTimer = timer
    }

    func checkPermission() {
        let allowed = CGPreflightListenEventAccess()
        guard allowed != keyboardAllowed || tap == nil else { return }
        NSLog("Odomouse: input monitoring %d -> %d, rebuilding the event tap", keyboardAllowed ? 1 : 0, allowed ? 1 : 0)
        keyboardAllowed = allowed
        destroyTap()
        createTap()
        scheduleWatch()
    }

    /// Clears this app's Input Monitoring entry and asks again. Fixes the case
    /// where System Settings shows the switch on but it belongs to an older
    /// build of the app (ad-hoc signed builds differ, so macOS treats each
    /// update as a new app). Returns false if macOS refused the reset.
    @discardableResult
    func resetPermission() -> Bool {
        var ok = false
        if let id = Bundle.main.bundleIdentifier {
            let p = Process()
            p.executableURL = URL(fileURLWithPath: "/usr/bin/tccutil")
            p.arguments = ["reset", "ListenEvent", id]
            p.standardOutput = FileHandle.nullDevice
            p.standardError = FileHandle.nullDevice
            if (try? p.run()) != nil {
                p.waitUntilExit()
                ok = p.terminationStatus == 0
            }
        }
        _ = CGRequestListenEventAccess()
        checkPermission()
        return ok
    }

    private func setRunning(_ value: Bool) {
        guard value != running else { return }
        running = value
        core.setHooksRunning(value)
        onStateChange?(value)
    }

    /// The event tap exists (mouse events arrive; keys only with permission).
    var tapActive: Bool { tap != nil }

    private func destroyTap() {
        if let tap = tap {
            CGEvent.tapEnable(tap: tap, enable: false)
            CFMachPortInvalidate(tap)
        }
        if let source = source { CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes) }
        tap = nil
        source = nil
    }

    private func createTap() {
        let types: [CGEventType] = [
            .mouseMoved, .leftMouseDragged, .rightMouseDragged, .otherMouseDragged,
            .leftMouseDown, .rightMouseDown, .otherMouseDown, .scrollWheel,
            .keyDown, .keyUp, .flagsChanged,
        ]
        var mask: CGEventMask = 0
        for t in types { mask |= CGEventMask(1) << CGEventMask(t.rawValue) }
        let refcon = Unmanaged.passUnretained(self).toOpaque()
        guard let tap = CGEvent.tapCreate(tap: .cgSessionEventTap, place: .tailAppendEventTap,
                                          options: .listenOnly, eventsOfInterest: mask,
                                          callback: inputTapCallback, userInfo: refcon) else {
            NSLog("Odomouse: event tap not created (input monitoring %d)", keyboardAllowed ? 1 : 0)
            setRunning(false)
            return
        }
        let source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
        CFRunLoopAddSource(CFRunLoopGetMain(), source, .commonModes)
        CGEvent.tapEnable(tap: tap, enable: true)
        self.tap = tap
        self.source = source
        setRunning(keyboardAllowed)
    }

    fileprivate func handle(type: CGEventType, event: CGEvent) {
        switch type {
        case .tapDisabledByTimeout, .tapDisabledByUserInput:
            if let tap = tap { CGEvent.tapEnable(tap: tap, enable: true) }
        case .mouseMoved, .leftMouseDragged, .rightMouseDragged, .otherMouseDragged:
            core.mouseMove(event.location)
        case .leftMouseDown:
            core.mouseDown(1)
        case .rightMouseDown:
            core.mouseDown(2)
        case .otherMouseDown:
            core.mouseDown(event.getIntegerValueField(.mouseEventButtonNumber) == 2 ? 3 : 4)
        case .scrollWheel:
            let dy = Double(event.getIntegerValueField(.scrollWheelEventPointDeltaAxis1))
            let dx = Double(event.getIntegerValueField(.scrollWheelEventPointDeltaAxis2))
            core.wheel(event.location, points: (dx * dx + dy * dy).squareRoot())
        case .keyDown:
            if event.getIntegerValueField(.keyboardEventAutorepeat) != 0 { return }
            let vc = Self.vc(event)
            if vc != 0 { core.keyDown(vc, mods: Self.mods(event.flags)) }
        case .keyUp:
            let vc = Self.vc(event)
            if vc != 0 { core.keyUp(vc) }
        case .flagsChanged:
            handleFlags(event)
        default:
            break
        }
    }

    /// Modifier keys arrive as flagsChanged, not keyDown/keyUp.
    private func handleFlags(_ event: CGEvent) {
        let code = event.getIntegerValueField(.keyboardEventKeycode)
        let vc = Self.vc(event)
        guard vc != 0 else { return }
        if code == Self.capsLock {
            core.keyDown(vc, mods: 0)
            core.keyUp(vc)
            return
        }
        guard let bit = Self.modifierBits[code] else { return }
        if event.flags.rawValue & bit != 0 {
            core.keyDown(vc, mods: Self.mods(event.flags))
        } else {
            core.keyUp(vc)
        }
    }

    private static func vc(_ event: CGEvent) -> UInt32 {
        mk_vc_from_mac(UInt16(truncatingIfNeeded: event.getIntegerValueField(.keyboardEventKeycode)))
    }

    /// MK_MOD_CTRL 1, MK_MOD_ALT 2, MK_MOD_SHIFT 4, MK_MOD_META 8
    private static func mods(_ f: CGEventFlags) -> UInt32 {
        var m: UInt32 = 0
        if f.contains(.maskControl) { m |= 1 }
        if f.contains(.maskAlternate) { m |= 2 }
        if f.contains(.maskShift) { m |= 4 }
        if f.contains(.maskCommand) { m |= 8 }
        return m
    }
}

private func inputTapCallback(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent,
                              refcon: UnsafeMutableRawPointer?) -> Unmanaged<CGEvent>? {
    if let refcon = refcon {
        Unmanaged<InputMonitor>.fromOpaque(refcon).takeUnretainedValue().handle(type: type, event: event)
    }
    return Unmanaged.passUnretained(event)
}
