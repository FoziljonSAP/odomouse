//! Languages: O'zbekcha (the original), English, Русский.
//!
//! The core formats everything the shells show without a web page (menu bar
//! text, notifications, menus, dialogs) and the pre-formatted pieces of the
//! JSON payloads, so the language lives here. The web UI has its own copy of
//! the page texts (src/ui/i18n.js); `strings()` below are the native ones.

use crate::json::Json;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Uz,
    En,
    Ru,
}

pub const LANGS: [&str; 3] = ["uz", "en", "ru"];

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Uz => "uz",
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    /// "uz" / "en" / "ru" exactly (a saved setting).
    pub fn parse(code: &str) -> Option<Lang> {
        match code {
            "uz" => Some(Lang::Uz),
            "en" => Some(Lang::En),
            "ru" => Some(Lang::Ru),
            _ => None,
        }
    }

    /// The OS language as the shells report it ("ru-RU", "uz-Latn-UZ",
    /// "en_US.UTF-8", "uz_UZ"...). Anything else gets English.
    pub fn from_system(tag: &str) -> Lang {
        let t = tag.trim().to_ascii_lowercase();
        if t.starts_with("uz") {
            Lang::Uz
        } else if t.starts_with("ru") {
            Lang::Ru
        } else {
            Lang::En
        }
    }

    pub fn pick<'a>(self, uz: &'a str, en: &'a str, ru: &'a str) -> &'a str {
        match self {
            Lang::Uz => uz,
            Lang::En => en,
            Lang::Ru => ru,
        }
    }
}

/// Russian noun form for a number as displayed: [one, few, many, fraction].
/// "1 банан", "2 банана", "5 бананов", "3.3 банана". `few` is separate from
/// `fraction` because adjectives differ: "2 футбольных поля" but
/// "3.3 футбольного поля".
pub fn ru_form<'a>(number: &str, forms: &[&'a str; 4]) -> &'a str {
    let digits: String = number.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    if digits.contains('.') {
        return forms[3];
    }
    let n: u64 = digits.parse().unwrap_or(0);
    let (d10, d100) = (n % 10, n % 100);
    if d10 == 1 && d100 != 11 {
        forms[0]
    } else if (2..=4).contains(&d10) && !(12..=14).contains(&d100) {
        forms[1]
    } else {
        forms[2]
    }
}

/// English: singular only for exactly "1".
pub fn en_form<'a>(number: &str, one: &'a str, many: &'a str) -> &'a str {
    if number == "1" { one } else { many }
}

/// Short units after a duration: "3 soat 25 daq", "3 h 25 min", "3 ч 25 мин".
pub fn minute_units(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Uz => ("soat", "daq"),
        Lang::En => ("h", "min"),
        Lang::Ru => ("ч", "мин"),
    }
}

/// A duration spelled out, for sentences: "1 soat 35 daqiqa",
/// "1 hour 35 minutes", "1 час 35 минут".
pub fn long_duration(lang: Lang, minutes: i64) -> String {
    let (h, m) = (minutes / 60, minutes % 60);
    let hours = |n: i64| {
        let s = n.to_string();
        match lang {
            Lang::Uz => format!("{} soat", n),
            Lang::En => format!("{} {}", n, en_form(&s, "hour", "hours")),
            Lang::Ru => format!("{} {}", n, ru_form(&s, &["час", "часа", "часов", "часа"])),
        }
    };
    let mins = |n: i64| {
        let s = n.to_string();
        match lang {
            Lang::Uz => format!("{} daqiqa", n),
            Lang::En => format!("{} {}", n, en_form(&s, "minute", "minutes")),
            Lang::Ru => format!("{} {}", n, ru_form(&s, &["минута", "минуты", "минут", "минуты"])),
        }
    };
    if h == 0 {
        mins(m)
    } else if m == 0 {
        hours(h)
    } else {
        format!("{} {}", hours(h), mins(m))
    }
}

/// Texts the native shells show: menus, dialogs, permission help.
/// (key, uz, en, ru)
const STRINGS: &[(&str, &str, &str, &str)] = &[
    ("stats", "Statistika", "Statistics", "Статистика"),
    ("settings", "Sozlamalar", "Settings", "Настройки"),
    ("settingsMenu", "Sozlamalar…", "Settings…", "Настройки…"),
    ("quit", "Odomouse'dan chiqish", "Quit Odomouse", "Выйти из Odomouse"),
    ("hide", "Odomouse'ni yashirish", "Hide Odomouse", "Скрыть Odomouse"),
    ("today", "Bugun", "Today", "Сегодня"),
    ("permissionNeeded", "⚠ ruxsat kerak", "⚠ permission needed", "⚠ нужен доступ"),
    ("grantKeyboard", "Klaviatura uchun ruxsat berish…", "Allow keyboard access…", "Разрешить доступ к клавиатуре…"),
    ("keysAllowed", "Klaviatura: ruxsat bor", "Keyboard: allowed", "Клавиатура: доступ есть"),
    ("keysDenied", "Klaviatura: ruxsat yo'q", "Keyboard: not allowed", "Клавиатура: нет доступа"),
    ("mouseOn", "Sichqoncha: ishlayapti", "Mouse: tracking", "Мышь: отслеживается"),
    ("mouseOff", "Sichqoncha: to'xtagan", "Mouse: stopped", "Мышь: остановлено"),
    ("cancel", "Bekor qilish", "Cancel", "Отмена"),
    (
        "hiddenTitle",
        "Odomouse ishlayapti, lekin belgisi ko'rinmayapti",
        "Odomouse is running, but its icon is hidden",
        "Odomouse работает, но значок скрыт",
    ),
    (
        "hiddenBody",
        "Menu bar'da joy yetmadi. ⌘ tugmasini bosib turib, keraksiz belgilarni sudrab olib tashlang yoki Sozlamalarda «Faqat ikonka»ni tanlang.",
        "The menu bar is full. Hold ⌘ and drag icons you don't need out of it, or choose \"Icon only\" in Settings.",
        "В строке меню не хватило места. Удерживая ⌘, перетащите лишние значки или выберите «Только значок» в Настройках.",
    ),
    ("trayHintTitle", "Odomouse shu yerda", "Odomouse is here", "Odomouse здесь"),
    (
        "trayHintBody",
        "Bugungi hisob vazifalar panelida, soat yonida turadi. Uni sichqoncha bilan istalgan joyga sudrab qo'yish mumkin.",
        "Today's count sits on the taskbar next to the clock. You can drag it anywhere with the mouse.",
        "Счёт за сегодня виден на панели задач рядом с часами. Его можно перетащить мышью в любое место.",
    ),
    ("counterReset", "Hisoblagichni joyiga qaytarish", "Put the counter back by the clock", "Вернуть счётчик к часам"),
    ("resetTitle", "Barcha statistika o'chirilsinmi?", "Delete all statistics?", "Удалить всю статистику?"),
    (
        "resetDetail",
        "Bu amalni qaytarib bo'lmaydi. Avval CSV eksport qilib olishingiz mumkin.",
        "This can't be undone. You can export a CSV first.",
        "Это действие нельзя отменить. Сначала можно сохранить CSV.",
    ),
    ("resetAction", "Hammasini o'chirish", "Delete everything", "Удалить всё"),
    ("appsTitle", "Ilovalar bo'yicha statistika o'chirilsinmi?", "Delete per-app statistics?", "Удалить статистику по приложениям?"),
    (
        "appsDetail",
        "Faqat ilova nomlari va ularga tegishli raqamlar o'chadi. Umumiy statistika qoladi.",
        "Only app names and their numbers are deleted. Overall statistics stay.",
        "Удалятся только названия приложений и их цифры. Общая статистика останется.",
    ),
    ("appsAction", "O'chirish", "Delete", "Удалить"),
    ("saveCsv", "Statistikani CSV sifatida saqlash", "Save statistics as CSV", "Сохранить статистику в CSV"),
    ("saveWrapped", "Wrapped kartani saqlash", "Save the Wrapped card", "Сохранить карточку Wrapped"),
    ("edit", "Tahrirlash", "Edit", "Правка"),
    ("undo", "Bekor qilish", "Undo", "Отменить"),
    ("redo", "Qaytarish", "Redo", "Повторить"),
    ("cut", "Kesish", "Cut", "Вырезать"),
    ("copy", "Nusxa olish", "Copy", "Копировать"),
    ("paste", "Joylashtirish", "Paste", "Вставить"),
    ("selectAll", "Hammasini belgilash", "Select All", "Выбрать всё"),
    ("window", "Oyna", "Window", "Окно"),
    ("close", "Yopish", "Close", "Закрыть"),
    ("minimize", "Kichraytirish", "Minimize", "Свернуть"),
    (
        "noPermissionNeededWindows",
        "Windows'da alohida ruxsat kerak emas. Kuzatish ishlayapti.",
        "Windows needs no extra permission. Tracking is on.",
        "В Windows отдельное разрешение не нужно. Отслеживание работает.",
    ),
    (
        "hooksFailedWindows",
        "Klaviatura va sichqonchani kuzatishni boshlab bo'lmadi. Kompyuterni qayta ishga tushirib ko'ring.",
        "Couldn't start tracking the keyboard and mouse. Try restarting the computer.",
        "Не удалось начать отслеживание клавиатуры и мыши. Попробуйте перезагрузить компьютер.",
    ),
    (
        "webview2Missing",
        "Statistika oynasi uchun Microsoft Edge WebView2 kerak. U Windows 10/11 da odatda bor.\n\nYuklab olish sahifasini ochaymi? Hisoblash WebView2'siz ham ishlayveradi.",
        "The statistics window needs Microsoft Edge WebView2. Windows 10/11 usually has it.\n\nOpen the download page? Counting works without WebView2 too.",
        "Для окна статистики нужен Microsoft Edge WebView2. Обычно он уже есть в Windows 10/11.\n\nОткрыть страницу загрузки? Подсчёт работает и без WebView2.",
    ),
    (
        "noPermissionNeededX11",
        "Linux (X11) da alohida ruxsat kerak emas.",
        "Linux (X11) needs no extra permission.",
        "В Linux (X11) отдельное разрешение не нужно.",
    ),
    ("trackingOn", "Kuzatish ishlayapti.", "Tracking is on.", "Отслеживание работает."),
    (
        "waylandHelp",
        "Wayland'da klaviatura va sichqonchani sanash uchun foydalanuvchingiz \"input\" guruhida bo'lishi kerak.\n\nTerminalda:\n  sudo usermod -aG input $USER\n\nSo'ng tizimdan chiqib, qayta kiring.",
        "On Wayland, your user must be in the \"input\" group to count the keyboard and mouse.\n\nIn a terminal:\n  sudo usermod -aG input $USER\n\nThen log out and back in.",
        "В Wayland для подсчёта клавиатуры и мыши ваш пользователь должен быть в группе \"input\".\n\nВ терминале:\n  sudo usermod -aG input $USER\n\nЗатем выйдите из системы и войдите снова.",
    ),
];

pub fn tr(lang: Lang, key: &str) -> &'static str {
    STRINGS
        .iter()
        .find(|s| s.0 == key)
        .map(|s| lang.pick(s.1, s.2, s.3))
        .unwrap_or("")
}

/// All native texts in one language, for `call("getStrings")`.
pub fn strings(lang: Lang) -> Json {
    let mut out = Json::obj();
    for s in STRINGS {
        out.insert(s.0, lang.pick(s.1, s.2, s.3));
    }
    out.set("lang", lang.code())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_tags() {
        assert_eq!(Lang::from_system("ru-RU"), Lang::Ru);
        assert_eq!(Lang::from_system("uz-Latn-UZ"), Lang::Uz);
        assert_eq!(Lang::from_system("uz_UZ.UTF-8"), Lang::Uz);
        assert_eq!(Lang::from_system("en_US.UTF-8"), Lang::En);
        assert_eq!(Lang::from_system("de-DE"), Lang::En);
        assert_eq!(Lang::from_system(""), Lang::En);
    }

    #[test]
    fn russian_plurals() {
        let f = ["банан", "банана", "бананов", "банана"];
        assert_eq!(ru_form("1", &f), "банан");
        assert_eq!(ru_form("21", &f), "банан");
        assert_eq!(ru_form("11", &f), "бананов");
        assert_eq!(ru_form("3", &f), "банана");
        assert_eq!(ru_form("13", &f), "бананов");
        assert_eq!(ru_form("25", &f), "бананов");
        assert_eq!(ru_form("1\u{a0}002", &f), "банана");
        assert_eq!(ru_form("3.3", &f), "банана");
    }

    #[test]
    fn durations() {
        assert_eq!(long_duration(Lang::En, 95), "1 hour 35 minutes");
        assert_eq!(long_duration(Lang::En, 61), "1 hour 1 minute");
        assert_eq!(long_duration(Lang::Ru, 50), "50 минут");
        assert_eq!(long_duration(Lang::Ru, 122), "2 часа 2 минуты");
        assert_eq!(long_duration(Lang::Uz, 60), "1 soat");
    }

    #[test]
    fn every_string_in_every_language() {
        for s in STRINGS {
            assert!(!s.1.is_empty() && !s.2.is_empty() && !s.3.is_empty(), "{}", s.0);
        }
        assert_eq!(tr(Lang::Ru, "stats"), "Статистика");
        assert_eq!(tr(Lang::En, "missing"), "");
    }
}
