//! Check ported knowledge guards and replies against every call PyBot made to them
//! (`harness/entry_calls.py` output in data/export/entry_calls_<split>/).
//!
//!     entry_check [gold|heldout] [--module bot.knowledge.cooking]... [--fn SUBSTR] [--show K] [--all]
//!
//! Per function: calls that give PyBot's result, calls that differ, and calls the port has no
//! function for. Without --all, functions that are not ported yet are summarised in one line.
use rsbot::data::Value;
use rsbot::knowledge::registry;
use rsbot::state::Memory;
use rsbot::understood::Understood;
use std::collections::{BTreeMap, HashMap};
use std::io::BufRead;

fn main() {
    rsbot::runtime::quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut split = "gold".to_string();
    let (mut modules, mut fns, mut show, mut all) = (Vec::new(), Vec::new(), 3usize, false);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "gold" | "heldout" => split = args[i].clone(),
            "--module" => { modules.push(args[i + 1].clone()); i += 1; }
            "--fn" => { fns.push(args[i + 1].clone()); i += 1; }
            "--show" => { show = args[i + 1].parse().unwrap(); i += 1; }
            "--all" => all = true,
            x => panic!("unknown argument {x}"),
        }
        i += 1;
    }
    let dir = format!("{}/../data/export/entry_calls_{split}", env!("CARGO_MANIFEST_DIR"));
    let mut readings: HashMap<String, Result<Understood, String>> = HashMap::new();
    for line in std::io::BufReader::new(std::fs::File::open(format!("{dir}/readings.jsonl")).unwrap()).lines() {
        let v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let h = v["h"].as_str().unwrap().to_string();
        readings.insert(h, serde_json::from_value(v["understood"].clone()).map_err(|e| e.to_string()));
    }
    rsbot::data::load();
    // fn -> (ok, differ, unreadable, examples)
    let mut stats: BTreeMap<String, (usize, usize, usize, Vec<String>)> = BTreeMap::new();
    let mut not_ported: BTreeMap<String, usize> = BTreeMap::new();
    let entries = rsbot::data::try_get("__entries_by_id__");
    let _ = entries;
    for line in std::io::BufReader::new(std::fs::File::open(format!("{dir}/calls.jsonl")).unwrap()).lines() {
        let c: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = c["fn"].as_str().unwrap();
        let module = id.split(':').next().unwrap();
        if !modules.is_empty() && !modules.iter().any(|m| m == module) { continue; }
        if !fns.is_empty() && !fns.iter().any(|f| id.contains(f.as_str())) { continue; }
        let kind = c["kind"].as_str().unwrap();
        let ported = if kind == "guard" { registry::has_guard(id) } else { registry::has_reply(id) };
        if !ported {
            *not_ported.entry(format!("{kind} {id}")).or_default() += 1;
            continue;
        }
        let st = stats.entry(format!("{kind} {id}")).or_default();
        let Some(Ok(reading)) = readings.get(c["h"].as_str().unwrap_or("")) else { st.2 += 1; continue };
        let fref = entry_fn(c["entry"].as_str().unwrap(), kind, id);
        let Some(fref) = fref else { st.2 += 1; continue };
        let (got, want) = if kind == "guard" {
            let want = if c["raised"].is_null() { c["result"].as_bool().unwrap_or(false) } else { false };
            (serde_json::json!(registry::guard(&fref, reading)), serde_json::json!(want))
        } else {
            let mut m = memory_from(&c["memory"], reading);
            let want = if c["raised"].is_null() { c["result"].as_str().unwrap_or("").to_string() } else { String::new() };
            (serde_json::json!(registry::reply(&fref, &mut m, reading)), serde_json::json!(want))
        };
        if got == want { st.0 += 1 } else {
            st.1 += 1;
            if st.3.len() < show {
                st.3.push(format!("entry {} text {:?}\n      rust {}\n      py   {}", c["entry"], reading.text, got, want));
            }
        }
    }
    let (mut tot_ok, mut tot_bad, mut fn_ok, mut fn_bad) = (0, 0, 0, 0);
    for (name, (ok, bad, unreadable, ex)) in &stats {
        tot_ok += ok; tot_bad += bad;
        if *bad == 0 { fn_ok += 1 } else { fn_bad += 1 }
        if *bad > 0 || all {
            println!("{name}: {ok} ok, {bad} differ{}", if *unreadable > 0 { format!(", {unreadable} unreadable") } else { String::new() });
            for e in ex { println!("    {e}"); }
        }
    }
    println!("ported functions: {fn_ok} match on every call, {fn_bad} differ somewhere; calls {tot_ok} ok / {tot_bad} differ");
    if !not_ported.is_empty() {
        let calls: usize = not_ported.values().sum();
        println!("not ported yet: {} functions ({} calls){}", not_ported.len(), calls,
            if all { format!(":\n  {}", not_ported.keys().cloned().collect::<Vec<_>>().join("\n  ")) } else { String::new() });
    }
}

/// The entry's own guard / reply function, with its captured values, from the bundle.
fn entry_fn(entry_id: &str, kind: &str, fn_id: &str) -> Option<rsbot::data::FnRef> {
    static INDEX: std::sync::LazyLock<HashMap<String, Vec<rsbot::data::FnRef>>> = std::sync::LazyLock::new(|| {
        // every Entry object anywhere in the bundle, by id
        let mut m: HashMap<String, Vec<rsbot::data::FnRef>> = HashMap::new();
        fn walk(v: &Value, m: &mut HashMap<String, Vec<rsbot::data::FnRef>>, depth: usize) {
            if depth > 12 { return; }
            match v {
                Value::Obj(class, _) if &**class == "Entry" => {
                    let id = v.attr("id").map(|x| x.str().to_string()).unwrap_or_default();
                    for k in ["guard", "reply"] {
                        if let Some(f) = v.attr(k).and_then(|x| x.as_fn()) {
                            m.entry(format!("{id}|{k}")).or_default().push(f.clone());
                        }
                    }
                }
                Value::List(xs) | Value::Tuple(xs) | Value::Set(xs) => for x in xs.iter() { walk(x, m, depth + 1) },
                Value::Dict(d) => for (_, x) in d.iter() { walk(x, m, depth + 1) },
                _ => {}
            }
        }
        for name in rsbot::data::names_with_prefix("bot.") {
            walk(rsbot::data::get(name), &mut m, 0);
        }
        m
    });
    INDEX.get(&format!("{entry_id}|{kind}"))?.iter().find(|f| f.id == fn_id).cloned()
}

fn memory_from(view: &serde_json::Value, reading: &Understood) -> Memory {
    let mut m = Memory::new();
    m.understood = Some(reading.clone());
    if let Some(o) = view.as_object() {
        for (k, v) in o {
            match k.as_str() {
                "intent_wants" => m.intent_wants = v.as_str().map(str::to_string),
                "asked_now" => m.asked_now = v.as_str().map(str::to_string),
                _ => { m.extra.insert(k.clone(), Value::from_plain(v)); }
            }
        }
    }
    m
}
