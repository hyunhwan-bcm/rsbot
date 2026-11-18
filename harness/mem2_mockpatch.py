"""Make a copy of the crate in which the functions the MEM2 files call and other workstreams have
not ported yet answer from a mem2_record.py recording, so the MEM2 ports can be checked alone.

    rsync -a --delete src Cargo.toml Cargo.lock COPY/
    python3 harness/mem2_mockpatch.py COPY
    cd COPY && CARGO_TARGET_DIR=COPY_TARGET cargo build --profile fast --bin mem2_check
    RSBOT_DATA=../data/export/bundle.json RSBOT_TODAY=2026-11-17 MEM2_MOCK=REC.jsonl.gz \
        COPY_TARGET/fast/mem2_check REC.jsonl.gz

Test-only: the copy is never merged. last_user and topics.resumed are written out (they only read
memory); the rest look up the recorded call by its arguments."""
import re, sys
root = sys.argv[1]

MOCK = r'''//! Test-only: answers for functions other workstreams have not ported, read from a
//! mem2_record.py recording (MEM2_MOCK=path). Never part of the real crate.
use crate::data::Value;
use serde_json::Value as J;
use std::collections::HashMap;
use std::io::BufRead;
use std::sync::LazyLock;

static TABLE: LazyLock<HashMap<String, J>> = LazyLock::new(|| {
    let mut out = HashMap::new();
    let Ok(path) = std::env::var("MEM2_MOCK") else { return out };
    for p in path.split(',') {
        let file = std::fs::File::open(p).unwrap();
        for line in std::io::BufReader::new(flate2::read::GzDecoder::new(file)).lines() {
            let row: J = serde_json::from_str(&line.unwrap()).unwrap();
            if row["raised"].is_string() { continue; }
            let key = format!("{}|{}", row["fn"].as_str().unwrap(), serde_json::to_string(&row["args"]).unwrap());
            out.insert(key, row["result"].clone());
        }
    }
    out
});

pub fn get(name: &str, args: J) -> J {
    let key = format!("{}|{}", name, serde_json::to_string(&args).unwrap());
    match TABLE.get(&key) {
        Some(j) => j.clone(),
        None => panic!("UNPORTED mock miss {key}"),
    }
}

pub fn value(name: &str, args: J) -> Value {
    Value::from_plain(&get(name, args))
}

/// The first of several argument shapes the recording has (Python's None for Rust's "").
pub fn get_any(name: &str, shapes: Vec<J>) -> J {
    for args in &shapes {
        let key = format!("{}|{}", name, serde_json::to_string(args).unwrap());
        if let Some(j) = TABLE.get(&key) {
            return j.clone();
        }
    }
    panic!("UNPORTED mock miss {name} {:?}", shapes)
}

pub fn re_arg(v: &Value) -> J {
    match v.as_re() {
        Some(r) => serde_json::json!({"__re__": [r.pattern(), r.flags()]}),
        None => J::Null,
    }
}
'''

def patch(path, old, new):
    p = f"{root}/{path}"
    s = open(p).read()
    assert old in s, (path, old)
    s = s.replace(old, new)
    open(p, "w").write(s)

open(f"{root}/src/mem2_mock.rs", "w").write(MOCK)
patch("src/lib.rs", "pub mod memory;", "pub mod memory;\npub mod mem2_mock;")
patch("src/memory/taste.rs", 'pub fn read(line: &str) -> Value {\n    crate::unported!("bot.memory.taste:read")',
      'pub fn read(line: &str) -> Value {\n    crate::mem2_mock::value("bot.memory.taste:read", serde_json::json!([line]))')
patch("src/memory/taste.rs", 'pub fn read_all(line: &str) -> Value {\n    crate::unported!("bot.memory.taste:read_all")',
      'pub fn read_all(line: &str) -> Value {\n    crate::mem2_mock::value("bot.memory.taste:read_all", serde_json::json!([line]))')
patch("src/memory/taste.rs", 'pub fn said_about(text: &str, position: i64) -> bool {\n    crate::unported!("bot.memory.taste:said_about")',
      'pub fn said_about(text: &str, position: i64) -> bool {\n    let p = crate::py::text::char_index(text, position as usize);\n'
      '    crate::mem2_mock::get("bot.memory.taste:said_about", serde_json::json!([text, p])).as_bool().unwrap()')
patch("src/memory/taste.rs", 'pub fn read_other(line: &str) -> Vec<String> {\n    crate::unported!("bot.memory.taste:read_other")',
      'pub fn read_other(line: &str) -> Vec<Value> {\n    crate::mem2_mock::value("bot.memory.taste:read_other", serde_json::json!([line])).items().to_vec()')
patch("src/memory/filler.rs", 'pub fn without_filler(said: &str) -> String {\n    crate::unported!("bot.memory.filler:without_filler")',
      'pub fn without_filler(said: &str) -> String {\n    crate::mem2_mock::get("bot.memory.filler:without_filler", serde_json::json!([said])).as_str().unwrap().to_string()')
patch("src/memory/named.rs", 'pub fn the_name_in(value: &Value) -> String {\n    crate::unported!("bot.memory.named:the_name_in")',
      'pub fn the_name_in(value: &Value) -> String {\n    crate::mem2_mock::get("bot.memory.named:the_name_in", serde_json::json!([value.to_plain()])).as_str().unwrap_or("").to_string()')
patch("src/compose/badnews.rs", 'pub fn reads_like_a_symptom(clean: &str) -> bool {\n    crate::unported!("bot.compose.badnews:reads_like_a_symptom")',
      'pub fn reads_like_a_symptom(clean: &str) -> bool {\n    crate::mem2_mock::get("bot.compose.badnews:reads_like_a_symptom", serde_json::json!([clean])).as_bool().unwrap()')
patch("src/memory/quotes.rs", 'pub fn closed(said: &str, rest: &[String], whole: &str) -> String {\n    crate::unported!("bot.memory.quotes:closed")',
      'pub fn closed(said: &str, rest: &[String], whole: &str) -> String {\n    crate::mem2_mock::get("bot.memory.quotes:closed", serde_json::json!([said, rest, whole])).as_str().unwrap().to_string()')
patch("src/memory/quotes.rs", 'pub fn sentences(text: &str, splitter: &Value) -> Vec<String> {\n    crate::unported!("bot.memory.quotes:sentences")',
      'pub fn sentences(text: &str, splitter: &Value) -> Vec<String> {\n    crate::mem2_mock::value("bot.memory.quotes:sentences", serde_json::json!([text, crate::mem2_mock::re_arg(splitter)])).strs()')
patch("src/memory/returned.rs", 'pub fn the_next_step(reply: &str) -> Value {\n    crate::unported!("bot.memory.returned:the_next_step")',
      'pub fn the_next_step(reply: &str) -> Value {\n    crate::mem2_mock::value("bot.memory.returned:the_next_step", serde_json::json!([reply]))')
patch("src/memory/returned.rs", 'pub fn says_it_happened_again(text: &str) -> bool {\n    crate::unported!("bot.memory.returned:says_it_happened_again")',
      'pub fn says_it_happened_again(text: &str) -> bool {\n    crate::mem2_mock::get("bot.memory.returned:says_it_happened_again", serde_json::json!([text])).as_bool().unwrap()')
patch("src/memory/repeat.rs", 'pub fn asks_for_the_move(text: &str) -> bool {\n    crate::unported!("bot.memory.repeat:asks_for_the_move")',
      'pub fn asks_for_the_move(text: &str) -> bool {\n    crate::mem2_mock::get("bot.memory.repeat:asks_for_the_move", serde_json::json!([text])).as_bool().unwrap()')
patch("src/memory/topics.rs", '    pub fn resumed(memory: &Memory) {\n        crate::unported!("bot.memory.topics:resumed")',
      '    pub fn resumed(memory: &Memory) -> Value {\n        let Some(record) = memory.get("returned") else { return Value::None };\n        if !record.is_dict() || !record.get("text").is_some_and(Value::truthy) { return Value::None }\n        if record.get("turn") != Some(&Value::Int(memory.turns.len() as i64)) { return Value::None }\n        return record.clone();')
patch("src/memory/named.rs", 'pub fn carries_a_name(text: &str) -> bool {\n    crate::unported!("bot.memory.named:carries_a_name")',
      'pub fn carries_a_name(text: &str) -> bool {\n    crate::mem2_mock::get("bot.memory.named:carries_a_name", serde_json::json!([text])).as_bool().unwrap()')
patch("src/memory/relations.rs", 'pub fn _tidy_name(value: &Value) -> Value {\n    crate::unported!("bot.memory.relations:_tidy_name")',
      'pub fn _tidy_name(value: &Value) -> Value {\n    crate::mem2_mock::value("bot.memory.relations:_tidy_name", serde_json::json!([value.to_plain()]))')
patch("src/memory/relations.rs", 'pub fn _animal_family(animal: &str) -> String {\n    crate::unported!("bot.memory.relations:_animal_family")',
      'pub fn _animal_family(animal: &str) -> String {\n    crate::mem2_mock::get_any("bot.memory.relations:_animal_family", vec![serde_json::json!([animal]), serde_json::json!([null])]).as_str().unwrap_or("").to_string()')
patch("src/memory/mod.rs", '    pub fn plan_sentence(plan: &str, said_line: &str) -> String {\n        crate::unported!("bot.memory:plan_sentence")',
      '    pub fn plan_sentence(plan: &str, said_line: &str) -> String {\n        crate::mem2_mock::get_any("bot.memory:plan_sentence", vec![serde_json::json!([plan, said_line]), serde_json::json!([plan, null])]).as_str().unwrap_or("").to_string()')
patch("src/memory/mod.rs", '    pub fn last_user(memory: &Memory, n: i64) -> Value {\n        crate::unported!("bot.memory:last_user")',
      '    pub fn last_user(memory: &Memory, n: i64) -> Value {\n        let from = memory.turns.len().saturating_sub(n.max(0) as usize);\n        let lines: Vec<Value> = memory.turns[from..].iter().map(|t| Value::from(t.user.as_str())).collect();\n        return if n > 1 { Value::from(lines) } else { lines.last().cloned().unwrap_or_else(|| Value::from("")) };')
print("patched")
