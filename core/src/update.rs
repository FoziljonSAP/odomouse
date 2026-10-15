//! New-version check. The shells download one small file once a day,
//! `latest.json` on the website, and hand its text to the core; the core
//! decides whether it is newer and whether to notify (once per version).
//! Nothing about the user is sent: it is a plain download of a public file.
//!
//! latest.json (written by CI next to the installers):
//! {"version":"1.2.0","files":{"windows":{"url":"Odomouse-Setup.exe","sha256":"…"},
//!                              "mac":{"url":"Odomouse.dmg","sha256":"…"},
//!                              "linux":{"url":"odomouse_amd64.deb","sha256":"…"}}}

use crate::json::{parse, Json};

/// The app version, from VERSION at the repository root (one place for all OSes).
pub fn version() -> &'static str {
    include_str!("../../VERSION").trim()
}

/// Where the shells look for a new version. When the site moves to its own
/// domain, GitHub Pages redirects this address there.
pub const FEED_URL: &str = "https://foziljonsap.github.io/odomouse/download/latest.json";
/// The download page, for when a direct file cannot be used.
pub const PAGE_URL: &str = "https://foziljonsap.github.io/odomouse/";

pub const DAY_MS: i64 = 24 * 3600 * 1000;
/// First check this long after start, so it never slows the launch.
pub const FIRST_CHECK_AFTER_MS: i64 = 60 * 1000;
/// After a failed download, try again in an hour.
pub const RETRY_MS: i64 = 3600 * 1000;

/// This build's OS, as named in latest.json.
pub fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub version: String,
    /// Absolute https address of this OS's installer.
    pub url: String,
    /// Lower-case hex SHA-256 of the installer ("" if the feed has none).
    pub sha256: String,
}

impl Update {
    pub fn to_json(&self) -> Json {
        Json::obj()
            .set("version", self.version.as_str())
            .set("url", self.url.as_str())
            .set("sha256", self.sha256.as_str())
            .set("page", PAGE_URL)
    }
}

/// "1.10.0" > "1.9.3"; missing parts count as 0; anything after a '-' or '+' is ignored.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    fn parts(v: &str) -> Option<Vec<u64>> {
        let core = v.trim().trim_start_matches('v').split(['-', '+']).next()?;
        if core.is_empty() {
            return None;
        }
        core.split('.').map(|p| p.parse::<u64>().ok()).collect()
    }
    match (parts(candidate), parts(current)) {
        (Some(mut a), Some(mut b)) => {
            let n = a.len().max(b.len());
            a.resize(n, 0);
            b.resize(n, 0);
            a > b
        }
        _ => false,
    }
}

/// Resolve a file name from the feed against the feed's own address.
fn resolve(feed_url: &str, file: &str) -> Option<String> {
    if file.starts_with("https://") {
        return Some(file.to_string());
    }
    if file.contains("://") || file.starts_with('/') || file.contains("..") || file.is_empty() {
        return None; // only https, and only next to the feed
    }
    let base = &feed_url[..feed_url.rfind('/')? + 1];
    Some(format!("{}{}", base, file))
}

/// Parse latest.json. Returns the update for `platform` if it is newer than `current`.
pub fn parse_feed(body: &str, feed_url: &str, platform: &str, current: &str) -> Result<Option<Update>, String> {
    let j = parse(body.trim_start_matches('\u{feff}')).map_err(|e| format!("bad feed: {}", e))?;
    let version = j.get("version").and_then(Json::as_str).ok_or("feed has no version")?;
    if !is_newer(version, current) {
        return Ok(None);
    }
    let file = j.get("files").and_then(|f| f.get(platform));
    let url = file
        .and_then(|f| f.get("url"))
        .and_then(Json::as_str)
        .and_then(|u| resolve(feed_url, u))
        .unwrap_or_else(|| PAGE_URL.to_string());
    let sha256 = file
        .and_then(|f| f.get("sha256"))
        .and_then(Json::as_str)
        .filter(|h| h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()))
        .map(|h| h.to_ascii_lowercase())
        .unwrap_or_default();
    Ok(Some(Update { version: version.to_string(), url, sha256 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number() {
        assert!(is_newer("1.1.0", "1.0.0"));
        assert!(is_newer("1.10.0", "1.9.9"));
        assert!(is_newer("2", "1.9"));
        assert!(is_newer("v1.0.1", "1.0.0"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("1.0", "1.0.0"));
        assert!(!is_newer("0.9.9", "1.0.0"));
        assert!(!is_newer("abc", "1.0.0"));
        assert!(!is_newer("", "1.0.0"));
    }

    #[test]
    fn the_version_file_is_a_version() {
        assert!(is_newer(version(), "0.0.1"), "VERSION = {:?}", version());
        // the Rust crates carry the same number (Windows and macOS read VERSION when building)
        for toml in [include_str!("../Cargo.toml"), include_str!("../../linux/Cargo.toml")] {
            let line = toml.lines().find(|l| l.starts_with("version")).unwrap();
            assert_eq!(line, format!("version = \"{}\"", version()));
        }
    }

    #[test]
    fn feed_gives_this_os_file() {
        let feed = "https://example.org/odomouse/download/latest.json";
        let body = r#"{"version":"9.0.0","files":{"windows":{"url":"Odomouse-Setup.exe","sha256":"ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef0123456789"},"linux":{"url":"../evil.deb"}}}"#;
        let u = parse_feed(body, feed, "windows", "1.0.0").unwrap().unwrap();
        assert_eq!(u.version, "9.0.0");
        assert_eq!(u.url, "https://example.org/odomouse/download/Odomouse-Setup.exe");
        assert_eq!(u.sha256, "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789");
        // a path outside the folder, or no entry: the download page instead
        let l = parse_feed(body, feed, "linux", "1.0.0").unwrap().unwrap();
        assert_eq!(l.url, PAGE_URL);
        assert_eq!(l.sha256, "");
        let m = parse_feed(body, feed, "mac", "1.0.0").unwrap().unwrap();
        assert_eq!(m.url, PAGE_URL);
        assert_eq!(parse_feed(body, feed, "mac", "9.0.0").unwrap(), None);
        assert!(parse_feed("<html>", feed, "mac", "1.0.0").is_err());
        assert!(parse_feed("{}", feed, "mac", "1.0.0").is_err());
        let http = r#"{"version":"9.0.0","files":{"mac":{"url":"http://x.org/a.dmg"}}}"#;
        assert_eq!(parse_feed(http, feed, "mac", "1.0.0").unwrap().unwrap().url, PAGE_URL);
    }
}
