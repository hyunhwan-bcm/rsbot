//! Port of `bot/memory/reminded.py`.
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

/// Python: bot/memory/reminded.py:96 `_trim(words)` (never called on the trace corpus)
pub fn _trim(words: &[String]) -> String {
    crate::unported!("bot.memory.reminded:_trim")
}

/// Python: bot/memory/reminded.py:102 `_said_before(thing, memory)` (never called on the trace corpus)
pub fn _said_before(thing: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.reminded:_said_before")
}

/// Python: bot/memory/reminded.py:123 `the_thing(text, memory=None)`
pub fn the_thing(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.reminded:the_thing")
}

/// Python: bot/memory/reminded.py:169 `_the_thing_itself(phrase)` (never called on the trace corpus)
pub fn _the_thing_itself(phrase: &str) -> String {
    crate::unported!("bot.memory.reminded:_the_thing_itself")
}

/// Python: bot/memory/reminded.py:177 `_resolved(pronoun, memory)` (never called on the trace corpus)
pub fn _resolved(pronoun: &Value, memory: &Memory) -> String {
    crate::unported!("bot.memory.reminded:_resolved")
}

/// Python: bot/memory/reminded.py:193 `_said_what_was_wrong(thing, memory)` (never called on the trace corpus)
pub fn _said_what_was_wrong(thing: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.reminded:_said_what_was_wrong")
}

/// Python: bot/memory/reminded.py:217 `about_a_thing_they_said(text, memory=None)`
pub fn about_a_thing_they_said(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.reminded:about_a_thing_they_said")
}

