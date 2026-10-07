//! The dashboard on Linux opens in the user's browser (an app window when
//! Chromium/Chrome is installed), served from 127.0.0.1 by this tiny HTTP
//! server. No web engine lives inside the app, so it stays a few MB.
//!
//! Security: listens on loopback only; the Host header must be ours (no DNS
//! rebinding); /api, /events and /boot.js need a random per-run token, which
//! the opened URL carries once and the page then keeps as an HttpOnly,
//! SameSite=Strict cookie.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub type ApiHandler = dyn Fn(&str) -> String + Send + Sync;
pub type BootScript = dyn Fn() -> String + Send + Sync;

pub struct Server {
    pub port: u16,
    token: String,
    root: PathBuf,
    sse: Mutex<Vec<TcpStream>>,
    api: Box<ApiHandler>,
    boot: Box<BootScript>,
}

const MAX_HEADER: usize = 64 * 1024;
const MAX_BODY: usize = 40 * 1024 * 1024; // a Wrapped PNG as a data URL

pub fn random_token() -> String {
    let mut b = [0u8; 16];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = f.read_exact(&mut b);
    }
    if b.iter().all(|x| *x == 0) {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
        b[..16].copy_from_slice(&t.to_le_bytes());
    }
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

struct Request {
    method: String,
    path: String,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn parse_query(q: &str) -> HashMap<String, String> {
    q.split('&')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            if k.is_empty() { None } else { Some((k.to_string(), v.to_string())) }
        })
        .collect()
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok()?;
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let mut headers = HashMap::new();
    let mut total = line.len();
    loop {
        let mut h = String::new();
        let n = reader.read_line(&mut h).ok()?;
        total += n;
        if n == 0 || total > MAX_HEADER {
            return None;
        }
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let len: usize = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    if len > MAX_BODY {
        return None;
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    let (path, q) = target.split_once('?').unwrap_or((&target, ""));
    Some(Request { method, path: path.to_string(), query: parse_query(q), headers, body })
}

fn content_type(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn respond(stream: &mut TcpStream, status: &str, ctype: &str, extra: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n{extra}\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

impl Server {
    pub fn start(root: PathBuf, api: Box<ApiHandler>, boot: Box<BootScript>) -> Option<Arc<Server>> {
        let listener = TcpListener::bind("127.0.0.1:0").ok()?;
        let port = listener.local_addr().ok()?.port();
        let server = Arc::new(Server { port, token: random_token(), root, sse: Mutex::new(Vec::new()), api, boot });
        let s = server.clone();
        std::thread::Builder::new()
            .name("http".into())
            .spawn(move || {
                for stream in listener.incoming().flatten() {
                    let s = s.clone();
                    std::thread::spawn(move || s.handle(stream));
                }
            })
            .ok()?;
        Some(server)
    }

    /// Address to open in the browser (carries the token once).
    pub fn url(&self, page: &str, query: &str) -> String {
        let extra = if query.is_empty() { String::new() } else { format!("&{}", query) };
        format!("http://127.0.0.1:{}/ui/{}?token={}{}", self.port, page, self.token, extra)
    }

    pub fn has_clients(&self) -> bool {
        !self.sse.lock().unwrap().is_empty()
    }

    /// Push an event to every open page: window.__odomouseEmit(name, payload).
    pub fn emit(&self, name: &str, payload_json: &str) {
        let msg = format!("data: {{\"name\":\"{}\",\"payload\":{}}}\n\n", name, payload_json);
        self.sse.lock().unwrap().retain_mut(|s| s.write_all(msg.as_bytes()).is_ok());
    }

    /// Comment line so dead connections are noticed and dropped.
    pub fn keepalive(&self) {
        self.sse.lock().unwrap().retain_mut(|s| s.write_all(b": ping\n\n").is_ok());
    }

    fn authorized(&self, req: &Request) -> bool {
        let cookie_ok = req.headers.get("cookie").map_or(false, |c| {
            c.split(';').any(|kv| kv.trim() == format!("odomouse_token={}", self.token))
        });
        cookie_ok || req.query.get("token") == Some(&self.token)
    }

    fn handle(&self, mut stream: TcpStream) {
        let Some(req) = read_request(&mut stream) else { return };
        let host_ok = req.headers.get("host").map_or(false, |h| {
            h == &format!("127.0.0.1:{}", self.port) || h == &format!("localhost:{}", self.port)
        });
        if !host_ok {
            respond(&mut stream, "403 Forbidden", "text/plain", "", b"forbidden");
            return;
        }
        let authed = self.authorized(&req);
        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/boot.js") if authed => {
                respond(&mut stream, "200 OK", "application/javascript; charset=utf-8", "", (self.boot)().as_bytes());
            }
            ("POST", "/api") if authed => {
                let body = String::from_utf8_lossy(&req.body);
                let out = (self.api)(&body);
                respond(&mut stream, "200 OK", "application/json", "", out.as_bytes());
            }
            ("GET", "/events") if authed => {
                let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: keep-alive\r\n\r\n: hi\n\n";
                if stream.write_all(head.as_bytes()).is_ok() {
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                    self.sse.lock().unwrap().push(stream);
                }
            }
            ("GET", "/") => {
                respond(&mut stream, "302 Found", "text/plain", "Location: /ui/dashboard.html\r\n", b"");
            }
            ("GET", p) if p.starts_with("/ui/") || p.starts_with("/core/") || p.starts_with("/assets/") => self.static_file(&mut stream, &req, authed),
            ("GET", _) | ("POST", _) => respond(&mut stream, "404 Not Found", "text/plain", "", b"not found"),
            _ => respond(&mut stream, "405 Method Not Allowed", "text/plain", "", b""),
        }
    }

    fn static_file(&self, stream: &mut TcpStream, req: &Request, authed: bool) {
        let rel = req.path.trim_start_matches('/');
        let safe = rel.split('/').all(|seg| !seg.is_empty() && seg != ".." && seg.chars().all(|c| c.is_ascii_alphanumeric() || "._-@".contains(c)));
        let path = self.root.join(rel);
        let Ok(mut body) = (if safe { std::fs::read(&path) } else { Err(std::io::ErrorKind::NotFound.into()) }) else {
            respond(stream, "404 Not Found", "text/plain", "", b"not found");
            return;
        };
        let mut extra = String::new();
        if rel.ends_with(".html") {
            // the native-bridge needs /boot.js first (inline scripts are blocked by the page's CSP)
            let html = String::from_utf8_lossy(&body).replace(
                "<script src=\"native-bridge.js\"></script>",
                "<script src=\"/boot.js\"></script>\n  <script src=\"native-bridge.js\"></script>",
            );
            body = html.into_bytes();
            if authed {
                extra = format!("Set-Cookie: odomouse_token={}; HttpOnly; SameSite=Strict; Path=/\r\n", self.token);
            }
        }
        respond(stream, "200 OK", content_type(rel), &extra, &body);
    }
}
