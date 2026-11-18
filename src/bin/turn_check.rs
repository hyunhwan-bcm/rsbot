//! Check every stage of every turn against the full PyBot (`harness/turn_oracle.py` output).
//!
//!     turn_check ORACLE.jsonl.gz [--module NAME]... [--stage observe|respond|arbiter|after] [--show K] [--limit N]
//!     turn_check ORACLE.jsonl.gz --e2e [--show K]     whole conversations from scratch, reply by reply
//!
//! Each stage gets exactly PyBot's input for that stage (memory and global RNG state), so a
//! difference is pinned to one module on one turn. Per module it reports: identical (reply and
//! memory changes), different, and how many differences came from a stub that is not ported yet.
use flate2::read::GzDecoder;
use rsbot::modules::registry;
use rsbot::py::random::{self, Random};
use rsbot::runtime::{guarded, quiet_panics, take_last_panic, Candidate};
use rsbot::state::Memory;
use serde_json::{Map, Value as J};
use std::collections::BTreeMap;
use std::io::BufRead;

#[derive(Default)]
struct Tally {
    ok: usize,
    bad: usize,
    unported: BTreeMap<String, usize>,
    shown: Vec<String>,
}

fn rng_set(state: &J) {
    if let Some(a) = state.as_array() {
        let words: Vec<u64> = a.iter().map(|x| x.as_u64().unwrap_or(0)).collect();
        random::setstate(Random::from_python_state(&words));
    }
}

/// Values that mean "absent" in Python reads (`d.get(k)` is falsy either way).
fn blank(v: &J) -> bool {
    match v {
        J::Null => true,
        J::Bool(b) => !b,
        J::String(s) => s.is_empty(),
        J::Array(a) => a.is_empty(),
        J::Object(o) => o.is_empty(),
        J::Number(n) => n.as_f64() == Some(0.0),
    }
}

fn norm(v: &J) -> J {
    match v {
        J::Object(o) => J::Object(o.iter().filter(|(_, x)| !blank(x)).map(|(k, x)| (k.clone(), norm(x))).collect()),
        J::Array(a) => J::Array(a.iter().map(norm).collect()),
        x => x.clone(),
    }
}

fn same(a: &J, b: &J) -> bool {
    norm(a) == norm(b)
}

fn diff(before: &Map<String, J>, after: &Map<String, J>) -> J {
    let mut set = Map::new();
    for (k, v) in after {
        if before.get(k).map(|b| !same(b, v)).unwrap_or(true) {
            set.insert(k.clone(), v.clone());
        }
    }
    let del: Vec<J> = before.keys().filter(|k| !after.contains_key(*k)).map(|k| J::String(k.clone())).collect();
    serde_json::json!({"set": set, "del": del})
}

/// Whether two diffs over the same `before` memory leave the same memory. A key only one side
/// lists is read from `before` on the other: Rust's diff is built with `same`, which reads a
/// blank value as absent, so a write that only adds a blank (`meal["offered"] = []`) is in
/// PyBot's diff and not in Rust's, and the memories are still the same.
fn diff_same(a: &J, b: &J, before: &Map<String, J>) -> bool {
    let (sa, sb) = (&a["set"], &b["set"]);
    let keys: std::collections::BTreeSet<&String> = sa.as_object().into_iter().flat_map(|o| o.keys()).chain(sb.as_object().into_iter().flat_map(|o| o.keys())).collect();
    let side = |s: &J, k: &String| -> J { s.get(k.as_str()).or_else(|| before.get(k.as_str())).cloned().unwrap_or(J::Null) };
    keys.into_iter().all(|k| same(&side(sa, k), &side(sb, k))) && {
        let mut da: Vec<String> = a["del"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let mut db: Vec<String> = b["del"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        da.sort();
        db.sort();
        da == db
    }
}

fn apply(mem: &Map<String, J>, d: &J) -> Map<String, J> {
    let mut m = mem.clone();
    if let Some(s) = d["set"].as_object() {
        for (k, v) in s {
            m.insert(k.clone(), v.clone());
        }
    }
    if let Some(del) = d["del"].as_array() {
        for k in del {
            m.shift_remove(k.as_str().unwrap_or(""));
        }
    }
    m
}

fn short(v: &J) -> String {
    let s = v.to_string();
    let t: String = s.chars().take(240).collect();
    if t.len() < s.len() { format!("{t}...") } else { t }
}

fn keys_differing(a: &J, b: &J) -> String {
    let mut out = Vec::new();
    let ka: Vec<&String> = a["set"].as_object().map(|o| o.keys().collect()).unwrap_or_default();
    let kb: Vec<&String> = b["set"].as_object().map(|o| o.keys().collect()).unwrap_or_default();
    for k in ka.iter().chain(kb.iter()) {
        if !same(&a["set"][k.as_str()], &b["set"][k.as_str()]) && !out.contains(k) {
            out.push(*k);
        }
    }
    out.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(",")
}

fn main() {
    // every oracle was recorded with the clock pinned to this day
    if std::env::var_os("RSBOT_TODAY").is_none() {
        unsafe { std::env::set_var("RSBOT_TODAY", "2026-11-17") };
    }
    quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args[0].clone();
    let (mut modules, mut stage, mut show, mut limit, mut e2e) = (Vec::<String>::new(), None::<String>, 3usize, usize::MAX, false);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--module" => { modules.push(args[i + 1].clone()); i += 1; }
            "--stage" => { stage = Some(args[i + 1].clone()); i += 1; }
            "--show" => { show = args[i + 1].parse().unwrap(); i += 1; }
            "--limit" => { limit = args[i + 1].parse().unwrap(); i += 1; }
            "--e2e" => e2e = true,
            x => panic!("unknown argument {x}"),
        }
        i += 1;
    }
    rsbot::data::load();
    let reader = std::io::BufReader::new(GzDecoder::new(std::fs::File::open(&path).unwrap()));
    let want = |m: &str| modules.is_empty() || modules.iter().any(|x| x == m);
    let st = |s: &str| stage.as_deref().map(|x| x == s).unwrap_or(true);
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    let (mut e2e_ok, mut e2e_n) = (0usize, 0usize);
    let mut e2e_mem: Option<(String, Memory)> = None;
    let mut n = 0;
    for line in reader.lines() {
        if n >= limit { break; }
        n += 1;
        let rec: J = serde_json::from_str(&line.unwrap()).unwrap();
        let text = rec["text"].as_str().unwrap_or("");
        let conv = rec["conv"].as_str().unwrap_or("").to_string();
        let turn_id = format!("{}#{}", conv, rec["turn"]);
        if e2e {
            if e2e_mem.as_ref().map(|(c, _)| c != &conv).unwrap_or(true) {
                random::seed(0);
                e2e_mem = Some((conv.clone(), Memory::new()));
            }
            let mem = &mut e2e_mem.as_mut().unwrap().1;
            let r = rsbot::runtime::turn(text, mem);
            e2e_n += 1;
            if r.reply == rec["reply"].as_str().unwrap_or("") {
                e2e_ok += 1;
            } else if tallies.entry("e2e".into()).or_default().shown.len() < show {
                tallies.get_mut("e2e").unwrap().shown.push(format!("{turn_id} rust[{}] py[{}] text {:?}\n      rust: {}\n      py:   {}", r.module.unwrap_or("none"), rec["chosen"].as_str().unwrap_or("none"), text, r.reply, rec["reply"].as_str().unwrap_or("")));
            }
            continue;
        }
        let mem_in = rec["mem_in"].as_object().cloned().unwrap_or_default();
        // observers
        let mut before = mem_in.clone();
        let mut state = rec["rng_in"].clone();
        for o in rec["observe"].as_array().unwrap() {
            let name = o["module"].as_str().unwrap();
            if st("observe") && want(name) {
                let def = registry().iter().find(|d| d.name == name).unwrap();
                let mut m = Memory::from_plain(&before);
                rng_set(&state);
                let ok_run = guarded(|| (def.observe.unwrap())(text, &mut m));
                let got = diff(&before, &m.to_plain());
                let t = tallies.entry(format!("observe {name}")).or_default();
                if diff_same(&got, &o["diff"], &before) {
                    t.ok += 1;
                } else {
                    t.bad += 1;
                    if ok_run.is_none() {
                        if let Some(p) = take_last_panic().filter(|p| p.starts_with("UNPORTED")) {
                            *t.unported.entry(p).or_default() += 1;
                        }
                    }
                    if t.shown.len() < show {
                        t.shown.push(format!("{turn_id} text {:?} keys [{}]\n      rust: {}\n      py:   {}", text, keys_differing(&got, &o["diff"]), short(&got), short(&o["diff"])));
                    }
                }
            }
            before = apply(&before, &o["diff"]);
            if o["rng_after"] != "same" { state = o["rng_after"].clone(); }
        }
        let post = before;
        // respond
        let mut state = rec["rng_before_respond"].clone();
        let mut cands = Vec::new();
        for r in rec["respond"].as_array().unwrap() {
            let name = r["module"].as_str().unwrap();
            let py_reply = r["reply"].as_str().filter(|s| !rsbot::py::text::strip(s).is_empty()).map(str::to_string);
            if py_reply.is_some() {
                let def = registry().iter().find(|d| d.name == name).unwrap();
                cands.push(Candidate { module: def.name, priority: def.priority, reply: py_reply.clone().unwrap() });
            }
            if st("respond") && want(name) {
                let def = registry().iter().find(|d| d.name == name).unwrap();
                let mut m = Memory::from_plain(&post);
                rng_set(&state);
                let res = guarded(|| (def.respond.unwrap())(text, &mut m));
                let got_reply = res.clone().flatten();
                let got = diff(&post, &m.to_plain());
                let reply_ok = got_reply.as_deref() == r["reply"].as_str();
                let t = tallies.entry(format!("respond {name}")).or_default();
                if reply_ok && diff_same(&got, &r["diff"], &post) {
                    t.ok += 1;
                } else {
                    t.bad += 1;
                    if res.is_none() {
                        if let Some(p) = take_last_panic().filter(|p| p.starts_with("UNPORTED")) {
                            *t.unported.entry(p).or_default() += 1;
                        }
                    }
                    if t.shown.len() < show {
                        t.shown.push(format!("{turn_id} text {:?}\n      reply rust: {:?}\n      reply py:   {:?}\n      memory keys differing: [{}]", text, got_reply, r["reply"].as_str(), keys_differing(&got, &r["diff"])));
                        if std::env::var_os("RSBOT_SHOW_DIFF").is_some() {
                            for k in keys_differing(&got, &r["diff"]).split(',') {
                                t.shown.push(format!("  {k}\n      rust: {}\n      py:   {}", got["set"][k], r["diff"]["set"][k]));
                            }
                        }
                    }
                }
            }
            if r["rng_after"] != "same" { state = r["rng_after"].clone(); }
        }
        // arbiter
        if st("arbiter") && want("arbiter") && cands.len() >= 2 {
            let m = Memory::from_plain(&post);
            rng_set(&rec["rng_before_respond"]);
            let i = guarded(|| rsbot::modules::choose(text, &m, &cands)).unwrap_or(0);
            let t = tallies.entry("arbiter".into()).or_default();
            if cands.get(i).map(|c| c.module) == rec["chosen"].as_str() {
                t.ok += 1;
            } else {
                t.bad += 1;
                if let Some(p) = take_last_panic().filter(|p| p.starts_with("UNPORTED")) {
                    *t.unported.entry(p).or_default() += 1;
                }
                if t.shown.len() < show {
                    t.shown.push(format!("{turn_id} picked {:?}, PyBot {:?} from {:?}", cands.get(i).map(|c| c.module), rec["chosen"], cands.iter().map(|c| c.module).collect::<Vec<_>>()));
                }
            }
        }
        // after hooks
        let chosen = rec["chosen"].as_str();
        let mut mem = match chosen {
            Some(c) => apply(&post, &rec["respond"].as_array().unwrap().iter().find(|r| r["module"] == c).unwrap()["diff"]),
            None => post.clone(),
        };
        let reply = rec["reply"].as_str().unwrap_or("");
        for a in rec["after"].as_array().unwrap() {
            let name = a["module"].as_str().unwrap();
            if st("after") && want(name) {
                let def = registry().iter().find(|d| d.name == name).unwrap();
                let mut m = Memory::from_plain(&mem);
                let res = guarded(|| (def.after.unwrap())(text, reply, &mut m));
                let got = diff(&mem, &m.to_plain());
                let t = tallies.entry(format!("after {name}")).or_default();
                if diff_same(&got, &a["diff"], &mem) { t.ok += 1 } else {
                    t.bad += 1;
                    if res.is_none() {
                        if let Some(p) = take_last_panic().filter(|p| p.starts_with("UNPORTED")) {
                            *t.unported.entry(p).or_default() += 1;
                        }
                    }
                    if t.shown.len() < show {
                        t.shown.push(format!("{turn_id} text {:?} keys [{}]", text, keys_differing(&got, &a["diff"])));
                    }
                }
            }
            mem = apply(&mem, &a["diff"]);
        }
    }
    if e2e {
        println!("e2e: {e2e_ok}/{e2e_n} replies identical ({:.1}%)", 100.0 * e2e_ok as f64 / e2e_n.max(1) as f64);
        if let Some(t) = tallies.get("e2e") { for s in &t.shown { println!("  -- {s}"); } }
        return;
    }
    let (mut all_ok, mut all_n) = (0, 0);
    for (name, t) in &tallies {
        all_ok += t.ok;
        all_n += t.ok + t.bad;
        if t.bad == 0 && modules.is_empty() { continue; }
        let unp: usize = t.unported.values().sum();
        println!("{name:44} {:6} ok {:6} differ{}", t.ok, t.bad, if unp > 0 { format!(" ({unp} from unported stubs)") } else { String::new() });
        for s in &t.shown { println!("    -- {s}"); }
        if !modules.is_empty() {
            for (p, c) in t.unported.iter().take(10) { println!("    unported: {p} x{c}"); }
        }
    }
    let clean = tallies.values().filter(|t| t.bad == 0).count();
    println!("{n} turns; stage checks {all_ok}/{all_n} identical; {clean}/{} module-stages fully identical", tallies.len());
}
