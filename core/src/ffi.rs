//! C ABI for the Swift (macOS) and C# (Windows) shells. See
//! include/odomouse_core.h. All strings are UTF-8; every `char*` returned
//! must be released with mk_string_free. The handle is thread-safe (one
//! mutex), so input hooks may call from any thread. No call ever unwinds
//! into the host: a panic is caught and turned into an error result.

use crate::engine::Engine;
use crate::json::{parse, Json};
use crate::keyboard;
use crate::time::Moment;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

pub struct MkHandle {
    engine: Mutex<Engine>,
    tz_offset_s: AtomicI32,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl MkHandle {
    fn now(&self) -> Moment {
        Moment::new(now_ms(), self.tz_offset_s.load(Ordering::Relaxed))
    }

    fn with<R>(&self, f: impl FnOnce(&mut Engine, Moment) -> R) -> R {
        let t = self.now();
        let mut e = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        f(&mut e, t)
    }
}

unsafe fn str_arg<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        CStr::from_ptr(p).to_str().ok()
    }
}

fn out(s: String) -> *mut c_char {
    CString::new(s.replace('\0', "")).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}

fn guard(f: impl FnOnce() -> String) -> *mut c_char {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(s) => out(s),
        Err(_) => out(Json::obj().set("error", "internal error").to_string()),
    }
}

fn quiet(f: impl FnOnce()) {
    let _ = catch_unwind(AssertUnwindSafe(f));
}

/// Open (or create) the data folder. `tz_offset_s`: local UTC offset now.
#[no_mangle]
pub unsafe extern "C" fn mk_open(data_dir: *const c_char, tz_offset_s: i32) -> *mut MkHandle {
    let Some(dir) = str_arg(data_dir) else { return std::ptr::null_mut() };
    let dir = PathBuf::from(dir);
    catch_unwind(|| {
        let engine = Engine::open(&dir, Moment::new(now_ms(), tz_offset_s));
        Box::into_raw(Box::new(MkHandle { engine: Mutex::new(engine), tz_offset_s: AtomicI32::new(tz_offset_s) }))
    })
    .unwrap_or(std::ptr::null_mut())
}

/// Save and free the handle.
#[no_mangle]
pub unsafe extern "C" fn mk_close(h: *mut MkHandle) {
    if h.is_null() {
        return;
    }
    let h = Box::from_raw(h);
    quiet(|| {
        h.with(|e, _| e.persist(true));
    });
}

#[no_mangle]
pub unsafe extern "C" fn mk_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

// ----------------------------------------------------------------- input

#[no_mangle]
pub unsafe extern "C" fn mk_mouse_move(h: *const MkHandle, x: f64, y: f64) {
    if let Some(h) = h.as_ref() {
        quiet(|| h.with(|e, t| e.mouse_move(t, x, y)));
    }
}

/// button: 1 left, 2 right, 3 middle, other values count as "other".
#[no_mangle]
pub unsafe extern "C" fn mk_mouse_down(h: *const MkHandle, button: u32) {
    if let Some(h) = h.as_ref() {
        quiet(|| h.with(|e, t| e.mouse_down(t, button)));
    }
}

/// points: scrolled distance in cursor coordinate units (sign ignored).
#[no_mangle]
pub unsafe extern "C" fn mk_wheel(h: *const MkHandle, x: f64, y: f64, points: f64) {
    if let Some(h) = h.as_ref() {
        quiet(|| h.with(|e, t| e.wheel(t, x, y, points)));
    }
}

/// vc: key id from mk_vc_from_*; mods: MK_MOD_* bits.
#[no_mangle]
pub unsafe extern "C" fn mk_key_down(h: *const MkHandle, vc: u32, mods: u32) {
    if let Some(h) = h.as_ref() {
        if vc != 0 {
            quiet(|| h.with(|e, t| e.key_down(t, vc, mods)));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mk_key_up(h: *const MkHandle, vc: u32) {
    if let Some(h) = h.as_ref() {
        quiet(|| h.with(|e, t| e.key_up(t, vc)));
    }
}

#[no_mangle]
pub extern "C" fn mk_vc_from_mac(kvk: u16) -> u32 {
    keyboard::from_mac(kvk).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn mk_vc_from_windows(scan_code: u32, extended: bool) -> u32 {
    keyboard::from_windows(scan_code, extended).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn mk_vc_from_evdev(code: u16) -> u32 {
    keyboard::from_evdev(code).unwrap_or(0)
}

// ----------------------------------------------------------------- state

/// JSON array of displays, see include/odomouse_core.h.
#[no_mangle]
pub unsafe extern "C" fn mk_set_displays(h: *const MkHandle, json: *const c_char) {
    if let (Some(h), Some(j)) = (h.as_ref(), str_arg(json)) {
        quiet(|| h.with(|e, _| e.set_displays_json(j)));
    }
}

#[no_mangle]
pub unsafe extern "C" fn mk_set_hooks_running(h: *const MkHandle, running: bool) {
    if let Some(h) = h.as_ref() {
        quiet(|| h.with(|e, _| e.set_hooks_running(running)));
    }
}

/// Frontmost app name, or NULL. Ignored unless app stats are enabled.
#[no_mangle]
pub unsafe extern "C" fn mk_set_app(h: *const MkHandle, name: *const c_char) {
    if let Some(h) = h.as_ref() {
        let name = str_arg(name);
        quiet(|| h.with(|e, _| e.set_app(name)));
    }
}

/// Once a second. Returns {"tray": "...", "notify": null | {title, body}}.
#[no_mangle]
pub unsafe extern "C" fn mk_tick(h: *const MkHandle, tz_offset_s: i32, has_cursor: bool, x: f64, y: f64) -> *mut c_char {
    let Some(h) = h.as_ref() else { return std::ptr::null_mut() };
    h.tz_offset_s.store(tz_offset_s, Ordering::Relaxed);
    guard(|| h.with(|e, t| e.tick(t, if has_cursor { Some((x, y)) } else { None }).to_string()))
}

/// UI bridge: method name + JSON arguments (an array or a single value).
/// Returns JSON. Methods: getLive, getDashboard, getWrapped, getWidget,
/// updateSettings, exportCsv, resetData, clearApps, markDashboardSeen,
/// getReferences, getSettings, getDisplays.
#[no_mangle]
pub unsafe extern "C" fn mk_call(h: *const MkHandle, method: *const c_char, args_json: *const c_char) -> *mut c_char {
    let Some(h) = h.as_ref() else { return std::ptr::null_mut() };
    let method = str_arg(method).unwrap_or("").to_string();
    let args = str_arg(args_json).and_then(|a| parse(a).ok()).unwrap_or(Json::Null);
    guard(|| h.with(|e, t| e.call(&method, &args, t).to_string()))
}

/// Text for the menu bar / tray ("" = icon only).
#[no_mangle]
pub unsafe extern "C" fn mk_tray_text(h: *const MkHandle) -> *mut c_char {
    let Some(h) = h.as_ref() else { return std::ptr::null_mut() };
    guard(|| h.with(|e, _| e.tray_text()))
}

/// Write history.json now if anything changed. Returns false on I/O error.
#[no_mangle]
pub unsafe extern "C" fn mk_save(h: *const MkHandle) -> bool {
    let Some(h) = h.as_ref() else { return false };
    catch_unwind(AssertUnwindSafe(|| h.with(|e, _| e.persist(false)))).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn mk_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}
