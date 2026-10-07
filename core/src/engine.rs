//! The whole app minus the OS: stores, tracker, displays, break coach and
//! every payload the UI shows. The native shells call this (directly from
//! Rust, or through ffi.rs).

use crate::breaks::{break_message, BreakCoach};
use crate::compare::{compare_periods, compare_today};
use crate::day::{aggregate, summarize, wpm, Day, GRID_H, GRID_W};
use crate::displays::{self, build_model, infos_from_json, Display, DisplayInfo};
use crate::fun::{distance_comparison_in, text_comparison_in, DISTANCE_REFS, TEXT_REFS};
use crate::i18n::{self, Lang};
use crate::json::{parse, Json};
use crate::keyboard;
use crate::store::{HistoryStore, Settings, SettingsStore};
use crate::time::{add_days, date_range, Moment};
use crate::tracker::Tracker;
use crate::units::{format_count, format_distance, format_minutes_in, to_fixed};
use std::path::Path;

/// history.json is written at most this often (only if something changed).
pub const SAVE_MS: i64 = 30000;

pub struct Engine {
    pub history: HistoryStore,
    pub settings: SettingsStore,
    pub tracker: Tracker,
    infos: Vec<DisplayInfo>,
    displays: Vec<Display>,
    hooks_running: bool,
    /// The OS language (setSystemLanguage); used while the setting is "auto".
    system_lang: Lang,
    coach: BreakCoach,
    last_save_ms: i64,
}

fn range_back(range: &str) -> Option<Option<i64>> {
    match range {
        "today" => Some(Some(0)),
        "7d" => Some(Some(6)),
        "30d" => Some(Some(29)),
        "all" => Some(None),
        _ => None,
    }
}

/// Folder names the app used before it was called Odomouse; each sits next
/// to the new folder (macOS/Windows "Mishka Tracker", Linux "mishka-tracker").
const OLD_DIR_NAMES: [&str; 2] = ["Mishka Tracker", "mishka-tracker"];
const DATA_FILES: [&str; 2] = ["history.json", "settings.json"];

/// First start under the new name: copy the statistics over from the old
/// folder (copied, not moved, so the old folder stays as a backup).
fn migrate_old_data(data_dir: &Path) {
    if DATA_FILES.iter().any(|f| data_dir.join(f).exists()) {
        return;
    }
    let Some(parent) = data_dir.parent() else { return };
    for name in OLD_DIR_NAMES {
        let old = parent.join(name);
        if old == data_dir || !old.join("history.json").is_file() {
            continue;
        }
        if std::fs::create_dir_all(data_dir).is_err() {
            return;
        }
        for f in DATA_FILES {
            let _ = std::fs::copy(old.join(f), data_dir.join(f));
        }
        // the old versions left "launch at login" off by default, so the
        // counter was gone after every restart: start with it on
        let file = data_dir.join("settings.json");
        if let Some(mut j) = std::fs::read_to_string(&file).ok().and_then(|t| parse(&t).ok()) {
            if j.as_obj().is_some() {
                j.insert("launchAtLogin", true);
                let _ = std::fs::write(&file, j.to_string());
            }
        }
        return;
    }
}

impl Engine {
    pub fn open(data_dir: &Path, now: Moment) -> Engine {
        migrate_old_data(data_dir);
        let history = HistoryStore::open(data_dir.join("history.json"));
        let settings = SettingsStore::open(data_dir.join("settings.json"));
        let resume = history.get_day(&now.day_key()).cloned();
        let tracker = Tracker::new(now, resume);
        let mut e = Engine {
            history,
            settings,
            tracker,
            infos: Vec::new(),
            displays: Vec::new(),
            hooks_running: false,
            system_lang: Lang::Uz,
            coach: BreakCoach::new(),
            last_save_ms: now.ms,
        };
        let today = e.tracker.day.clone();
        e.history.put_day(&today);
        e
    }

    pub fn settings(&self) -> &Settings {
        &self.settings.data
    }

    // ------------------------------------------------------------ inputs

    pub fn mouse_move(&mut self, t: Moment, x: f64, y: f64) {
        self.tracker.mouse_move(t, x, y);
        self.after_event();
    }

    pub fn mouse_down(&mut self, t: Moment, button: u32) {
        self.tracker.mouse_down(t, button);
        self.after_event();
    }

    pub fn wheel(&mut self, t: Moment, x: f64, y: f64, points: f64) {
        self.tracker.wheel(t, x, y, points);
        self.after_event();
    }

    pub fn key_down(&mut self, t: Moment, code: u32, mods: u32) {
        self.tracker.key_down(t, code, mods);
        self.after_event();
    }

    pub fn key_up(&mut self, t: Moment, code: u32) {
        self.tracker.key_up(t, code);
    }

    /// Displays as the shell sees them (see displays::infos_from_json).
    pub fn set_displays(&mut self, infos: Vec<DisplayInfo>) {
        self.infos = infos;
        self.rebuild_displays();
    }

    pub fn set_displays_json(&mut self, json: &str) {
        if let Ok(j) = parse(json) {
            self.set_displays(infos_from_json(&j));
        }
    }

    pub fn set_hooks_running(&mut self, running: bool) {
        if running && !self.hooks_running {
            self.tracker.reset_input_state();
        }
        self.hooks_running = running;
    }

    /// The language in use: the setting, or the OS language for "auto".
    pub fn lang(&self) -> Lang {
        Lang::parse(&self.settings.data.language).unwrap_or(self.system_lang)
    }

    pub fn set_system_language(&mut self, tag: &str) {
        self.system_lang = Lang::from_system(tag);
    }

    /// Settings as the pages get them, plus the resolved language.
    pub fn settings_json(&self) -> Json {
        self.settings.data.to_json().set("lang", self.lang().code()).set("systemLang", self.system_lang.code())
    }

    pub fn hooks_running(&self) -> bool {
        self.hooks_running
    }

    /// Frontmost app; ignored unless the user turned app stats on.
    pub fn set_app(&mut self, name: Option<&str>) {
        let name = if self.settings.data.track_apps { name } else { None };
        self.tracker.set_app(name);
    }

    fn rebuild_displays(&mut self) {
        self.displays = build_model(&self.infos, &self.settings.data.diagonal_overrides);
        self.tracker.set_displays(self.displays.clone());
    }

    fn after_event(&mut self) {
        let finished = self.tracker.take_finished();
        if !finished.is_empty() {
            for d in &finished {
                self.history.put_day(d);
            }
            self.persist(true);
        }
    }

    // -------------------------------------------------------------- timer

    /// Call once a second. Handles midnight, the break reminder, the cursor
    /// heatmap and autosave. Returns {tray, notify}.
    pub fn tick(&mut self, t: Moment, cursor: Option<(f64, f64)>) -> Json {
        self.tracker.check_rollover(t);
        self.after_event();
        let s = &self.settings.data;
        let session_ms = if self.hooks_running { self.tracker.session_ms(t) } else { 0 };
        let due = self.coach.check(
            t.ms,
            self.tracker.session_start,
            if s.break_reminder { session_ms } else { 0 },
            s.break_minutes,
        );
        if let Some((x, y)) = cursor {
            if self.hooks_running {
                self.tracker.sample_cursor(t, x, y);
            }
        }
        if t.ms - self.last_save_ms >= SAVE_MS {
            self.persist(false);
            self.last_save_ms = t.ms;
        }
        let notify = if due {
            let (title, body) = break_message(session_ms as f64 / 60000.0, self.lang());
            Json::obj().set("title", title).set("body", body)
        } else {
            Json::Null
        };
        Json::obj().set("tray", self.tray_text()).set("notify", notify)
    }

    /// Text next to the menu bar / tray icon ("" = icon only).
    pub fn tray_text(&self) -> String {
        if !self.hooks_running {
            return i18n::tr(self.lang(), "permissionNeeded").to_string();
        }
        let s = &self.settings.data;
        match s.tray_shows.as_str() {
            "icon" => String::new(),
            "keys" => format_count(self.tracker.day.keystrokes),
            _ => format_distance(self.tracker.day.mouse_mm / 1000.0, &s.unit, None).text,
        }
    }

    /// Save today if it changed (or always with `force`).
    pub fn persist(&mut self, force: bool) -> bool {
        if !self.tracker.dirty && !force {
            return true;
        }
        self.history.put_day(&self.tracker.day.clone());
        match self.history.save() {
            Ok(()) => {
                self.tracker.dirty = false;
                true
            }
            Err(_) => false,
        }
    }

    // ----------------------------------------------------------- payloads

    fn comparisons(&self, summary: &Json) -> Json {
        let s = &self.settings.data;
        let meters = summary.get("mouseMeters").and_then(Json::as_f64).unwrap_or(0.0);
        let chars = summary.get("charsTyped").and_then(Json::as_f64).unwrap_or(0.0);
        let opt = |c: Option<crate::fun::Comparison>| c.map(|c| c.to_json()).unwrap_or(Json::Null);
        Json::obj()
            .set("distance", opt(distance_comparison_in(meters, &s.compare_distance, self.lang())))
            .set("text", opt(text_comparison_in(chars, &s.compare_text, self.lang())))
    }

    fn session_info(&self, t: Moment) -> Json {
        let ms = if self.hooks_running { self.tracker.session_ms(t) } else { 0 };
        let break_in = if self.settings.data.break_reminder && ms > 0 { self.coach.minutes_left(t.ms) } else { None };
        Json::obj().set("minutes", ms / 60000).set("breakIn", break_in)
    }

    fn yesterday(&self) -> Option<&Day> {
        self.history.get_day(&add_days(&self.tracker.day.date, -1))
    }

    pub fn live(&self, t: Moment) -> Json {
        let summary = summarize(&self.tracker.day);
        Json::obj()
            .set("comparisons", self.comparisons(&summary))
            .set("summary", summary)
            .set("compare", compare_today(&self.tracker.day, self.yesterday(), t, self.lang()))
            .set("session", self.session_info(t))
            .set("settings", self.settings_json())
            .set("hooksRunning", self.hooks_running)
            .set("estimatedDisplays", self.displays.iter().any(|d| d.source == "estimate"))
    }

    fn range_start(&self, back: Option<i64>) -> String {
        let today = &self.tracker.day.date;
        match back {
            None => self.history.first_date().unwrap_or(today).to_string(),
            Some(b) => add_days(today, -b),
        }
    }

    fn series(&self, from: &str, to: &str) -> Json {
        Json::Arr(
            date_range(from, to)
                .into_iter()
                .map(|date| {
                    let d = self.history.get_day(&date);
                    let f = |g: fn(&Day) -> f64| d.map_or(0.0, g);
                    Json::obj()
                        .set("mouseMeters", f(|d| d.mouse_mm / 1000.0))
                        .set("scrollMeters", f(|d| d.scroll_mm / 1000.0))
                        .set("keystrokes", f(|d| d.keystrokes))
                        .set("clicks", f(|d| d.clicks.total()))
                        .set("activeMinutes", f(|d| d.active_minutes))
                        .set("date", date)
                })
                .map(|j| reorder_date_first(j))
                .collect(),
        )
    }

    pub fn dashboard(&mut self, range: &str, t: Moment) -> Json {
        let (range, back) = match range_back(range) {
            Some(b) => (range, b),
            None => ("7d", Some(6)),
        };
        let today = self.tracker.day.clone();
        self.history.put_day(&today);
        let to = today.date.clone();
        let from = self.range_start(back);
        let days = self.history.get_days(&from, &to);
        let total = aggregate(days.iter().copied());
        let summary = summarize(&total);
        let compare = match back {
            Some(0) => compare_today(&self.tracker.day, self.history.get_day(&add_days(&to, -1)), t, self.lang()),
            None => Json::Null,
            Some(b) => {
                let n = b + 1;
                let prev_to = add_days(&from, -1);
                let prev = self.history.get_days(&add_days(&prev_to, -(n - 1)), &prev_to);
                compare_periods(&days, &prev, n, self.lang())
            }
        };
        let num_arr = |a: &[f64]| Json::Arr(a.iter().map(|&n| Json::Num(n)).collect());
        let mut keys = Json::obj();
        for (k, v) in &total.keys {
            keys.insert(&k.to_string(), *v);
        }
        let aspect = self.displays.first().map_or(16.0 / 10.0, |d| d.bounds.width / d.bounds.height);
        Json::obj()
            .set("range", range)
            .set("from", from.as_str())
            .set("to", to.as_str())
            .set("comparisons", self.comparisons(&summary))
            .set("summary", summary)
            .set("compare", compare)
            .set("session", self.session_info(t))
            .set("series", if back == Some(0) { Json::Arr(vec![]) } else { self.series(&from, &to) })
            .set("hourly", num_arr(&total.hourly))
            .set("keys", keys)
            .set("grid", num_arr(&total.grid))
            .set("gridW", GRID_W)
            .set("gridH", GRID_H)
            .set("gridAspect", aspect)
            .set("settings", self.settings_json())
            .set("displays", displays::public_json_in(&self.displays, self.lang()))
            .set("hooksRunning", self.hooks_running)
    }

    pub fn wrapped(&mut self, period: &str) -> Json {
        let l = self.lang();
        let (period, back, label) = match period {
            "month" => ("month", Some(29), l.pick("Shu oy", "This month", "Этот месяц")),
            "all" => ("all", None, l.pick("Butun vaqt", "All time", "За всё время")),
            _ => ("week", Some(6), l.pick("Shu hafta", "This week", "Эта неделя")),
        };
        let today = self.tracker.day.clone();
        self.history.put_day(&today);
        let to = today.date.clone();
        let from = self.range_start(back);
        let summary = summarize(&aggregate(self.history.get_days(&from, &to)));
        Json::obj()
            .set("period", period)
            .set("label", label)
            .set("from", from.as_str())
            .set("to", to.as_str())
            .set("comparisons", self.comparisons(&summary))
            .set("summary", summary)
            .set("settings", self.settings_json())
    }

    /// Settings page change. Returns {settings, displays}.
    pub fn update_settings(&mut self, partial: &Json) -> Json {
        let _ = self.settings.update(partial);
        self.rebuild_displays();
        if !self.settings.data.track_apps {
            self.tracker.set_app(None);
        }
        Json::obj()
            .set("settings", self.settings_json())
            .set("displays", displays::public_json_in(&self.displays, self.lang()))
    }

    pub fn public_displays(&self) -> Json {
        displays::public_json_in(&self.displays, self.lang())
    }

    pub fn csv(&mut self) -> String {
        self.persist(true);
        let mut out = String::from(
            "date,mouse_m,scroll_m,keystrokes,clicks_left,clicks_right,clicks_middle,clicks_other,active_minutes,chars_typed,typing_minutes,backspaces,avg_wpm,best_wpm\n",
        );
        for d in self.history.days.values() {
            let w = wpm(&d.typing).map(|w| to_fixed(w, 1)).unwrap_or_default();
            let best = if d.typing.best_wpm > 0.0 { format!("{}", d.typing.best_wpm) } else { String::new() };
            let row = [
                d.date.clone(),
                to_fixed(d.mouse_mm / 1000.0, 2),
                to_fixed(d.scroll_mm / 1000.0, 2),
                fmt_num(d.keystrokes),
                fmt_num(d.clicks.left),
                fmt_num(d.clicks.right),
                fmt_num(d.clicks.middle),
                fmt_num(d.clicks.other),
                fmt_num(d.active_minutes),
                fmt_num(d.typing.chars),
                to_fixed(d.typing.ms / 60000.0, 1),
                fmt_num(d.typing.backspaces),
                w,
                best,
            ];
            out.push_str(&row.join(","));
            out.push('\n');
        }
        out
    }

    /// Delete all statistics (the shell asks for confirmation first).
    pub fn reset_data(&mut self) {
        let _ = self.history.clear();
        let date = self.tracker.day.date.clone();
        self.tracker.day = Day::empty(&date);
        self.tracker.dirty = true;
        self.persist(true);
    }

    pub fn clear_apps(&mut self) {
        let _ = self.history.clear_apps();
        self.tracker.day.apps.clear();
        self.persist(true);
    }

    /// Compact, pre-formatted snapshot for widgets (WidgetKit, a Windows
    /// flyout, a Linux panel) that should not re-implement formatting.
    pub fn widget(&self, t: Moment) -> Json {
        let d = &self.tracker.day;
        let s = &self.settings.data;
        let summary = summarize(d);
        let comps = self.comparisons(&summary);
        let ctext = |k: &str| comps.get(k).and_then(|c| c.get("text")).cloned().unwrap_or(Json::Null);
        let compare = compare_today(d, self.yesterday(), t, self.lang());
        let mut deltas = Json::obj();
        if let Some(dl) = compare.get("deltas").and_then(Json::as_obj) {
            for (k, v) in dl {
                let text = v.as_f64().map(|r| {
                    let pct = crate::units::js_round(r * 100.0);
                    if pct > 0.0 { format!("+{}%", pct) } else { format!("{}%", pct) }
                });
                deltas.insert(k, text);
            }
        }
        let wpm_text = wpm(&d.typing).map(|w| format!("{}", crate::units::js_round(w)));
        let letters: Vec<Json> = summary
            .get("topLetters")
            .and_then(Json::as_arr)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|l| {
                let count = l.get("count").and_then(Json::as_f64).unwrap_or(0.0);
                l.set("countText", format_count(count))
            })
            .collect();
        let top_key = d.keys.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(k, _)| keyboard::label_for(*k));
        Json::obj()
            .set("v", 1)
            .set("updatedAt", t.ms)
            .set("date", d.date.as_str())
            .set("hooksRunning", self.hooks_running)
            .set("theme", s.theme.as_str())
            .set("unit", s.unit.as_str())
            .set(
                "distance",
                Json::obj().set("meters", d.mouse_mm / 1000.0).set("text", format_distance(d.mouse_mm / 1000.0, &s.unit, None).text),
            )
            .set(
                "scroll",
                Json::obj().set("meters", d.scroll_mm / 1000.0).set("text", format_distance(d.scroll_mm / 1000.0, &s.unit, None).text),
            )
            .set("keys", Json::obj().set("count", d.keystrokes).set("text", format_count(d.keystrokes)))
            .set("clicks", Json::obj().set("count", d.clicks.total()).set("text", format_count(d.clicks.total())))
            .set("active", Json::obj().set("minutes", d.active_minutes).set("text", format_minutes_in(d.active_minutes, self.lang())))
            .set("wpm", wpm_text)
            .set("topKey", top_key)
            .set("topLetters", Json::Arr(letters))
            .set("distanceComparison", ctext("distance"))
            .set("textComparison", ctext("text"))
            .set("deltas", deltas)
            .set("hourly", Json::Arr(d.hourly.iter().map(|&n| Json::Num(n)).collect()))
            .set("session", self.session_info(t))
    }

    /// Reference lists for the settings page (labels for every option).
    pub fn references(lang: Lang) -> Json {
        let refs = |list: &[crate::fun::Ref]| {
            Json::Arr(list.iter().map(|r| Json::obj().set("key", r.key).set("label", r.label_in(lang)).set("size", r.size)).collect())
        };
        Json::obj().set("distance", refs(&DISTANCE_REFS)).set("text", refs(&TEXT_REFS))
    }

    // ---------------------------------------------------------- dispatch

    /// One entry point for the UI bridge (window.odomouse.* in the web UI).
    /// Methods the shell must handle itself (dialogs, clipboard, windows)
    /// are not here: openDashboard, saveWrapped, copyWrapped,
    /// openPermissions, quit.
    pub fn call(&mut self, method: &str, args: &Json, t: Moment) -> Json {
        let arg0 = args.as_arr().and_then(|a| a.first()).unwrap_or(args);
        let arg_str = arg0.as_str().unwrap_or("");
        match method {
            "getLive" => self.live(t),
            "getDashboard" => self.dashboard(arg_str, t),
            "getWrapped" => self.wrapped(arg_str),
            "getWidget" => self.widget(t),
            "updateSettings" => self.update_settings(arg0),
            "exportCsv" => Json::obj().set("csv", self.csv()).set("date", self.tracker.day.date.as_str()),
            "resetData" => {
                self.reset_data();
                Json::obj().set("ok", true)
            }
            "clearApps" => {
                self.clear_apps();
                Json::obj().set("ok", true)
            }
            "markDashboardSeen" => {
                if !self.settings.data.seen_dashboard {
                    self.update_settings(&Json::obj().set("seenDashboard", true));
                }
                self.settings_json()
            }
            "setSystemLanguage" => {
                self.system_lang = Lang::from_system(arg_str);
                self.settings_json()
            }
            "getStrings" => i18n::strings(self.lang()),
            "getReferences" => Engine::references(self.lang()),
            "getSettings" => self.settings_json(),
            "getDisplays" => self.public_displays(),
            _ => Json::obj().set("error", format!("unknown method: {}", method)),
        }
    }
}

/// Integers without ".0", like JavaScript's String(n).
fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// seriesFor puts "date" first; keep that key order for readable JSON.
fn reorder_date_first(j: Json) -> Json {
    match j {
        Json::Obj(mut items) => {
            if let Some(i) = items.iter().position(|(k, _)| k == "date") {
                let d = items.remove(i);
                items.insert(0, d);
            }
            Json::Obj(items)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::k;
    use crate::store::tests::temp_dir;
    use crate::tracker::tests::{adv, at};

    fn displays_json() -> &'static str {
        r#"[{"id":1,"internal":true,"scaleFactor":2,"bounds":{"x":0,"y":0,"width":1470,"height":956},"widthMm":294,"heightMm":191}]"#
    }

    fn engine(dir: &Path, t: Moment) -> Engine {
        let mut e = Engine::open(dir, t);
        e.set_displays_json(displays_json());
        e.set_hooks_running(true);
        e
    }

    #[test]
    fn events_to_live_payload_and_tray() {
        let dir = temp_dir("engine-live");
        let t = at("2026-10-05", 10, 0, 0);
        let mut e = engine(&dir, t);
        e.mouse_move(t, 0.0, 0.0);
        e.mouse_move(t, 3000.0, 0.0); // longer than a step can be: a warp, ignored
        e.mouse_move(t, 3300.0, 400.0); // 500 pt, outside every display: nearest one's scale
        e.key_down(t, k::E, 0);
        e.mouse_down(t, 1);
        let live = e.live(t);
        let s = live.get("summary").unwrap();
        assert_eq!(s.get("keystrokes").and_then(Json::as_f64), Some(1.0));
        assert!((s.get("mouseMeters").and_then(Json::as_f64).unwrap() - 0.1).abs() < 1e-12);
        assert_eq!(live.get("hooksRunning"), Some(&Json::Bool(true)));
        assert_eq!(live.get("estimatedDisplays"), Some(&Json::Bool(false)));
        let tick = e.tick(t, Some((10.0, 10.0)));
        assert!(tick.get("tray").and_then(Json::as_str).unwrap().ends_with("\u{a0}m"));
        e.update_settings(&parse(r#"{"trayShows":"keys"}"#).unwrap());
        assert_eq!(e.tray_text(), "1");
        e.set_hooks_running(false);
        assert_eq!(e.tray_text(), "⚠ ruxsat kerak");
    }

    #[test]
    fn midnight_saves_yesterday_and_dashboard_compares() {
        let dir = temp_dir("engine-midnight");
        // yesterday: 300 keys in the 8 o'clock hour
        let mut t = at("2026-10-04", 8, 0, 0);
        let mut e = engine(&dir, t);
        for i in 0..300 {
            e.key_down(t, if i % 2 == 0 { k::A } else { k::S }, 0);
            t = adv(t, 1100);
        }
        // next day, same time
        let mut t = at("2026-10-05", 9, 0, 0);
        e.tick(t, None);
        assert!(e.history.get_day("2026-10-04").is_some());
        assert!(std::fs::read_to_string(dir.join("history.json")).unwrap().contains("2026-10-04"));
        for i in 0..600 {
            e.key_down(t, if i % 2 == 0 { k::A } else { k::S }, 0);
            t = adv(t, 1100);
        }
        let d = e.dashboard("today", t);
        let ks = d.get("compare").unwrap().get("deltas").unwrap().get("keystrokes").and_then(Json::as_f64).unwrap();
        assert!((ks - 1.0).abs() < 0.01, "{}", ks);
        let week = e.dashboard("7d", t);
        assert_eq!(week.get("from").and_then(Json::as_str), Some("2026-09-29"));
        assert_eq!(week.get("series").unwrap().as_arr().unwrap().len(), 7);
        assert_eq!(week.get("summary").unwrap().get("keystrokes").and_then(Json::as_f64), Some(900.0));
        let all = e.dashboard("bogus", t);
        assert_eq!(all.get("range").and_then(Json::as_str), Some("7d"));
        let a = e.dashboard("all", t);
        assert_eq!(a.get("from").and_then(Json::as_str), Some("2026-10-04"));
        assert_eq!(a.get("compare"), Some(&Json::Null));
        let w = e.wrapped("month");
        assert_eq!(w.get("label").and_then(Json::as_str), Some("Shu oy"));
    }

    #[test]
    fn break_reminder_fires_from_tick() {
        let dir = temp_dir("engine-break");
        let mut t = at("2026-10-05", 9, 0, 0);
        let mut e = engine(&dir, t);
        e.update_settings(&parse(r#"{"breakMinutes":25}"#).unwrap());
        let mut fired = 0;
        for _ in 0..(26 * 60) {
            if t.ms % 30000 == 0 {
                e.mouse_down(t, 1);
            }
            if e.tick(t, None).get("notify") != Some(&Json::Null) {
                fired += 1;
            }
            t = adv(t, 1000);
        }
        assert_eq!(fired, 1);
        let live = e.live(t);
        assert_eq!(live.get("session").unwrap().get("breakIn").and_then(Json::as_f64), Some(19.0));
    }

    #[test]
    fn csv_reset_and_apps() {
        let dir = temp_dir("engine-csv");
        let t = at("2026-10-05", 9, 0, 0);
        let mut e = engine(&dir, t);
        e.update_settings(&parse(r#"{"trackApps":true}"#).unwrap());
        e.set_app(Some("Safari"));
        e.mouse_move(t, 0.0, 0.0);
        e.mouse_move(t, 300.0, 400.0);
        e.key_down(t, k::A, 0);
        let csv = e.csv();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], "2026-10-05,0.10,0.00,1,0,0,0,0,1,1,0.0,0,,");
        assert_eq!(e.tracker.day.apps.len(), 1);
        e.call("clearApps", &Json::Null, t);
        assert!(e.tracker.day.apps.is_empty());
        e.update_settings(&parse(r#"{"trackApps":false}"#).unwrap());
        e.set_app(Some("Safari"));
        e.key_down(adv(t, 2000), k::B, 0);
        assert!(e.tracker.day.apps.is_empty());
        e.call("resetData", &Json::Null, t);
        assert_eq!(e.tracker.day.keystrokes, 0.0);
        let again = Engine::open(&dir, t);
        assert_eq!(again.tracker.day.keystrokes, 0.0);
    }

    #[test]
    fn widget_snapshot_is_preformatted() {
        let dir = temp_dir("engine-widget");
        let t = at("2026-10-05", 9, 0, 0);
        let mut e = engine(&dir, t);
        e.mouse_move(t, 0.0, 0.0);
        e.mouse_move(t, 300.0, 400.0);
        for _ in 0..3 {
            e.key_down(t, k::E, 0);
            e.key_up(t, k::E);
        }
        let w = e.call("getWidget", &Json::Null, t);
        assert_eq!(w.get("distance").unwrap().get("text").and_then(Json::as_str), Some("0.10\u{a0}m"));
        assert_eq!(w.get("topKey").and_then(Json::as_str), Some("E"));
        assert_eq!(w.get("active").unwrap().get("text").and_then(Json::as_str), Some("1\u{a0}daq"));
        let u = e.call("updateSettings", &parse(r#"[{"unit":"km"}]"#).unwrap(), t);
        assert_eq!(u.get("settings").unwrap().get("unit").and_then(Json::as_str), Some("km"));
        assert!(e.call("nope", &Json::Null, t).get("error").is_some());
        assert_eq!(e.call("getSettings", &Json::Null, t).get("unit").and_then(Json::as_str), Some("km"));
        assert_eq!(e.call("getDisplays", &Json::Null, t).as_arr().map(|a| a.len()), Some(1));
        let seen = e.call("markDashboardSeen", &Json::Null, t);
        assert_eq!(seen.get("seenDashboard"), Some(&Json::Bool(true)));
    }

    #[test]
    fn language_follows_the_os_until_chosen() {
        let dir = temp_dir("engine-lang");
        let t = at("2026-10-05", 9, 0, 0);
        let mut e = engine(&dir, t);
        let s = e.call("setSystemLanguage", &parse(r#"["ru-RU"]"#).unwrap(), t);
        assert_eq!(s.get("lang").and_then(Json::as_str), Some("ru"));
        assert_eq!(s.get("languageChosen"), Some(&Json::Bool(false)));
        assert_eq!(s.get("clock").and_then(Json::as_str), Some("auto"));
        assert_eq!(e.call("getStrings", &Json::Null, t).get("stats").and_then(Json::as_str), Some("Статистика"));
        assert_eq!(e.wrapped("week").get("label").and_then(Json::as_str), Some("Эта неделя"));

        let u = e.call("updateSettings", &parse(r#"[{"language":"en","languageChosen":true,"clock":"24"}]"#).unwrap(), t);
        let u = u.get("settings").unwrap();
        assert_eq!(u.get("lang").and_then(Json::as_str), Some("en"));
        assert_eq!(u.get("systemLang").and_then(Json::as_str), Some("ru"));
        e.set_hooks_running(false);
        assert_eq!(e.tray_text(), "⚠ permission needed");
        let d = e.dashboard("today", t);
        assert_eq!(d.get("settings").and_then(|s| s.get("clock")).and_then(Json::as_str), Some("24"));

        // bad values fall back; the choice survives a restart
        e.call("updateSettings", &parse(r#"[{"language":"de","clock":"13"}]"#).unwrap(), t);
        let again = Engine::open(&dir, t);
        assert_eq!(again.settings.data.language, "auto");
        assert_eq!(again.settings.data.clock, "auto");
        assert!(again.settings.data.language_chosen);
    }

    #[test]
    fn statistics_move_over_from_the_old_name() {
        let root = temp_dir("engine-rename");
        let old = root.join("Mishka Tracker");
        std::fs::create_dir_all(&old).unwrap();
        let t = at("2026-10-05", 9, 0, 0);
        {
            let mut e = engine(&old, t);
            e.key_down(t, keyboard::k::A, 0);
            e.update_settings(&parse(r#"{"unit":"km","language":"ru"}"#).unwrap());
            e.persist(true);
        }
        let new = root.join("Odomouse");
        let e = Engine::open(&new, t);
        assert_eq!(e.settings.data.unit, "km");
        assert_eq!(e.lang(), Lang::Ru);
        assert!(e.settings.data.launch_at_login);
        assert_eq!(e.tracker.day.keystrokes, 1.0);
        assert!(old.join("history.json").is_file(), "the old copy stays");
        // a second start does not copy again over newer data
        std::fs::write(old.join("settings.json"), r#"{"unit":"mi"}"#).unwrap();
        assert_eq!(Engine::open(&new, t).settings.data.unit, "km");
    }
}
