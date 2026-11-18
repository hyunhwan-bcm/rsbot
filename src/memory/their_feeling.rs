//! Port of `bot/memory/their_feeling.py`.
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

/// Python: bot/memory/their_feeling.py:109 `_the_last_person(text, memory)` (never called on the trace corpus)
pub fn _the_last_person(text: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.their_feeling:_the_last_person")
}

/// Python: bot/memory/their_feeling.py:136 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.their_feeling:reply_for")
}

/// Python: bot/memory/their_feeling.py:171 `_says_something(thing)` (never called on the trace corpus)
pub fn _says_something(thing: &str) -> bool {
    crate::unported!("bot.memory.their_feeling:_says_something")
}

/// Python: bot/memory/their_feeling.py:177 `_an_earlier_turn(memory, low)` (never called on the trace corpus)
pub fn _an_earlier_turn(memory: &Memory, low: &str) -> Option<bool> {
    crate::unported!("bot.memory.their_feeling:_an_earlier_turn")
}

/// Python: bot/memory/their_feeling.py:185 `_fill(frame, person, thing)` (never called on the trace corpus)
pub fn _fill(frame: &Value, person: &Value, thing: &str) -> String {
    crate::unported!("bot.memory.their_feeling:_fill")
}

/// Python: bot/memory/their_feeling.py:197 `_vary(options, memory, line='')` (never called on the trace corpus)
pub fn _vary(options: &Value, memory: &Memory, line: &str) -> Value {
    crate::unported!("bot.memory.their_feeling:_vary")
}

