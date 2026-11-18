//! Port of `bot/memory/rates.py`.
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

/// Python: bot/memory/rates.py:103 `_the_rates(memory)`
pub fn _the_rates(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.rates:_the_rates")
}

/// Python: bot/memory/rates.py:113 `_keep_a_rate_of_a_doing(said, rate, memory)`
pub fn _keep_a_rate_of_a_doing(said: &str, rate: &Value, memory: &mut Memory) {
    crate::unported!("bot.memory.rates:_keep_a_rate_of_a_doing")
}

/// Python: bot/memory/rates.py:137 `_keep_a_rate(memory, what, how, said, own=True)`
pub fn _keep_a_rate(memory: &mut Memory, what: &str, how: &str, said: &str, own: bool) {
    crate::unported!("bot.memory.rates:_keep_a_rate")
}

/// Python: bot/memory/rates.py:162 `_the_newest_recent_rate(memory, within=2)` (never called on the trace corpus)
pub fn _the_newest_recent_rate(memory: &Memory, within: &Value) -> Value {
    crate::unported!("bot.memory.rates:_the_newest_recent_rate")
}

/// Python: bot/memory/rates.py:173 `_the_rate_put_right(text, memory)`
pub fn _the_rate_put_right(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.rates:_the_rate_put_right")
}

/// Python: bot/memory/rates.py:211 `keep(text, memory)`
pub fn keep(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.rates:keep")
}

/// Python: bot/memory/rates.py:300 `is_a_rate_ask(text, memory)`
pub fn is_a_rate_ask(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.rates:is_a_rate_ask")
}

/// Python: bot/memory/rates.py:305 `answer(text, memory)`
pub fn answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.rates:answer")
}

