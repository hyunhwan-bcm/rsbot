//! Port of `bot/compose/missing_half.py`.
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

/// Python: bot/compose/missing_half.py:138 `_choose(pool, text)` (never called on the trace corpus)
pub fn _choose(pool: &Value, text: &str) -> Value {
    crate::unported!("bot.compose.missing_half:_choose")
}

/// Python: bot/compose/missing_half.py:143 `_a_sentence(line)` (never called on the trace corpus)
pub fn _a_sentence(line: &str) -> Value {
    crate::unported!("bot.compose.missing_half:_a_sentence")
}

/// Python: bot/compose/missing_half.py:154 `_fits(low, who)` (never called on the trace corpus)
pub fn _fits(low: &str, who: &str) -> bool {
    crate::unported!("bot.compose.missing_half:_fits")
}

/// Python: bot/compose/missing_half.py:167 `_given_beside(low, who)` (never called on the trace corpus)
pub fn _given_beside(low: &str, who: &str) -> bool {
    crate::unported!("bot.compose.missing_half:_given_beside")
}

/// Python: bot/compose/missing_half.py:176 `_the_ages(memory)`
pub fn _the_ages(memory: &Memory) -> Value {
    crate::unported!("bot.compose.missing_half:_the_ages")
}

/// Python: bot/compose/missing_half.py:199 `_the_known_list(others)` (never called on the trace corpus)
pub fn _the_known_list(others: &Value) -> Value {
    crate::unported!("bot.compose.missing_half:_the_known_list")
}

/// Python: bot/compose/missing_half.py:212 `the_missing_half(text, memory)`
pub fn the_missing_half(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.missing_half:the_missing_half")
}

