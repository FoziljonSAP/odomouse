//! Raw input events -> today's stats. Every
//! call carries the current time as a `Moment`, so the core never reads the
//! clock itself and all of it is testable.

use crate::day::{clean_app_name, AppStats, Day, GRID_H, GRID_W};
use crate::displays::{normalized_position, step_mm, Display};
use crate::keyboard::{self, k};
use crate::shortcuts;
use crate::time::Moment;
use std::collections::{HashMap, VecDeque};

/// A keydown for a key still held, within this window, is auto-repeat.
pub const REPEAT_WINDOW_MS: i64 = 1000;
/// Gaps longer than this between keystrokes are pauses, not typing time.
pub const TYPING_GAP_MS: i64 = 2000;
const BURST_WINDOW_MS: i64 = 30000;
const BURST_MIN_SPAN_MS: i64 = 15000;
const BURST_MIN_CHARS: usize = 40;
const WPM_CAP: f64 = 250.0;
/// Cursor dwell is only sampled while the user is around.
pub const IDLE_AFTER_MS: i64 = 60000;
/// No input for this long is a break: it ends the work session.
pub const BREAK_MS: i64 = 5 * 60000;
/// Single scroll events above this (points) are bogus.
const MAX_SCROLL_POINTS: f64 = 20000.0;

pub struct Tracker {
    pub day: Day,
    pub dirty: bool,
    pub displays: Vec<Display>,
    last_mouse: Option<(f64, f64)>,
    pressed: HashMap<u32, i64>,
    last_active_minute: Option<i64>,
    pub last_input_at: i64,
    last_type_at: i64,
    burst: VecDeque<i64>,
    pub session_start: Option<i64>,
    app: Option<String>,
    /// Days that ended at midnight, waiting to be saved by the engine.
    finished: Vec<Day>,
}

impl Tracker {
    pub fn new(now: Moment, resume_day: Option<Day>) -> Tracker {
        let today = now.day_key();
        let day = match resume_day {
            Some(d) if d.date == today => d,
            _ => Day::empty(&today),
        };
        Tracker {
            day,
            dirty: false,
            displays: Vec::new(),
            last_mouse: None,
            pressed: HashMap::new(),
            last_active_minute: None,
            last_input_at: 0,
            last_type_at: 0,
            burst: VecDeque::new(),
            session_start: None,
            app: None,
            finished: Vec::new(),
        }
    }

    /// Frontmost app name, or None to stop attributing.
    pub fn set_app(&mut self, name: Option<&str>) {
        self.app = name.and_then(clean_app_name);
    }

    pub fn set_displays(&mut self, model: Vec<Display>) {
        self.displays = model;
    }

    /// The input hook (re)started: forget keys and cursor from before.
    pub fn reset_input_state(&mut self) {
        self.pressed.clear();
        self.last_mouse = None;
    }

    /// Length of the current work stretch in ms, 0 when on a break.
    pub fn session_ms(&self, t: Moment) -> i64 {
        match self.session_start {
            Some(s) if t.ms - self.last_input_at < BREAK_MS => t.ms - s,
            _ => 0,
        }
    }

    pub fn take_finished(&mut self) -> Vec<Day> {
        std::mem::take(&mut self.finished)
    }

    // ------------------------------------------------------------ events

    pub fn mouse_move(&mut self, t: Moment, x: f64, y: f64) {
        self.tick(t);
        if let Some((x0, y0)) = self.last_mouse {
            let mm = step_mm(&self.displays, x0, y0, x, y);
            if mm > 0.0 {
                self.day.mouse_mm += mm;
                self.day.by_hour.mm[t.hour()] += mm;
                if let Some(a) = self.app_stats() {
                    a.mm += mm;
                }
                self.dirty = true;
            }
        }
        self.last_mouse = Some((x, y));
        self.mark_active(t);
    }

    /// button: 1 left, 2 right, 3 middle, anything else "other".
    pub fn mouse_down(&mut self, t: Moment, button: u32) {
        self.tick(t);
        let c = &mut self.day.clicks;
        match button {
            1 => c.left += 1.0,
            2 => c.right += 1.0,
            3 => c.middle += 1.0,
            _ => c.other += 1.0,
        }
        self.day.by_hour.clicks[t.hour()] += 1.0;
        if let Some(a) = self.app_stats() {
            a.clicks += 1.0;
        }
        self.dirty = true;
        self.mark_active(t);
    }

    /// `points`: scrolled distance in the same units as cursor coordinates.
    pub fn wheel(&mut self, t: Moment, x: f64, y: f64, points: f64) {
        self.tick(t);
        let points = points.abs();
        if points > 0.0 && points < MAX_SCROLL_POINTS {
            let scale = normalized_position(&self.displays, x, y).map_or(0.0, |(i, _, _)| self.displays[i].mm_per_point);
            let mm = points * scale;
            self.day.scroll_mm += mm;
            self.day.by_hour.scroll[t.hour()] += mm;
            if let Some(a) = self.app_stats() {
                a.scroll += mm;
            }
            self.dirty = true;
        }
        self.mark_active(t);
    }

    /// `code`: VC keycode; `mods`: shortcuts::{CTRL, ALT, SHIFT, META} bits.
    pub fn key_down(&mut self, t: Moment, code: u32, mods: u32) {
        self.tick(t);
        let last = self.pressed.insert(code, t.ms);
        if matches!(last, Some(l) if t.ms - l < REPEAT_WINDOW_MS) {
            return; // auto-repeat
        }
        self.day.keystrokes += 1.0;
        *self.day.keys.entry(code).or_insert(0.0) += 1.0;
        self.day.by_hour.keys[t.hour()] += 1.0;
        if let Some(a) = self.app_stats() {
            a.keys += 1.0;
        }
        if shortcuts::is_shortcut(code, mods) {
            self.day.shortcuts.add(&shortcuts::id_for(mods & 15, code), 1.0);
        }
        self.dirty = true;
        self.mark_active(t);

        let shortcut = mods & (shortcuts::META | shortcuts::CTRL) != 0;
        if code == k::BACKSPACE {
            self.typing_time(t.ms);
            self.day.typing.backspaces += 1.0;
        } else if keyboard::is_printable(code) && !shortcut {
            self.typing_time(t.ms);
            self.day.typing.chars += 1.0;
            self.burst_add(t.ms);
        } else if !keyboard::is_modifier(code) {
            // arrows, Tab, shortcuts: a pause as far as WPM is concerned
            self.last_type_at = 0;
            self.burst.clear();
        }
    }

    pub fn key_up(&mut self, _t: Moment, code: u32) {
        self.pressed.remove(&code);
    }

    /// Called every second with the cursor position (heatmap dwell).
    pub fn sample_cursor(&mut self, t: Moment, x: f64, y: f64) {
        self.tick(t);
        if t.ms - self.last_input_at > IDLE_AFTER_MS {
            return;
        }
        if let Some((_, fx, fy)) = normalized_position(&self.displays, x, y) {
            let col = (fx * GRID_W as f64) as usize;
            let row = (fy * GRID_H as f64) as usize;
            self.day.grid[row * GRID_W + col] += 1.0;
            self.dirty = true;
        }
    }

    /// Check the date even when idle, so midnight is handled without input.
    pub fn check_rollover(&mut self, t: Moment) {
        self.tick(t);
    }

    // --------------------------------------------------------- internals

    fn tick(&mut self, t: Moment) {
        let key = t.day_key();
        if key != self.day.date {
            let done = std::mem::replace(&mut self.day, Day::empty(&key));
            self.dirty = true;
            self.last_active_minute = None;
            self.last_type_at = 0;
            self.burst.clear();
            self.finished.push(done);
        }
    }

    fn mark_active(&mut self, t: Moment) {
        let start = match self.session_start {
            Some(s) if t.ms - self.last_input_at < BREAK_MS => s,
            _ => t.ms,
        };
        self.session_start = Some(start);
        self.last_input_at = t.ms;
        let session_min = ((t.ms - start) / 60000) as f64;
        if session_min > self.day.longest_session_min {
            self.day.longest_session_min = session_min;
        }
        let minute = t.ms.div_euclid(60000);
        if self.last_active_minute != Some(minute) {
            self.last_active_minute = Some(minute);
            self.day.active_minutes += 1.0;
            self.day.hourly[t.hour()] += 1.0;
            if let Some(a) = self.app_stats() {
                a.minutes += 1.0;
            }
            self.dirty = true;
        }
    }

    fn app_stats(&mut self) -> Option<&mut AppStats> {
        let name = self.app.as_deref()?;
        self.day.app_mut(name)
    }

    fn typing_time(&mut self, t: i64) {
        if self.last_type_at != 0 && t - self.last_type_at <= TYPING_GAP_MS {
            self.day.typing.ms += (t - self.last_type_at) as f64;
        } else {
            self.burst.clear();
        }
        self.last_type_at = t;
    }

    fn burst_add(&mut self, t: i64) {
        self.burst.push_back(t);
        while let Some(&first) = self.burst.front() {
            if t - first > BURST_WINDOW_MS {
                self.burst.pop_front();
            } else {
                break;
            }
        }
        let span = t - self.burst[0];
        if self.burst.len() >= BURST_MIN_CHARS && span >= BURST_MIN_SPAN_MS {
            let w = (self.burst.len() - 1) as f64 / 5.0 / (span as f64 / 60000.0);
            if w <= WPM_CAP && w > self.day.typing.best_wpm {
                self.day.typing.best_wpm = crate::units::js_round(w);
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::displays::{build_model, DisplayInfo, Rect};
    use crate::shortcuts::{META, SHIFT};
    use crate::time::days_from_civil;

    /// Local wall-clock time in a +5 zone (Tashkent), like the user's Mac.
    pub fn at(date: &str, h: i64, m: i64, s: i64) -> Moment {
        let (y, mo, d) = crate::time::parse_key(date).unwrap();
        let local = days_from_civil(y, mo, d) * 86_400_000 + ((h * 60 + m) * 60 + s) * 1000;
        Moment::new(local - 5 * 3_600_000, 5 * 3600)
    }

    pub fn adv(t: Moment, ms: i64) -> Moment {
        Moment::new(t.ms + ms, t.tz_offset_s)
    }

    /// MacBook Air: 1470x956 pt, 294x191 mm -> exactly 0.2 mm per point.
    pub fn mba() -> Vec<Display> {
        build_model(
            &[DisplayInfo {
                id: "1".into(),
                label: None,
                internal: true,
                scale_factor: 2.0,
                bounds: Rect { x: 0.0, y: 0.0, width: 1470.0, height: 956.0 },
                width_mm: 294.0,
                height_mm: 191.0,
                estimate_ppi: 0.0,
            }],
            &[],
        )
    }

    fn tracker(t: Moment) -> Tracker {
        let mut tr = Tracker::new(t, None);
        tr.set_displays(mba());
        tr
    }

    fn approx(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() <= eps, "{} != {}", a, b);
    }

    #[test]
    fn mouse_distance_in_real_mm() {
        let t = at("2026-10-03", 10, 0, 0);
        let mut tr = tracker(t);
        tr.mouse_move(t, 0.0, 0.0);
        tr.mouse_move(t, 300.0, 400.0);
        approx(tr.day.mouse_mm, 100.0, 1e-9);
    }

    #[test]
    fn auto_repeat_counted_once_and_missed_keyup_recovers() {
        let mut t = at("2026-10-03", 10, 0, 0);
        let mut tr = tracker(t);
        tr.key_down(t, k::D, 0);
        for _ in 0..30 {
            t = adv(t, 40);
            tr.key_down(t, k::D, 0);
        }
        tr.key_up(t, k::D);
        assert_eq!(tr.day.keystrokes, 1.0);
        t = adv(t, 100);
        tr.key_down(t, k::D, 0);
        assert_eq!(tr.day.keystrokes, 2.0);
        t = adv(t, 5000); // keyup never came
        tr.key_down(t, k::D, 0);
        assert_eq!(tr.day.keystrokes, 3.0);
    }

    #[test]
    fn clicks_and_scroll() {
        let t = at("2026-10-03", 10, 0, 0);
        let mut tr = tracker(t);
        for b in [1, 1, 2, 3, 5] {
            tr.mouse_down(t, b);
        }
        let c = &tr.day.clicks;
        assert_eq!((c.left, c.right, c.middle, c.other), (2.0, 1.0, 1.0, 1.0));
        tr.wheel(t, 100.0, 100.0, -30.0);
        approx(tr.day.scroll_mm, 6.0, 1e-9);
        tr.wheel(t, 100.0, 100.0, 0.0);
        approx(tr.day.scroll_mm, 6.0, 1e-9);
    }

    #[test]
    fn active_minutes_and_local_hours() {
        let mut t = at("2026-10-03", 9, 59, 30);
        let mut tr = tracker(t);
        tr.mouse_down(t, 1);
        t = adv(t, 10000);
        tr.mouse_down(t, 1);
        t = adv(t, 30000);
        tr.mouse_down(t, 1);
        assert_eq!(tr.day.active_minutes, 2.0);
        assert_eq!(tr.day.hourly[9], 1.0);
        assert_eq!(tr.day.hourly[10], 1.0);
    }

    #[test]
    fn wpm_counts_typing_time_only() {
        let mut t = at("2026-10-03", 10, 0, 0);
        let mut tr = tracker(t);
        for _ in 0..300 {
            tr.key_down(t, k::E, 0);
            t = adv(t, 100);
            tr.key_up(t, k::E);
        }
        assert_eq!(tr.day.typing.chars, 300.0);
        approx(tr.day.typing.ms, 29900.0, 1.0);
        t = adv(t, 60000);
        tr.key_down(t, k::C, META);
        assert_eq!(tr.day.typing.chars, 300.0);
        assert_eq!(tr.day.keystrokes, 301.0);
        approx(tr.day.typing.ms, 29900.0, 1.0);
        approx(tr.day.typing.best_wpm, 120.0, 1.0);
        tr.key_down(t, k::BACKSPACE, 0);
        assert_eq!(tr.day.typing.backspaces, 1.0);
    }

    #[test]
    fn midnight_rollover_hands_over_the_finished_day() {
        let t = at("2026-10-03", 23, 59, 50);
        let mut tr = tracker(t);
        tr.mouse_move(t, 0.0, 0.0);
        tr.mouse_move(t, 300.0, 400.0);
        tr.key_down(t, k::A, 0);
        let t = at("2026-10-04", 0, 0, 5);
        tr.key_up(t, k::A);
        tr.key_down(t, k::B, 0);
        let done = tr.take_finished();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].date, "2026-10-03");
        assert_eq!(done[0].keystrokes, 1.0);
        approx(done[0].mouse_mm, 100.0, 1e-9);
        assert_eq!(tr.day.date, "2026-10-04");
        assert_eq!(tr.day.keystrokes, 1.0);
        assert_eq!(tr.day.mouse_mm, 0.0);
        // idle rollover
        let t2 = at("2026-10-05", 8, 0, 0);
        tr.check_rollover(t2);
        tr.check_rollover(t2);
        assert_eq!(tr.take_finished().len(), 1);
    }

    #[test]
    fn resume_only_today() {
        let t = at("2026-10-03", 12, 0, 0);
        let mut saved = Day::empty("2026-10-03");
        saved.keystrokes = 77.0;
        assert_eq!(Tracker::new(t, Some(saved)).day.keystrokes, 77.0);
        let mut stale = Day::empty("2026-10-01");
        stale.keystrokes = 5.0;
        assert_eq!(Tracker::new(t, Some(stale)).day.keystrokes, 0.0);
    }

    #[test]
    fn heatmap_samples_only_while_active() {
        let t = at("2026-10-03", 10, 0, 0);
        let mut tr = tracker(t);
        tr.mouse_move(t, 735.0, 478.0);
        tr.sample_cursor(t, 735.0, 478.0);
        assert_eq!(tr.day.grid[10 * GRID_W + 16], 1.0);
        tr.sample_cursor(adv(t, IDLE_AFTER_MS + 1000), 735.0, 478.0);
        assert_eq!(tr.day.grid.iter().sum::<f64>(), 1.0);
    }

    #[test]
    fn shortcuts_per_combo_without_repeat() {
        let mut t = at("2026-10-04", 10, 0, 0);
        let mut tr = tracker(t);
        let mut press = |tr: &mut Tracker, code, mods| {
            tr.key_down(t, code, mods);
            t = adv(t, 80);
            tr.key_up(t, code);
            t = adv(t, 80);
        };
        press(&mut tr, k::META, META);
        press(&mut tr, k::C, META);
        press(&mut tr, k::C, META);
        press(&mut tr, k::V, META);
        let t = at("2026-10-04", 10, 1, 0);
        tr.key_down(t, k::Z, META | SHIFT);
        for i in 1..=10 {
            tr.key_down(adv(t, 40 * i), k::Z, META | SHIFT);
        }
        assert_eq!(tr.day.shortcuts.get("8:46"), Some(&2.0));
        assert_eq!(tr.day.shortcuts.get("8:47"), Some(&1.0));
        assert_eq!(tr.day.shortcuts.get("12:44"), Some(&1.0));
        assert_eq!(tr.day.shortcuts.len(), 3);
    }

    #[test]
    fn hour_buckets_add_up() {
        let t = at("2026-10-04", 9, 59, 59);
        let mut tr = tracker(t);
        tr.mouse_move(t, 0.0, 0.0);
        tr.mouse_move(t, 300.0, 400.0);
        tr.key_down(t, k::A, 0);
        let t = at("2026-10-04", 10, 30, 0);
        tr.mouse_move(t, 300.0, 800.0);
        tr.mouse_down(t, 1);
        tr.wheel(t, 10.0, 10.0, 10.0);
        tr.key_down(t, k::B, 0);
        approx(tr.day.by_hour.mm[9], 100.0, 1e-9);
        approx(tr.day.by_hour.mm[10], 80.0, 1e-9);
        approx(tr.day.by_hour.mm.iter().sum(), tr.day.mouse_mm, 1e-9);
        assert_eq!(tr.day.by_hour.keys.iter().sum::<f64>(), tr.day.keystrokes);
        assert_eq!(tr.day.by_hour.clicks.iter().sum::<f64>(), 1.0);
        approx(tr.day.by_hour.scroll.iter().sum(), tr.day.scroll_mm, 1e-9);
    }

    #[test]
    fn work_session_and_breaks() {
        let mut t = at("2026-10-04", 9, 0, 0);
        let mut tr = tracker(t);
        for _ in 0..=70 {
            tr.mouse_down(t, 1);
            t = adv(t, 60000);
        }
        t = adv(t, -60000);
        assert_eq!(tr.day.longest_session_min, 70.0);
        assert_eq!(tr.session_ms(t), 70 * 60000);
        t = adv(t, 5 * 60000);
        assert_eq!(tr.session_ms(t), 0);
        tr.mouse_down(t, 1);
        assert_eq!(tr.session_ms(t), 0);
        t = adv(t, 4 * 60000);
        tr.mouse_down(t, 1);
        assert_eq!(tr.session_ms(t), 4 * 60000);
        assert_eq!(tr.day.longest_session_min, 70.0);
    }

    #[test]
    fn app_attribution_only_when_set() {
        let mut t = at("2026-10-04", 10, 0, 0);
        let mut tr = tracker(t);
        tr.key_down(t, k::A, 0);
        tr.key_up(t, k::A);
        assert!(tr.day.apps.is_empty());
        tr.set_app(Some("Visual Studio Code"));
        t = adv(t, 61000);
        tr.key_down(t, k::A, 0);
        tr.mouse_move(t, 0.0, 0.0);
        tr.mouse_move(t, 300.0, 400.0);
        tr.mouse_down(t, 1);
        tr.set_app(Some("Telegram"));
        tr.mouse_down(t, 1);
        let vs = &tr.day.apps[0].1;
        assert_eq!((vs.keys, vs.clicks, vs.minutes), (1.0, 1.0, 1.0));
        approx(vs.mm, 100.0, 1e-9);
        tr.set_app(None);
        tr.mouse_down(t, 1);
        assert_eq!(tr.day.apps[1].1.clicks, 1.0);
    }
}
