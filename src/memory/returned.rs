//! Port of `bot/memory/returned.py`.
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

/// Python: bot/memory/returned.py:91 `reads_as_returning(text)`
pub fn reads_as_returning(text: &str) -> bool {
    crate::unported!("bot.memory.returned:reads_as_returning")
}

/// Python: bot/memory/returned.py:103 `_the_turn_before(memory)` (never called on the trace corpus)
pub fn _the_turn_before(memory: &Memory) -> Value {
    crate::unported!("bot.memory.returned:_the_turn_before")
}

/// Python: bot/memory/returned.py:108 `_the_answer_before(memory, keep=4)` (never called on the trace corpus)
pub fn _the_answer_before(memory: &Memory, keep: i64) -> Value {
    crate::unported!("bot.memory.returned:_the_answer_before")
}

/// Python: bot/memory/returned.py:141 `the_next_step(reply)` (never called on the trace corpus)
pub fn the_next_step(reply: &str) -> Value {
    crate::unported!("bot.memory.returned:the_next_step")
}

/// Python: bot/memory/returned.py:185 `says_it_happened_again(text)`
pub fn says_it_happened_again(text: &str) -> bool {
    crate::unported!("bot.memory.returned:says_it_happened_again")
}

/// Python: bot/memory/returned.py:201 `reads_as_a_conditional(text)`
pub fn reads_as_a_conditional(text: &str) -> bool {
    crate::unported!("bot.memory.returned:reads_as_a_conditional")
}

/// Python: bot/memory/returned.py:211 `pool_for(text, memory)`
pub fn pool_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.returned:pool_for")
}

/// Python: bot/memory/returned.py:240 `the_whole_of_it(text, memory)` (never called on the trace corpus)
pub fn the_whole_of_it(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.returned:the_whole_of_it")
}

