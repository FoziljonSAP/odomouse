//! Distance units and number formatting. Mirrors src/core/units.js exactly:
//! '.' for decimals, a no-break space between thousands ("12 345.6"), and
//! JavaScript `toFixed` rounding (ties go up), so the core and the web UI
//! show the same text.

pub const NBSP: char = '\u{a0}';

pub const UNITS: [(&str, &str, f64); 4] = [
    ("m", "Metr", 1.0),
    ("km", "Kilometr", 1000.0),
    ("ft", "Fut", 0.3048),
    ("mi", "Mil", 1609.344),
];

fn meters_per(unit: &str) -> Option<f64> {
    UNITS.iter().find(|u| u.0 == unit).map(|u| u.2)
}

/// Any stored unit key -> one of m / km / ft / mi (older versions saved
/// "auto-metric", "auto-imperial" and "yd").
pub fn normalize_unit(key: &str) -> &'static str {
    if let Some(u) = UNITS.iter().find(|u| u.0 == key) {
        return u.0;
    }
    match key {
        "auto-imperial" | "yd" => "ft",
        _ => "m",
    }
}

/// Fewer decimals as the number grows: 3.42 / 34.2 / 342.
pub fn auto_decimals(value: f64) -> usize {
    let a = value.abs();
    if a == 0.0 {
        0
    } else if a < 10.0 {
        2
    } else if a < 100.0 {
        1
    } else {
        0
    }
}

/// `Math.abs(x).toFixed(d)` for x >= 0: exact decimal expansion, ties rounded up.
pub fn to_fixed(x: f64, d: usize) -> String {
    let x = if x.is_finite() { x.abs() } else { 0.0 };
    // 400 fractional digits print every f64 >= 1e-76 exactly; anything
    // smaller rounds to zero at the few decimals we use anyway.
    let exact = format!("{:.400}", x);
    let (int_part, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut digits: Vec<u8> = int_part.bytes().chain(frac.bytes().take(d)).collect();
    let round_up = frac.as_bytes().get(d).map_or(false, |&c| c >= b'5');
    if round_up {
        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, b'1');
                break;
            }
            i -= 1;
            if digits[i] == b'9' {
                digits[i] = b'0';
            } else {
                digits[i] += 1;
                break;
            }
        }
    }
    let int_len = digits.len() - d;
    let mut out = String::from_utf8(digits[..int_len].to_vec()).unwrap();
    if d > 0 {
        out.push('.');
        out.push_str(std::str::from_utf8(&digits[int_len..]).unwrap());
    }
    out
}

/// Same as units.js formatNumber; `decimals = None` picks auto_decimals.
pub fn format_number(value: f64, decimals: Option<usize>) -> String {
    let value = if value.is_finite() { value } else { 0.0 };
    let d = decimals.unwrap_or_else(|| auto_decimals(value));
    let fixed = to_fixed(value, d);
    let (int_part, frac) = match fixed.split_once('.') {
        Some((i, f)) => (i.to_string(), Some(f.to_string())),
        None => (fixed.clone(), None),
    };
    let mut grouped = String::new();
    let len = int_part.len();
    for (i, c) in int_part.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            grouped.push(NBSP);
        }
        grouped.push(c);
    }
    let is_zero = fixed.bytes().all(|c| c == b'0' || c == b'.');
    let sign = if value < 0.0 && !is_zero { "-" } else { "" };
    match frac {
        Some(f) => format!("{}{}.{}", sign, grouped, f),
        None => format!("{}{}", sign, grouped),
    }
}

/// Math.round as in JavaScript (x.5 rounds towards +infinity).
pub fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

pub fn format_count(n: f64) -> String {
    format_number(js_round(if n.is_finite() { n } else { 0.0 }), Some(0))
}

pub struct Distance {
    pub value: f64,
    pub unit: &'static str,
    pub number: String,
    pub text: String,
}

pub fn format_distance(meters: f64, unit_key: &str, decimals: Option<usize>) -> Distance {
    let m = if meters.is_finite() { meters.max(0.0) } else { 0.0 };
    let unit = normalize_unit(unit_key);
    let value = m / meters_per(unit).unwrap();
    let number = format_number(value, decimals);
    let text = format!("{}{}{}", number, NBSP, unit);
    Distance { value, unit, number, text }
}

/// 205 -> "3 soat 25 daq", 45 -> "45 daq", 0 -> "0 daq".
pub fn format_minutes(total: f64) -> String {
    format_minutes_in(total, crate::i18n::Lang::Uz)
}

/// 205 -> "3 soat 25 daq" / "3 h 25 min" / "3 ч 25 мин".
pub fn format_minutes_in(total: f64, lang: crate::i18n::Lang) -> String {
    let mins = js_round(if total.is_finite() { total } else { 0.0 }).max(0.0) as i64;
    let (h, m) = (mins / 60, mins % 60);
    let (hu, mu) = crate::i18n::minute_units(lang);
    if h == 0 {
        format!("{}{}{}", m, NBSP, mu)
    } else if m == 0 {
        format!("{}{}{}", h, NBSP, hu)
    } else {
        format!("{}{}{} {}{}{}", h, NBSP, hu, m, NBSP, mu)
    }
}

pub fn format_percent(ratio: f64, decimals: Option<usize>) -> String {
    format_number(ratio * 100.0, Some(decimals.unwrap_or(1))) + "%"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.replace(' ', "\u{a0}")
    }

    #[test]
    fn numbers_match_javascript() {
        assert_eq!(format_number(12345.67, None), s("12 346"));
        assert_eq!(format_number(3.4249, None), "3.42");
        assert_eq!(format_number(34.25, None), "34.3"); // 34.25 is exact in binary: tie goes up
        assert_eq!(format_number(1.005, Some(2)), "1.00"); // 1.005 is really 1.00499999...
        assert_eq!(format_number(2.5, Some(0)), "3");
        assert_eq!(format_number(999.96, Some(1)), s("1 000.0"));
        assert_eq!(format_number(-0.001, Some(2)), "0.00");
        assert_eq!(format_number(-1234.5, Some(1)), s("-1 234.5"));
        assert_eq!(format_number(f64::NAN, None), "0");
        assert_eq!(format_number(0.0, None), "0");
        assert_eq!(format_count(1234567.4), s("1 234 567"));
    }

    #[test]
    fn units_and_legacy_keys() {
        assert_eq!(format_distance(1234.0, "m", None).text, s("1 234 m"));
        assert_eq!(format_distance(1234.0, "km", None).text, s("1.23 km"));
        assert_eq!(format_distance(1609.344, "mi", None).text, s("1.00 mi"));
        assert_eq!(format_distance(100.0, "ft", None).number, "328");
        assert_eq!(normalize_unit("auto-metric"), "m");
        assert_eq!(normalize_unit("auto-imperial"), "ft");
        assert_eq!(normalize_unit("yd"), "ft");
        assert_eq!(normalize_unit("parsec"), "m");
        assert_eq!(format_distance(-5.0, "m", None).text, s("0 m"));
    }

    #[test]
    fn minutes_and_percent() {
        assert_eq!(format_minutes(205.0), "3\u{a0}soat 25\u{a0}daq");
        assert_eq!(format_minutes(45.0), s("45 daq"));
        assert_eq!(format_minutes(120.0), s("2 soat"));
        assert_eq!(format_minutes(0.0), s("0 daq"));
        assert_eq!(format_percent(0.1234, None), "12.3%");
    }
}
