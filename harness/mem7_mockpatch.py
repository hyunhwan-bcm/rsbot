"""Make a test-only copy of the crate in which the functions the MEM7 files call and other
workstreams have not ported yet answer from a mem7_record.py recording ("dep" rows), so the MEM7
ports can be checked past them. The few that only read memory are written out from the Python.

    rsync -a --delete src Cargo.toml Cargo.lock COPY/
    python3 harness/mem7_mockpatch.py COPY
    cd COPY && CARGO_TARGET_DIR=COPY_TARGET cargo build --profile fast --bin mem7_check
    RSBOT_DATA=../data/export/bundle.json RSBOT_TODAY=2026-11-17 MEM7_MOCK=REC.jsonl.gz \\
        COPY_TARGET/fast/mem7_check REC.jsonl.gz

Never merged: the copy exists only to run the checker."""
import sys

root = sys.argv[1]

MOCK = r'''//! Test-only: answers for functions other workstreams have not ported, read from the "dep" rows
//! of a mem7_record.py recording (MEM7_MOCK=path). Never part of the real crate.
use crate::data::Value;
use serde_json::{json, Value as J};
use std::collections::HashMap;
use std::io::BufRead;
use std::sync::LazyLock;

static TABLE: LazyLock<HashMap<String, J>> = LazyLock::new(|| {
    let mut out = HashMap::new();
    let Ok(path) = std::env::var("MEM7_MOCK") else { return out };
    for p in path.split(',') {
        let file = std::fs::File::open(p).unwrap();
        for line in std::io::BufReader::new(flate2::read::GzDecoder::new(file)).lines() {
            let Ok(line) = line else { break };
            let row: J = serde_json::from_str(&line).unwrap();
            if row.get("dep").is_none() { continue; }
            out.insert(row["key"].as_str().unwrap().to_string(), row["result"].clone());
        }
    }
    out
});

pub fn get(name: &str, key: J) -> J {
    let key = serde_json::to_string(&json!([name, key])).unwrap();
    match TABLE.get(&key) {
        Some(j) => j.clone(),
        None => panic!("UNPORTED mock miss {key}"),
    }
}

pub fn value(name: &str, key: J) -> Value {
    Value::from_plain(&get(name, key))
}

/// `compose.pick(options, memory, seed)`, keyed on the options and the seed. The NUL-led options
/// rates.rs passes for PyBot's `pick([0, 1], memory)` are the ints they stand for.
pub fn pick(options: &[String], seed: &str) -> String {
    let ints = options.len() == 2 && options[0] == "\u{0}0" && options[1] == "\u{0}1";
    let key = if ints { json!([[0, 1], seed]) } else { json!([options, seed]) };
    let got = get("bot.compose:pick", key);
    match got {
        J::Number(n) => options[n.as_u64().unwrap() as usize].clone(),
        J::String(s) => s,
        other => panic!("UNPORTED mock pick gave {other}"),
    }
}
'''


def patch(path, old, new):
    p = f"{root}/{path}"
    s = open(p).read()
    assert old in s, (path, old)
    s = s.replace(old, new)
    open(p, "w").write(s)


open(f"{root}/src/mem7_mock.rs", "w").write(MOCK)
patch("src/lib.rs", "pub mod memory;", "pub mod memory;\npub mod mem7_mock;")
patch("src/memory/kinds.rs",
      'pub fn the_line_that_gives(text: &str, lines: &[String]) {\n    crate::unported!("bot.memory.kinds:the_line_that_gives")',
      'pub fn the_line_that_gives(text: &str, lines: &[String]) -> Value {\n'
      '    crate::mem7_mock::value("bot.memory.kinds:the_line_that_gives", serde_json::json!([text, lines]))')
patch("src/memory/choice.rs",
      'pub fn options_in(reply: &str) -> Vec<String> {\n    crate::unported!("bot.memory.choice:options_in")',
      'pub fn options_in(reply: &str) -> Vec<String> {\n'
      '    crate::mem7_mock::value("bot.memory.choice:options_in", serde_json::json!([reply])).strs()')
patch("src/memory/quotes.rs",
      'pub fn sentences(text: &str, splitter: &Value) -> Vec<String> {\n    crate::unported!("bot.memory.quotes:sentences")',
      'pub fn sentences(text: &str, splitter: &Value) -> Vec<String> {\n'
      '    let re = splitter.as_re().unwrap();\n'
      '    crate::mem7_mock::value("bot.memory.quotes:sentences", serde_json::json!([text, re.pattern(), re.flags()])).strs()')
patch("src/memory/detail.rs",
      'pub fn _a_step(said: &str, questions_are_steps: &Value) -> Value {\n    crate::unported!("bot.memory.detail:_a_step")',
      'pub fn _a_step(said: &str, questions_are_steps: &Value) -> Value {\n'
      '    crate::mem7_mock::value("bot.memory.detail:_a_step", serde_json::json!([said, questions_are_steps.truthy()]))')
patch("src/memory/detail.rs",
      'pub fn _first_clause(step: &Value) -> String {\n    crate::unported!("bot.memory.detail:_first_clause")',
      'pub fn _first_clause(step: &Value) -> String {\n'
      '    crate::mem7_mock::get("bot.memory.detail:_first_clause", serde_json::json!([step.to_plain()])).as_str().unwrap_or("").to_string()')
patch("src/compose/mod.rs",
      '    pub fn pick(options: &[String], memory: &Value, seed_text: &str) -> String {\n        crate::unported!("bot.compose:pick")',
      '    pub fn pick(options: &[String], memory: &Value, seed_text: &str) -> String {\n'
      '        return crate::mem7_mock::pick(options, seed_text);')
# The readers of memory that only read it, written out from bot/memory/history.py and __init__.py.
patch("src/memory/history.rs",
      'pub fn an_earlier_turn(memory: &Memory, text: &str) -> bool {\n    crate::unported!("bot.memory.history:an_earlier_turn")',
      'pub fn an_earlier_turn(memory: &Memory, text: &str) -> bool {\n'
      '    let mine = crate::py::text::lower(&crate::py::text::squash(text));\n'
      '    memory.turns.iter().any(|t| { let line = crate::py::text::squash(&t.user); !line.is_empty() && crate::py::text::lower(&line) != mine })')
patch("src/memory/history.rs",
      'pub fn turns_before(memory: &Memory) -> Value {\n    crate::unported!("bot.memory.history:turns_before")',
      'pub fn turns_before(memory: &Memory) -> Value {\n'
      '    Value::from(memory.turns.iter().map(|t| { let mut d = Value::dict(); d.set_key("user", t.user.as_str()); d.set_key("bot", t.bot.as_str()); d }).collect::<Vec<_>>())')
patch("src/memory/history.rs",
      'pub fn the_last_answer(memory: &Memory) -> String {\n    crate::unported!("bot.memory.history:the_last_answer")',
      'pub fn the_last_answer(memory: &Memory) -> String {\n'
      '    memory.turns.iter().rev().find(|t| !crate::py::text::strip(&t.bot).is_empty()).map_or(String::new(), |t| crate::py::text::squash(&t.bot))')
patch("src/memory/mod.rs",
      '    pub fn there_is_a_chat(memory: &Memory) -> bool {\n        crate::unported!("bot.memory:there_is_a_chat")',
      '    pub fn there_is_a_chat(memory: &Memory) -> bool {\n'
      '        return memory.turns.iter().any(|t| !crate::py::text::strip(&t.user).is_empty());')
patch("src/memory/mod.rs",
      '    pub fn last_user(memory: &Memory, n: i64) -> Value {\n        crate::unported!("bot.memory:last_user")',
      '    pub fn last_user(memory: &Memory, n: i64) -> Value {\n'
      '        let from = memory.turns.len().saturating_sub(n.max(0) as usize);\n'
      '        let lines: Vec<Value> = memory.turns[from..].iter().map(|t| Value::from(t.user.as_str())).collect();\n'
      '        return if n > 1 { Value::from(lines) } else { lines.last().cloned().unwrap_or_else(|| Value::from("")) };')
print("patched")
