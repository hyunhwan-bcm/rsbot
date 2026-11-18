//! Check the port against PyBot, stage by stage, on the benchmark prompts.
//!
//!     oracle_check understood [gold|heldout] [-n N] [--show K]
//!     oracle_check module NAME [gold|heldout] [--show K]   (memory from PyBot's own reading)
//!     oracle_check arbiter [gold|heldout] [--show K]       (candidates from PyBot)
//!     oracle_check e2e [gold|heldout] [--show K]           (the whole turn)
//!     oracle_check all [gold|heldout]
//!
//! `module` and `arbiter` feed each stage PyBot's own inputs, so a workstream can be checked
//! before the stages in front of it are ported. Set RSBOT_ORACLE_UNDERSTAND=1 to have
//! `understanding::understand` read PyBot's results for texts it has seen.
use rsbot::modules::registry;
use rsbot::oracle::{load, Row};
use rsbot::py::random;
use rsbot::runtime::{guarded, quiet_panics, Candidate};
use rsbot::state::Memory;
use std::collections::BTreeMap;
use std::time::Instant;

struct Opts {
    split: String,
    show: usize,
    limit: Option<usize>,
}

fn memory_from(row: &Row) -> Memory {
    let mut m = Memory::new();
    m.understood = Some(row.understood.clone());
    m.intent_wants = row.intent_wants.clone();
    m
}

fn short(s: &str) -> String {
    let t: String = s.chars().take(220).collect();
    if t.len() < s.len() { format!("{t}...") } else { t }
}

fn check_understood(rows: &[Row], o: &Opts) -> (usize, usize) {
    let mut field_bad: BTreeMap<String, usize> = BTreeMap::new();
    let (mut ok, mut shown) = (0, 0);
    for row in rows {
        random::seed(0);
        let mut m = Memory::new();
        for def in registry() {
            if let Some(obs) = def.observe {
                let _ = guarded(|| obs(&row.text, &mut m));
            }
        }
        let want = serde_json::to_value(&row.understood).unwrap();
        let got = match &m.understood {
            Some(u) => serde_json::to_value(u).unwrap(),
            None => serde_json::Value::Null,
        };
        let mut same = m.intent_wants == row.intent_wants;
        if !same {
            *field_bad.entry("(intent_wants)".into()).or_default() += 1;
        }
        if let (Some(w), Some(g)) = (want.as_object(), got.as_object()) {
            for (k, wv) in w {
                if g.get(k) != Some(wv) {
                    same = false;
                    *field_bad.entry(k.clone()).or_default() += 1;
                    if shown < o.show {
                        println!("-- {} field {k}\n   text: {}\n   rust: {}\n   py:   {}", row.id, short(&row.text), short(&g.get(k).map(|v| v.to_string()).unwrap_or("<missing>".into())), short(&wv.to_string()));
                        shown += 1;
                    }
                }
            }
        } else {
            same = false;
            *field_bad.entry("(no reading)".into()).or_default() += 1;
        }
        if same {
            ok += 1;
        }
    }
    if !field_bad.is_empty() {
        let mut v: Vec<_> = field_bad.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        println!("fields that differ (rows): {}", v.iter().map(|(k, n)| format!("{k}={n}")).collect::<Vec<_>>().join(" "));
    }
    (ok, rows.len())
}

fn check_module(name: &str, rows: &[Row], o: &Opts) -> (usize, usize) {
    let def = registry().iter().find(|d| d.name == name).unwrap_or_else(|| panic!("no module {name}"));
    let respond = def.respond.expect("module has no respond");
    let (mut ok, mut shown) = (0, 0);
    for row in rows {
        random::seed(0);
        let mut m = memory_from(row);
        let got = guarded(|| respond(&row.text, &mut m)).flatten().filter(|s| !rsbot::py::text::strip(s).is_empty());
        let want = row.candidates.get(name).cloned().flatten();
        if got == want {
            ok += 1;
        } else if shown < o.show {
            println!("-- {} {}\n   text: {}\n   rust: {}\n   py:   {}", row.id, name, short(&row.text), got.as_deref().map(short).unwrap_or("None".into()), want.as_deref().map(short).unwrap_or("None".into()));
            shown += 1;
        }
    }
    (ok, rows.len())
}

fn check_arbiter(rows: &[Row], o: &Opts) -> (usize, usize) {
    let (mut ok, mut n, mut shown) = (0, 0, 0);
    for row in rows {
        if row.order.len() < 2 {
            continue;
        }
        n += 1;
        let cands: Vec<Candidate> = row
            .order
            .iter()
            .map(|name| {
                let def = registry().iter().find(|d| d.name == name).unwrap();
                Candidate { module: def.name, priority: def.priority, reply: row.candidates[name].clone().unwrap() }
            })
            .collect();
        random::seed(0);
        let m = memory_from(row);
        let i = guarded(|| rsbot::modules::arbiter::choose(&row.text, &m, &cands)).unwrap_or(0);
        let got = cands.get(i).map(|c| c.module.to_string());
        if got == row.chosen {
            ok += 1;
        } else if shown < o.show {
            println!("-- {} picked {:?}, PyBot picked {:?} from {:?}\n   text: {}", row.id, got, row.chosen, row.order, short(&row.text));
            shown += 1;
        }
    }
    (ok, n)
}

fn check_e2e(rows: &[Row], o: &Opts) -> (usize, usize) {
    let (mut ok, mut shown) = (0, 0);
    for row in rows {
        random::seed(0);
        let mut m = Memory::new();
        let (reply, module) = rsbot::runtime::answer(&row.text, &mut m);
        if reply == row.reply {
            ok += 1;
        } else if shown < o.show {
            println!("-- {} rust[{}] py[{}]\n   text: {}\n   rust: {}\n   py:   {}", row.id, module.unwrap_or("none"), row.chosen.as_deref().unwrap_or("none"), short(&row.text), short(&reply), short(&row.reply));
            shown += 1;
        }
    }
    (ok, rows.len())
}

fn main() {
    quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut o = Opts { split: "gold".into(), show: 5, limit: None };
    let mut pos = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--show" => {
                o.show = args[i + 1].parse().unwrap();
                i += 1;
            }
            "-n" => {
                o.limit = Some(args[i + 1].parse().unwrap());
                i += 1;
            }
            "gold" | "heldout" => o.split = args[i].clone(),
            a => pos.push(a.to_string()),
        }
        i += 1;
    }
    let mut rows = load(&o.split);
    if let Some(n) = o.limit {
        rows.truncate(n);
    }
    rsbot::data::load();
    let t = Instant::now();
    let report = |stage: &str, (ok, n): (usize, usize)| {
        println!("{stage:28} {ok:5}/{n:<5} identical ({:.1}%)", 100.0 * ok as f64 / n.max(1) as f64);
    };
    match pos.first().map(String::as_str) {
        Some("understood") => report("understood", check_understood(&rows, &o)),
        Some("module") => {
            let name = pos.get(1).expect("module NAME");
            report(&format!("module {name}"), check_module(name, &rows, &o))
        }
        Some("arbiter") => report("arbiter", check_arbiter(&rows, &o)),
        Some("e2e") => report("e2e", check_e2e(&rows, &o)),
        Some("all") | None => {
            let quiet = Opts { show: 0, split: o.split.clone(), limit: o.limit };
            report("understood", check_understood(&rows, &quiet));
            for def in registry() {
                if def.respond.is_some() {
                    report(&format!("module {}", def.name), check_module(def.name, &rows, &quiet));
                }
            }
            report("arbiter", check_arbiter(&rows, &quiet));
            report("e2e", check_e2e(&rows, &quiet));
        }
        Some(x) => panic!("unknown stage {x}"),
    }
    eprintln!("[{} split, {:.1}s]", o.split, t.elapsed().as_secs_f64());
}
