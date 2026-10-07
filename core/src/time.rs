//! Local calendar time without a timezone database: the host OS passes the
//! current UTC offset with every timestamp, the core does the date maths.

/// A point in time as seen by the user: Unix milliseconds + local UTC offset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Moment {
    pub ms: i64,
    pub tz_offset_s: i32,
}

impl Moment {
    pub fn new(ms: i64, tz_offset_s: i32) -> Moment {
        Moment { ms, tz_offset_s }
    }

    fn local_ms(&self) -> i64 {
        self.ms + self.tz_offset_s as i64 * 1000
    }

    /// (year, month 1-12, day 1-31) in local time.
    pub fn date(&self) -> (i64, u32, u32) {
        civil_from_days(self.local_ms().div_euclid(86_400_000))
    }

    /// Local hour 0-23.
    pub fn hour(&self) -> usize {
        (self.local_ms().rem_euclid(86_400_000) / 3_600_000) as usize
    }

    /// Seconds since local midnight.
    pub fn seconds_of_day(&self) -> i64 {
        self.local_ms().rem_euclid(86_400_000) / 1000
    }

    /// "YYYY-MM-DD" in local time.
    pub fn day_key(&self) -> String {
        let (y, m, d) = self.date();
        format!("{:04}-{:02}-{:02}", y, m, d)
    }
}

/// Howard Hinnant's algorithm: days since 1970-01-01 -> (y, m, d).
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// (y, m, d) -> days since 1970-01-01.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let m = m as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn parse_key(key: &str) -> Option<(i64, u32, u32)> {
    let mut parts = key.split('-');
    let y = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

fn key_of_days(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// "2026-12-31" + 1 -> "2027-01-01". Invalid keys come back unchanged.
pub fn add_days(key: &str, delta: i64) -> String {
    match parse_key(key) {
        Some((y, m, d)) => key_of_days(days_from_civil(y, m, d) + delta),
        None => key.to_string(),
    }
}

/// Inclusive list of day keys from `from` to `to` (max 4000 days).
pub fn date_range(from: &str, to: &str) -> Vec<String> {
    let (Some(a), Some(b)) = (parse_key(from), parse_key(to)) else { return Vec::new() };
    let (a, b) = (days_from_civil(a.0, a.1, a.2), days_from_civil(b.0, b.1, b.2));
    (a..=b).take(4000).map(key_of_days).collect()
}

/// Weekday 0 = Sunday ... 6 = Saturday for a day key.
pub fn weekday(key: &str) -> Option<u32> {
    let (y, m, d) = parse_key(key)?;
    Some((days_from_civil(y, m, d) + 4).rem_euclid(7) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_date_uses_offset() {
        // 2026-10-03T19:30:00Z is already 2026-10-04 00:30 in Tashkent (+5)
        let t = Moment::new(1_791_055_800_000, 5 * 3600);
        assert_eq!(t.day_key(), "2026-10-04");
        assert_eq!(t.hour(), 0);
        assert_eq!(Moment::new(1_791_055_800_000, 0).day_key(), "2026-10-03");
        assert_eq!(Moment::new(1_791_055_800_000, 0).hour(), 19);
    }

    #[test]
    fn day_arithmetic_crosses_months_and_years() {
        assert_eq!(add_days("2026-10-01", -1), "2026-09-30");
        assert_eq!(add_days("2026-12-31", 1), "2027-01-01");
        assert_eq!(add_days("2028-02-28", 1), "2028-02-29");
        assert_eq!(date_range("2026-09-29", "2026-10-02"), vec!["2026-09-29", "2026-09-30", "2026-10-01", "2026-10-02"]);
        assert_eq!(date_range("2026-10-02", "2026-10-01"), Vec::<String>::new());
    }

    #[test]
    fn civil_round_trip_over_wide_range() {
        for days in (-100_000..100_000).step_by(37) {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days);
        }
        assert_eq!(weekday("2026-10-04"), Some(0)); // Sunday
        assert_eq!(parse_key("2026-13-01"), None);
    }
}
