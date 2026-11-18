//! Chat with rsbot in a terminal, with `pybot.py`'s command line.
//!
//!     rsbot                         interactive chat
//!     rsbot -p "your message"       one reply, then exit
//!     echo "your message" | rsbot   one reply per input line
//!     rsbot --jsonl IN OUT          benchmark mode: one fresh conversation per JSONL row
use rsbot::py::random;
use rsbot::runtime::answer;
use rsbot::state::Memory;
use std::io::{BufRead, IsTerminal, Write};
use std::time::Instant;

const LOGO: &str = r"    ____        ____        __
   / __ \_____/ __ )____  / /_
  / /_/ / ___/ __  / __ \/ __/
 / _, _(__  ) /_/ / /_/ / /_
/_/ |_/____/_____/\____/\__/";
const TAGLINE: &str = "PyBot's rules, in Rust. No weights, no training, just rules.";
const HINT: &str = "Type a message and press Enter. /reset starts a new conversation, /exit quits.";

struct Args {
    prompt: Option<String>,
    interactive: bool,
    seed: Option<i128>,
    verbose: bool,
    jsonl: Option<(String, String)>,
}

fn parse() -> Args {
    let mut a = Args { prompt: None, interactive: false, seed: None, verbose: false, jsonl: None };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        match x.as_str() {
            "-p" | "--prompt" => a.prompt = it.next(),
            "-i" | "--interactive" => a.interactive = true,
            "-s" | "--seed" => a.seed = it.next().and_then(|s| s.parse().ok()),
            "-v" | "--verbose" => a.verbose = true,
            "--jsonl" => a.jsonl = Some((it.next().unwrap(), it.next().unwrap())),
            "--version" => {
                println!("rsbot {} (port of pybot at commit 24cbc0439e07)", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-h" | "--help" => {
                println!("usage: rsbot [-p TEXT] [-i] [-s N] [-v] [--jsonl IN OUT] [--version]");
                std::process::exit(0);
            }
            other => {
                eprintln!("rsbot: unknown argument {other}");
                std::process::exit(2);
            }
        }
    }
    a
}

fn details(module: Option<&str>, secs: f64) -> String {
    format!("[module: {} | {:.0} ms]", module.unwrap_or("none"), secs * 1000.0)
}

fn one(text: &str, memory: &mut Memory) -> (String, Option<&'static str>, f64) {
    let t = Instant::now();
    let (reply, module) = answer(text, memory);
    (reply, module, t.elapsed().as_secs_f64())
}

fn jsonl(inp: &str, out: &str, seed: i128) {
    let t0 = Instant::now();
    rsbot::data::load();
    eprintln!("loaded data in {:.2}s", t0.elapsed().as_secs_f64());
    let mut w = std::io::BufWriter::new(std::fs::File::create(out).unwrap());
    let mut n = 0;
    for line in std::io::BufReader::new(std::fs::File::open(inp).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let text = row["instruction"].as_str().unwrap_or("");
        random::seed(seed);
        let mut memory = Memory::new();
        let (reply, module, secs) = one(text, &mut memory);
        let rec = serde_json::json!({"id": row["id"], "reply": reply, "module": module, "ms": (secs * 10000.0).round() / 10.0});
        writeln!(w, "{rec}").unwrap();
        n += 1;
    }
    eprintln!("done {n} rows in {:.1}s", t0.elapsed().as_secs_f64());
}

fn main() {
    let args = parse();
    if let Some((inp, out)) = &args.jsonl {
        jsonl(inp, out, args.seed.unwrap_or(0));
        return;
    }
    if let Some(s) = args.seed {
        random::seed(s);
    }
    let stdin = std::io::stdin();
    let interactive = (args.prompt.is_none() && stdin.is_terminal()) || (args.prompt.is_some() && args.interactive);
    rsbot::data::load();
    if !interactive {
        if let Some(p) = &args.prompt {
            let (reply, module, secs) = one(p, &mut Memory::new());
            println!("{reply}");
            if args.verbose {
                eprintln!("{}", details(module, secs));
            }
        } else {
            let mut memory = Memory::new();
            for line in stdin.lock().lines() {
                let (reply, module, secs) = one(line.unwrap().trim_end_matches(['\r', '\n']), &mut memory);
                println!("{reply}");
                if args.verbose {
                    eprintln!("{}", details(module, secs));
                }
            }
        }
        return;
    }
    let opening = || {
        println!("{LOGO}\n");
        println!("{TAGLINE}");
        println!("{HINT}\n");
    };
    opening();
    let mut memory = Memory::new();
    let mut first = args.prompt.clone();
    let mut lines = stdin.lock().lines();
    loop {
        let text = match first.take() {
            Some(t) => {
                println!("user: {t}");
                t
            }
            None => {
                print!("user: ");
                std::io::stdout().flush().ok();
                match lines.next() {
                    Some(Ok(l)) => l,
                    _ => {
                        println!();
                        break;
                    }
                }
            }
        };
        let command = text.trim().to_lowercase();
        match command.as_str() {
            "/exit" | "/quit" | "/bye" => break,
            "/reset" => {
                memory = Memory::new();
                if let Some(s) = args.seed {
                    random::seed(s);
                }
                print!("\x1b[H\x1b[2J\x1b[3J");
                opening();
                continue;
            }
            "/help" => {
                println!("Commands:\n  /reset   clear the screen and start again\n  /help    show this help\n  /exit    quit (Ctrl+D works too)\n");
                continue;
            }
            "" => continue,
            _ => {}
        }
        let (reply, module, secs) = one(&text, &mut memory);
        println!("rsbot: {reply}");
        if args.verbose {
            println!("{}", details(module, secs));
        }
        println!();
    }
}
