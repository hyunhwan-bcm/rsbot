//! DUP ports (group 6 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.
//! The first section holds small Python-number helpers the group's modules share.

// ---------------------------------------------------------------------------------------------
// Python number helpers (not DUPs: the arithmetic Python's builtins do).

/// Python's `round(x, n)` on a float: correctly rounded to `n` decimals (ties to even on the
/// exact binary value), as `"%.nf"` prints it. Negative `n` rounds to tens, hundreds, ...
pub fn py_round(x: f64, n: i32) -> f64 {
    if n >= 0 {
        format!("{:.*}", n as usize, x).parse().unwrap_or(x)
    } else {
        let p = 10f64.powi(-n);
        crate::py::text::round_half_even(x / p) as f64 * p
    }
}

/// Python's `a // b` on floats (CPython's `float_floor_div`).
pub fn py_floordiv(a: f64, b: f64) -> f64 {
    let m = a % b;
    let mut div = (a - m) / b;
    if m != 0.0 && ((b < 0.0) != (m < 0.0)) {
        div -= 1.0;
    }
    if div != 0.0 {
        let f = div.floor();
        if div - f > 0.5 { f + 1.0 } else { f }
    } else {
        0.0f64.copysign(a / b)
    }
}

/// `"%d" % x` for a float: the integer part (truncated toward zero).
pub fn pct_d(x: f64) -> String {
    format!("{}", x.trunc() as i64)
}

// ---------------------------------------------------------------------------------------------
// DUP ports of helpers owned by other workstreams (full original logic).

use crate::state::Memory;

/// DUP(bot.memory::last_bot): the bot's last reply; "" when there is no earlier turn.
pub fn last_bot(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// A differential check for the group's modules (test only): replays calls recorded by calling
// every function guard / reply / shape of a module's entries on many readings, whether or not
// PyBot would route the line there. `RSBOT_G6_DIFF=<dir> cargo test --release --lib g6_diff`.
#[cfg(test)]
mod g6_diff {
    use crate::data::{FnRef, Value};
    use crate::knowledge::registry;
    use crate::state::Memory;
    use crate::understood::Understood;
    use std::collections::{BTreeMap, HashMap};
    use std::io::BufRead;

    fn entry_fn(entry_id: &str, kind: &str, fn_id: &str) -> Option<FnRef> {
        static INDEX: std::sync::LazyLock<HashMap<String, Vec<FnRef>>> = std::sync::LazyLock::new(|| {
            let mut m: HashMap<String, Vec<FnRef>> = HashMap::new();
            fn walk(v: &Value, m: &mut HashMap<String, Vec<FnRef>>, depth: usize) {
                if depth > 12 {
                    return;
                }
                match v {
                    Value::Obj(class, _) if &**class == "Entry" => {
                        let id = v.attr("id").map(|x| x.str().to_string()).unwrap_or_default();
                        for k in ["guard", "reply"] {
                            if let Some(f) = v.attr(k).and_then(|x| x.as_fn()) {
                                m.entry(format!("{id}|{k}")).or_default().push(f.clone());
                            }
                        }
                        if let Some(s) = v.attr("shape") {
                            for f in s.items().iter().filter_map(|x| x.as_fn()) {
                                m.entry(format!("{id}|shape")).or_default().push(f.clone());
                            }
                        }
                    }
                    Value::List(xs) | Value::Tuple(xs) | Value::Set(xs) => {
                        for x in xs.iter() {
                            walk(x, m, depth + 1)
                        }
                    }
                    Value::Dict(d) => {
                        for (_, x) in d.iter() {
                            walk(x, m, depth + 1)
                        }
                    }
                    _ => {}
                }
            }
            for name in crate::data::names_with_prefix("bot.") {
                walk(crate::data::get(name), &mut m, 0);
            }
            m
        });
        INDEX.get(&format!("{entry_id}|{kind}"))?.iter().find(|f| f.id == fn_id).cloned()
    }

    fn shape(f: &FnRef, arg: &str) -> Option<bool> {
        let tables: Vec<&[(&str, crate::knowledge::d1_shared::ShapeFn)]> = vec![];
        for t in tables {
            if let Some((_, s)) = t.iter().find(|(k, _)| *k == f.id) {
                return Some(crate::runtime::guarded(|| s(f, arg)).unwrap_or(false));
            }
        }
        None
    }

    #[test]
    fn g6_diff() {
        let Ok(dir) = std::env::var("RSBOT_G6_DIFF") else { return };
        crate::runtime::quiet_panics();
        let show: usize = std::env::var("RSBOT_G6_SHOW").ok().and_then(|s| s.parse().ok()).unwrap_or(3);
        let mut readings: HashMap<String, Understood> = HashMap::new();
        for line in std::io::BufReader::new(std::fs::File::open(format!("{dir}/readings.jsonl")).unwrap()).lines() {
            let mut v: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
            // `kept` is a dict on some single-turn readings ({"place", "time"}) while the typed
            // reading holds a string; none of the group's functions read it.
            if v["understood"]["kept"].is_object() {
                v["understood"]["kept"] = serde_json::Value::Null;
            }
            match serde_json::from_value(v["understood"].clone()) {
                Ok(u) => {
                    readings.insert(v["h"].as_str().unwrap().to_string(), u);
                }
                Err(e) => println!("unreadable reading: {e}"),
            }
        }
        let mut stats: BTreeMap<String, (usize, usize, usize, Vec<String>)> = BTreeMap::new();
        for line in std::io::BufReader::new(std::fs::File::open(format!("{dir}/calls.jsonl")).unwrap()).lines() {
            let c: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
            let (kind, id) = (c["kind"].as_str().unwrap(), c["fn"].as_str().unwrap());
            let st = stats.entry(format!("{kind} {id}")).or_default();
            let Some(reading) = readings.get(c["h"].as_str().unwrap()) else { st.2 += 1; continue };
            let Some(f) = entry_fn(c["entry"].as_str().unwrap(), kind, id) else { st.2 += 1; continue };
            let raised = !c["raised"].is_null();
            let (got, want) = match kind {
                "guard" => (
                    serde_json::json!(registry::guard(&f, reading)),
                    serde_json::json!(!raised && c["result"].as_bool().unwrap_or(false)),
                ),
                "reply" => {
                    let mut m = Memory::new();
                    m.understood = Some(reading.clone());
                    (
                        serde_json::json!(registry::reply(&f, &mut m, reading)),
                        serde_json::json!(if raised { "" } else { c["result"].as_str().unwrap_or("") }),
                    )
                }
                _ => {
                    let Some(got) = shape(&f, c["arg"].as_str().unwrap_or("")) else { st.2 += 1; continue };
                    (serde_json::json!(got), serde_json::json!(!raised && c["result"].as_bool().unwrap_or(false)))
                }
            };
            if got == want {
                st.0 += 1;
            } else {
                st.1 += 1;
                if st.3.len() < show {
                    st.3.push(format!("entry {} text {:?}\n      rust {}\n      py   {}", c["entry"], reading.text, got, want));
                }
            }
        }
        let (mut ok, mut bad) = (0, 0);
        for (name, (o, b, u, ex)) in &stats {
            ok += o;
            bad += b;
            println!("{name}: {o} ok, {b} differ{}", if *u > 0 { format!(", {u} unrunnable") } else { String::new() });
            for e in ex {
                println!("    {e}");
            }
        }
        println!("calls {ok} ok / {bad} differ");
        let misses = registry::misses();
        if !misses.is_empty() {
            println!("functions with no port that were called: {misses:?}");
        }
    }

    /// Direct calls of the group's non-entry functions (`RSBOT_G6_CALLS=<file.jsonl>`): each line
    /// names a function, its arguments and what Python returned.
    #[test]
    fn g6_calls() {
        let Ok(path) = std::env::var("RSBOT_G6_CALLS") else { return };
        let (mut ok, mut bad) = (0, 0);
        for line in std::io::BufReader::new(std::fs::File::open(path).unwrap()).lines() {
            let c: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
            let s = |k: &str| c[k].as_str().unwrap_or("").to_string();
            let mut memory = Memory::new();
            if !s("user").is_empty() || !s("bot").is_empty() {
                memory.turns.push(crate::state::Turn { user: s("user"), bot: s("bot"), module: None });
            }
            let reading = Understood { clean: s("text"), ..Default::default() };
            let got: serde_json::Value = match s("fn").as_str() {
                "the_still_ok_follow_up" => serde_json::json!(crate::knowledge::freshness::the_still_ok_follow_up(&s("text"), &memory)),
                "_left_out" => serde_json::json!(crate::knowledge::freshness::_left_out(&memory, &reading)),
                "crosses_a_border" => serde_json::json!(crate::knowledge::geography::crosses_a_border(&s("text"))),
                "carries_it_out_of_britain" => serde_json::json!(crate::knowledge::geography::carries_it_out_of_britain(&s("text"))),
                "directed_place" => serde_json::json!(crate::knowledge::geography::directed_place(&s("text"))),
                "destination_place" => serde_json::json!(crate::knowledge::geography::destination_place(&s("text"))),
                "_countries_said" => serde_json::json!(crate::knowledge::geography::_countries_said(&s("text"))),
                other => panic!("no dispatch for {other}"),
            };
            if got == c["want"] {
                ok += 1;
            } else {
                bad += 1;
                println!("{} {:?}\n   rust {}\n   py   {}", s("fn"), s("text"), got, c["want"]);
            }
        }
        println!("direct calls {ok} ok / {bad} differ");
    }
}
