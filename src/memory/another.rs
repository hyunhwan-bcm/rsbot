//! Port of `bot/memory/another.py`.
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

/// Python: bot/memory/another.py:251 `names_nothing(text)`
pub fn names_nothing(text: &str) -> bool {
    crate::unported!("bot.memory.another:names_nothing")
}

/// Python: bot/memory/another.py:269 `ruled_out_words(text)`
pub fn ruled_out_words(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.another:ruled_out_words")
}

/// Python: bot/memory/another.py:277 `_names_one(part, ruled_out)` (never called on the trace corpus)
pub fn _names_one(part: &str, ruled_out: &Value) -> bool {
    crate::unported!("bot.memory.another:_names_one")
}

/// Python: bot/memory/another.py:284 `_the_turn_to_lean_on(turns, memory=None)`
pub fn _the_turn_to_lean_on(turns: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.another:_the_turn_to_lean_on")
}

/// Python: bot/memory/another.py:305 `split_sentences(said)`
pub fn split_sentences(said: &str) -> Vec<String> {
    crate::unported!("bot.memory.another:split_sentences")
}

/// Python: bot/memory/another.py:340 `_sentences(said)`
pub fn _sentences(said: &str) -> Vec<String> {
    crate::unported!("bot.memory.another:_sentences")
}

/// Python: bot/memory/another.py:347 `_said_before(memory, said, before=None)` (never called on the trace corpus)
pub fn _said_before(memory: &Memory, said: &str, before: &str) -> bool {
    crate::unported!("bot.memory.another:_said_before")
}

/// Python: bot/memory/another.py:367 `_cut(said)` (never called on the trace corpus)
pub fn _cut(said: &str) -> Value {
    crate::unported!("bot.memory.another:_cut")
}

/// Python: bot/memory/another.py:397 `_a_caveat_about_getting_help(part)`
pub fn _a_caveat_about_getting_help(part: &str) -> bool {
    crate::unported!("bot.memory.another:_a_caveat_about_getting_help")
}

/// Python: bot/memory/another.py:417 `is_a_step(part)`
pub fn is_a_step(part: &str) -> bool {
    crate::unported!("bot.memory.another:is_a_step")
}

/// Python: bot/memory/another.py:422 `_the_rest(said, memory, ruled_out=(), at=None)`
pub fn _the_rest(said: &str, memory: &Memory, ruled_out: &[String], at: i64) -> String {
    crate::unported!("bot.memory.another:_the_rest")
}

/// Python: bot/memory/another.py:478 `other_option(text, memory)`
pub fn other_option(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.another:other_option")
}

