//! DUP ports (group 2 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn set_of(name: &str) -> HashSet<String> {
    data::get(name).str_set()
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup

static WEIGHT_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.knowledge.lookup.WEIGHT_UNITS"));

/// DUP(bot.knowledge.lookup::a_weight): true when the line gives a weight in kilos, grams or
/// pounds (a kitchen line, not a question about the animal or the place the noun also names).
pub fn a_weight(understood: &Understood) -> bool {
    understood
        .numbers
        .iter()
        .any(|number| WEIGHT_UNITS.contains(number.unit.as_deref().unwrap_or("").to_lowercase().as_str()))
}

// ---------------------------------------------------------------------------------------------
// bot.memory.topics / bot.memory.named: what the chat is on, read from the memory store's
// records (`memory["topic"]`, `memory["topics"]`, `memory["user"]`), which live in
// `Memory::extra` under their Python keys.

/// `memory.get(key)` from the store's own keys.
fn mem<'a>(memory: &'a Memory, key: &str) -> Option<&'a Value> {
    memory.extra.get(key)
}

/// `str(record.get(key) or "")` for a dict record.
fn rec_str<'a>(record: &'a Value, key: &str) -> &'a str {
    match record.get(key) {
        Some(Value::Str(s)) => s,
        _ => "",
    }
}

/// DUP(bot.memory.topics::context): the last n topic texts, oldest first. None where Python
/// would raise (a record that is not a dict), which the caller catches.
pub fn topics_context(memory: &Memory, n: usize) -> Option<Vec<String>> {
    let Some(Value::List(records)) = mem(memory, "topics") else {
        return Some(Vec::new());
    };
    let start = records.len().saturating_sub(n);
    let mut out = Vec::new();
    for r in &records[start..] {
        if !matches!(r, Value::Dict(_)) {
            return None; // r.get on a non-dict raises
        }
        // r.get("text", "") keeps a non-str text as it is; only truthy ones are kept
        match r.get("text") {
            Some(Value::Str(s)) if !s.is_empty() => out.push(s.to_string()),
            Some(v) if v.truthy() => out.push(String::new()), // non-str text: str() of it is not kept by callers
            _ => {}
        }
    }
    Some(out)
}

/// DUP(bot.memory.topics::current): the current topic record, or None.
fn topics_current(memory: &Memory) -> Option<&Value> {
    let record = mem(memory, "topic")?;
    if matches!(record, Value::Dict(_)) && record.get("text").map(Value::truthy).unwrap_or(false) {
        return Some(record);
    }
    None
}

/// DUP(bot.memory.topics::topic_text).
fn topics_topic_text(memory: &Memory) -> String {
    match topics_current(memory) {
        Some(r) => rec_str(r, "text").to_string(),
        None => String::new(),
    }
}

static ROOM_LIMIT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics._ROOM_LIMIT"));
static HOME_SPACES: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.topics._HOME_SPACES"));

/// DUP(bot.memory.topics::_room_limit): the home space a line only says the size of, or "".
fn topics_room_limit(text_: &str) -> String {
    let words = text::squash(&text_.to_lowercase());
    let Some(found) = ROOM_LIMIT.match_(&words) else {
        return String::new();
    };
    let head = text::strip_chars(found.name_str("head"), " ,.;:!?-");
    if head.is_empty() {
        return String::new();
    }
    match text::split_ref(head).last() {
        Some(last) if HOME_SPACES.contains(*last) => head.to_string(),
        _ => String::new(),
    }
}

/// DUP(bot.memory.topics::gist), PARTIAL: the short noun phrase a line is about. Only the empty
/// line is answered exactly (""); the full reader (switchers, the wish and lead strippers,
/// `keywords`) is the memory workstream's, and a non-empty line reads as "" here until it is
/// ported. Single-turn replies never carry a topic record, so they only ever pass "".
pub fn topics_gist(text_: &str) -> String {
    if text::squash(text_).is_empty() {
        return String::new();
    }
    String::new() // PARTIAL: see the doc comment
}

/// DUP(bot.memory.topics::_names_nothing).
fn topics_names_nothing(record: &Value) -> bool {
    if !text::squash(rec_str(record, "subject")).is_empty() {
        return false;
    }
    topics_gist(rec_str(record, "text")).is_empty()
}

/// DUP(bot.memory.topics::_about_record): the last topic that names something.
fn topics_about_record(memory: &Memory) -> Option<&Value> {
    let Some(Value::List(records)) = mem(memory, "topics") else {
        return topics_current(memory);
    };
    for record in records.iter().rev() {
        if !matches!(record, Value::Dict(_)) || !topics_room_limit(rec_str(record, "text")).is_empty() {
            continue;
        }
        if topics_names_nothing(record) {
            continue;
        }
        return Some(record);
    }
    topics_current(memory)
}

/// DUP(bot.memory.topics::_thing_of): the record a detail line points back at.
fn topics_thing_of<'a>(memory: &'a Memory, record: Option<&Value>) -> Option<&'a Value> {
    let record = record?;
    let back = rec_str(record, "of");
    if back.is_empty() {
        return None;
    }
    if let Some(Value::List(records)) = mem(memory, "topics") {
        for other in records.iter().rev() {
            if matches!(other, Value::Dict(_)) && rec_str(other, "text") == back {
                return Some(other);
            }
        }
    }
    None
}

/// DUP(bot.memory.topics::subject): what the conversation is about now, as a noun phrase.
pub fn topics_subject(memory: &Memory) -> String {
    let known = match mem(memory, "user") {
        Some(v @ Value::Dict(_)) => Some(v),
        _ => None,
    };
    let mut record = topics_about_record(memory);
    if let Some(thing) = topics_thing_of(memory, record) {
        record = Some(thing);
    }
    let subject = record.map(|r| text::squash(rec_str(r, "subject"))).unwrap_or_default();
    let now = if !subject.is_empty() {
        subject
    } else {
        let t = record.map(|r| rec_str(r, "text")).unwrap_or("");
        let t = if t.is_empty() { topics_topic_text(memory) } else { t.to_string() };
        topics_gist(&t)
    };
    let planned = topics_gist(known.map(|k| rec_str(k, "plan")).unwrap_or(""));
    if !planned.is_empty() && !now.is_empty() && now.contains(&planned) {
        return planned;
    }
    if now.is_empty() { planned } else { now }
}

static NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.NOT_THE_THING"));
static MEASURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.MEASURE_WORDS"));
static A_FAULT: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.A_FAULT"));
static A_GETTING: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.A_GETTING"));
static PLACE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.PLACE_WORDS"));

/// DUP(bot.memory.named::_the_noun_in): the content words of a phrase, in order.
fn named_the_noun_in(said: &str) -> Vec<String> {
    if said.is_empty() {
        return Vec::new();
    }
    said.split(|c: char| !c.is_ascii_lowercase())
        .filter(|w| {
            w.chars().count() > 2
                && !NOT_THE_THING.contains(*w)
                && !MEASURE_WORDS.contains(*w)
                && !A_FAULT.contains(*w)
                && !["the", "and", "for", "with", "some", "any", "new"].contains(w)
        })
        .map(str::to_string)
        .collect()
}

/// DUP(bot.memory.named::_the_record_before): the record before the newest one.
fn named_the_record_before(memory: &Memory) -> Option<&Value> {
    let now = mem(memory, "topic").map(|n| rec_str(n, "text")).unwrap_or("");
    let Some(Value::List(records)) = mem(memory, "topics") else {
        return None;
    };
    records
        .iter()
        .filter(|r| matches!(r, Value::Dict(_)) && r.get("text").map(Value::truthy).unwrap_or(false))
        .rev()
        .find(|r| rec_str(r, "text") != now)
}

/// DUP(bot.memory.named::subject_word): the thing the chat is on, in one word, or "".
pub fn subject_word(memory: &Memory) -> String {
    let said = text::squash(&topics_subject(memory)).to_lowercase();
    let mut found = named_the_noun_in(&said);
    if !found.is_empty() && found.iter().all(|w| A_GETTING.contains(w.as_str())) {
        let own = mem(memory, "topic").map(|n| rec_str(n, "text")).unwrap_or("");
        found = named_the_noun_in(&text::squash(own).to_lowercase());
    }
    if let Some(last) = found.last() {
        if PLACE_WORDS.contains(last.as_str()) {
            if let Some(before) = named_the_record_before(memory) {
                let again = named_the_noun_in(&text::squash(rec_str(before, "text")).to_lowercase());
                if !again.is_empty() {
                    found = again;
                }
            }
        }
    }
    found.last().cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// Differential checks against PyBot's own functions (development aid; run with
// `G2_DIFF_DIR=<dir> cargo test --release --lib <module>::difftest -- --ignored --nocapture`,
// on cases written by the group's diff generator).

#[cfg(test)]
pub mod difftest {
    use crate::data::Value;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;
    use serde_json::{json, Value as J};
    use std::collections::BTreeMap;
    use std::io::BufRead;

    pub trait ToJ {
        fn to_j(&self) -> J;
    }
    impl ToJ for bool {
        fn to_j(&self) -> J {
            json!(self)
        }
    }
    impl ToJ for f64 {
        fn to_j(&self) -> J {
            json!(self)
        }
    }
    impl ToJ for i64 {
        fn to_j(&self) -> J {
            json!(self)
        }
    }
    impl ToJ for String {
        fn to_j(&self) -> J {
            json!(self)
        }
    }
    impl ToJ for &str {
        fn to_j(&self) -> J {
            json!(self)
        }
    }
    impl<T: ToJ> ToJ for Option<T> {
        fn to_j(&self) -> J {
            match self {
                Some(x) => x.to_j(),
                None => J::Null,
            }
        }
    }
    impl<T: ToJ> ToJ for Vec<T> {
        fn to_j(&self) -> J {
            J::Array(self.iter().map(ToJ::to_j).collect())
        }
    }

    /// One case: the function, the reading, the memory, the text argument.
    pub struct Case<'a> {
        pub f: &'a str,
        pub reading: &'a Understood,
        pub memory: Memory,
        pub text: Option<&'a str>,
    }

    fn memory_of(m: &J, reading: &Understood, wants: Option<&str>) -> Memory {
        let mut mem = Memory::new();
        let mut r = reading.clone();
        if let Some(w) = wants {
            r.wants = Some(w.to_string());
            mem.intent_wants = Some(w.to_string());
        }
        mem.understood = Some(r);
        if let Some(o) = m.as_object() {
            for (k, v) in o {
                match k.as_str() {
                    "turns" => {
                        for t in v.as_array().unwrap() {
                            mem.turns.push(Turn {
                                user: t["user"].as_str().unwrap_or("").to_string(),
                                bot: t["bot"].as_str().unwrap_or("").to_string(),
                                module: None,
                            })
                        }
                    }
                    "intent_wants" => mem.intent_wants = v.as_str().map(str::to_string),
                    _ => {
                        mem.extra.insert(k.clone(), Value::from_plain(v));
                    }
                }
            }
        }
        mem
    }

    /// Run every case in `$G2_DIFF_DIR` through `dispatch` and report the functions that differ.
    pub fn run(dispatch: impl Fn(&mut Case) -> Option<J>) {
        let Ok(dir) = std::env::var("G2_DIFF_DIR") else {
            eprintln!("G2_DIFF_DIR not set");
            return;
        };
        let readings: Vec<Option<Understood>> = std::io::BufReader::new(std::fs::File::open(format!("{dir}/readings.jsonl")).unwrap())
            .lines()
            .map(|l| serde_json::from_str(&l.unwrap()).ok())
            .collect();
        let memories: Vec<J> = std::io::BufReader::new(std::fs::File::open(format!("{dir}/memories.jsonl")).unwrap())
            .lines()
            .map(|l| serde_json::from_str(&l.unwrap()).unwrap())
            .collect();
        let only = std::env::var("G2_FN").ok();
        let mut stats: BTreeMap<String, (usize, usize, usize, Vec<String>)> = BTreeMap::new();
        crate::runtime::quiet_panics();
        for line in std::io::BufReader::new(std::fs::File::open(format!("{dir}/cases.jsonl")).unwrap()).lines() {
            let c: J = serde_json::from_str(&line.unwrap()).unwrap();
            let f = c["f"].as_str().unwrap();
            if let Some(o) = &only {
                if f != o {
                    continue;
                }
            }
            let st = stats.entry(f.to_string()).or_default();
            let Some(Some(reading)) = readings.get(c["r"].as_u64().unwrap() as usize) else {
                st.2 += 1;
                continue;
            };
            let memory = match c["m"].as_u64() {
                Some(i) => memory_of(&memories[i as usize], reading, c["w"].as_str()),
                None => memory_of(&J::Null, reading, None),
            };
            let text = c["t"].as_str();
            let mut case = Case { f, reading, memory, text };
            let got = crate::runtime::guarded(|| dispatch(&mut case));
            let raised = !c["raised"].is_null();
            let want = &c["want"];
            let ok = match &got {
                None => raised,                              // panicked
                Some(None) => {
                    st.2 += 1;
                    continue;
                } // not dispatched
                Some(Some(g)) => (raised && g.is_null()) || (!raised && g == want),
            };
            if ok {
                st.0 += 1;
            } else {
                st.1 += 1;
                if st.3.len() < 3 {
                    st.3.push(format!(
                        "text {:?} mem {} t {:?}\n      rust {}\n      py   {} {}",
                        reading.text,
                        c["m"],
                        text,
                        got.map(|g| g.unwrap().to_string()).unwrap_or("PANIC".into()),
                        want,
                        if raised { c["raised"].to_string() } else { String::new() }
                    ));
                }
            }
        }
        let (mut ok, mut bad) = (0, 0);
        for (f, (o, b, skipped, ex)) in &stats {
            ok += o;
            bad += b;
            if *b > 0 || *skipped > 0 {
                println!("{f}: {o} ok, {b} differ, {skipped} skipped");
                for e in ex {
                    println!("    {e}");
                }
            }
        }
        println!("functions {}; cases {ok} ok / {bad} differ", stats.len());
    }
}
