//! New versions on Linux. Downloads go through the system's own tools
//! (curl, wget or python3; one of them is on every desktop distro), so the
//! app needs no TLS library. Only a public file is fetched; nothing is sent.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Save `url` (https only) to `out`.
pub fn download(url: &str, out: &Path) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("https only".into());
    }
    let _ = std::fs::remove_file(out);
    let o = out.to_string_lossy().to_string();
    let py = "import sys,urllib.request as u\n\
              r=u.urlopen(u.Request(sys.argv[1],headers={'User-Agent':'Odomouse','Cache-Control':'no-cache'}),timeout=120)\n\
              open(sys.argv[2],'wb').write(r.read())";
    let tries: [(&str, Vec<&str>); 3] = [
        ("curl", vec!["-fsSL", "--proto", "=https", "--max-time", "300", "-A", "Odomouse", "-H", "Cache-Control: no-cache", "-o", &o, url]),
        ("wget", vec!["-q", "--https-only", "-T", "120", "-U", "Odomouse", "-O", &o, url]),
        ("python3", vec!["-c", py, url, &o]),
    ];
    let mut last = String::from("no download tool (curl, wget or python3)");
    for (tool, args) in tries.iter() {
        match Command::new(tool).args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status() {
            Ok(s) if s.success() && out.exists() => return Ok(()),
            Ok(s) => last = format!("{} failed ({})", tool, s),
            Err(_) => continue, // not installed: try the next one
        }
    }
    let _ = std::fs::remove_file(out);
    Err(last)
}

/// The text of a small file (latest.json).
pub fn fetch_text(url: &str, scratch: &Path) -> Result<String, String> {
    let _ = std::fs::create_dir_all(scratch);
    let file = scratch.join(format!("latest-{}.json", std::process::id()));
    download(url, &file)?;
    let text = std::fs::read_to_string(&file).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&file);
    text
}

/// Lower-case hex SHA-256 of a file, by coreutils' sha256sum.
fn sha256_of(file: &Path) -> Option<String> {
    let out = Command::new("sha256sum").arg(file).stderr(Stdio::null()).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let hex = text.split_whitespace().next()?.to_ascii_lowercase();
    (hex.len() == 64).then_some(hex)
}

/// Download the new .deb into `dir` and check it. Returns its path.
pub fn download_package(url: &str, sha256: &str, version: &str, dir: &Path) -> Result<PathBuf, String> {
    if sha256.len() != 64 {
        return Err("no checksum".into());
    }
    let safe: String = version.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-').collect();
    let _ = std::fs::create_dir_all(dir);
    let file = dir.join(format!("odomouse_{}_amd64.deb", if safe.is_empty() { "new" } else { &safe }));
    let part = file.with_extension("part");
    download(url, &part)?;
    if sha256_of(&part).as_deref() != Some(&sha256.to_ascii_lowercase()) {
        let _ = std::fs::remove_file(&part);
        return Err("checksum mismatch".into());
    }
    std::fs::rename(&part, &file).map_err(|e| e.to_string())?;
    Ok(file)
}
