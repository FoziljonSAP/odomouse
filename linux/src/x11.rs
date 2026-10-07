//! X11: global input through the RECORD extension (the same way libuiohook
//! does it, so no special permission is needed), monitors with physical
//! size through RandR, and the active window's class for per-app stats.

use crate::dl::Lib;
use crate::sym;
use std::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void, CStr};
use std::ptr;

type Display = c_void;
type XId = c_ulong;

pub struct XLibs {
    x11: Lib,
    xtst: Option<Lib>,
    xrandr: Option<Lib>,
}

impl XLibs {
    pub fn load() -> Option<XLibs> {
        let x11 = Lib::open(&["libX11.so.6", "libX11.so"])?;
        let init: unsafe extern "C" fn() -> c_int = sym!(x11, "XInitThreads": fn() -> c_int)?;
        unsafe { init() };
        Some(XLibs {
            x11,
            xtst: Lib::open(&["libXtst.so.6", "libXtst.so"]),
            xrandr: Lib::open(&["libXrandr.so.2", "libXrandr.so"]),
        })
    }

    pub fn open_display(&self) -> Option<*mut Display> {
        let open = sym!(self.x11, "XOpenDisplay": fn(*const c_char) -> *mut Display)?;
        let d = unsafe { open(ptr::null()) };
        if d.is_null() { None } else { Some(d) }
    }
}

// ------------------------------------------------------------------ input

/// One decoded X input event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Input {
    Motion(f64, f64),
    Button { button: u8, x: f64, y: f64 },
    KeyDown { keycode: u8, state: u16 },
    KeyUp { keycode: u8 },
}

/// Decode a raw 32-byte xEvent recorded from the server.
pub fn decode_event(data: &[u8], swapped: bool) -> Option<Input> {
    if data.len() < 32 {
        return None;
    }
    let u16_at = |i: usize| {
        let b = [data[i], data[i + 1]];
        if swapped { u16::from_be_bytes(b) } else { u16::from_ne_bytes(b) }
    };
    let i16_at = |i: usize| u16_at(i) as i16 as f64;
    let (x, y) = (i16_at(20), i16_at(22));
    match data[0] & 0x7f {
        2 => Some(Input::KeyDown { keycode: data[1], state: u16_at(28) }),
        3 => Some(Input::KeyUp { keycode: data[1] }),
        4 => Some(Input::Button { button: data[1], x, y }),
        6 => Some(Input::Motion(x, y)),
        _ => None,
    }
}

/// X modifier state -> core MK_MOD_* bits (Shift 1, Control 4, Mod1/Alt 8, Mod4/Super 64).
pub fn mods_from_state(state: u16) -> u32 {
    let mut m = 0;
    if state & 4 != 0 { m |= 1; }
    if state & 8 != 0 { m |= 2; }
    if state & 1 != 0 { m |= 4; }
    if state & 64 != 0 { m |= 8; }
    m
}

type InterceptProc = unsafe extern "C" fn(*mut c_char, *mut RecordData);

#[repr(C)]
pub struct RecordData {
    id_base: XId,
    server_time: c_ulong,
    client_seq: c_ulong,
    category: c_int,
    client_swapped: c_int,
    data: *mut c_uchar,
    data_len: c_ulong,
}

struct Recorder {
    free_data: unsafe extern "C" fn(*mut RecordData),
    sink: Box<dyn FnMut(Input) + Send>,
}

unsafe extern "C" fn on_record(closure: *mut c_char, data: *mut RecordData) {
    let rec = &mut *(closure as *mut Recorder);
    let d = &*data;
    if d.category == 0 && !d.data.is_null() && d.data_len >= 8 {
        let bytes = std::slice::from_raw_parts(d.data, 32);
        if let Some(ev) = decode_event(bytes, d.client_swapped != 0) {
            (rec.sink)(ev);
        }
    }
    (rec.free_data)(data);
}

/// Start recording core input events on a background thread. Returns
/// false when RECORD is unavailable. `sink` runs on that thread.
pub fn start_record(libs: &'static XLibs, sink: Box<dyn FnMut(Input) + Send>) -> bool {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("x11-record".into())
        .spawn(move || {
            let ok = unsafe { record_loop(libs, sink, &tx) };
            if !ok {
                let _ = tx.send(false);
            }
        })
        .ok();
    rx.recv().unwrap_or(false)
}

unsafe fn record_loop(libs: &XLibs, sink: Box<dyn FnMut(Input) + Send>, started: &std::sync::mpsc::Sender<bool>) -> bool {
    let Some(xtst) = libs.xtst.as_ref() else { return false };
    let (Some(ctrl), Some(data)) = (libs.open_display(), libs.open_display()) else { return false };
    let Some(query_version) = sym!(xtst, "XRecordQueryVersion": fn(*mut Display, *mut c_int, *mut c_int) -> c_int) else { return false };
    let (mut major, mut minor) = (0, 0);
    if query_version(ctrl, &mut major, &mut minor) == 0 {
        return false;
    }
    let (Some(alloc_range), Some(create), Some(enable), Some(free_data)) = (
        sym!(xtst, "XRecordAllocRange": fn() -> *mut u8),
        sym!(xtst, "XRecordCreateContext": fn(*mut Display, c_int, *mut XId, c_int, *mut *mut u8, c_int) -> XId),
        sym!(xtst, "XRecordEnableContext": fn(*mut Display, XId, InterceptProc, *mut c_char) -> c_int),
        sym!(xtst, "XRecordFreeData": fn(*mut RecordData)),
    ) else {
        return false;
    };
    if let Some(sync) = sym!(libs.x11, "XSynchronize": fn(*mut Display, c_int) -> *mut c_void) {
        sync(ctrl, 1);
    }
    let range = alloc_range();
    if range.is_null() {
        return false;
    }
    // XRecordRange.device_events {first, last} lives at byte 18: KeyPress(2)..MotionNotify(6)
    *range.add(18) = 2;
    *range.add(19) = 6;
    let mut all_clients: XId = 3; // XRecordAllClients
    let mut ranges = [range];
    let ctx = create(ctrl, 0, &mut all_clients, 1, ranges.as_mut_ptr(), 1);
    if ctx == 0 {
        return false;
    }
    if let Some(flush) = sym!(libs.x11, "XFlush": fn(*mut Display) -> c_int) {
        flush(ctrl);
    }
    let _ = started.send(true);
    let rec = Box::into_raw(Box::new(Recorder { free_data, sink }));
    // blocks for the life of the app
    enable(data, ctx, on_record, rec as *mut c_char);
    true
}

// ------------------------------------------------------------ queries

/// RandR monitor info (Xrandr 1.5), see X11/extensions/Xrandr.h.
#[repr(C)]
struct MonitorInfo {
    name: XId,
    primary: c_int,
    automatic: c_int,
    noutput: c_int,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    mwidth: c_int,
    mheight: c_int,
    outputs: *mut XId,
}

#[repr(C)]
struct ClassHint {
    res_name: *mut c_char,
    res_class: *mut c_char,
}

/// A display connection owned by one thread (the tick thread).
pub struct Query {
    libs: &'static XLibs,
    d: *mut Display,
    root: XId,
    active_atom: XId,
}

impl Query {
    pub fn new(libs: &'static XLibs) -> Option<Query> {
        let d = libs.open_display()?;
        unsafe {
            let root = sym!(libs.x11, "XDefaultRootWindow": fn(*mut Display) -> XId)?(d);
            let intern = sym!(libs.x11, "XInternAtom": fn(*mut Display, *const c_char, c_int) -> XId)?;
            let active_atom = intern(d, b"_NET_ACTIVE_WINDOW\0".as_ptr() as *const c_char, 0);
            Some(Query { libs, d, root, active_atom })
        }
    }

    pub fn cursor(&self) -> Option<(f64, f64)> {
        let q = sym!(self.libs.x11, "XQueryPointer": fn(*mut Display, XId, *mut XId, *mut XId, *mut c_int, *mut c_int, *mut c_int, *mut c_int, *mut c_uint) -> c_int)?;
        let (mut r, mut c) = (0, 0);
        let (mut rx, mut ry, mut wx, mut wy, mut mask) = (0, 0, 0, 0, 0);
        let ok = unsafe { q(self.d, self.root, &mut r, &mut c, &mut rx, &mut ry, &mut wx, &mut wy, &mut mask) };
        if ok != 0 { Some((rx as f64, ry as f64)) } else { None }
    }

    fn atom_name(&self, atom: XId) -> Option<String> {
        let get = sym!(self.libs.x11, "XGetAtomName": fn(*mut Display, XId) -> *mut c_char)?;
        let free = sym!(self.libs.x11, "XFree": fn(*mut c_void) -> c_int)?;
        unsafe {
            let p = get(self.d, atom);
            if p.is_null() {
                return None;
            }
            let s = CStr::from_ptr(p).to_string_lossy().into_owned();
            free(p as *mut c_void);
            Some(s)
        }
    }

    /// Displays JSON for the core (see core/include/odomouse_core.h).
    pub fn displays_json(&self) -> String {
        let mut out = Vec::new();
        if let Some(xr) = self.libs.xrandr.as_ref() {
            if let (Some(get), Some(free)) = (
                sym!(xr, "XRRGetMonitors": fn(*mut Display, XId, c_int, *mut c_int) -> *mut MonitorInfo),
                sym!(xr, "XRRFreeMonitors": fn(*mut MonitorInfo)),
            ) {
                unsafe {
                    let mut n = 0;
                    let list = get(self.d, self.root, 1, &mut n);
                    if !list.is_null() {
                        for m in std::slice::from_raw_parts(list, n.max(0) as usize) {
                            let name = self.atom_name(m.name).unwrap_or_else(|| format!("monitor-{}", out.len()));
                            out.push(display_entry(&name, m.x, m.y, m.width, m.height, m.mwidth, m.mheight));
                        }
                        free(list);
                    }
                }
            }
        }
        if out.is_empty() {
            // no RandR 1.5: the whole X screen as one display
            let f = |n: &str| self.libs.x11.sym::<unsafe extern "C" fn(*mut Display, c_int) -> c_int>(n);
            if let (Some(w), Some(h), Some(wmm), Some(hmm)) = (f("XDisplayWidth"), f("XDisplayHeight"), f("XDisplayWidthMM"), f("XDisplayHeightMM")) {
                unsafe { out.push(display_entry("screen", 0, 0, w(self.d, 0), h(self.d, 0), wmm(self.d, 0), hmm(self.d, 0))) };
            }
        }
        format!("[{}]", out.join(","))
    }

    /// WM_CLASS of the active window ("firefox", "Code"), for per-app stats.
    pub fn active_app(&self) -> Option<String> {
        let get_prop = sym!(self.libs.x11, "XGetWindowProperty": fn(*mut Display, XId, XId, c_long, c_long, c_int, XId, *mut XId, *mut c_int, *mut c_ulong, *mut c_ulong, *mut *mut c_uchar) -> c_int)?;
        let free = sym!(self.libs.x11, "XFree": fn(*mut c_void) -> c_int)?;
        let class_hint = sym!(self.libs.x11, "XGetClassHint": fn(*mut Display, XId, *mut ClassHint) -> c_int)?;
        unsafe {
            let (mut ty, mut fmt, mut n, mut after) = (0, 0, 0, 0);
            let mut data: *mut c_uchar = ptr::null_mut();
            let r = get_prop(self.d, self.root, self.active_atom, 0, 1, 0, 33 /* XA_WINDOW */, &mut ty, &mut fmt, &mut n, &mut after, &mut data);
            if r != 0 || data.is_null() || n == 0 {
                if !data.is_null() { free(data as *mut c_void); }
                return None;
            }
            let win = *(data as *const XId);
            free(data as *mut c_void);
            if win == 0 {
                return None;
            }
            let mut hint = ClassHint { res_name: ptr::null_mut(), res_class: ptr::null_mut() };
            if class_hint(self.d, win, &mut hint) == 0 {
                return None;
            }
            let name = if !hint.res_class.is_null() { Some(CStr::from_ptr(hint.res_class).to_string_lossy().into_owned()) } else { None };
            if !hint.res_name.is_null() { free(hint.res_name as *mut c_void); }
            if !hint.res_class.is_null() { free(hint.res_class as *mut c_void); }
            name
        }
    }
}

fn display_entry(name: &str, x: i32, y: i32, w: i32, h: i32, wmm: i32, hmm: i32) -> String {
    let internal = ["eDP", "LVDS", "DSI"].iter().any(|p| name.starts_with(p));
    // X11 uses real pixels: a laptop panel is ~140 px/in, a desktop monitor ~96.
    let ppi = if internal { 140 } else { 96 };
    let esc = name.replace('\\', "").replace('"', "");
    format!(
        r#"{{"id":"{esc}","label":"{esc}","internal":{internal},"scaleFactor":1,"bounds":{{"x":{x},"y":{y},"width":{w},"height":{h}}},"widthMm":{wmm},"heightMm":{hmm},"estimatePpi":{ppi}}}"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_core_events() {
        let mut ev = [0u8; 32];
        ev[0] = 6;
        ev[20..22].copy_from_slice(&300i16.to_ne_bytes());
        ev[22..24].copy_from_slice(&(-20i16).to_ne_bytes());
        assert_eq!(decode_event(&ev, false), Some(Input::Motion(300.0, -20.0)));
        ev[0] = 2 | 0x80; // sent-event bit
        ev[1] = 38;
        ev[28..30].copy_from_slice(&5u16.to_ne_bytes());
        assert_eq!(decode_event(&ev, false), Some(Input::KeyDown { keycode: 38, state: 5 }));
        assert_eq!(mods_from_state(5), 1 | 4);
        assert_eq!(mods_from_state(64 | 8), 8 | 2);
        ev[0] = 9;
        assert_eq!(decode_event(&ev, false), None);
    }
}
