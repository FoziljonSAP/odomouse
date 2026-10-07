//! Wayland: compositors do not let apps watch global input, so we read the
//! kernel's input devices (/dev/input/event*) directly. That needs the user
//! to be in the `input` group. Mice report relative motion, which we add up
//! on a virtual plane; the distance is an estimate (pointer acceleration is
//! applied later, by the compositor).

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Input {
    /// relative motion in device counts
    Motion(f64, f64),
    Button(u32),
    /// wheel notches (vertical or horizontal)
    Wheel(f64),
    KeyDown(u16),
    KeyUp(u16),
}

const EV_KEY: u16 = 1;
const EV_REL: u16 = 2;
const REL_X: u16 = 0;
const REL_Y: u16 = 1;
const REL_HWHEEL: u16 = 6;
const REL_WHEEL: u16 = 8;
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;
const BTN_TASK: u16 = 0x117;

/// One `struct input_event` (24 bytes on 64-bit Linux) -> Input.
pub fn decode(buf: &[u8]) -> Option<Input> {
    if buf.len() < 24 {
        return None;
    }
    let ty = u16::from_ne_bytes([buf[16], buf[17]]);
    let code = u16::from_ne_bytes([buf[18], buf[19]]);
    let value = i32::from_ne_bytes([buf[20], buf[21], buf[22], buf[23]]);
    match ty {
        EV_REL => match code {
            REL_X => Some(Input::Motion(value as f64, 0.0)),
            REL_Y => Some(Input::Motion(0.0, value as f64)),
            REL_WHEEL | REL_HWHEEL => Some(Input::Wheel(value as f64)),
            _ => None,
        },
        EV_KEY => match (code, value) {
            (BTN_LEFT..=BTN_TASK, 1) => Some(Input::Button(match code {
                BTN_LEFT => 1,
                BTN_RIGHT => 2,
                BTN_MIDDLE => 3,
                _ => 4,
            })),
            (BTN_LEFT..=BTN_TASK, _) => None,
            (c, 1) if c < 0x100 => Some(Input::KeyDown(c)),
            (c, 0) if c < 0x100 => Some(Input::KeyUp(c)),
            _ => None, // 2 = auto-repeat
        },
        _ => None,
    }
}

/// Watch every readable /dev/input/event* (rescanning for new devices).
/// Returns false if none could be opened (no permission).
pub fn start(sink: Arc<Mutex<dyn FnMut(Input) + Send>>) -> bool {
    let opened = Arc::new(Mutex::new(HashSet::new()));
    let any = scan(&sink, &opened);
    let sink2 = sink.clone();
    std::thread::Builder::new()
        .name("evdev-scan".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(10));
            scan(&sink2, &opened);
        })
        .ok();
    any
}

fn scan(sink: &Arc<Mutex<dyn FnMut(Input) + Send>>, opened: &Arc<Mutex<HashSet<String>>>) -> bool {
    let Ok(dir) = std::fs::read_dir("/dev/input") else { return false };
    let mut any = !opened.lock().unwrap().is_empty();
    for entry in dir.flatten() {
        let path = entry.path().to_string_lossy().into_owned();
        if !path.contains("/event") || opened.lock().unwrap().contains(&path) {
            continue;
        }
        let Ok(mut file) = File::open(&path) else { continue };
        any = true;
        opened.lock().unwrap().insert(path.clone());
        let sink = sink.clone();
        let opened = opened.clone();
        std::thread::Builder::new()
            .name("evdev".into())
            .spawn(move || {
                let mut buf = [0u8; 24 * 64];
                loop {
                    match file.read(&mut buf) {
                        Ok(n) if n >= 24 => {
                            let mut f = sink.lock().unwrap();
                            for chunk in buf[..n - n % 24].chunks(24) {
                                if let Some(ev) = decode(chunk) {
                                    f(ev);
                                }
                            }
                        }
                        _ => break, // device unplugged
                    }
                }
                opened.lock().unwrap().remove(&path);
            })
            .ok();
    }
    any
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(ty: u16, code: u16, value: i32) -> [u8; 24] {
        let mut b = [0u8; 24];
        b[16..18].copy_from_slice(&ty.to_ne_bytes());
        b[18..20].copy_from_slice(&code.to_ne_bytes());
        b[20..24].copy_from_slice(&value.to_ne_bytes());
        b
    }

    #[test]
    fn decodes_kernel_events() {
        assert_eq!(decode(&ev(EV_REL, REL_X, -7)), Some(Input::Motion(-7.0, 0.0)));
        assert_eq!(decode(&ev(EV_REL, REL_WHEEL, 1)), Some(Input::Wheel(1.0)));
        assert_eq!(decode(&ev(EV_KEY, BTN_RIGHT, 1)), Some(Input::Button(2)));
        assert_eq!(decode(&ev(EV_KEY, BTN_RIGHT, 0)), None);
        assert_eq!(decode(&ev(EV_KEY, 30, 1)), Some(Input::KeyDown(30)));
        assert_eq!(decode(&ev(EV_KEY, 30, 2)), None);
        assert_eq!(decode(&ev(EV_KEY, 30, 0)), Some(Input::KeyUp(30)));
        assert_eq!(decode(&ev(3, 0, 5)), None);
    }
}
