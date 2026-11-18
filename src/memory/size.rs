//! Port of `bot/memory/size.py`.
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

/// Python: bot/memory/size.py:75 `_flat(text)` (never called on the trace corpus)
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.size:_flat")
}

/// Python: bot/memory/size.py:93 `_one_of_ours(answer)` (never called on the trace corpus)
pub fn _one_of_ours(answer: &str) -> bool {
    crate::unported!("bot.memory.size:_one_of_ours")
}

/// Python: bot/memory/size.py:123 `_the_thing_asked(memory, text)` (never called on the trace corpus)
pub fn _the_thing_asked(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.size:_the_thing_asked")
}

/// Python: bot/memory/size.py:143 `_the_last_line(memory)` (never called on the trace corpus)
pub fn _the_last_line(memory: &Memory) -> String {
    crate::unported!("bot.memory.size:_the_last_line")
}

/// Python: bot/memory/size.py:152 `_the_answer(memory)` (never called on the trace corpus)
pub fn _the_answer(memory: &Memory) -> Value {
    crate::unported!("bot.memory.size:_the_answer")
}

/// Python: bot/memory/size.py:171 `_the_rule(answer)` (never called on the trace corpus)
pub fn _the_rule(answer: &str) -> Value {
    crate::unported!("bot.memory.size:_the_rule")
}

/// Python: bot/memory/size.py:195 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.size:answer_for")
}

