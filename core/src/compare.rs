//! "Compared with yesterday / the previous period".
//! Today is compared with yesterday up to the same time of day; tiny
//! baselines are skipped so the UI never shows "+900%".

use crate::day::{aggregate, Day};
use crate::json::Json;
use crate::i18n::Lang;
use crate::time::Moment;

pub const METRICS: [&str; 5] = ["mouseMeters", "scrollMeters", "keystrokes", "clicks", "activeMinutes"];
const MIN_BASE: [f64; 5] = [5.0, 1.0, 100.0, 20.0, 10.0];

pub fn totals_of(d: &Day) -> [f64; 5] {
    [d.mouse_mm / 1000.0, d.scroll_mm / 1000.0, d.keystrokes, d.clicks.total(), d.active_minutes]
}

/// Totals from midnight until `hour` + `fraction` of that hour.
pub fn totals_until(d: &Day, hour: usize, fraction: f64) -> [f64; 5] {
    let up_to = |a: &[f64; 24]| a[..hour].iter().sum::<f64>() + a[hour] * fraction;
    [
        up_to(&d.by_hour.mm) / 1000.0,
        up_to(&d.by_hour.scroll) / 1000.0,
        up_to(&d.by_hour.keys),
        up_to(&d.by_hour.clicks),
        up_to(&d.hourly),
    ]
}

fn deltas(cur: [f64; 5], prev: [f64; 5]) -> Json {
    let mut out = Json::obj();
    for i in 0..5 {
        let v = if prev[i] >= MIN_BASE[i] { Json::Num((cur[i] - prev[i]) / prev[i]) } else { Json::Null };
        out.insert(METRICS[i], v);
    }
    out
}

pub fn compare_today(today: &Day, yesterday: Option<&Day>, now: Moment, lang: Lang) -> Json {
    let Some(y) = yesterday.filter(|y| y.hourly_detail) else { return Json::Null };
    let fraction = (now.seconds_of_day() % 3600) as f64 / 3600.0;
    let prev = totals_until(y, now.hour(), fraction);
    Json::obj()
        .set("label", lang.pick("Kechagi shu vaqtga nisbatan", "Compared with yesterday at this time", "По сравнению со вчера в это же время"))
        .set("short", lang.pick("kecha shu vaqtga", "vs. yesterday", "ко вчера"))
        .set("deltas", deltas(totals_of(today), prev))
}

/// Range vs the same-length range right before it.
pub fn compare_periods(cur: &[&Day], prev: &[&Day], days: i64, lang: Lang) -> Json {
    if prev.is_empty() {
        return Json::Null;
    }
    let p = totals_of(&aggregate(prev.iter().copied()));
    if (0..5).all(|i| !(p[i] >= MIN_BASE[i])) {
        return Json::Null;
    }
    Json::obj()
        .set("label", match lang {
            Lang::Uz => format!("Oldingi {} kunga nisbatan", days),
            Lang::En => format!("Compared with the previous {} days", days),
            Lang::Ru => format!("По сравнению с предыдущими {} {}", days, if days % 10 == 1 && days % 100 != 11 { "днём" } else { "днями" }),
        })
        .set("short", match lang {
            Lang::Uz => format!("oldingi {} kun", days),
            Lang::En => format!("vs. previous {} days", days),
            Lang::Ru => format!("к пред. {} {}", days, if days % 10 == 1 && days % 100 != 11 { "дню" } else { "дням" }),
        })
        .set("deltas", deltas(totals_of(&aggregate(cur.iter().copied())), p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::tests::at;

    #[test]
    fn today_vs_yesterday_by_this_time() {
        let mut y = Day::empty("2026-10-03");
        y.by_hour.keys[9] = 1000.0;
        y.by_hour.keys[10] = 1000.0;
        y.by_hour.keys[20] = 5000.0; // after "now": must not count
        let mut t = Day::empty("2026-10-04");
        t.keystrokes = 2250.0;
        let c = compare_today(&t, Some(&y), at("2026-10-04", 10, 30, 0), Lang::Uz);
        let d = c.get("deltas").unwrap();
        assert!((d.get("keystrokes").unwrap().as_f64().unwrap() - 0.5).abs() < 1e-9);
        assert_eq!(d.get("mouseMeters"), Some(&Json::Null));
        y.hourly_detail = false;
        assert_eq!(compare_today(&t, Some(&y), at("2026-10-04", 10, 30, 0), Lang::Uz), Json::Null);
        assert_eq!(compare_today(&t, None, at("2026-10-04", 10, 30, 0), Lang::Uz), Json::Null);
    }

    #[test]
    fn periods_skip_tiny_baselines() {
        let mut a = Day::empty("2026-09-27");
        a.keystrokes = 50.0;
        let b = Day::empty("2026-10-04");
        assert_eq!(compare_periods(&[&b], &[&a], 7, Lang::Uz), Json::Null);
        a.keystrokes = 200.0;
        let mut b = b;
        b.keystrokes = 300.0;
        let c = compare_periods(&[&b], &[&a], 7, Lang::Uz);
        assert_eq!(c.get("short").and_then(Json::as_str), Some("oldingi 7 kun"));
        assert_eq!(c.get("deltas").unwrap().get("keystrokes").and_then(Json::as_f64), Some(0.5));
        assert_eq!(compare_periods(&[&b], &[], 7, Lang::Uz), Json::Null);
        let en = compare_periods(&[&b], &[&a], 7, Lang::En);
        assert_eq!(en.get("short").and_then(Json::as_str), Some("vs. previous 7 days"));
        let ru = compare_periods(&[&b], &[&a], 30, Lang::Ru);
        assert_eq!(ru.get("short").and_then(Json::as_str), Some("к пред. 30 дням"));
    }
}
