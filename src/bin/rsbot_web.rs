//! A small web chat for rsbot: a page to talk to the bot, one conversation per browser tab.
//!
//!     rsbot_web [--port 8787] [--host 127.0.0.1] [--seed 0]
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
//!   GET  /api/health            {"ok": true, "sessions": n}

use rsbot::py::random::{self, Random};
use rsbot::runtime::answer;
use rsbot::state::Memory;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
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
        serde_json::json!({"reply": reply, "module": module, "ms": (ms * 10.0).round() / 10.0, "turn": s.turns})
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
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" => port = args.next().and_then(|p| p.parse().ok()).expect("--port N"),
            "--host" => host = args.next().expect("--host ADDR"),
            "--seed" => seed = args.next().and_then(|s| s.parse().ok()).expect("--seed N"),
            _ => {
                eprintln!("usage: rsbot_web [--port 8787] [--host 127.0.0.1] [--seed 0]");
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
    let mut server = Server { sessions: HashMap::new(), seed };
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let Some(req) = read_request(&mut stream) else {
            respond(&mut stream, "400 Bad Request", "text/plain", b"bad request");
            continue;
        };
        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/") | ("GET", "/index.html") => respond(&mut stream, "200 OK", "text/html; charset=utf-8", PAGE.as_bytes()),
            ("GET", "/api/health") => json(&mut stream, "200 OK", &serde_json::json!({"ok": true, "sessions": server.sessions.len()})),
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
                    server.sessions.remove(id);
                }
                json(&mut stream, "200 OK", &serde_json::json!({"ok": true}))
            }
            _ => respond(&mut stream, "404 Not Found", "text/plain", b"not found"),
        }
    }
}
