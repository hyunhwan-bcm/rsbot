//! Entries and closures a knowledge module builds in code, for the Python factories that
//! build them (`_verdict(...)`, `_reply_for(...)`, `_pick(...)`).
//!
//! At import time PyBot ran every factory once and the bundle holds what they gave, which is
//! what the knowledge base reads. The factories are ported all the same (PORTING.md: every
//! function), building the same objects: an [`Entry`] through `lookup.entry`, and a closure as
//! the [`FnRef`] the registry runs, with the values it captured by name. `same_entry` is how a
//! test holds a factory's output against the exported one.

use crate::data::{FnRef, Value};
use crate::knowledge::entry::{Entry, KeyPart, Lead, Reply};
use std::sync::Arc;

/// A tuple of strings, as Python writes a word list.
pub fn words(items: &[String]) -> Value {
    Value::Tuple(Arc::new(items.iter().map(Value::from).collect()))
}

/// A tuple of values.
pub fn tuple(items: Vec<Value>) -> Value {
    Value::Tuple(Arc::new(items))
}

/// A closure: the function's Python id ("module:qualname:line") and what it captured.
pub fn closure(id: &str, cells: Vec<(&str, Value)>) -> FnRef {
    FnRef { id: id.to_string(), cells: cells.into_iter().map(|(k, v)| (k.to_string(), v)).collect(), defaults: Vec::new() }
}

/// A plain module function, as an entry names it.
pub fn function(id: &str) -> FnRef {
    closure(id, Vec::new())
}

/// One key part: a word, or a group of words any of which fills the position.
pub fn part(words: &[String]) -> KeyPart {
    KeyPart::Any(words.to_vec())
}

/// `entry(id, key, reply, avoid=..., bias=..., ask=..., guard=...)`, with the rest of
/// `lookup.entry`'s arguments at their defaults; set the fields for any other.
pub fn entry(id: &str, key: Vec<KeyPart>, reply: Reply, avoid: &[Value], bias: i64, ask: Vec<KeyPart>, guard: Option<FnRef>) -> Entry {
    crate::knowledge::lookup::entry(id, key, reply, avoid, bias, ask, guard)
}

/// What a test compares of two entries: everything `lookup.entry` set, and the reply / guard
/// functions by id and captured values. `None` when they agree, else what differs.
pub fn same_entry(built: &Entry, exported: &Entry) -> Option<String> {
    let reply = |r: &Reply| match r {
        Reply::Text(s) => format!("text {s:?}"),
        Reply::Choices(v) => format!("choices {v:?}"),
        Reply::Fn(f) => format!("fn {} {:?}", f.id, f.cells),
    };
    let lead = |l: &Lead| match l {
        Lead::Text(s) => format!("{s:?}"),
        Lead::Shaped(v) => format!("{v:?}"),
    };
    let checks: [(&str, bool); 12] = [
        ("id", built.id == exported.id),
        ("key", built.key == exported.key),
        ("avoid", built.avoid == exported.avoid),
        ("bias", built.bias == exported.bias),
        ("ask", built.ask == exported.ask),
        ("slips", built.slips == exported.slips),
        ("fault", built.fault == exported.fault),
        ("report", built.report == exported.report),
        ("explain", built.explain == exported.explain),
        ("lead", lead(&built.lead) == lead(&exported.lead)),
        ("reply", same_fn_reply(&built.reply, &exported.reply)),
        ("guard", same_fn(built.guard.as_ref(), exported.guard.as_ref())),
    ];
    let bad: Vec<&str> = checks.iter().filter(|(_, ok)| !ok).map(|(n, _)| *n).collect();
    if bad.is_empty() {
        return None;
    }
    Some(format!("{}: {bad:?} differ; built reply {} / exported {}", built.id, reply(&built.reply), reply(&exported.reply)))
}

fn same_fn_reply(a: &Reply, b: &Reply) -> bool {
    match (a, b) {
        (Reply::Text(x), Reply::Text(y)) => x == y,
        (Reply::Choices(x), Reply::Choices(y)) => x == y,
        (Reply::Fn(x), Reply::Fn(y)) => same_fn(Some(x), Some(y)),
        _ => false,
    }
}

fn same_fn(a: Option<&FnRef>, b: Option<&FnRef>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => {
            x.id == y.id && x.cells.len() == y.cells.len() && x.cells.iter().all(|(k, v)| y.cell(k).is_some_and(|w| same_value(v, w)))
        }
        _ => false,
    }
}

/// Python `==`, where a closure's captured function compares by its id and captures.
fn same_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Fn(x), Value::Fn(y)) => same_fn(Some(x), Some(y)),
        (Value::List(x) | Value::Tuple(x), Value::List(y) | Value::Tuple(y)) => x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| same_value(p, q)),
        _ => a == b,
    }
}

/// The exported entries of a module's `ENTRIES` (or another list of them), as entries.
pub fn exported(name: &str) -> Vec<Entry> {
    crate::data::get(name).items().iter().enumerate().map(|(i, v)| Entry::from_value(i, v)).collect()
}

/// For a test: every built row agrees with the exported row of the same id (and one exists).
pub fn assert_rows_match(built: &[Entry], exported_name: &str) {
    let exported = exported(exported_name);
    assert!(!built.is_empty(), "no rows built");
    for mine in built {
        let theirs = exported.iter().find(|r| r.id == mine.id).unwrap_or_else(|| panic!("no exported row {}", mine.id));
        assert_eq!(same_entry(mine, theirs), None);
    }
}

/// A reply as Python held it: a string, a function, or a pool of lines.
pub fn reply_of(v: &Value) -> Reply {
    match v {
        Value::Str(s) => Reply::Text(s.to_string()),
        Value::Fn(f) => Reply::Fn((**f).clone()),
        other => Reply::Choices(other.strs()),
    }
}
