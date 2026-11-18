//! Port of `bot/memory/prefer.py`.
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

/// Python: bot/memory/prefer.py:79 `_offered_choices(reply)` (never called on the trace corpus)
pub fn _offered_choices(reply: &str) -> bool {
    crate::unported!("bot.memory.prefer:_offered_choices")
}

/// Python: bot/memory/prefer.py:84 `_said_the_medium(said)` (never called on the trace corpus)
pub fn _said_the_medium(said: &str) -> Value {
    crate::unported!("bot.memory.prefer:_said_the_medium")
}

/// Python: bot/memory/prefer.py:97 `_last_turn(memory)` (never called on the trace corpus)
pub fn _last_turn(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.prefer:_last_turn")
}

/// Python: bot/memory/prefer.py:105 `_taste(text)`
pub fn _taste(text: &str) -> String {
    crate::unported!("bot.memory.prefer:_taste")
}

/// Python: bot/memory/prefer.py:148 `about_the_medium(answer, memory)` (never called on the trace corpus)
pub fn about_the_medium(answer: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.prefer:about_the_medium")
}

/// Python: bot/memory/prefer.py:162 `served_already(text, memory)`
pub fn served_already(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.prefer:served_already")
}

/// Python: bot/memory/prefer.py:177 `follow_up(text, memory)`
pub fn follow_up(text: &str, memory: &Value) -> String {
    crate::unported!("bot.memory.prefer:follow_up")
}

