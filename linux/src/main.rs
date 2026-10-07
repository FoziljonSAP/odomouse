//! Odomouse for Linux.
//!
//! Threads: input (XRecord on X11, /dev/input on Wayland), a 1-second tick,
//! the D-Bus tray, and the loopback HTTP server for the dashboard. All of
//! them share one core Engine behind a mutex.

mod dbus;
mod dl;
mod evdev;
mod server;
mod tray;
mod x11;

use odomouse_core::engine::Engine;
use odomouse_core::json::{parse, Json};
use odomouse_core::keyboard;
use odomouse_core::time::Moment;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tray::{Action, Tray};

// ------------------------------------------------------------ time zone

#[repr(C)]
struct Tm {
    sec: i32,
    min: i32,
    hour: i32,
    mday: i32,
    mon: i32,
    year: i32,
    wday: i32,
    yday: i32,
    isdst: i32,
    gmtoff: std::ffi::c_long,
    zone: *const std::ffi::c_char,
}

extern "C" {
    fn localtime_r(t: *const i64, out: *mut Tm) -> *mut Tm;
    fn tzset();
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Local UTC offset in seconds (follows DST).
fn tz_offset() -> i32 {
    let t = now_ms() / 1000;
    let mut tm: Tm = unsafe { std::mem::zeroed() };
    unsafe {
        if localtime_r(&t, &mut tm).is_null() {
            return 0;
        }
    }
    tm.gmtoff as i32
}

// ------------------------------------------------------------------ app

struct App {
    engine: Mutex<Engine>,
    tz: AtomicI32,
    wayland: bool,
    input_ok: AtomicBool,
    actions: Mutex<Sender<Action>>,
    server: OnceLock<Arc<server::Server>>,
    bus: OnceLock<&'static dbus::Bus>,
    tray: OnceLock<&'static Tray>,
}

impl App {
    fn now(&self) -> Moment {
        Moment::new(now_ms(), self.tz.load(Ordering::Relaxed))
    }

    fn with<R>(&self, f: impl FnOnce(&mut Engine, Moment) -> R) -> R {
        let t = self.now();
        let mut e = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        f(&mut e, t)
    }

    fn act(&self, a: Action) {
        let _ = self.actions.lock().unwrap().send(a);
    }

    fn settings(&self) -> Json {
        self.with(|e, _| e.settings().to_json())
    }

    fn after_settings_change(&self) {
        let s = self.settings();
        sync_autostart(s.get("launchAtLogin").and_then(Json::as_bool).unwrap_or(false));
        if let Some(srv) = self.server.get() {
            srv.emit("settings", &s.to_string());
        }
        self.refresh_label();
    }

    fn refresh_label(&self) {
        let (text, lang) = self.with(|e, _| (e.tray_text(), e.lang()));
        if let (Some(bus), Some(tray)) = (self.bus.get(), self.tray.get()) {
            tray.set_label(bus, &text);
            let tr = |k| odomouse_core::i18n::tr(lang, k);
            tray.set_menu_texts(bus, [tr("stats"), tr("settings"), tr("quit")]);
        }
    }

    /// window.odomouse.* from the dashboard page.
    fn api(&self, body: &str) -> String {
        let Ok(req) = parse(body) else { return r#"{"error":"bad request"}"#.into() };
        let method = req.get("method").and_then(Json::as_str).unwrap_or("").to_string();
        let args = req.get("args").cloned().unwrap_or(Json::Arr(vec![]));
        let first = args.as_arr().and_then(|a| a.first()).and_then(Json::as_str).unwrap_or("").to_string();
        match method.as_str() {
            "getLive" | "getDashboard" | "getWrapped" | "resetData" | "clearApps" => {
                let out = self.with(|e, t| e.call(&method, &args, t)).to_string();
                if method == "resetData" {
                    self.refresh_label();
                }
                out
            }
            "updateSettings" => {
                let out = self.with(|e, t| e.call(&method, &args, t)).to_string();
                self.after_settings_change();
                out
            }
            "openDashboard" => {
                self.act(Action::Open(if first == "settings" { "settings" } else { "stats" }));
                "null".into()
            }
            "exportCsv" => {
                let (csv, date) = self.with(|e, t| {
                    let r = e.call("exportCsv", &Json::Null, t);
                    (r.get("csv").and_then(Json::as_str).unwrap_or("").to_string(), r.get("date").and_then(Json::as_str).unwrap_or("data").to_string())
                });
                save_file(&user_dir("DOCUMENTS"), &format!("odomouse-{}.csv", date), csv.as_bytes())
            }
            "saveWrapped" => {
                let period = args.as_arr().and_then(|a| a.get(1)).and_then(Json::as_str).unwrap_or("week").to_string();
                match png_from_data_url(&first) {
                    Some(png) => {
                        let date = self.now().day_key();
                        save_file(&user_dir("PICTURES"), &format!("odomouse-wrapped-{}-{}.png", period, date), &png)
                    }
                    None => r#"{"ok":false}"#.into(),
                }
            }
            "copyWrapped" => match png_from_data_url(&first) {
                Some(png) if copy_png(&png) => r#"{"ok":true}"#.into(),
                _ => r#"{"ok":false}"#.into(),
            },
            "openPermissions" => {
                let key = if !self.wayland {
                    "noPermissionNeededX11"
                } else if self.input_ok.load(Ordering::Relaxed) {
                    "trackingOn"
                } else {
                    "waylandHelp"
                };
                let msg = self.with(|e, _| odomouse_core::i18n::tr(e.lang(), key));
                Json::obj().set("ok", true).set("alert", msg).to_string()
            }
            "quit" => {
                self.act(Action::Quit);
                "null".into()
            }
            _ => Json::obj().set("error", format!("unknown method: {}", method)).to_string(),
        }
    }
}

// ----------------------------------------------------------- file helpers

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(".local/share"))
        .join("odomouse")
}

/// XDG user dir (DOCUMENTS, PICTURES ...) from ~/.config/user-dirs.dirs, else ~.
fn user_dir(kind: &str) -> PathBuf {
    let cfg = home().join(".config/user-dirs.dirs");
    if let Ok(text) = std::fs::read_to_string(cfg) {
        let key = format!("XDG_{}_DIR=", kind);
        for line in text.lines() {
            if let Some(v) = line.trim().strip_prefix(&key) {
                let v = v.trim_matches('"').replace("$HOME", &home().to_string_lossy());
                let p = PathBuf::from(v);
                if p.is_dir() {
                    return p;
                }
            }
        }
    }
    home()
}

fn save_file(dir: &Path, name: &str, data: &[u8]) -> String {
    let mut path = dir.join(name);
    let mut n = 2;
    while path.exists() {
        let (stem, ext) = name.rsplit_once('.').unwrap_or((name, ""));
        path = dir.join(format!("{} ({}).{}", stem, n, ext));
        n += 1;
    }
    match std::fs::write(&path, data) {
        Ok(()) => Json::obj().set("ok", true).set("filePath", path.to_string_lossy().into_owned()).to_string(),
        Err(_) => r#"{"ok":false}"#.into(),
    }
}

fn png_from_data_url(s: &str) -> Option<Vec<u8>> {
    let b64 = s.strip_prefix("data:image/png;base64,")?;
    base64_decode(b64)
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let bytes: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=').collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let mut acc = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            acc |= val(*c)? << (18 - 6 * i);
        }
        let n = chunk.len();
        if n < 2 {
            return None;
        }
        out.push((acc >> 16) as u8);
        if n > 2 { out.push((acc >> 8) as u8); }
        if n > 3 { out.push(acc as u8); }
    }
    Some(out)
}

fn copy_png(png: &[u8]) -> bool {
    let tools: [(&str, &[&str]); 2] = [("wl-copy", &["--type", "image/png"]), ("xclip", &["-selection", "clipboard", "-t", "image/png"])];
    for (tool, args) in tools {
        if let Ok(mut child) = Command::new(tool).args(args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(png);
            }
            return true; // the tool keeps running to serve the clipboard
        }
    }
    false
}

fn sync_autostart(want: bool) {
    let dir = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config")).join("autostart");
    let file = dir.join("odomouse.desktop");
    let _ = std::fs::remove_file(dir.join("mishka-tracker.desktop")); // the old name
    if want {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("odomouse"));
        let text = format!(
            "[Desktop Entry]\nType=Application\nName=Odomouse\nExec=\"{}\" --background\nIcon=odomouse\nX-GNOME-Autostart-enabled=true\nNoDisplay=false\n",
            exe.display()
        );
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(file, text);
    } else {
        let _ = std::fs::remove_file(file);
    }
}

/// The desktop language as the locale variables say ("ru_RU.UTF-8").
fn system_language() -> String {
    for var in ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(var) {
            let first = v.split(':').next().unwrap_or("").trim().to_string();
            if !first.is_empty() && first != "C" && first != "POSIX" && !first.starts_with("C.") {
                return first;
            }
        }
    }
    "en".to_string()
}

/// web/ next to the binary, or the installed copy.
fn web_root() -> PathBuf {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default();
    let candidates = [
        exe_dir.join("web"),
        exe_dir.join("../share/odomouse/web"),
        home().join(".local/share/odomouse/app/web"),
        PathBuf::from("/usr/share/odomouse/web"),
        exe_dir.join("../../../src"), // development: linux/target/<profile>/ -> repo src/ (ui/, core/)
    ];
    candidates.iter().find(|p| p.join("ui/dashboard.html").is_file()).cloned().unwrap_or_else(|| exe_dir.join("web"))
}

/// Chromium-family browsers open the page as an app window; anything else as a tab.
fn open_in_browser(url: &str, small: bool) {
    eprintln!("Odomouse: {}", url);
    let size = if small { "--window-size=380,680" } else { "--window-size=1060,800" };
    for b in ["chromium", "chromium-browser", "google-chrome", "google-chrome-stable", "brave-browser", "microsoft-edge"] {
        let ok = Command::new(b)
            .arg(format!("--app={}", url))
            .arg(size)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();
        if ok {
            return;
        }
    }
    let _ = Command::new("xdg-open").arg(url).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

// --------------------------------------------------------------- input

/// Feed X11 events to the core.
fn x11_sink(app: Arc<App>) -> Box<dyn FnMut(x11::Input) + Send> {
    Box::new(move |ev| {
        let t = app.now();
        let mut e = app.engine.lock().unwrap_or_else(|p| p.into_inner());
        match ev {
            x11::Input::Motion(x, y) => e.mouse_move(t, x, y),
            x11::Input::Button { button, x, y } => match button {
                1 => e.mouse_down(t, 1),
                2 => e.mouse_down(t, 3),
                3 => e.mouse_down(t, 2),
                4..=7 => e.wheel(t, x, y, 60.0), // one notch ≈ 3 lines
                _ => e.mouse_down(t, 4),
            },
            x11::Input::KeyDown { keycode, state } => {
                if let Some(vc) = keycode.checked_sub(8).and_then(|k| keyboard::from_evdev(k as u16)) {
                    e.key_down(t, vc, x11::mods_from_state(state));
                }
            }
            x11::Input::KeyUp { keycode } => {
                if let Some(vc) = keycode.checked_sub(8).and_then(|k| keyboard::from_evdev(k as u16)) {
                    e.key_up(t, vc);
                }
            }
        }
    })
}

/// Feed kernel input events (Wayland) to the core.
fn evdev_sink(app: Arc<App>) -> Arc<Mutex<dyn FnMut(evdev::Input) + Send>> {
    let (mut vx, mut vy) = (0.0f64, 0.0f64);
    let mut mods = 0u32;
    Arc::new(Mutex::new(move |ev: evdev::Input| {
        let t = app.now();
        let mut e = app.engine.lock().unwrap_or_else(|p| p.into_inner());
        match ev {
            evdev::Input::Motion(dx, dy) => {
                vx += dx;
                vy += dy;
                e.mouse_move(t, vx, vy);
            }
            evdev::Input::Button(b) => e.mouse_down(t, b),
            evdev::Input::Wheel(n) => e.wheel(t, vx, vy, n.abs() * 60.0),
            evdev::Input::KeyDown(code) => {
                mods |= mod_bit(code);
                if let Some(vc) = keyboard::from_evdev(code) {
                    e.key_down(t, vc, mods);
                }
            }
            evdev::Input::KeyUp(code) => {
                mods &= !mod_bit(code);
                if let Some(vc) = keyboard::from_evdev(code) {
                    e.key_up(t, vc);
                }
            }
        }
    }))
}

fn mod_bit(code: u16) -> u32 {
    match code {
        29 | 97 => 1,
        56 | 100 => 2,
        42 | 54 => 4,
        125 | 126 => 8,
        _ => 0,
    }
}

/// Wayland gives no screen coordinates: one large virtual plane, ~100 counts per inch.
const WAYLAND_DISPLAY_JSON: &str = r#"[{"id":"wayland","label":"Ekran (taxminiy)","internal":false,"scaleFactor":1,"bounds":{"x":-1000000000,"y":-1000000000,"width":2000000000,"height":2000000000},"widthMm":0,"heightMm":0,"estimatePpi":100}]"#;

// ---------------------------------------------------------------- main

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
}

/// Set by SIGTERM/SIGINT/SIGHUP (logout, shutdown, package upgrade): the
/// tick thread then quits the normal way, which saves today first.
static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_stop_signal(_: std::os::raw::c_int) {
    STOP.store(true, Ordering::Relaxed);
}

fn catch_stop_signals() {
    extern "C" {
        fn signal(sig: std::os::raw::c_int, handler: extern "C" fn(std::os::raw::c_int)) -> usize;
    }
    for sig in [1, 2, 15] {
        unsafe { signal(sig, on_stop_signal) };
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let background = args.iter().any(|a| a == "--background");

    // single instance: a second launch asks the running copy to open the dashboard
    let sock = runtime_dir().join(format!("odomouse-{}.sock", std::env::var("USER").unwrap_or_default()));
    if let Ok(mut s) = UnixStream::connect(&sock) {
        let _ = s.write_all(b"show");
        return;
    }
    let _ = std::fs::remove_file(&sock);
    let listener = UnixListener::bind(&sock).ok();

    unsafe { tzset() };
    catch_stop_signals();
    let (tx, rx) = channel::<Action>();
    let wayland = std::env::var("XDG_SESSION_TYPE").map_or(false, |v| v == "wayland")
        || (std::env::var_os("WAYLAND_DISPLAY").is_some() && std::env::var_os("DISPLAY").is_none());
    let tz = tz_offset();
    let mut engine = Engine::open(&data_dir(), Moment::new(now_ms(), tz));
    // "auto" language follows the desktop until the user picks one
    engine.set_system_language(&system_language());
    let app = Arc::new(App {
        engine: Mutex::new(engine),
        tz: AtomicI32::new(tz),
        wayland,
        input_ok: AtomicBool::new(false),
        actions: Mutex::new(tx.clone()),
        server: OnceLock::new(),
        bus: OnceLock::new(),
        tray: OnceLock::new(),
    });

    if let Some(listener) = listener {
        let tx = tx.clone();
        std::thread::spawn(move || {
            for mut s in listener.incoming().flatten() {
                let mut buf = [0u8; 16];
                let _ = s.read(&mut buf);
                let _ = tx.send(Action::Open("stats"));
            }
        });
    }

    // ---- input
    let xlibs: Option<&'static x11::XLibs> = if std::env::var_os("DISPLAY").is_some() {
        x11::XLibs::load().map(|l| &*Box::leak(Box::new(l)))
    } else {
        None
    };
    let mut recording = false;
    if !wayland {
        if let Some(libs) = xlibs {
            recording = x11::start_record(libs, x11_sink(app.clone()));
        }
    }
    let input_ok = recording || evdev::start(evdev_sink(app.clone()));
    app.input_ok.store(input_ok, Ordering::Relaxed);
    app.with(|e, _| e.set_hooks_running(input_ok));
    if recording {
        if let Some(q) = xlibs.and_then(x11::Query::new) {
            let json = q.displays_json();
            app.with(|e, _| e.set_displays_json(&json));
        }
    } else {
        app.with(|e, _| e.set_displays_json(WAYLAND_DISPLAY_JSON));
    }

    // ---- dashboard server
    let a1 = app.clone();
    let a2 = app.clone();
    let srv = server::Server::start(
        web_root(),
        Box::new(move |body| a1.api(body)),
        Box::new(move || {
            let theme = a2.settings().get("theme").and_then(Json::as_str).unwrap_or("system").to_string();
            format!("window.ODOMOUSE_PLATFORM = 'linux'; window.ODOMOUSE_HTTP = true; window.ODOMOUSE_THEME = {};", Json::from(theme).to_string())
        }),
    );
    if let Some(srv) = srv {
        let _ = app.server.set(srv);
    }

    // ---- tray on the session bus
    if let Some(bus) = dbus::Bus::session() {
        let bus: &'static dbus::Bus = Box::leak(Box::new(bus));
        let tray: &'static Tray = Box::leak(Box::new(Tray::new(tx.clone())));
        let name = format!("org.kde.StatusNotifierItem-{}-1", std::process::id());
        bus.request_name(&name);
        bus.register(tray::ITEM_PATH, tray);
        bus.register(tray::MENU_PATH, tray);
        let _ = app.bus.set(bus);
        let _ = app.tray.set(tray);
        app.refresh_label();
        std::thread::Builder::new()
            .name("dbus".into())
            .spawn(move || {
                let mut watcher_seen = false;
                let mut last_check = std::time::Instant::now() - Duration::from_secs(60);
                loop {
                    if last_check.elapsed() >= Duration::from_secs(5) {
                        last_check = std::time::Instant::now();
                        let present = Tray::watcher_present(bus);
                        if present && !watcher_seen {
                            Tray::register_with_watcher(bus, &name);
                        }
                        watcher_seen = present;
                    }
                    if !bus.dispatch(500) {
                        break;
                    }
                }
            })
            .ok();
    }

    // ---- 1-second tick
    let ticker = app.clone();
    let stop_tx = tx.clone();
    std::thread::Builder::new()
        .name("tick".into())
        .spawn(move || {
            let query = if recording { xlibs.and_then(x11::Query::new) } else { None };
            let mut n: u64 = 0;
            let mut last_displays = String::new();
            loop {
                std::thread::sleep(Duration::from_millis(1000));
                if STOP.load(Ordering::Relaxed) {
                    let _ = stop_tx.send(Action::Quit);
                    break;
                }
                n += 1;
                if n % 600 == 0 {
                    unsafe { tzset() };
                }
                ticker.tz.store(tz_offset(), Ordering::Relaxed);
                let cursor = query.as_ref().and_then(|q| q.cursor());
                if let Some(q) = query.as_ref() {
                    if n % 2 == 0 && ticker.with(|e, _| e.settings().track_apps) {
                        let name = q.active_app();
                        ticker.with(|e, _| e.set_app(name.as_deref()));
                    }
                    if n % 10 == 0 {
                        let json = q.displays_json();
                        if json != last_displays {
                            ticker.with(|e, _| e.set_displays_json(&json));
                            if !last_displays.is_empty() {
                                if let Some(s) = ticker.server.get() {
                                    s.emit("displays", &ticker.with(|e, _| e.public_displays()).to_string());
                                }
                            }
                            last_displays = json;
                        }
                    }
                }
                let out = ticker.with(|e, t| e.tick(t, cursor));
                if let (Some(bus), Some(tray)) = (ticker.bus.get(), ticker.tray.get()) {
                    tray.set_label(bus, out.get("tray").and_then(Json::as_str).unwrap_or(""));
                    if let Some(note) = out.get("notify").filter(|n| **n != Json::Null) {
                        dbus::notify(bus, note.get("title").and_then(Json::as_str).unwrap_or(""), note.get("body").and_then(Json::as_str).unwrap_or(""));
                    }
                }
                if n % 15 == 0 {
                    if let Some(s) = ticker.server.get() {
                        s.keepalive();
                    }
                }
            }
        })
        .ok();

    if !background {
        let _ = tx.send(Action::Open("stats"));
    }

    // ---- main loop: actions from the tray menu, second launches and the page
    for action in rx {
        match action {
            Action::Open(tab) => {
                let Some(srv) = app.server.get() else { continue };
                {
                    let seen = app.settings().get("seenDashboard").and_then(Json::as_bool).unwrap_or(false);
                    if !seen {
                        app.with(|e, t| e.call("markDashboardSeen", &Json::Null, t));
                        app.after_settings_change();
                    }
                }
                if srv.has_clients() {
                    srv.emit("tab", &Json::from(tab).to_string());
                } else {
                    open_in_browser(&srv.url("dashboard.html", &format!("tab={}", tab)), false);
                }
            }
            Action::Quit => break,
        }
    }
    app.with(|e, _| e.persist(true));
    let _ = std::fs::remove_file(&sock);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trip() {
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64_decode("aGVsbG8gd29ybGQ=").unwrap(), b"hello world");
        assert_eq!(base64_decode("YQ==").unwrap(), b"a");
        assert!(base64_decode("@@@@").is_none());
        assert_eq!(png_from_data_url("data:image/png;base64,iVBORw0KGgo=").unwrap()[..4], [0x89, b'P', b'N', b'G']);
    }

    #[test]
    fn tz_offset_is_sane() {
        let o = tz_offset();
        assert!((-14 * 3600..=14 * 3600).contains(&o));
    }
}
