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

// was DUP(bot.memory::last_bot): now the real port
pub use crate::memory::last_bot;

/// DUP(bot.knowledge.lookup::asks_how_often), for a line already flattened by `the_line`: true
/// when it asks the interval rather than the method.
pub fn asks_how_often_text(said: &str) -> bool {
    static HOW_OFTEN_WORDS: std::sync::LazyLock<Vec<String>> =
        std::sync::LazyLock::new(|| crate::data::strs("bot.knowledge.lookup.HOW_OFTEN_WORDS"));
    HOW_OFTEN_WORDS.iter().any(|w| said.contains(w.as_str()))
}

// was DUP(bot.knowledge.lookup::asks_how_often): now the real port
pub use crate::knowledge::lookup::asks_how_often;

// --- bot.knowledge.lookup: food that has gone off, food being kept, counts into a dish ---------

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn lookup_set(name: &str) -> HashSet<String> {
    data::set(&format!("bot.knowledge.lookup.{name}"))
}

// was DUP(bot.knowledge.lookup::tokens): now the real port
pub use crate::knowledge::lookup::tokens;

// was DUP(bot.knowledge.lookup::food_has_gone_off): now the real port
pub use crate::knowledge::lookup::food_has_gone_off;

// was DUP(bot.knowledge.lookup::not_a_count_for_the_box_or_the_dish): now the real port
pub use crate::knowledge::lookup::not_a_count_for_the_box_or_the_dish;

// was DUP(bot.knowledge.lookup::a_food_is_being_kept): now the real port
pub use crate::knowledge::lookup::a_food_is_being_kept;

// --- bot.understanding.topics / bot.knowledge.shopping: a present to buy ---------------------

fn topics_set(name: &str) -> &'static HashSet<String> {
    static SETS: LazyLock<std::collections::HashMap<&'static str, HashSet<String>>> = LazyLock::new(|| {
        [
            "GIFT_ALWAYS", "GIFT_WORDS", "GIFT_ASKING", "GIVE_VERB", "GIFT_OCCASION", "GIFT_WANTING", "GIVE_TO",
            "GIVE_THINGS", "GIVE_THING", "GIVE_DETERMINERS", "GIVE_RELATIVE", "GIFTS_OR_OCCASIONS",
        ]
        .into_iter()
        .map(|n| (n, data::set(&format!("bot.understanding.topics.{n}"))))
        .collect()
    });
    &SETS[name]
}

fn meets(words: &HashSet<&str>, name: &str) -> bool {
    topics_set(name).iter().any(|w| words.contains(w.as_str()))
}

// was DUP(bot.understanding.topics::gift_ask): now the real port
pub use crate::understanding::topics::gift_ask;

/// The person a giving verb is followed by (within its first two words), and what comes after
/// them; None when no person follows the verb.
fn after_the_person(rest: &[String]) -> Option<&[String]> {
    for (index, one) in rest.iter().take(2).enumerate() {
        if topics_set("GIVE_TO").contains(one) {
            return Some(&rest[index + 1..]);
        }
        if topics_set("GIVE_DETERMINERS").contains(one) && rest.get(index + 1).is_some_and(|n| topics_set("GIVE_TO").contains(n)) {
            return Some(&rest[index + 2..]);
        }
    }
    None
}

/// DUP(bot.understanding.topics::_gift_ask): true when the line asks what to give someone.
pub fn _gift_ask(words: &HashSet<&str>, norm: &[String]) -> bool {
    let set = |n: &str| topics_set(n);
    if meets(words, "GIFT_ALWAYS") {
        return true;
    }
    if meets(words, "GIFT_WORDS")
        && (meets(words, "GIFT_ASKING") || meets(words, "GIVE_VERB") || meets(words, "GIFT_OCCASION") || words.contains("for"))
    {
        return true;
    }
    if meets(words, "GIFT_WANTING") && _gift_still_to_get(words, norm) {
        return true;
    }
    if !(meets(words, "GIFT_ASKING") || meets(words, "GIFT_OCCASION")) {
        return false;
    }
    if meets(words, "GIFT_OCCASION") && norm.len() > 1 {
        for (position, word) in norm[..norm.len() - 1].iter().enumerate() {
            let next = &norm[position + 1..(position + 3).min(norm.len())];
            if set("GIVE_TO").contains(word) && next.iter().any(|w| set("GIVE_THINGS").contains(w)) {
                return true;
            }
        }
    }
    for (position, word) in norm.iter().enumerate() {
        if !set("GIVE_VERB").contains(word) {
            continue;
        }
        let rest = &norm[position + 1..];
        let Some(head) = rest.first() else { continue };
        if set("GIVE_THING").contains(head) || set("GIFT_WORDS").contains(head) || set("GIFT_OCCASION").contains(head) {
            return true;
        }
        let Some(tail) = after_the_person(rest) else { continue };
        if tail.is_empty() || set("GIVE_RELATIVE").contains(&tail[0]) {
            return true;
        }
        if tail.iter().take(3).any(|w| set("GIFTS_OR_OCCASIONS").contains(w)) {
            return true;
        }
        if tail[0] == "for" && tail.iter().skip(1).take(3).any(|one| one.chars().any(|c| c.is_ascii_digit())) {
            return true;
        }
    }
    false
}

/// DUP(bot.understanding.topics::_gift_still_to_get): a present still not got.
pub fn _gift_still_to_get(words: &HashSet<&str>, norm: &[String]) -> bool {
    let set = |n: &str| topics_set(n);
    let owed = words.contains("nothing") || words.contains("anything");
    if meets(words, "GIVE_TO") && words.contains("for") && owed {
        return true;
    }
    for (position, word) in norm.iter().enumerate() {
        if !set("GIVE_VERB").contains(word) {
            continue;
        }
        let Some(tail) = after_the_person(&norm[position + 1..]) else { continue };
        let Some(first) = tail.first() else { continue };
        if set("GIVE_THING").contains(first) || set("GIFT_WORDS").contains(first) {
            return true;
        }
        if tail.iter().any(|w| w == "for") && owed {
            return true;
        }
        if tail.iter().take(3).any(|w| set("GIFT_OCCASION").contains(w)) {
            return true;
        }
    }
    false
}

// was DUP(bot.knowledge.shopping::not_a_present): now the real port
pub use crate::knowledge::shopping::not_a_present;

// --- bot.knowledge.cooking: the one food a line names a portion of --------------------------

fn cooking(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.cooking.{name}"))
}

// was DUP(bot.knowledge.cooking::_plurals): now the real port
pub use crate::knowledge::cooking_b::_plurals;

// was DUP(bot.knowledge.cooking::_portion_count): now the real port
pub use crate::knowledge::cooking_b::_portion_count;

// was DUP(bot.knowledge.cooking::_squashed): now the real port
pub use crate::knowledge::cooking_b::_squashed;

// was DUP(bot.knowledge.cooking::_one_letter_apart): now the real port
pub use crate::knowledge::cooking_b::_one_letter_apart;

/// DUP(bot.knowledge.cooking::_a_food_word_misspelt): the name this word is a slip of, or None.
pub fn _a_food_word_misspelt<'a>(word: &str, names: &[&'a str]) -> Option<&'a str> {
    let wlen = word.chars().count();
    if wlen < 5 || !crate::py::text::is_alpha(word) {
        return None;
    }
    if names.contains(&word) {
        return None;
    }
    let squashed = _squashed(word);
    let wchars: Vec<char> = word.chars().collect();
    for &name in names {
        let nchars: Vec<char> = name.chars().collect();
        if name.contains(' ') || nchars.len() < 5 || nchars[0] != wchars[0] {
            continue;
        }
        if nchars.len() == wlen {
            if squashed == _squashed(name) {
                return Some(name);
            }
            if nchars[nchars.len() - 1] == wchars[wlen - 1]
                && wchars.iter().zip(nchars.iter()).filter(|(a, b)| a != b).count() == 1
            {
                return Some(name);
            }
        } else if nchars.len().abs_diff(wlen) == 1 && nchars[nchars.len() - 1] == wchars[wlen - 1] {
            let (shorter, longer) = if wlen <= nchars.len() { (word, name) } else { (name, word) };
            if _one_letter_apart(shorter, longer) {
                return Some(name);
            }
        }
    }
    None
}

/// DUP(bot.knowledge.cooking::_the_food_named_by_a_slip): the one food a word of the line is a
/// slip of, with the number of portions.
pub fn _the_food_named_by_a_slip(words: &[String], table: &[&str]) -> Option<(String, f64)> {
    let dish = cooking("_NUTRIENT_DISH").strs();
    let join = cooking("_NUTRIENT_JOIN").strs();
    for (index, word) in words.iter().enumerate() {
        let Some(name) = _a_food_word_misspelt(word, table) else { continue };
        let before = if index > 0 { words.get(index - 1) } else { None };
        let after = words.get(index + 1);
        if after.is_some_and(|a| dish.contains(a) || join.contains(a)) {
            return None;
        }
        if before.is_some_and(|b| join.contains(b)) {
            return None;
        }
        return Some((name.to_string(), _portion_count(words, index)));
    }
    None
}

/// `cooking._nutrient_food(clean)` (the real port, over its FOOD_NUTRIENTS table).
pub fn _nutrient_food(clean: &str) -> Option<(String, f64)> {
    crate::knowledge::cooking_b::_nutrient_food(clean, &crate::knowledge::cooking_b::NUTRIENT_TABLE)
}

// --- bot.understanding.topics::the_bin_is_on_a_machine, bot.memory.topics::context ----------

// was DUP(bot.understanding.topics::the_bin_is_on_a_machine): now the real port
pub use crate::understanding::topics::the_bin_is_on_a_machine;

/// `memory.topics.context(memory, n)` (the real port).
pub fn memory_topics_context(memory: &Memory, n: usize) -> Vec<String> {
    crate::memory::topics::context(memory, n as i64)
}

/// Python's `repr()` of an exported value (`str()` of a container is its repr): enough to
/// search the printed form the way PyBot does with `str(memory.get(key))`.
pub fn py_repr(v: &Value) -> String {
    fn quote(s: &str) -> String {
        let q = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
        let mut out = String::new();
        out.push(q);
        for c in s.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c == q => {
                    out.push('\\');
                    out.push(c);
                }
                c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push(q);
        out
    }
    let join = |xs: &[Value]| xs.iter().map(py_repr).collect::<Vec<_>>().join(", ");
    match v {
        Value::None => "None".into(),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::Int(i) => i.to_string(),
        Value::Float(f) => crate::py::text::float_repr(*f),
        Value::Str(s) => quote(s),
        Value::List(xs) => format!("[{}]", join(xs)),
        Value::Tuple(xs) if xs.len() == 1 => format!("({},)", py_repr(&xs[0])),
        Value::Tuple(xs) => format!("({})", join(xs)),
        Value::Set(xs) if xs.is_empty() => "set()".into(),
        Value::Set(xs) => format!("{{{}}}", join(xs)),
        Value::Dict(d) => format!(
            "{{{}}}",
            d.iter().map(|(k, v)| format!("{}: {}", py_repr(k), py_repr(v))).collect::<Vec<_>>().join(", ")
        ),
        Value::Re(r) => format!("re.compile({})", quote(r.pattern())),
        Value::Fn(f) => format!("<function {}>", f.name()),
        Value::Obj(c, _) => format!("<{c} object>"),
        Value::Module(m) => format!("<module '{m}'>"),
        Value::Opaque(o) => format!("<{o}>"),
    }
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
        let tables: Vec<&[(&str, crate::knowledge::d1_shared::ShapeFn)]> = vec![crate::knowledge::cars::SHAPES, crate::knowledge::technology::SHAPES];
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
            // `kept` and `care_home` are dicts on some single-turn readings while the typed reading
            // holds strings; none of the group's functions read them.
            for key in ["kept", "care_home", "setup", "taking_up", "looks_back", "leans_on", "over_time_what"] {
                if v["understood"][key].is_object() {
                    v["understood"][key] = serde_json::Value::Null;
                }
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
            for (k, v) in c["extra"].as_object().into_iter().flatten() {
                memory.extra.insert(k.clone(), Value::from_plain(v));
            }
            for t in c["turns"].as_array().into_iter().flatten() {
                let (user, bot) = (t[0].as_str().unwrap_or("").to_string(), t[1].as_str().unwrap_or("").to_string());
                memory.turns.push(crate::state::Turn { user, bot, module: None });
            }
            let reading = Understood { clean: s("text"), ..Default::default() };
            let got: serde_json::Value = match s("fn").as_str() {
                "reply" => {
                    let f = FnRef { id: s("id"), cells: vec![], defaults: vec![] };
                    serde_json::json!(registry::reply(&f, &mut memory, &reading))
                }
                "the_still_ok_follow_up" => serde_json::json!(crate::knowledge::freshness::the_still_ok_follow_up(&s("text"), &memory)),
                "_left_out" => serde_json::json!(crate::knowledge::freshness::_left_out(&memory, &reading)),
                "crosses_a_border" => serde_json::json!(crate::knowledge::geography::crosses_a_border(&s("text"))),
                "carries_it_out_of_britain" => serde_json::json!(crate::knowledge::geography::carries_it_out_of_britain(&s("text"))),
                "directed_place" => serde_json::json!(crate::knowledge::geography::directed_place(&s("text"))),
                "destination_place" => serde_json::json!(crate::knowledge::geography::destination_place(&s("text"))),
                "_countries_said" => serde_json::json!(crate::knowledge::geography::_countries_said(&s("text"))),
                "gift_ask" => {
                    let r = Understood { norm: crate::py::text::split(&s("text")), ..Default::default() };
                    serde_json::json!(super::gift_ask(&r))
                }
                "_nutrient_food" => serde_json::json!(super::_nutrient_food(&s("text")).map(|(n, c)| (n, c))),
                "food_has_gone_off" => serde_json::json!(super::food_has_gone_off(&reading)),
                "a_food_is_being_kept" => {
                    let r = Understood { clean: s("text"), words: crate::py::text::split(&s("text")), ..Default::default() };
                    serde_json::json!(super::a_food_is_being_kept(&r))
                }
                "not_a_count" => serde_json::json!(super::not_a_count_for_the_box_or_the_dish(&reading)),
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

    /// `sport._win_years()` rebuilds the `WIN_YEARS` table the bundle holds.
    #[test]
    fn win_years_is_the_bundle_table() {
        let bundle: Vec<(String, Vec<i64>)> = crate::data::get("bot.knowledge.sport.WIN_YEARS")
            .pairs()
            .iter()
            .map(|(k, v)| (k.str().to_string(), v.items().iter().filter_map(Value::as_int).collect()))
            .collect();
        assert_eq!(crate::knowledge::sport::_win_years(), bundle);
    }
}
