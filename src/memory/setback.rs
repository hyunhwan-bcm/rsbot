//! Port of `bot/memory/setback.py`.
//!
//! GENERATED SKELETON: every function is a stub until its workstream ports it. The
//! signatures are the contract other files call against; keep them.
#![allow(unused_variables, unused_imports, non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::Re;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::sync::LazyLock;

/// Python: bot/memory/setback.py:188 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.setback:_flat")
}

/// Python: bot/memory/setback.py:192 `_thing(text)` (never called on the trace corpus)
pub fn _thing(text: &str) -> String {
    crate::unported!("bot.memory.setback:_thing")
}

/// Python: bot/memory/setback.py:212 `_named_after(text)` (never called on the trace corpus)
pub fn _named_after(text: &str) -> String {
    crate::unported!("bot.memory.setback:_named_after")
}

/// Python: bot/memory/setback.py:228 `_bare_subject(head)` (never called on the trace corpus)
pub fn _bare_subject(head: &str) -> Value {
    crate::unported!("bot.memory.setback:_bare_subject")
}

/// Python: bot/memory/setback.py:245 `_refused(name)` (never called on the trace corpus)
pub fn _refused(name: &str) -> bool {
    crate::unported!("bot.memory.setback:_refused")
}

/// Python: bot/memory/setback.py:250 `reading(text)`
pub fn reading(text: &str) {
    crate::unported!("bot.memory.setback:reading")
}

/// Python: bot/memory/setback.py:288 `_a_plural_thing(name)` (never called on the trace corpus)
pub fn _a_plural_thing(name: &str) -> bool {
    crate::unported!("bot.memory.setback:_a_plural_thing")
}

/// Python: bot/memory/setback.py:299 `_agrees_with_the_thing(line, name)` (never called on the trace corpus)
pub fn _agrees_with_the_thing(line: &str, name: &str) -> Value {
    crate::unported!("bot.memory.setback:_agrees_with_the_thing")
}

/// Python: bot/memory/setback.py:306 `pool(text)` (never called on the trace corpus)
pub fn pool(text: &str) -> Value {
    crate::unported!("bot.memory.setback:pool")
}

