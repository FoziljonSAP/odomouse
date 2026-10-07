//! Session bus through libdbus-1 (dlopen): the tray item
//! (org.kde.StatusNotifierItem, shown by KDE, GNOME's AppIndicator
//! extension, XFCE, Cinnamon, ...), its right-click menu
//! (com.canonical.dbusmenu) and desktop notifications.

use crate::dl::Lib;
use crate::sym;
use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::ptr;
use std::sync::Mutex;

type Conn = c_void;
type Msg = c_void;
type Iter = [u64; 16]; // DBusMessageIter is 72-80 bytes; 128 is plenty

#[repr(C)]
struct VTable {
    unregister: Option<unsafe extern "C" fn(*mut Conn, *mut c_void)>,
    message: Option<unsafe extern "C" fn(*mut Conn, *mut Msg, *mut c_void) -> c_int>,
    pad: [*mut c_void; 4],
}

const HANDLED: c_int = 0;

struct Api {
    error_init: unsafe extern "C" fn(*mut u8),
    error_is_set: unsafe extern "C" fn(*const u8) -> c_uint,
    error_free: unsafe extern "C" fn(*mut u8),
    bus_get: unsafe extern "C" fn(c_int, *mut u8) -> *mut Conn,
    request_name: unsafe extern "C" fn(*mut Conn, *const c_char, c_uint, *mut u8) -> c_int,
    name_has_owner: unsafe extern "C" fn(*mut Conn, *const c_char, *mut u8) -> c_uint,
    register_path: unsafe extern "C" fn(*mut Conn, *const c_char, *const VTable, *mut c_void, *mut u8) -> c_uint,
    dispatch: unsafe extern "C" fn(*mut Conn, c_int) -> c_uint,
    send: unsafe extern "C" fn(*mut Conn, *mut Msg, *mut c_uint) -> c_uint,
    flush: unsafe extern "C" fn(*mut Conn),
    new_call: unsafe extern "C" fn(*const c_char, *const c_char, *const c_char, *const c_char) -> *mut Msg,
    new_return: unsafe extern "C" fn(*mut Msg) -> *mut Msg,
    new_signal: unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> *mut Msg,
    new_error: unsafe extern "C" fn(*mut Msg, *const c_char, *const c_char) -> *mut Msg,
    unref: unsafe extern "C" fn(*mut Msg),
    get_interface: unsafe extern "C" fn(*mut Msg) -> *const c_char,
    get_member: unsafe extern "C" fn(*mut Msg) -> *const c_char,
    get_path: unsafe extern "C" fn(*mut Msg) -> *const c_char,
    iter_init: unsafe extern "C" fn(*mut Msg, *mut Iter) -> c_uint,
    iter_arg_type: unsafe extern "C" fn(*mut Iter) -> c_int,
    iter_get_basic: unsafe extern "C" fn(*mut Iter, *mut c_void),
    iter_next: unsafe extern "C" fn(*mut Iter) -> c_uint,
    iter_init_append: unsafe extern "C" fn(*mut Msg, *mut Iter),
    iter_append_basic: unsafe extern "C" fn(*mut Iter, c_int, *const c_void) -> c_uint,
    iter_open: unsafe extern "C" fn(*mut Iter, c_int, *const c_char, *mut Iter) -> c_uint,
    iter_close: unsafe extern "C" fn(*mut Iter, *mut Iter) -> c_uint,
}

impl Api {
    fn load(lib: &Lib) -> Option<Api> {
        Some(Api {
            error_init: sym!(lib, "dbus_error_init": fn(*mut u8))?,
            error_is_set: sym!(lib, "dbus_error_is_set": fn(*const u8) -> c_uint)?,
            error_free: sym!(lib, "dbus_error_free": fn(*mut u8))?,
            bus_get: sym!(lib, "dbus_bus_get": fn(c_int, *mut u8) -> *mut Conn)?,
            request_name: sym!(lib, "dbus_bus_request_name": fn(*mut Conn, *const c_char, c_uint, *mut u8) -> c_int)?,
            name_has_owner: sym!(lib, "dbus_bus_name_has_owner": fn(*mut Conn, *const c_char, *mut u8) -> c_uint)?,
            register_path: sym!(lib, "dbus_connection_try_register_object_path": fn(*mut Conn, *const c_char, *const VTable, *mut c_void, *mut u8) -> c_uint)?,
            dispatch: sym!(lib, "dbus_connection_read_write_dispatch": fn(*mut Conn, c_int) -> c_uint)?,
            send: sym!(lib, "dbus_connection_send": fn(*mut Conn, *mut Msg, *mut c_uint) -> c_uint)?,
            flush: sym!(lib, "dbus_connection_flush": fn(*mut Conn))?,
            new_call: sym!(lib, "dbus_message_new_method_call": fn(*const c_char, *const c_char, *const c_char, *const c_char) -> *mut Msg)?,
            new_return: sym!(lib, "dbus_message_new_method_return": fn(*mut Msg) -> *mut Msg)?,
            new_signal: sym!(lib, "dbus_message_new_signal": fn(*const c_char, *const c_char, *const c_char) -> *mut Msg)?,
            new_error: sym!(lib, "dbus_message_new_error": fn(*mut Msg, *const c_char, *const c_char) -> *mut Msg)?,
            unref: sym!(lib, "dbus_message_unref": fn(*mut Msg))?,
            get_interface: sym!(lib, "dbus_message_get_interface": fn(*mut Msg) -> *const c_char)?,
            get_member: sym!(lib, "dbus_message_get_member": fn(*mut Msg) -> *const c_char)?,
            get_path: sym!(lib, "dbus_message_get_path": fn(*mut Msg) -> *const c_char)?,
            iter_init: sym!(lib, "dbus_message_iter_init": fn(*mut Msg, *mut Iter) -> c_uint)?,
            iter_arg_type: sym!(lib, "dbus_message_iter_get_arg_type": fn(*mut Iter) -> c_int)?,
            iter_get_basic: sym!(lib, "dbus_message_iter_get_basic": fn(*mut Iter, *mut c_void))?,
            iter_next: sym!(lib, "dbus_message_iter_next": fn(*mut Iter) -> c_uint)?,
            iter_init_append: sym!(lib, "dbus_message_iter_init_append": fn(*mut Msg, *mut Iter))?,
            iter_append_basic: sym!(lib, "dbus_message_iter_append_basic": fn(*mut Iter, c_int, *const c_void) -> c_uint)?,
            iter_open: sym!(lib, "dbus_message_iter_open_container": fn(*mut Iter, c_int, *const c_char, *mut Iter) -> c_uint)?,
            iter_close: sym!(lib, "dbus_message_iter_close_container": fn(*mut Iter, *mut Iter) -> c_uint)?,
        })
    }
}

// ------------------------------------------------------------- values

/// A D-Bus value to write.
#[derive(Debug, Clone)]
pub enum V {
    S(String),
    O(String),
    B(bool),
    I(i32),
    U(u32),
    Y(u8),
    Var(Box<V>),
    /// element signature, items
    Arr(String, Vec<V>),
    Struct(Vec<V>),
    Entry(Box<V>, Box<V>),
}

impl V {
    pub fn s(x: &str) -> V { V::S(x.to_string()) }
    pub fn var(v: V) -> V { V::Var(Box::new(v)) }
    pub fn dict(items: Vec<(&str, V)>) -> V {
        V::Arr("{sv}".into(), items.into_iter().map(|(k, v)| V::Entry(Box::new(V::s(k)), Box::new(V::var(v)))).collect())
    }

    pub fn sig(&self) -> String {
        match self {
            V::S(_) => "s".into(),
            V::O(_) => "o".into(),
            V::B(_) => "b".into(),
            V::I(_) => "i".into(),
            V::U(_) => "u".into(),
            V::Y(_) => "y".into(),
            V::Var(_) => "v".into(),
            V::Arr(el, _) => format!("a{}", el),
            V::Struct(items) => format!("({})", items.iter().map(V::sig).collect::<String>()),
            V::Entry(k, v) => format!("{{{}{}}}", k.sig(), v.sig()),
        }
    }
}

/// Top-level arguments we read from incoming calls.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    I(i32),
    U(u32),
    S(String),
    Other,
}

// --------------------------------------------------------------- bus

pub trait Handler: Send + Sync {
    /// Returns reply values, or None for "unknown method".
    fn call(&self, path: &str, iface: &str, member: &str, args: &[Arg]) -> Option<Vec<V>>;
}

pub struct Bus {
    api: Api,
    conn: *mut Conn,
    lock: Mutex<()>,
}

unsafe impl Send for Bus {}
unsafe impl Sync for Bus {}

struct PathData {
    bus: *const Bus,
    handler: *const dyn Handler,
}

unsafe extern "C" fn on_message(_c: *mut Conn, msg: *mut Msg, data: *mut c_void) -> c_int {
    let pd = &*(data as *const PathData);
    let bus = &*pd.bus;
    let handler = &*pd.handler;
    let s = |p: *const c_char| if p.is_null() { String::new() } else { CStr::from_ptr(p).to_string_lossy().into_owned() };
    let (path, iface, member) = (s((bus.api.get_path)(msg)), s((bus.api.get_interface)(msg)), s((bus.api.get_member)(msg)));
    let args = bus.read_args(msg);
    let reply = match handler.call(&path, &iface, &member, &args) {
        Some(values) => {
            let r = (bus.api.new_return)(msg);
            bus.write(r, &values);
            r
        }
        None => {
            let name = CString::new("org.freedesktop.DBus.Error.UnknownMethod").unwrap();
            let text = CString::new(format!("{}.{}", iface, member)).unwrap_or_default();
            (bus.api.new_error)(msg, name.as_ptr(), text.as_ptr())
        }
    };
    if !reply.is_null() {
        (bus.api.send)(bus.conn, reply, ptr::null_mut());
        (bus.api.unref)(reply);
    }
    HANDLED
}

impl Bus {
    pub fn session() -> Option<Bus> {
        let lib = Lib::open(&["libdbus-1.so.3", "libdbus-1.so"])?;
        let init = sym!(lib, "dbus_threads_init_default": fn() -> c_uint)?;
        let api = Api::load(&lib)?;
        std::mem::forget(lib); // keep libdbus loaded for the life of the app
        unsafe {
            init();
            let mut err = [0u8; 64];
            (api.error_init)(err.as_mut_ptr());
            let conn = (api.bus_get)(0, err.as_mut_ptr());
            if conn.is_null() || (api.error_is_set)(err.as_ptr()) != 0 {
                (api.error_free)(err.as_mut_ptr());
                return None;
            }
            Some(Bus { api, conn, lock: Mutex::new(()) })
        }
    }

    pub fn request_name(&self, name: &str) -> bool {
        let n = CString::new(name).unwrap();
        unsafe {
            let mut err = [0u8; 64];
            (self.api.error_init)(err.as_mut_ptr());
            let r = (self.api.request_name)(self.conn, n.as_ptr(), 4 /* DO_NOT_QUEUE */, err.as_mut_ptr());
            (self.api.error_free)(err.as_mut_ptr());
            r == 1 // PRIMARY_OWNER
        }
    }

    pub fn has_owner(&self, name: &str) -> bool {
        let n = CString::new(name).unwrap();
        unsafe {
            let mut err = [0u8; 64];
            (self.api.error_init)(err.as_mut_ptr());
            let r = (self.api.name_has_owner)(self.conn, n.as_ptr(), err.as_mut_ptr());
            (self.api.error_free)(err.as_mut_ptr());
            r != 0
        }
    }

    /// Serve `path` with `handler`. Both must live for the rest of the app.
    pub fn register(&'static self, path: &str, handler: &'static dyn Handler) -> bool {
        let vtable: &'static VTable = Box::leak(Box::new(VTable { unregister: None, message: Some(on_message), pad: [ptr::null_mut(); 4] }));
        let data = Box::leak(Box::new(PathData { bus: self, handler }));
        let p = CString::new(path).unwrap();
        unsafe {
            let mut err = [0u8; 64];
            (self.api.error_init)(err.as_mut_ptr());
            let ok = (self.api.register_path)(self.conn, p.as_ptr(), vtable, data as *mut PathData as *mut c_void, err.as_mut_ptr());
            (self.api.error_free)(err.as_mut_ptr());
            ok != 0
        }
    }

    /// Process incoming calls; returns false when the connection closed.
    pub fn dispatch(&self, timeout_ms: i32) -> bool {
        unsafe { (self.api.dispatch)(self.conn, timeout_ms) != 0 }
    }

    pub fn call_no_reply(&self, dest: &str, path: &str, iface: &str, member: &str, args: &[V]) {
        let (d, p, i, m) = (CString::new(dest).unwrap(), CString::new(path).unwrap(), CString::new(iface).unwrap(), CString::new(member).unwrap());
        let _g = self.lock.lock().unwrap();
        unsafe {
            let msg = (self.api.new_call)(d.as_ptr(), p.as_ptr(), i.as_ptr(), m.as_ptr());
            if msg.is_null() {
                return;
            }
            self.write(msg, args);
            (self.api.send)(self.conn, msg, ptr::null_mut());
            (self.api.unref)(msg);
            (self.api.flush)(self.conn);
        }
    }

    pub fn signal(&self, path: &str, iface: &str, member: &str, args: &[V]) {
        let (p, i, m) = (CString::new(path).unwrap(), CString::new(iface).unwrap(), CString::new(member).unwrap());
        let _g = self.lock.lock().unwrap();
        unsafe {
            let msg = (self.api.new_signal)(p.as_ptr(), i.as_ptr(), m.as_ptr());
            if msg.is_null() {
                return;
            }
            self.write(msg, args);
            (self.api.send)(self.conn, msg, ptr::null_mut());
            (self.api.unref)(msg);
            (self.api.flush)(self.conn);
        }
    }

    unsafe fn read_args(&self, msg: *mut Msg) -> Vec<Arg> {
        let mut it: Iter = [0; 16];
        let mut out = Vec::new();
        if (self.api.iter_init)(msg, &mut it) == 0 {
            return out;
        }
        loop {
            let ty = (self.api.iter_arg_type)(&mut it);
            if ty == 0 {
                break;
            }
            out.push(match ty as u8 {
                b'i' => {
                    let mut v: i32 = 0;
                    (self.api.iter_get_basic)(&mut it, &mut v as *mut i32 as *mut c_void);
                    Arg::I(v)
                }
                b'u' => {
                    let mut v: u32 = 0;
                    (self.api.iter_get_basic)(&mut it, &mut v as *mut u32 as *mut c_void);
                    Arg::U(v)
                }
                b's' | b'o' => {
                    let mut p: *const c_char = ptr::null();
                    (self.api.iter_get_basic)(&mut it, &mut p as *mut *const c_char as *mut c_void);
                    Arg::S(if p.is_null() { String::new() } else { CStr::from_ptr(p).to_string_lossy().into_owned() })
                }
                _ => Arg::Other,
            });
            if (self.api.iter_next)(&mut it) == 0 {
                break;
            }
        }
        out
    }

    unsafe fn write(&self, msg: *mut Msg, values: &[V]) {
        let mut it: Iter = [0; 16];
        (self.api.iter_init_append)(msg, &mut it);
        for v in values {
            self.append(&mut it, v);
        }
    }

    unsafe fn append(&self, it: &mut Iter, v: &V) {
        let a = &self.api;
        match v {
            V::S(s) | V::O(s) => {
                let c = CString::new(s.replace('\0', "")).unwrap();
                let p = c.as_ptr();
                let ty = if matches!(v, V::O(_)) { b'o' } else { b's' };
                (a.iter_append_basic)(it, ty as c_int, &p as *const *const c_char as *const c_void);
            }
            V::B(b) => {
                let x: u32 = *b as u32;
                (a.iter_append_basic)(it, b'b' as c_int, &x as *const u32 as *const c_void);
            }
            V::I(x) => {
                (a.iter_append_basic)(it, b'i' as c_int, x as *const i32 as *const c_void);
            }
            V::U(x) => {
                (a.iter_append_basic)(it, b'u' as c_int, x as *const u32 as *const c_void);
            }
            V::Y(x) => {
                (a.iter_append_basic)(it, b'y' as c_int, x as *const u8 as *const c_void);
            }
            V::Var(inner) => {
                let sig = CString::new(inner.sig()).unwrap();
                let mut sub: Iter = [0; 16];
                (a.iter_open)(it, b'v' as c_int, sig.as_ptr(), &mut sub);
                self.append(&mut sub, inner);
                (a.iter_close)(it, &mut sub);
            }
            V::Arr(el, items) => {
                let sig = CString::new(el.as_str()).unwrap();
                let mut sub: Iter = [0; 16];
                (a.iter_open)(it, b'a' as c_int, sig.as_ptr(), &mut sub);
                for x in items {
                    self.append(&mut sub, x);
                }
                (a.iter_close)(it, &mut sub);
            }
            V::Struct(items) => {
                let mut sub: Iter = [0; 16];
                (a.iter_open)(it, b'r' as c_int, ptr::null(), &mut sub);
                for x in items {
                    self.append(&mut sub, x);
                }
                (a.iter_close)(it, &mut sub);
            }
            V::Entry(k, val) => {
                let mut sub: Iter = [0; 16];
                (a.iter_open)(it, b'e' as c_int, ptr::null(), &mut sub);
                self.append(&mut sub, k);
                self.append(&mut sub, val);
                (a.iter_close)(it, &mut sub);
            }
        }
    }
}

/// Desktop notification (org.freedesktop.Notifications).
pub fn notify(bus: &Bus, title: &str, body: &str) {
    bus.call_no_reply(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
        &[
            V::s("Odomouse"),
            V::U(0),
            V::s("odomouse"),
            V::s(title),
            V::s(body),
            V::Arr("s".into(), vec![]),
            V::dict(vec![]),
            V::I(15000),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures() {
        let pixmap = V::Arr("(iiay)".into(), vec![V::Struct(vec![V::I(1), V::I(1), V::Arr("y".into(), vec![V::Y(0)])])]);
        assert_eq!(pixmap.sig(), "a(iiay)");
        assert_eq!(V::dict(vec![("label", V::s("x"))]).sig(), "a{sv}");
        assert_eq!(V::Struct(vec![V::I(0), V::dict(vec![]), V::Arr("v".into(), vec![])]).sig(), "(ia{sv}av)");
    }
}
