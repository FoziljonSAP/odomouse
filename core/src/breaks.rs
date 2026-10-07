//! Break reminder. After `work_minutes` of continuous
//! work (no 5-minute pause) it asks for a break once, then every 20 minutes
//! while you keep going. A real break resets it.

use crate::i18n::{long_duration, Lang};

pub const WORK_OPTIONS: [u32; 4] = [25, 50, 60, 90];
pub const REPEAT_MS: i64 = 20 * 60000;

#[derive(Default)]
pub struct BreakCoach {
    session_start: Option<i64>,
    work_ms: i64,
    next_at: Option<i64>,
}

impl BreakCoach {
    pub fn new() -> BreakCoach {
        BreakCoach::default()
    }

    /// `session_ms`: 0 while on a break (or when reminders are off).
    pub fn check(&mut self, now: i64, session_start: Option<i64>, session_ms: i64, work_minutes: u32) -> bool {
        let Some(start) = session_start.filter(|_| session_ms > 0) else {
            self.session_start = None;
            self.next_at = None;
            return false;
        };
        let work_ms = work_minutes as i64 * 60000;
        if self.session_start != Some(start) || self.next_at.is_none() {
            self.session_start = Some(start);
            self.work_ms = work_ms;
            self.next_at = Some(start + work_ms);
        } else if work_ms != self.work_ms {
            self.work_ms = work_ms;
            self.next_at = Some(start + work_ms);
        }
        let next = self.next_at.unwrap();
        if now >= next {
            self.next_at = Some(now + REPEAT_MS);
            return true;
        }
        false
    }

    /// Minutes until the next reminder, None when not in a session.
    pub fn minutes_left(&self, now: i64) -> Option<i64> {
        self.next_at.map(|n| ((n - now) as f64 / 60000.0).ceil().max(0.0) as i64)
    }
}

/// 50 -> "50 daqiqadan", 60 -> "1 soatdan", 95 -> "1 soat 35 daqiqadan"
fn since_text(minutes: f64) -> String {
    let m = crate::units::js_round(minutes).max(0.0) as i64;
    let (h, r) = (m / 60, m % 60);
    if h == 0 {
        format!("{} daqiqadan", r)
    } else if r == 0 {
        format!("{} soatdan", h)
    } else {
        format!("{} soat {} daqiqadan", h, r)
    }
}

pub fn break_message(session_minutes: f64, lang: Lang) -> (String, String) {
    let m = crate::units::js_round(session_minutes).max(0.0) as i64;
    match lang {
        Lang::Uz => (
            "Tanaffus vaqti".to_string(),
            format!(
                "{} beri to'xtovsiz ishlayapsiz. 5 daqiqa o'rningizdan turing va uzoqroqqa qarang.",
                since_text(session_minutes)
            ),
        ),
        Lang::En => (
            "Time for a break".to_string(),
            format!(
                "You've been working for {} without a break. Stand up for 5 minutes and look into the distance.",
                long_duration(Lang::En, m)
            ),
        ),
        Lang::Ru => (
            "Пора сделать перерыв".to_string(),
            format!(
                "Вы работаете без перерыва уже {}. Встаньте на 5 минут и посмотрите вдаль.",
                long_duration(Lang::Ru, m)
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MIN: i64 = 60000;

    #[test]
    fn reminds_once_then_every_20_minutes_and_resets_after_break() {
        let mut c = BreakCoach::new();
        let s = 1_000_000;
        assert!(!c.check(s + 49 * MIN, Some(s), 49 * MIN, 50));
        assert_eq!(c.minutes_left(s + 49 * MIN), Some(1));
        assert!(c.check(s + 50 * MIN, Some(s), 50 * MIN, 50));
        assert!(!c.check(s + 51 * MIN, Some(s), 51 * MIN, 50));
        assert!(c.check(s + 70 * MIN, Some(s), 70 * MIN, 50));
        assert!(!c.check(s + 76 * MIN, Some(s), 0, 50)); // break
        assert_eq!(c.minutes_left(s + 76 * MIN), None);
        let s2 = s + 80 * MIN;
        assert!(!c.check(s2 + MIN, Some(s2), MIN, 50));
        // setting changed mid-session
        assert!(c.check(s2 + 26 * MIN, Some(s2), 26 * MIN, 25));
    }

    #[test]
    fn message_text() {
        assert!(break_message(50.0, Lang::Uz).1.starts_with("50 daqiqadan beri"));
        assert!(break_message(60.0, Lang::Uz).1.starts_with("1 soatdan beri"));
        assert!(break_message(95.0, Lang::Uz).1.starts_with("1 soat 35 daqiqadan beri"));
        assert!(break_message(95.0, Lang::En).1.starts_with("You've been working for 1 hour 35 minutes"));
        assert_eq!(break_message(50.0, Lang::Ru).0, "Пора сделать перерыв");
        assert!(break_message(122.0, Lang::Ru).1.contains("уже 2 часа 2 минуты."));
    }
}
