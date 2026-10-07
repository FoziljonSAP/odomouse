//! history.json (all days) and settings.json.
//! Atomic writes (temp file + rename); a corrupt file is kept as
//! `.bak-<time>` and the app starts fresh instead of crashing.

use crate::breaks::WORK_OPTIONS;
use crate::day::{normalize_day, Day};
use crate::fun::{is_ref_key, DISTANCE_REFS, TEXT_REFS};
use crate::json::{parse, Json};
use crate::units::normalize_unit;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const HISTORY_VERSION: u32 = 2;

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Ok(None) if missing; Ok(None) + a .bak copy if unreadable.
fn read_json(file: &Path) -> Option<Json> {
    let text = fs::read(file).ok()?;
    match std::str::from_utf8(&text).map_err(|e| e.to_string()).and_then(parse) {
        Ok(v) => Some(v),
        Err(_) => {
            let mut bak = file.as_os_str().to_owned();
            bak.push(format!(".bak-{}", now_ms()));
            let _ = fs::copy(file, PathBuf::from(bak));
            None
        }
    }
}

pub fn write_atomic(file: &Path, text: &str) -> io::Result<()> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut tmp = file.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, text)?;
    fs::rename(&tmp, file).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        e
    })
}

// ------------------------------------------------------------- history

pub struct HistoryStore {
    pub file: PathBuf,
    /// date key -> day; BTreeMap keeps dates sorted
    pub days: BTreeMap<String, Day>,
}

impl HistoryStore {
    pub fn open(file: PathBuf) -> HistoryStore {
        let mut s = HistoryStore { file, days: BTreeMap::new() };
        s.load();
        s
    }

    pub fn load(&mut self) {
        let value = read_json(&self.file);
        let mut migrated = false;
        self.days.clear();
        if let Some(obj) = value.as_ref().and_then(|v| v.get("days")).and_then(Json::as_obj) {
            for (date, raw) in obj {
                let day = normalize_day(raw, date);
                migrated |= day.migrated_from.is_some();
                self.days.insert(date.clone(), day);
            }
        }
        let old_version = value.as_ref().map_or(false, |v| v.get("version").and_then(Json::as_f64) != Some(HISTORY_VERSION as f64));
        if migrated || old_version {
            let _ = self.save();
        }
    }

    pub fn to_json(&self) -> Json {
        let mut days = Json::obj();
        for (k, d) in &self.days {
            days.insert(k, d.to_json());
        }
        Json::obj().set("version", HISTORY_VERSION).set("days", days)
    }

    pub fn save(&self) -> io::Result<()> {
        write_atomic(&self.file, &self.to_json().to_string())
    }

    pub fn put_day(&mut self, day: &Day) {
        self.days.insert(day.date.clone(), day.clone());
    }

    pub fn get_day(&self, date: &str) -> Option<&Day> {
        self.days.get(date)
    }

    /// Days in [from, to], oldest first; missing days are skipped.
    pub fn get_days(&self, from: &str, to: &str) -> Vec<&Day> {
        if from > to {
            return Vec::new();
        }
        self.days.range(from.to_string()..=to.to_string()).map(|(_, d)| d).collect()
    }

    pub fn first_date(&self) -> Option<&str> {
        self.days.keys().next().map(String::as_str)
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.days.clear();
        self.save()
    }

    pub fn clear_apps(&mut self) -> io::Result<()> {
        for d in self.days.values_mut() {
            d.apps.clear();
        }
        self.save()
    }
}

// ------------------------------------------------------------ settings

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub unit: String,
    pub theme: String,
    pub compare_distance: String,
    pub compare_text: String,
    pub onboarding_done: bool,
    pub seen_dashboard: bool,
    pub tray_shows: String,
    pub diagonal_overrides: Vec<(String, f64)>,
    pub break_reminder: bool,
    pub break_minutes: u32,
    pub track_apps: bool,
    /// Show the icon in the menu bar / system tray (off = widget only on macOS).
    pub menu_bar: bool,
    pub launch_at_login: bool,
    /// "auto" (the OS language) or "uz" / "en" / "ru".
    pub language: String,
    /// The first-run language question was answered.
    pub language_chosen: bool,
    /// Clock style for hours: "auto" (12-hour in English), "24" or "12".
    pub clock: String,
}

const THEMES: [&str; 3] = ["system", "light", "dark"];
const TRAY_KEYS: [&str; 3] = ["distance", "keys", "icon"];
const CLOCKS: [&str; 3] = ["auto", "24", "12"];

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            unit: "m".into(),
            theme: "system".into(),
            compare_distance: "auto".into(),
            compare_text: "a4".into(),
            onboarding_done: false,
            seen_dashboard: false,
            tray_shows: "distance".into(),
            diagonal_overrides: Vec::new(),
            break_reminder: true,
            break_minutes: 50,
            track_apps: false,
            menu_bar: true,
            // on by default: a tray counter that is gone after a restart looks broken
            launch_at_login: true,
            language: "auto".into(),
            language_chosen: false,
            clock: "auto".into(),
        }
    }
}

impl Settings {
    pub fn to_json(&self) -> Json {
        let mut ov = Json::obj();
        for (id, v) in &self.diagonal_overrides {
            ov.insert(id, *v);
        }
        Json::obj()
            .set("unit", self.unit.as_str())
            .set("theme", self.theme.as_str())
            .set("compareDistance", self.compare_distance.as_str())
            .set("compareText", self.compare_text.as_str())
            .set("onboardingDone", self.onboarding_done)
            .set("seenDashboard", self.seen_dashboard)
            .set("trayShows", self.tray_shows.as_str())
            .set("diagonalOverrides", ov)
            .set("breakReminder", self.break_reminder)
            .set("breakMinutes", self.break_minutes)
            .set("trackApps", self.track_apps)
            .set("menuBar", self.menu_bar)
            .set("launchAtLogin", self.launch_at_login)
            .set("language", self.language.as_str())
            .set("languageChosen", self.language_chosen)
            .set("clock", self.clock.as_str())
    }

    /// Validate a full settings object; anything invalid falls back to the default.
    pub fn sanitize(j: &Json) -> Settings {
        let d = Settings::default();
        let s = |k: &str| j.get(k).and_then(Json::as_str).unwrap_or("");
        let pick = |v: &str, ok: bool, def: &str| if ok { v.to_string() } else { def.to_string() };
        let mut overrides = Vec::new();
        if let Some(o) = j.get("diagonalOverrides").and_then(Json::as_obj) {
            for (id, v) in o {
                let n = crate::day::num(Some(v));
                if (5.0..=120.0).contains(&n) {
                    overrides.push((id.clone(), crate::units::js_round(n * 10.0) / 10.0));
                }
            }
        }
        let minutes = crate::day::num(j.get("breakMinutes"));
        let unit_raw = s("unit");
        Settings {
            unit: normalize_unit(unit_raw).to_string(),
            theme: pick(s("theme"), THEMES.contains(&s("theme")), &d.theme),
            compare_distance: pick(s("compareDistance"), is_ref_key(&DISTANCE_REFS, s("compareDistance")), &d.compare_distance),
            compare_text: pick(s("compareText"), is_ref_key(&TEXT_REFS, s("compareText")), &d.compare_text),
            onboarding_done: j.get("onboardingDone") == Some(&Json::Bool(true)),
            seen_dashboard: j.get("seenDashboard") == Some(&Json::Bool(true)),
            tray_shows: pick(s("trayShows"), TRAY_KEYS.contains(&s("trayShows")), &d.tray_shows),
            diagonal_overrides: overrides,
            break_reminder: j.get("breakReminder") != Some(&Json::Bool(false)),
            break_minutes: WORK_OPTIONS.iter().copied().find(|m| *m as f64 == minutes).unwrap_or(d.break_minutes),
            track_apps: j.get("trackApps") == Some(&Json::Bool(true)),
            menu_bar: j.get("menuBar") != Some(&Json::Bool(false)),
            launch_at_login: j.get("launchAtLogin") != Some(&Json::Bool(false)),
            language: pick(s("language"), s("language") == "auto" || crate::i18n::LANGS.contains(&s("language")), &d.language),
            language_chosen: j.get("languageChosen") == Some(&Json::Bool(true)),
            clock: pick(s("clock"), CLOCKS.contains(&s("clock")), &d.clock),
        }
    }

    /// `{...self, ...partial}` then sanitize, like the JS store.
    pub fn merged(&self, partial: &Json) -> Settings {
        let mut base = self.to_json();
        if let Some(o) = partial.as_obj() {
            for (k, v) in o {
                base.insert(k, v.clone());
            }
        }
        Settings::sanitize(&base)
    }
}

pub struct SettingsStore {
    pub file: PathBuf,
    pub data: Settings,
}

impl SettingsStore {
    pub fn open(file: PathBuf) -> SettingsStore {
        let data = match read_json(&file) {
            Some(v) => Settings::default().merged(&v),
            None => Settings::default(),
        };
        SettingsStore { file, data }
    }

    pub fn update(&mut self, partial: &Json) -> io::Result<&Settings> {
        self.data = self.data.merged(partial);
        write_atomic(&self.file, &self.data.to_json().to_string())?;
        Ok(&self.data)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("odomouse-test-{}-{}-{}", name, std::process::id(), now_ms()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn files_in(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    }

    #[test]
    fn migrates_v1_file_and_rewrites_it() {
        let dir = temp_dir("v1");
        let file = dir.join("history.json");
        fs::write(&file, r#"{"days":{"2026-09-30":{"date":"2026-09-30","keystrokes":10,"clicks":2,"mouseMeters":1.5,"topLetters":[]}}}"#).unwrap();
        let h = HistoryStore::open(file.clone());
        assert_eq!(h.get_day("2026-09-30").unwrap().mouse_mm, 1500.0);
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.starts_with(r#"{"version":2,"days":{"2026-09-30":{"v":3,"#));
        assert_eq!(files_in(&dir), vec!["history.json"]);
    }

    #[test]
    fn atomic_save_round_trip_and_range_queries() {
        let dir = temp_dir("save");
        let file = dir.join("sub").join("history.json");
        let mut h = HistoryStore::open(file.clone());
        for date in ["2026-10-01", "2026-10-03", "2026-10-02"] {
            let mut d = Day::empty(date);
            d.keystrokes = 5.0;
            h.put_day(&d);
        }
        h.save().unwrap();
        let h2 = HistoryStore::open(file.clone());
        assert_eq!(h2.days, h.days);
        let got: Vec<&str> = h2.get_days("2026-10-02", "2026-10-09").iter().map(|d| d.date.as_str()).collect();
        assert_eq!(got, vec!["2026-10-02", "2026-10-03"]);
        assert_eq!(h2.first_date(), Some("2026-10-01"));
        assert_eq!(files_in(&dir.join("sub")), vec!["history.json"]);
    }

    #[test]
    fn corrupt_history_is_backed_up() {
        let dir = temp_dir("corrupt");
        let file = dir.join("history.json");
        fs::write(&file, "{\"days\": {oops").unwrap();
        let h = HistoryStore::open(file);
        assert!(h.days.is_empty());
        let files = files_in(&dir);
        assert!(files.iter().any(|f| f.starts_with("history.json.bak-")));
    }

    #[test]
    fn settings_defaults_validation_and_persistence() {
        let dir = temp_dir("settings");
        let file = dir.join("settings.json");
        let mut s = SettingsStore::open(file.clone());
        assert_eq!(s.data, Settings::default());
        let p = parse(r#"{"unit":"auto-imperial","theme":"neon","compareDistance":"eiffel","compareText":"bus",
            "trayShows":"keys","diagonalOverrides":{"1":"13.62","2":300},"breakMinutes":"25","breakReminder":0,"trackApps":"yes"}"#).unwrap();
        s.update(&p).unwrap();
        let d = &s.data;
        assert_eq!(d.unit, "ft");
        assert_eq!(d.theme, "system");
        assert_eq!(d.compare_distance, "eiffel");
        assert_eq!(d.compare_text, "a4");
        assert_eq!(d.tray_shows, "keys");
        assert_eq!(d.diagonal_overrides, vec![("1".to_string(), 13.6)]);
        assert_eq!(d.break_minutes, 25);
        assert!(d.break_reminder, "only an explicit false turns it off");
        assert!(!d.track_apps);
        let again = SettingsStore::open(file);
        assert_eq!(&again.data, d);
        // a partial update keeps the other fields
        let mut s = again;
        s.update(&parse(r#"{"theme":"dark"}"#).unwrap()).unwrap();
        assert_eq!((s.data.theme.as_str(), s.data.unit.as_str()), ("dark", "ft"));
    }
}
