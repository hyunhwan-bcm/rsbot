//! Port of `bot/memory/times.py`.
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

/// Python: bot/memory/times.py:29 `_a_routine(phrase, ask=False)`
pub fn _a_routine(phrase: &str, ask: bool) -> String {
    crate::unported!("bot.memory.times:_a_routine")
}

/// Python: bot/memory/times.py:62 `_the_times(memory)`
pub fn _the_times(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.times:_the_times")
}

/// Python: bot/memory/times.py:69 `_keep_a_time(memory, what, at, said)`
pub fn _keep_a_time(memory: &mut Memory, what: &str, at: &str, said: &str) {
    crate::unported!("bot.memory.times:_keep_a_time")
}

/// Python: bot/memory/times.py:106 `keep(text, memory)`
pub fn keep(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.times:keep")
}

/// Python: bot/memory/times.py:166 `answer(text, memory)`
pub fn answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.times:answer")
}

/// Python: bot/memory/times.py:202 `is_a_time_ask(text, memory)`
pub fn is_a_time_ask(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.times:is_a_time_ask")
}

