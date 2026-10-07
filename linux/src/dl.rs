//! Tiny dlopen wrapper: system libraries are optional at run time.

use std::ffi::{c_char, c_int, c_void, CString};

#[link(name = "dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

const RTLD_NOW: c_int = 2;
const RTLD_GLOBAL: c_int = 0x100;

pub struct Lib(*mut c_void);

unsafe impl Send for Lib {}
unsafe impl Sync for Lib {}

impl Lib {
    /// First name that loads wins ("libX11.so.6", "libX11.so").
    pub fn open(names: &[&str]) -> Option<Lib> {
        for n in names {
            let c = CString::new(*n).ok()?;
            let h = unsafe { dlopen(c.as_ptr(), RTLD_NOW | RTLD_GLOBAL) };
            if !h.is_null() {
                return Some(Lib(h));
            }
        }
        None
    }

    /// The symbol as a function pointer of type `T` (must be an `extern "C" fn`).
    pub fn sym<T: Copy>(&self, name: &str) -> Option<T> {
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<*mut c_void>());
        let c = CString::new(name).ok()?;
        let p = unsafe { dlsym(self.0, c.as_ptr()) };
        if p.is_null() {
            None
        } else {
            Some(unsafe { std::mem::transmute_copy::<*mut c_void, T>(&p) })
        }
    }
}

/// `sym!(lib, "XOpenDisplay": fn(*const c_char) -> *mut c_void)`
#[macro_export]
macro_rules! sym {
    ($lib:expr, $name:literal : fn($($arg:ty),*) $(-> $ret:ty)?) => {
        $lib.sym::<unsafe extern "C" fn($($arg),*) $(-> $ret)?>($name)
    };
}
