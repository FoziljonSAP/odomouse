//! One calendar day of stats: the shape saved in history.json (v3), migration
//! from older formats, and aggregation for ranges.

use crate::json::Json;
use crate::keyboard;
use std::collections::BTreeMap;

pub const GRID_W: usize = 32;
pub const GRID_H: usize = 20;
pub const DAY_VERSION: u32 = 3;
pub const MAX_APPS_PER_DAY: usize = 200;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Clicks {
    pub left: f64,
    pub right: f64,
    pub middle: f64,
    pub other: f64,
}

impl Clicks {
    pub fn total(&self) -> f64 {
        self.left + self.right + self.middle + self.other
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Typing {
    pub chars: f64,
    pub ms: f64,
    pub backspaces: f64,
    pub best_wpm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ByHour {
    pub mm: [f64; 24],
    pub scroll: [f64; 24],
    pub keys: [f64; 24],
    pub clicks: [f64; 24],
}

/// Insertion-ordered string -> count map (a JavaScript object's key order,
/// which decides the order of ties when sorting).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Counts(Vec<(String, f64)>);

impl Counts {
    pub fn get(&self, key: &str) -> Option<&f64> {
        self.0.iter().find(|e| e.0 == key).map(|e| &e.1)
    }
    pub fn insert(&mut self, key: String, value: f64) {
        match self.0.iter_mut().find(|e| e.0 == key) {
            Some(e) => e.1 = value,
            None => self.0.push((key, value)),
        }
    }
    pub fn add(&mut self, key: &str, value: f64) {
        match self.0.iter_mut().find(|e| e.0 == key) {
            Some(e) => e.1 += value,
            None => self.0.push((key.to_string(), value)),
        }
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &f64)> {
        self.0.iter().map(|e| (&e.0, &e.1))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppStats {
    pub keys: f64,
    pub clicks: f64,
    pub mm: f64,
    pub scroll: f64,
    pub minutes: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Day {
    pub date: String,
    pub keystrokes: f64,
    /// VC keycode -> count
    pub keys: BTreeMap<u32, f64>,
    pub clicks: Clicks,
    pub mouse_mm: f64,
    pub scroll_mm: f64,
    pub active_minutes: f64,
    /// active minutes in each local hour
    pub hourly: [f64; 24],
    pub typing: Typing,
    /// seconds the cursor rested in each cell
    pub grid: Vec<f64>,
    pub by_hour: ByHour,
    /// false for days recorded before by_hour existed
    pub hourly_detail: bool,
    /// "mask:keycode" -> count
    pub shortcuts: Counts,
    pub longest_session_min: f64,
    /// insertion-ordered, opt-in
    pub apps: Vec<(String, AppStats)>,
    pub migrated_from: Option<u32>,
    /// only set on aggregated ranges
    pub days: Option<f64>,
    pub active_days: Option<f64>,
}

impl Day {
    pub fn empty(date: &str) -> Day {
        Day {
            date: date.to_string(),
            keystrokes: 0.0,
            keys: BTreeMap::new(),
            clicks: Clicks::default(),
            mouse_mm: 0.0,
            scroll_mm: 0.0,
            active_minutes: 0.0,
            hourly: [0.0; 24],
            typing: Typing::default(),
            grid: vec![0.0; GRID_W * GRID_H],
            by_hour: ByHour { mm: [0.0; 24], scroll: [0.0; 24], keys: [0.0; 24], clicks: [0.0; 24] },
            hourly_detail: true,
            shortcuts: Counts::default(),
            longest_session_min: 0.0,
            apps: Vec::new(),
            migrated_from: None,
            days: None,
            active_days: None,
        }
    }

    pub fn app_mut(&mut self, name: &str) -> Option<&mut AppStats> {
        if let Some(i) = self.apps.iter().position(|a| a.0 == name) {
            return Some(&mut self.apps[i].1);
        }
        if self.apps.len() >= MAX_APPS_PER_DAY {
            return None;
        }
        self.apps.push((name.to_string(), AppStats::default()));
        self.apps.last_mut().map(|a| &mut a.1)
    }

    pub fn to_json(&self) -> Json {
        let arr = |a: &[f64]| Json::Arr(a.iter().map(|&n| Json::Num(n)).collect());
        let mut keys = Json::obj();
        for (k, v) in &self.keys {
            keys.insert(&k.to_string(), *v);
        }
        let mut shortcuts = Json::obj();
        for (k, v) in self.shortcuts.iter() {
            shortcuts.insert(k, *v);
        }
        let mut apps = Json::obj();
        for (name, a) in &self.apps {
            apps.insert(
                name,
                Json::obj()
                    .set("keys", a.keys)
                    .set("clicks", a.clicks)
                    .set("mm", a.mm)
                    .set("scroll", a.scroll)
                    .set("minutes", a.minutes),
            );
        }
        Json::obj()
            .set("v", DAY_VERSION)
            .set("date", self.date.as_str())
            .set("keystrokes", self.keystrokes)
            .set("keys", keys)
            .set(
                "clicks",
                Json::obj()
                    .set("left", self.clicks.left)
                    .set("right", self.clicks.right)
                    .set("middle", self.clicks.middle)
                    .set("other", self.clicks.other),
            )
            .set("mouseMm", self.mouse_mm)
            .set("scrollMm", self.scroll_mm)
            .set("activeMinutes", self.active_minutes)
            .set("hourly", arr(&self.hourly))
            .set(
                "typing",
                Json::obj()
                    .set("chars", self.typing.chars)
                    .set("ms", self.typing.ms)
                    .set("backspaces", self.typing.backspaces)
                    .set("bestWpm", self.typing.best_wpm),
            )
            .set("grid", arr(&self.grid))
            .set(
                "byHour",
                Json::obj()
                    .set("mm", arr(&self.by_hour.mm))
                    .set("scroll", arr(&self.by_hour.scroll))
                    .set("keys", arr(&self.by_hour.keys))
                    .set("clicks", arr(&self.by_hour.clicks)),
            )
            .set("hourlyDetail", self.hourly_detail)
            .set("shortcuts", shortcuts)
            .set("longestSessionMin", self.longest_session_min)
            .set("apps", apps)
    }
}

/// JavaScript `Number(v)` kept only when finite and > 0.
pub fn num(v: Option<&Json>) -> f64 {
    let n = match v {
        Some(Json::Num(n)) => *n,
        Some(Json::Str(s)) => s.trim().parse::<f64>().unwrap_or(0.0),
        Some(Json::Bool(true)) => 1.0,
        _ => 0.0,
    };
    if n.is_finite() && n > 0.0 { n } else { 0.0 }
}

fn arr_n<const N: usize>(v: Option<&Json>) -> Option<[f64; N]> {
    let a = v?.as_arr()?;
    if a.len() != N {
        return None;
    }
    let mut out = [0.0; N];
    for (i, x) in a.iter().enumerate() {
        out[i] = num(Some(x));
    }
    Some(out)
}

pub fn clean_app_name(name: &str) -> Option<String> {
    let n: String = name.chars().filter(|c| (*c as u32) > 0x1f).collect();
    let n: String = n.trim().chars().take(80).collect();
    if n.is_empty() { None } else { Some(n) }
}

/// Any stored day (v1 prototype, v2, v3) -> a complete v3 day.
pub fn normalize_day(raw: &Json, fallback_date: &str) -> Day {
    let date = raw.get("date").and_then(Json::as_str).filter(|s| !s.is_empty()).unwrap_or(fallback_date);
    let mut day = Day::empty(date);
    if raw.as_obj().is_none() {
        return day;
    }
    let g = |k: &str| raw.get(k);
    if num(g("v")) >= 2.0 {
        day.keystrokes = num(g("keystrokes"));
        if let Some(o) = g("keys").and_then(Json::as_obj) {
            for (k, v) in o {
                if let (Ok(code), n) = (k.parse::<u32>(), num(Some(v))) {
                    if n > 0.0 {
                        day.keys.insert(code, n);
                    }
                }
            }
        }
        let c = g("clicks");
        let cg = |k: &str| num(c.and_then(|c| c.get(k)));
        day.clicks = Clicks { left: cg("left"), right: cg("right"), middle: cg("middle"), other: cg("other") };
        day.mouse_mm = num(g("mouseMm"));
        day.scroll_mm = num(g("scrollMm"));
        day.active_minutes = num(g("activeMinutes"));
        if let Some(h) = arr_n::<24>(g("hourly")) {
            day.hourly = h;
        }
        let t = g("typing");
        let tg = |k: &str| num(t.and_then(|t| t.get(k)));
        day.typing = Typing { chars: tg("chars"), ms: tg("ms"), backspaces: tg("backspaces"), best_wpm: tg("bestWpm") };
        if let Some(a) = g("grid").and_then(Json::as_arr) {
            if a.len() == GRID_W * GRID_H {
                day.grid = a.iter().map(|x| num(Some(x))).collect();
            }
        }
        let bh = g("byHour");
        let bg = |k: &str| arr_n::<24>(bh.and_then(|b| b.get(k)));
        match (bg("mm"), bg("scroll"), bg("keys"), bg("clicks")) {
            (Some(mm), Some(scroll), Some(keys), Some(clicks)) => {
                day.by_hour = ByHour { mm, scroll, keys, clicks };
                day.hourly_detail = g("hourlyDetail") != Some(&Json::Bool(false));
            }
            _ => day.hourly_detail = false,
        }
        if let Some(o) = g("shortcuts").and_then(Json::as_obj) {
            for (k, v) in o {
                let n = num(Some(v));
                if n > 0.0 {
                    day.shortcuts.insert(k.clone(), n);
                }
            }
        }
        day.longest_session_min = num(g("longestSessionMin"));
        if let Some(o) = g("apps").and_then(Json::as_obj) {
            for (name, a) in o.iter().take(MAX_APPS_PER_DAY) {
                let Some(n) = clean_app_name(name) else { continue };
                if a.as_obj().is_none() || day.apps.iter().any(|x| x.0 == n) {
                    continue;
                }
                let ag = |k: &str| num(a.get(k));
                day.apps.push((
                    n,
                    AppStats { keys: ag("keys"), clicks: ag("clicks"), mm: ag("mm"), scroll: ag("scroll"), minutes: ag("minutes") },
                ));
            }
        }
        return day;
    }

    // v1 prototype: { date, keystrokes, clicks: n, mouseMeters, topLetters: [{letter, count}] }
    day.keystrokes = num(g("keystrokes"));
    day.clicks.left = num(g("clicks"));
    day.mouse_mm = num(g("mouseMeters")) * 1000.0;
    if let Some(list) = g("topLetters").and_then(Json::as_arr) {
        for item in list {
            let letter = item.get("letter").and_then(Json::as_str).and_then(|s| s.chars().next());
            if let Some(code) = letter.and_then(keyboard::code_of_letter) {
                day.keys.insert(code, num(item.get("count")));
            }
        }
    }
    day.hourly_detail = false;
    day.migrated_from = Some(1);
    day
}

/// Words per minute (5 chars = 1 word); None until 30 s of typing.
pub fn wpm(t: &Typing) -> Option<f64> {
    if t.ms < 30000.0 || t.chars == 0.0 {
        return None;
    }
    Some((t.chars / 5.0) / (t.ms / 60000.0))
}

pub fn backspace_ratio(t: &Typing) -> Option<f64> {
    if t.chars < 50.0 {
        return None;
    }
    Some(t.backspaces / t.chars)
}

pub fn aggregate<'a>(days: impl IntoIterator<Item = &'a Day>) -> Day {
    let mut total = Day::empty("");
    let (mut n, mut active) = (0.0, 0.0);
    for d in days {
        n += 1.0;
        if d.active_minutes > 0.0 || d.keystrokes > 0.0 || d.mouse_mm > 0.0 {
            active += 1.0;
        }
        total.keystrokes += d.keystrokes;
        for (k, v) in &d.keys {
            *total.keys.entry(*k).or_insert(0.0) += v;
        }
        total.clicks.left += d.clicks.left;
        total.clicks.right += d.clicks.right;
        total.clicks.middle += d.clicks.middle;
        total.clicks.other += d.clicks.other;
        total.mouse_mm += d.mouse_mm;
        total.scroll_mm += d.scroll_mm;
        total.active_minutes += d.active_minutes;
        for h in 0..24 {
            total.hourly[h] += d.hourly[h];
        }
        total.typing.chars += d.typing.chars;
        total.typing.ms += d.typing.ms;
        total.typing.backspaces += d.typing.backspaces;
        total.typing.best_wpm = total.typing.best_wpm.max(d.typing.best_wpm);
        for (i, v) in d.grid.iter().enumerate().take(total.grid.len()) {
            total.grid[i] += v;
        }
        for (k, v) in d.shortcuts.iter() {
            total.shortcuts.add(k, *v);
        }
        total.longest_session_min = total.longest_session_min.max(d.longest_session_min);
        for (name, a) in &d.apps {
            if let Some(i) = total.apps.iter().position(|x| &x.0 == name) {
                let t = &mut total.apps[i].1;
                t.keys += a.keys;
                t.clicks += a.clicks;
                t.mm += a.mm;
                t.scroll += a.scroll;
                t.minutes += a.minutes;
            } else {
                total.apps.push((name.clone(), a.clone()));
            }
        }
    }
    total.days = Some(n);
    total.active_days = Some(active);
    total
}

/// Everything the popup / dashboard / Wrapped card shows (day.js summarize).
pub fn summarize(day: &Day) -> Json {
    let mut top_keys: Vec<(u32, f64)> = day.keys.iter().map(|(k, v)| (*k, *v)).collect();
    top_keys.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let hourly_max = day.hourly.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut shortcuts: Vec<(&String, f64)> = day.shortcuts.iter().map(|(k, v)| (k, *v)).collect();
    shortcuts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut apps: Vec<&(String, AppStats)> = day.apps.iter().collect();
    apps.sort_by(|a, b| {
        (b.1.minutes, b.1.keys).partial_cmp(&(a.1.minutes, a.1.keys)).unwrap_or(std::cmp::Ordering::Equal)
    });
    let peak = if hourly_max > 0.0 { day.hourly.iter().position(|&h| h == hourly_max) } else { None };
    let opt_num = |v: Option<f64>| v.map(Json::Num).unwrap_or(Json::Null);

    Json::obj()
        .set("date", if day.date.is_empty() { Json::Null } else { Json::from(day.date.as_str()) })
        .set("mouseMeters", day.mouse_mm / 1000.0)
        .set("scrollMeters", day.scroll_mm / 1000.0)
        .set("keystrokes", day.keystrokes)
        .set(
            "clicks",
            Json::obj()
                .set("left", day.clicks.left)
                .set("right", day.clicks.right)
                .set("middle", day.clicks.middle)
                .set("other", day.clicks.other)
                .set("total", day.clicks.total()),
        )
        .set("activeMinutes", day.active_minutes)
        .set("wpm", opt_num(wpm(&day.typing)))
        .set("bestWpm", opt_num(Some(day.typing.best_wpm).filter(|w| *w > 0.0)))
        .set("backspaceRatio", opt_num(backspace_ratio(&day.typing)))
        .set("charsTyped", day.typing.chars)
        .set(
            "topKeys",
            Json::Arr(top_keys.iter().take(10).map(|(c, n)| Json::obj().set("code", *c).set("count", *n)).collect()),
        )
        .set(
            "topLetters",
            Json::Arr(
                top_keys
                    .iter()
                    .filter_map(|(c, n)| keyboard::letter(*c).map(|l| (l, *n)))
                    .take(5)
                    .map(|(l, n)| Json::obj().set("letter", l.to_string()).set("count", n))
                    .collect(),
            ),
        )
        .set("peakHour", peak.map(Json::from).unwrap_or(Json::Null))
        .set(
            "topShortcuts",
            Json::Arr(shortcuts.iter().take(10).map(|(id, n)| Json::obj().set("id", id.as_str()).set("count", *n)).collect()),
        )
        .set("shortcutTotal", shortcuts.iter().map(|s| s.1).sum::<f64>())
        .set("distinctShortcuts", shortcuts.len())
        .set("longestSessionMin", day.longest_session_min)
        .set(
            "apps",
            Json::Arr(
                apps.iter()
                    .take(12)
                    .map(|(name, a)| {
                        Json::obj()
                            .set("name", name.as_str())
                            .set("minutes", a.minutes)
                            .set("keys", a.keys)
                            .set("clicks", a.clicks)
                            .set("meters", a.mm / 1000.0)
                            .set("scrollMeters", a.scroll / 1000.0)
                    })
                    .collect(),
            ),
        )
        .set("days", opt_num(day.days))
        .set("activeDays", opt_num(day.active_days))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse;
    use crate::keyboard::k;

    #[test]
    fn v1_prototype_migrates() {
        let raw = parse(r#"{"date":"2026-09-01","keystrokes":120,"clicks":7,"mouseMeters":12.5,
            "topLetters":[{"letter":"e","count":30},{"letter":"T","count":20}]}"#).unwrap();
        let d = normalize_day(&raw, "x");
        assert_eq!(d.date, "2026-09-01");
        assert_eq!(d.clicks.left, 7.0);
        assert_eq!(d.mouse_mm, 12500.0);
        assert_eq!(d.keys.get(&k::E), Some(&30.0));
        assert_eq!(d.keys.get(&k::T), Some(&20.0));
        assert!(!d.hourly_detail);
        assert_eq!(d.migrated_from, Some(1));
    }

    #[test]
    fn v2_day_has_no_hourly_detail_and_bad_values_are_dropped() {
        let raw = parse(r#"{"v":2,"date":"2026-09-02","keystrokes":"50","keys":{"18":3,"19":-1,"x":5},
            "clicks":{"left":1,"right":null},"mouseMm":1e400,"hourly":[1,2],
            "apps":{"Safari\u0007":{"keys":2},"  ":{"keys":1}}}"#).unwrap();
        let d = normalize_day(&raw, "2026-09-02");
        assert_eq!(d.keystrokes, 50.0);
        assert_eq!(d.keys.len(), 1);
        assert_eq!(d.clicks.right, 0.0);
        assert_eq!(d.mouse_mm, 0.0);
        assert_eq!(d.hourly, [0.0; 24]);
        assert!(!d.hourly_detail);
        assert_eq!(d.apps.len(), 1);
        assert_eq!(d.apps[0].0, "Safari");
    }

    #[test]
    fn v3_round_trips_through_json() {
        let mut d = Day::empty("2026-10-05");
        d.keystrokes = 3.0;
        d.keys.insert(k::A, 2.0);
        d.keys.insert(k::SPACE, 1.0);
        d.by_hour.mm[9] = 1234.5;
        d.shortcuts.insert("8:46".into(), 4.0);
        d.app_mut("Code").unwrap().keys = 3.0;
        d.grid[5] = 7.0;
        let back = normalize_day(&parse(&d.to_json().to_string()).unwrap(), "x");
        assert_eq!(back, d);
    }

    #[test]
    fn aggregate_and_summarize() {
        let mut a = Day::empty("2026-10-04");
        a.keystrokes = 100.0;
        a.keys.insert(k::E, 40.0);
        a.keys.insert(k::SPACE, 50.0);
        a.keys.insert(k::T, 10.0);
        a.typing = Typing { chars: 300.0, ms: 60000.0, backspaces: 30.0, best_wpm: 70.0 };
        a.hourly[10] = 30.0;
        a.active_minutes = 30.0;
        let mut b = Day::empty("2026-10-05");
        b.keys.insert(k::T, 45.0);
        b.typing.best_wpm = 80.0;
        b.clicks.left = 2.0;
        b.longest_session_min = 42.0;
        let empty = Day::empty("2026-10-03");
        let total = aggregate([&a, &b, &empty]);
        assert_eq!(total.days, Some(3.0));
        assert_eq!(total.active_days, Some(1.0));
        let s = summarize(&total);
        assert_eq!(s.get("wpm").and_then(Json::as_f64), Some(60.0));
        assert_eq!(s.get("bestWpm").and_then(Json::as_f64), Some(80.0));
        assert_eq!(s.get("backspaceRatio").and_then(Json::as_f64), Some(0.1));
        assert_eq!(s.get("peakHour").and_then(Json::as_f64), Some(10.0));
        assert_eq!(s.get("longestSessionMin").and_then(Json::as_f64), Some(42.0));
        let letters = s.get("topLetters").unwrap().to_string();
        assert_eq!(letters, r#"[{"letter":"T","count":55},{"letter":"E","count":40}]"#);
        assert_eq!(s.get("topKeys").unwrap().as_arr().unwrap()[0].get("code").and_then(Json::as_f64), Some(k::T as f64));
        let empty_summary = summarize(&Day::empty("2026-10-05"));
        assert_eq!(empty_summary.get("wpm"), Some(&Json::Null));
        assert_eq!(empty_summary.get("peakHour"), Some(&Json::Null));
        assert_eq!(empty_summary.get("days"), Some(&Json::Null));
    }
}
