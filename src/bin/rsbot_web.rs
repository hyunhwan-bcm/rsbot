//! A small web chat for rsbot: a page to talk to the bot, one conversation per browser tab.
//!
//!     rsbot_web [--port 8787] [--host 127.0.0.1] [--seed 0] [--log-dir logs | --no-log]
//!
//! Standard library only (no web framework): a single thread accepts one request at a time,
//! which is also what keeps each conversation exact. PyBot's random generator is one global
//! state, so every conversation keeps its own copy, swapped in for its turn and saved after it,
//! as if each conversation ran alone in its own `pybot.py`.
//!
//! Routes:
//!   GET  /                      the chat page
//!   POST /api/chat              {"session": id, "text": line} -> {"reply", "module", "ms", "turn"}
//!   POST /api/reset             {"session": id} -> {"ok": true}
//!   GET  /api/health            {"ok": true, "sessions": n, "log": dir or null}
//!
//! Conversation logs: every turn is appended, as one JSON line, to `<log-dir>/rsbot-YYYY-MM-DD.jsonl`
//! (one file per UTC day, written and flushed as each reply is sent):
//!   {"time": "2026-11-18T23:05:12.345Z", "session": id, "turn": 3, "user": line,
//!    "reply": text, "module": name or null, "ms": 41.2}
//! and a "New chat" is logged as {"time", "session", "event": "reset"}. `--no-log` turns logging off.

use rsbot::py::random::{self, Random};
use rsbot::runtime::answer;
use rsbot::state::Memory;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const PAGE: &str = include_str!("../web/index.html");
/// Longest line the page accepts, in bytes (a pasted essay is fine; a file is not).
const MAX_BODY: usize = 64 * 1024;
/// Conversations kept in memory; the least recently used is dropped past this.
const MAX_SESSIONS: usize = 1000;

/// One conversation: PyBot's memory dict, its random generator, and when it was last used.
struct Session {
    memory: Memory,
    rng: Random,
    turns: usize,
    last_used: Instant,
}

struct Server {
    sessions: HashMap<String, Session>,
    seed: i128,
    log: Option<ConvLog>,
}

/// The conversation log: JSON lines appended to one file per UTC day in a directory.
struct ConvLog {
    dir: PathBuf,
    day: String,
    file: Option<std::fs::File>,
}

impl ConvLog {
    fn new(dir: PathBuf) -> std::io::Result<ConvLog> {
        std::fs::create_dir_all(&dir)?;
        Ok(ConvLog { dir, day: String::new(), file: None })
    }

    /// Append one record, stamped with the time; a write error is reported and the chat goes on.
    fn write(&mut self, mut record: serde_json::Value) {
        let now = chrono::Utc::now();
        let day = now.format("%Y-%m-%d").to_string();
        if self.file.is_none() || day != self.day {
            let path = self.dir.join(format!("rsbot-{day}.jsonl"));
            match std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                Ok(f) => self.file = Some(f),
                Err(e) => {
                    eprintln!("conversation log: cannot open {}: {e}", path.display());
                    return;
                }
            }
            self.day = day;
        }
        record["time"] = serde_json::json!(now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());
        let mut line = record.to_string();
        line.push('\n');
        if let Some(f) = self.file.as_mut() {
            if let Err(e) = f.write_all(line.as_bytes()).and_then(|_| f.flush()) {
                eprintln!("conversation log: write failed: {e}");
            }
        }
    }
}

impl Server {
    fn chat(&mut self, id: &str, text: &str) -> serde_json::Value {
        if !self.sessions.contains_key(id) {
            if self.sessions.len() >= MAX_SESSIONS {
                let oldest = self.sessions.iter().min_by_key(|(_, s)| s.last_used).map(|(k, _)| k.clone());
                if let Some(k) = oldest {
                    self.sessions.remove(&k);
                }
            }
            // a fresh conversation starts as `pybot.py` does: empty memory, seeded generator
            random::seed(self.seed);
            self.sessions.insert(
                id.to_string(),
                Session { memory: Memory::new(), rng: random::getstate(), turns: 0, last_used: Instant::now() },
            );
        }
        let s = self.sessions.get_mut(id).expect("session exists");
        random::setstate(s.rng.clone());
        let t = Instant::now();
        let mut memory = std::mem::take(&mut s.memory);
        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (reply, module) = answer(text, &mut memory);
            (reply, module, memory)
        }));
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let (reply, module) = match out {
            Ok((reply, module, memory)) => {
                s.memory = memory;
                (reply, module)
            }
            // the runtime already treats a module's failure as Python does; this is the last net
            Err(_) => ("Hello!".to_string(), None),
        };
        s.rng = random::getstate();
        s.turns += 1;
        s.last_used = Instant::now();
        let ms = (ms * 10.0).round() / 10.0;
        let turn = s.turns;
        if let Some(log) = self.log.as_mut() {
            log.write(serde_json::json!({
                "session": id, "turn": turn, "user": text, "reply": reply, "module": module, "ms": ms
            }));
        }
        serde_json::json!({"reply": reply, "module": module, "ms": ms, "turn": turn})
    }
}

/// A parsed request: method, path and body.
struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.split('?').next()?.to_string();
    let mut length = 0usize;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).ok()? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().ok()?;
            }
        }
    }
    if length > MAX_BODY {
        return None;
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some(Request { method, path, body })
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn json(stream: &mut TcpStream, status: &str, v: &serde_json::Value) {
    respond(stream, status, "application/json", v.to_string().as_bytes());
}

fn field<'a>(v: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(|x| x.as_str())
}

fn main() {
    let mut port = std::env::var("RSBOT_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787u16);
    let mut host = "127.0.0.1".to_string();
    let mut seed = 0i128;
    let mut log_dir: Option<PathBuf> = Some(PathBuf::from("logs"));
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" => port = args.next().and_then(|p| p.parse().ok()).expect("--port N"),
            "--host" => host = args.next().expect("--host ADDR"),
            "--seed" => seed = args.next().and_then(|s| s.parse().ok()).expect("--seed N"),
            "--log-dir" => log_dir = Some(PathBuf::from(args.next().expect("--log-dir DIR"))),
            "--no-log" => log_dir = None,
            _ => {
                eprintln!("usage: rsbot_web [--port 8787] [--host 127.0.0.1] [--seed 0] [--log-dir logs | --no-log]");
                std::process::exit(2);
            }
        }
    }
    rsbot::runtime::quiet_panics();
    let t = Instant::now();
    rsbot::data::load();
    eprintln!("loaded PyBot's data in {:.2}s", t.elapsed().as_secs_f64());
    let listener = match TcpListener::bind((host.as_str(), port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot listen on {host}:{port}: {e} (another program may be using the port; try --port)");
            std::process::exit(1);
        }
    };
    eprintln!("rsbot web chat on http://{host}:{port}/");
    let log = match log_dir {
        Some(dir) => match ConvLog::new(dir.clone()) {
            Ok(l) => {
                eprintln!("conversation logs in {}/", dir.display());
                Some(l)
            }
            Err(e) => {
                eprintln!("cannot create log directory {}: {e}", dir.display());
                std::process::exit(1);
            }
        },
        None => {
            eprintln!("conversation logging is off");
            None
        }
    };
    let mut server = Server { sessions: HashMap::new(), seed, log };
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let Some(req) = read_request(&mut stream) else {
            respond(&mut stream, "400 Bad Request", "text/plain", b"bad request");
            continue;
        };
        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/") | ("GET", "/index.html") => respond(&mut stream, "200 OK", "text/html; charset=utf-8", PAGE.as_bytes()),
            ("GET", "/api/health") => json(&mut stream, "200 OK", &serde_json::json!({"ok": true, "sessions": server.sessions.len(), "log": server.log.as_ref().map(|l| l.dir.display().to_string())})),
            ("POST", "/api/chat") => {
                let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap_or_default();
                match (field(&body, "session"), field(&body, "text")) {
                    (Some(id), Some(text)) if !id.is_empty() && id.len() <= 64 => {
                        let out = server.chat(id, text);
                        json(&mut stream, "200 OK", &out)
                    }
                    _ => json(&mut stream, "400 Bad Request", &serde_json::json!({"error": "need session and text"})),
                }
            }
            ("POST", "/api/reset") => {
                let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap_or_default();
                if let Some(id) = field(&body, "session") {
                    if server.sessions.remove(id).is_some() {
                        if let Some(log) = server.log.as_mut() {
                            log.write(serde_json::json!({"session": id, "event": "reset"}));
                        }
                    }
                }
                json(&mut stream, "200 OK", &serde_json::json!({"ok": true}))
            }
            _ => respond(&mut stream, "404 Not Found", "text/plain", b"not found"),
        }
    }
}
