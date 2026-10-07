//! "How much is that?" comparisons (src/core/fun.js).

use crate::i18n::{en_form, ru_form, Lang};
use crate::json::Json;
use crate::units::format_number;

pub struct Ref {
    pub key: &'static str,
    pub size: f64,
    /// Settings label: uz, en, ru.
    pub label: [&'static str; 3],
    /// After a number. uz: no plural; en: [one, many]; ru: [one, few, many, fraction].
    pub uz: &'static str,
    pub en: [&'static str; 2],
    pub ru: [&'static str; 4],
}

impl Ref {
    pub fn label_in(&self, lang: Lang) -> &'static str {
        lang.pick(self.label[0], self.label[1], self.label[2])
    }

    pub fn unit_for(&self, lang: Lang, number: &str) -> &'static str {
        match lang {
            Lang::Uz => self.uz,
            Lang::En => en_form(number, self.en[0], self.en[1]),
            Lang::Ru => ru_form(number, &self.ru),
        }
    }
}

pub const DISTANCE_REFS: [Ref; 7] = [
    Ref { key: "banana", size: 0.18, label: ["Banan", "Banana", "Банан"], uz: "banan",
          en: ["banana", "bananas"], ru: ["банан", "банана", "бананов", "банана"] },
    Ref { key: "a4", size: 0.297, label: ["A4 varaq", "A4 sheet", "Лист A4"], uz: "A4 varaq",
          en: ["A4 sheet", "A4 sheets"], ru: ["лист A4", "листа A4", "листов A4", "листа A4"] },
    Ref { key: "bus", size: 12.0, label: ["Avtobus", "Bus", "Автобус"], uz: "avtobus",
          en: ["bus", "buses"], ru: ["автобус", "автобуса", "автобусов", "автобуса"] },
    Ref { key: "pool", size: 50.0, label: ["Basseyn", "Pool", "Бассейн"], uz: "olimpiya basseyni",
          en: ["Olympic pool", "Olympic pools"],
          ru: ["олимпийский бассейн", "олимпийских бассейна", "олимпийских бассейнов", "олимпийского бассейна"] },
    Ref { key: "football", size: 105.0, label: ["Futbol maydoni", "Football pitch", "Футбольное поле"], uz: "futbol maydoni",
          en: ["football pitch", "football pitches"],
          ru: ["футбольное поле", "футбольных поля", "футбольных полей", "футбольного поля"] },
    Ref { key: "eiffel", size: 330.0, label: ["Eyfel minorasi", "Eiffel Tower", "Эйфелева башня"], uz: "Eyfel minorasi",
          en: ["Eiffel Tower", "Eiffel Towers"],
          ru: ["Эйфелева башня", "Эйфелевы башни", "Эйфелевых башен", "Эйфелевой башни"] },
    Ref { key: "marathon", size: 42195.0, label: ["Marafon", "Marathon", "Марафон"], uz: "marafon",
          en: ["marathon", "marathons"], ru: ["марафон", "марафона", "марафонов", "марафона"] },
];

pub const TEXT_REFS: [Ref; 4] = [
    Ref { key: "sms", size: 160.0, label: ["SMS", "SMS", "SMS"], uz: "SMS",
          en: ["SMS", "SMS"], ru: ["SMS", "SMS", "SMS", "SMS"] },
    Ref { key: "tweet", size: 280.0, label: ["Tvit", "Tweet", "Твит"], uz: "tvit",
          en: ["tweet", "tweets"], ru: ["твит", "твита", "твитов", "твита"] },
    Ref { key: "page", size: 1800.0, label: ["Kitob sahifasi", "Book page", "Страница книги"], uz: "kitob sahifasi",
          en: ["book page", "book pages"], ru: ["страница книги", "страницы книги", "страниц книги", "страницы книги"] },
    Ref { key: "a4", size: 3000.0, label: ["A4 varaq", "A4 sheet", "Лист A4"], uz: "A4 varaq",
          en: ["A4 sheet", "A4 sheets"], ru: ["лист A4", "листа A4", "листов A4", "листа A4"] },
];

pub struct Comparison {
    pub key: &'static str,
    pub ratio: f64,
    pub text: String,
}

impl Comparison {
    pub fn to_json(&self) -> Json {
        Json::obj().set("key", self.key).set("ratio", self.ratio).set("text", self.text.as_str())
    }
}

/// "auto": the biggest reference that still fits at least twice.
fn pick<'a>(refs: &'a [Ref], amount: f64, key: &str) -> &'a Ref {
    if key != "auto" && !key.is_empty() {
        return refs.iter().find(|r| r.key == key).unwrap_or(&refs[0]);
    }
    let mut best = &refs[0];
    for r in refs {
        if amount >= 2.0 * r.size {
            best = r;
        }
    }
    best
}

fn compare(refs: &[Ref], amount: f64, key: &str, lang: Lang) -> Option<Comparison> {
    if !(amount > 0.0) {
        return None;
    }
    let rf = pick(refs, amount, key);
    let ratio = amount / rf.size;
    if ratio < 0.1 {
        return None;
    }
    let number = format_number(ratio, Some(if ratio < 10.0 { 1 } else { 0 }));
    let unit = rf.unit_for(lang, &number);
    Some(Comparison { key: rf.key, ratio, text: format!("≈ {} {}", number, unit) })
}

/// Cursor distance in metres -> "≈ 3.3 futbol maydoni".
pub fn distance_comparison(meters: f64, key: &str) -> Option<Comparison> {
    distance_comparison_in(meters, key, Lang::Uz)
}

pub fn distance_comparison_in(meters: f64, key: &str, lang: Lang) -> Option<Comparison> {
    compare(&DISTANCE_REFS, meters, key, lang)
}

/// Characters typed -> "≈ 4.1 A4 varaq".
pub fn text_comparison(chars: f64, key: &str) -> Option<Comparison> {
    text_comparison_in(chars, key, Lang::Uz)
}

pub fn text_comparison_in(chars: f64, key: &str, lang: Lang) -> Option<Comparison> {
    compare(&TEXT_REFS, chars, key, lang)
}

pub fn is_ref_key(refs: &[Ref], key: &str) -> bool {
    key == "auto" || refs.iter().any(|r| r.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages() {
        assert_eq!(distance_comparison_in(350.0, "football", Lang::En).unwrap().text, "≈ 3.3 football pitches");
        assert_eq!(distance_comparison_in(350.0, "football", Lang::Ru).unwrap().text, "≈ 3.3 футбольного поля");
        assert_eq!(distance_comparison_in(2310.0, "football", Lang::Ru).unwrap().text, "≈ 22 футбольных поля");
        assert_eq!(distance_comparison_in(5.4, "banana", Lang::Ru).unwrap().text, "≈ 30 бананов");
        assert_eq!(text_comparison_in(3000.0 * 21.0, "a4", Lang::Ru).unwrap().text, "≈ 21 лист A4");
        assert_eq!(text_comparison_in(12300.0, "a4", Lang::En).unwrap().text, "≈ 4.1 A4 sheets");
    }

    #[test]
    fn auto_picks_biggest_reference_that_fits_twice() {
        let c = distance_comparison(350.0, "auto").unwrap();
        assert_eq!(c.key, "football");
        assert_eq!(c.text, "≈ 3.3 futbol maydoni");
        assert_eq!(distance_comparison(1.0, "auto").unwrap().key, "a4");
        assert_eq!(distance_comparison(0.2, "auto").unwrap().key, "banana");
        assert_eq!(distance_comparison(100000.0, "auto").unwrap().key, "marathon");
    }

    #[test]
    fn chosen_reference_and_edge_cases() {
        assert_eq!(distance_comparison(350.0, "eiffel").unwrap().text, "≈ 1.1 Eyfel minorasi");
        assert_eq!(distance_comparison(350.0, "nope").unwrap().key, "banana");
        assert!(distance_comparison(0.0, "auto").is_none());
        assert!(distance_comparison(1.0, "marathon").is_none());
        let t = text_comparison(12300.0, "a4").unwrap();
        assert_eq!(t.text, "≈ 4.1 A4 varaq");
        assert_eq!(text_comparison(28000.0, "tweet").unwrap().text, "≈ 100 tvit");
        assert!(is_ref_key(&TEXT_REFS, "auto") && is_ref_key(&TEXT_REFS, "sms") && !is_ref_key(&TEXT_REFS, "bus"));
    }
}
