//! Physical key ids. The canonical id is the libuiohook "VC" code (PC set-1
//! scancode, extended keys = 0xE000 | code), the same ids the first version
//! stored, so history.json files stay compatible. Each native shell translates its own
//! key codes to these (see the `from_*` functions).

pub mod k {
    pub const ESCAPE: u32 = 1;
    pub const D1: u32 = 2;
    pub const D2: u32 = 3;
    pub const D3: u32 = 4;
    pub const D4: u32 = 5;
    pub const D5: u32 = 6;
    pub const D6: u32 = 7;
    pub const D7: u32 = 8;
    pub const D8: u32 = 9;
    pub const D9: u32 = 10;
    pub const D0: u32 = 11;
    pub const MINUS: u32 = 12;
    pub const EQUAL: u32 = 13;
    pub const BACKSPACE: u32 = 14;
    pub const TAB: u32 = 15;
    pub const Q: u32 = 16;
    pub const W: u32 = 17;
    pub const E: u32 = 18;
    pub const R: u32 = 19;
    pub const T: u32 = 20;
    pub const Y: u32 = 21;
    pub const U: u32 = 22;
    pub const I: u32 = 23;
    pub const O: u32 = 24;
    pub const P: u32 = 25;
    pub const BRACKET_LEFT: u32 = 26;
    pub const BRACKET_RIGHT: u32 = 27;
    pub const ENTER: u32 = 28;
    pub const CTRL: u32 = 29;
    pub const A: u32 = 30;
    pub const S: u32 = 31;
    pub const D: u32 = 32;
    pub const F: u32 = 33;
    pub const G: u32 = 34;
    pub const H: u32 = 35;
    pub const J: u32 = 36;
    pub const K: u32 = 37;
    pub const L: u32 = 38;
    pub const SEMICOLON: u32 = 39;
    pub const QUOTE: u32 = 40;
    pub const BACKQUOTE: u32 = 41;
    pub const SHIFT: u32 = 42;
    pub const BACKSLASH: u32 = 43;
    pub const Z: u32 = 44;
    pub const X: u32 = 45;
    pub const C: u32 = 46;
    pub const V: u32 = 47;
    pub const B: u32 = 48;
    pub const N: u32 = 49;
    pub const M: u32 = 50;
    pub const COMMA: u32 = 51;
    pub const PERIOD: u32 = 52;
    pub const SLASH: u32 = 53;
    pub const SHIFT_RIGHT: u32 = 54;
    pub const ALT: u32 = 56;
    pub const SPACE: u32 = 57;
    pub const CAPS_LOCK: u32 = 58;
    pub const F1: u32 = 59;
    pub const F10: u32 = 68;
    pub const F11: u32 = 87;
    pub const F12: u32 = 88;
    pub const CTRL_RIGHT: u32 = 3613;
    pub const ALT_RIGHT: u32 = 3640;
    pub const META: u32 = 3675;
    pub const META_RIGHT: u32 = 3676;
    pub const ARROW_UP: u32 = 57416;
    pub const ARROW_LEFT: u32 = 57419;
    pub const ARROW_RIGHT: u32 = 57421;
    pub const ARROW_DOWN: u32 = 57424;
}

use k::*;

const LETTER_CODES: [(u32, char); 26] = [
    (A, 'A'), (B, 'B'), (C, 'C'), (D, 'D'), (E, 'E'), (F, 'F'), (G, 'G'), (H, 'H'), (I, 'I'),
    (J, 'J'), (K, 'K'), (L, 'L'), (M, 'M'), (N, 'N'), (O, 'O'), (P, 'P'), (Q, 'Q'), (R, 'R'),
    (S, 'S'), (T, 'T'), (U, 'U'), (V, 'V'), (W, 'W'), (X, 'X'), (Y, 'Y'), (Z, 'Z'),
];

/// 'A'..'Z' for letter keys.
pub fn letter(code: u32) -> Option<char> {
    LETTER_CODES.iter().find(|l| l.0 == code).map(|l| l.1)
}

pub fn code_of_letter(ch: char) -> Option<u32> {
    let up = ch.to_ascii_uppercase();
    LETTER_CODES.iter().find(|l| l.1 == up).map(|l| l.0)
}

/// Keys that produce a character (counted for WPM). Enter counts like it
/// does in typing tests; Tab, arrows and modifiers do not.
pub fn is_printable(code: u32) -> bool {
    letter(code).is_some()
        || (D1..=D0).contains(&code)
        || matches!(
            code,
            SPACE | ENTER | BACKQUOTE | MINUS | EQUAL | BRACKET_LEFT | BRACKET_RIGHT | BACKSLASH
                | SEMICOLON | QUOTE | COMMA | PERIOD | SLASH
        )
}

pub fn is_modifier(code: u32) -> bool {
    matches!(
        code,
        SHIFT | SHIFT_RIGHT | CTRL | CTRL_RIGHT | ALT | ALT_RIGHT | META | META_RIGHT | CAPS_LOCK
    )
}

/// Short label as printed on a Mac key: 'E', '⌘', 'space'.
pub fn label_for(code: u32) -> String {
    if let Some(c) = letter(code) {
        return c.to_string();
    }
    let s = match code {
        ESCAPE => "esc",
        D1..=D9 => return (code - 1).to_string(),
        D0 => "0",
        MINUS => "-",
        EQUAL => "=",
        BACKSPACE => "⌫",
        TAB => "⇥",
        BRACKET_LEFT => "[",
        BRACKET_RIGHT => "]",
        BACKSLASH => "\\",
        CAPS_LOCK => "⇪",
        SEMICOLON => ";",
        QUOTE => "'",
        ENTER => "↩",
        SHIFT | SHIFT_RIGHT => "⇧",
        COMMA => ",",
        PERIOD => ".",
        SLASH => "/",
        BACKQUOTE => "`",
        CTRL | CTRL_RIGHT => "⌃",
        ALT | ALT_RIGHT => "⌥",
        META | META_RIGHT => "⌘",
        SPACE => "space",
        ARROW_LEFT => "←",
        ARROW_UP => "↑",
        ARROW_DOWN => "↓",
        ARROW_RIGHT => "→",
        59..=68 => return format!("F{}", code - 58),
        F11 => "F11",
        F12 => "F12",
        _ => return format!("#{}", code),
    };
    s.to_string()
}

// ------------------------------------------------------------ native codes

/// macOS virtual key code (kVK_*, Carbon HIToolbox/Events.h) -> VC code.
pub fn from_mac(kvk: u16) -> Option<u32> {
    Some(match kvk {
        0x00 => A, 0x01 => S, 0x02 => D, 0x03 => F, 0x04 => H, 0x05 => G, 0x06 => Z, 0x07 => X,
        0x08 => C, 0x09 => V, 0x0B => B, 0x0C => Q, 0x0D => W, 0x0E => E, 0x0F => R, 0x10 => Y,
        0x11 => T, 0x12 => D1, 0x13 => D2, 0x14 => D3, 0x15 => D4, 0x16 => D6, 0x17 => D5,
        0x18 => EQUAL, 0x19 => D9, 0x1A => D7, 0x1B => MINUS, 0x1C => D8, 0x1D => D0,
        0x1E => BRACKET_RIGHT, 0x1F => O, 0x20 => U, 0x21 => BRACKET_LEFT, 0x22 => I, 0x23 => P,
        0x24 => ENTER, 0x25 => L, 0x26 => J, 0x27 => QUOTE, 0x28 => K, 0x29 => SEMICOLON,
        0x2A => BACKSLASH, 0x2B => COMMA, 0x2C => SLASH, 0x2D => N, 0x2E => M, 0x2F => PERIOD,
        0x30 => TAB, 0x31 => SPACE, 0x32 => BACKQUOTE, 0x33 => BACKSPACE, 0x35 => ESCAPE,
        0x36 => META_RIGHT, 0x37 => META, 0x38 => SHIFT, 0x39 => CAPS_LOCK, 0x3A => ALT,
        0x3B => CTRL, 0x3C => SHIFT_RIGHT, 0x3D => ALT_RIGHT, 0x3E => CTRL_RIGHT,
        0x7A => F1, 0x78 => F1 + 1, 0x63 => F1 + 2, 0x76 => F1 + 3, 0x60 => F1 + 4, 0x61 => F1 + 5,
        0x62 => F1 + 6, 0x64 => F1 + 7, 0x65 => F1 + 8, 0x6D => F10, 0x67 => F11, 0x6F => F12,
        0x7B => ARROW_LEFT, 0x7C => ARROW_RIGHT, 0x7D => ARROW_DOWN, 0x7E => ARROW_UP,
        0x0A => BACKQUOTE, // ISO section key, next to left shift
        0x75 => 0xE053,    // forward delete
        0x73 => 0xE047,    // home
        0x77 => 0xE04F,    // end
        0x74 => 0xE049,    // page up
        0x79 => 0xE051,    // page down
        _ => return None,
    })
}

/// Windows: a low-level keyboard hook gives the set-1 scan code plus an
/// "extended" flag. libuiohook writes extended keys as 0x0Exx, except the
/// navigation cluster that shares codes with the numpad (0x47..0x53), which
/// is 0xE0xx (ArrowLeft = 0xE04B, right ⌃ = 0x0E1D, left Win = 0x0E5B).
pub fn from_windows(scan_code: u32, extended: bool) -> Option<u32> {
    if scan_code == 0 || scan_code > 0x7F {
        return None;
    }
    Some(match (extended, scan_code) {
        (false, c) => c,
        (true, c @ 0x47..=0x53) => 0xE000 | c,
        (true, c) => 0x0E00 | c,
    })
}

/// Linux evdev KEY_* code -> VC code. evdev codes 1..88 equal set-1 scan
/// codes; the extended keys have their own evdev numbers.
pub fn from_evdev(code: u16) -> Option<u32> {
    let c = code as u32;
    Some(match c {
        1..=88 => c,
        96 => 0x0E1C,  // KEY_KPENTER
        97 => CTRL_RIGHT,
        98 => 0x0E35,  // KEY_KPSLASH
        100 => ALT_RIGHT,
        102 => 0xE047, // home
        103 => ARROW_UP,
        104 => 0xE049, // page up
        105 => ARROW_LEFT,
        106 => ARROW_RIGHT,
        107 => 0xE04F, // end
        108 => ARROW_DOWN,
        109 => 0xE051, // page down
        110 => 0xE052, // insert
        111 => 0xE053, // delete
        125 => META,
        126 => META_RIGHT,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_and_labels() {
        assert_eq!(letter(E), Some('E'));
        assert_eq!(code_of_letter('q'), Some(Q));
        assert_eq!(label_for(D1), "1");
        assert_eq!(label_for(D9), "9");
        assert_eq!(label_for(D0), "0");
        assert_eq!(label_for(META), "⌘");
        assert_eq!(label_for(SPACE), "space");
        assert_eq!(label_for(F1 + 4), "F5");
        assert_eq!(label_for(9999), "#9999");
        assert!(is_printable(SPACE) && is_printable(D5) && !is_printable(TAB));
        assert!(is_modifier(META_RIGHT) && !is_modifier(A));
    }

    #[test]
    fn native_codes_map_to_the_same_keys() {
        assert_eq!(from_mac(0x00), Some(A));
        assert_eq!(from_mac(0x37), Some(META));
        assert_eq!(from_mac(0x7B), Some(ARROW_LEFT));
        assert_eq!(from_mac(0x16), Some(D6));
        assert_eq!(from_windows(0x1E, false), Some(A));
        assert_eq!(from_windows(0x4B, true), Some(ARROW_LEFT));
        assert_eq!(from_windows(0x5B, true), Some(META));
        assert_eq!(from_windows(0x1D, true), Some(CTRL_RIGHT));
        assert_eq!(from_windows(0x38, true), Some(ALT_RIGHT));
        assert_eq!(from_evdev(30), Some(A));
        assert_eq!(from_evdev(105), Some(ARROW_LEFT));
        assert_eq!(from_evdev(125), Some(META));
    }
}
