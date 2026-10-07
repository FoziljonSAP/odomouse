//! Keyboard shortcuts (⌘C, ⇧⌘4 ...). A shortcut is a non-modifier key
//! pressed while ⌘/Win or Ctrl is held. Option/Alt alone is not a shortcut
//! (on a Mac it types special characters). Stored as "mask:keycode".

use crate::keyboard::{self, k};

pub const CTRL: u32 = 1;
pub const ALT: u32 = 2;
pub const SHIFT: u32 = 4;
pub const META: u32 = 8;

pub fn is_shortcut(code: u32, mods: u32) -> bool {
    mods & (META | CTRL) != 0 && !keyboard::is_modifier(code)
}

pub fn id_for(mask: u32, code: u32) -> String {
    format!("{}:{}", mask, code)
}

pub fn parse_id(id: &str) -> Option<(u32, u32)> {
    let (a, b) = id.split_once(':')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// Mac order: ⌃⌥⇧⌘ then the key, e.g. "⇧⌘4".
pub fn combo_label(id: &str) -> String {
    let Some((mask, code)) = parse_id(id) else { return id.to_string() };
    let mut s = String::new();
    if mask & CTRL != 0 { s.push('⌃'); }
    if mask & ALT != 0 { s.push('⌥'); }
    if mask & SHIFT != 0 { s.push('⇧'); }
    if mask & META != 0 { s.push('⌘'); }
    if code == k::SPACE { s.push_str("Space") } else { s.push_str(&keyboard::label_for(code)) }
    s
}

pub fn name_for(id: &str) -> Option<&'static str> {
    let (mask, code) = parse_id(id)?;
    Some(match (mask, code) {
        (META, k::C) => "Nusxa olish",
        (META, k::V) => "Joylashtirish",
        (META, k::X) => "Kesib olish",
        (META, k::Z) => "Bekor qilish",
        (m, k::Z) if m == META | SHIFT => "Qaytarish",
        (META, k::A) => "Hammasini belgilash",
        (META, k::S) => "Saqlash",
        (META, k::F) => "Qidirish",
        (META, k::T) => "Yangi tab",
        (META, k::W) => "Yopish",
        (META, k::Q) => "Ilovadan chiqish",
        (META, k::N) => "Yangi oyna",
        (META, k::R) => "Yangilash",
        (META, k::P) => "Chop etish",
        (META, k::L) => "Manzil satri",
        (META, k::TAB) => "Ilova almashtirish",
        (META, k::BACKQUOTE) => "Oyna almashtirish",
        (META, k::SPACE) => "Spotlight",
        (m, k::D3) if m == META | SHIFT => "Ekran skrinshoti",
        (m, k::D4) if m == META | SHIFT => "Qism skrinshoti",
        (m, k::D5) if m == META | SHIFT => "Skrinshot paneli",
        (META, k::BACKSPACE) => "Qatorni o'chirish",
        (META, k::ARROW_LEFT) => "Qator boshiga",
        (META, k::ARROW_RIGHT) => "Qator oxiriga",
        (META, k::SLASH) => "Izohga aylantirish",
        (CTRL, k::C) => "To'xtatish (Terminal)",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_rules_and_labels() {
        assert!(is_shortcut(k::C, META));
        assert!(is_shortcut(k::C, CTRL | SHIFT));
        assert!(!is_shortcut(k::C, ALT));
        assert!(!is_shortcut(k::SHIFT, META));
        assert_eq!(combo_label(&id_for(META | SHIFT, k::D4)), "⇧⌘4");
        assert_eq!(combo_label(&id_for(META, k::SPACE)), "⌘Space");
        assert_eq!(name_for("8:46"), Some("Nusxa olish"));
        assert_eq!(name_for("12:44"), Some("Qaytarish"));
        assert_eq!(name_for("2:46"), None);
    }
}
