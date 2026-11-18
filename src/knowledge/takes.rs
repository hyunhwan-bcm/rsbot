//! Port of PyBot's `bot/knowledge/takes.py`: how long a job takes, for rows whose subject must be
//! in the line the person typed.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;
use std::sync::LazyLock;

static _ASK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.takes._ASK"));

// `_typed(*words)` is an import-time factory: it flattens the words into the closure's `flat`
// cell, and the guard below reads that cell.

/// `_typed(...)`'s guard: the row's subject has to be in the line the person typed.
pub fn _typed_guard(flat: &[Value], understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood).to_lowercase();
    flat.iter().any(|word| said.contains(word.str()))
}

/// A shape: whether the line asks for a length of time ("how long", "how soon", ...).
pub fn _asks_a_length(text: &str) -> bool {
    let low = text.to_lowercase();
    _ASK.iter().any(|part| low.contains(part.as_str()))
}

fn g_typed(f: &FnRef, u: &Understood) -> bool {
    _typed_guard(f.cell("flat").map(Value::items).unwrap_or_default(), u)
}
fn s_asks_a_length(_f: &FnRef, text: &str) -> bool {
    _asks_a_length(text)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.takes:_typed.<locals>.guard:59", g_typed)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
pub const SHAPES: &[(&str, ShapeFn)] = &[("bot.knowledge.takes:_asks_a_length:65", s_asks_a_length)];


// ---------------------------------------------------------------------------------------------
// The guard factory.

/// `_typed(*words)`: a guard - the row's subject has to be in the line the person typed. Each
/// argument is a word or a group of words; the closure keeps them flat.
pub fn _typed(words: &[Value]) -> FnRef {
    let mut flat: Vec<Value> = Vec::new();
    for word in words {
        match word {
            Value::Tuple(_) | Value::List(_) => flat.extend(word.items().iter().cloned()),
            other => flat.push(other.clone()),
        }
    }
    crate::knowledge::built::closure("bot.knowledge.takes:_typed.<locals>.guard:59", vec![("flat", Value::from(flat))])
}
