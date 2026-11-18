//! Port of `bot/memory/opening.py`.
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

/// Python: bot/memory/opening.py:139 `_the_day_is_the_point(clean, match)` (never called on the trace corpus)
pub fn _the_day_is_the_point(clean: &str, r#match: &Value) -> bool {
    crate::unported!("bot.memory.opening:_the_day_is_the_point")
}

/// Python: bot/memory/opening.py:150 `_names_my(lower, word)`
pub fn _names_my(lower: &str, word: &str) -> bool {
    crate::unported!("bot.memory.opening:_names_my")
}

/// Python: bot/memory/opening.py:156 `_who(clean)`
pub fn _who(clean: &str) -> Vec<String> {
    crate::unported!("bot.memory.opening:_who")
}

/// Python: bot/memory/opening.py:180 `_said_before(memory)`
pub fn _said_before(memory: &Memory) -> bool {
    crate::unported!("bot.memory.opening:_said_before")
}

/// Python: bot/memory/opening.py:189 `the_opening(thing, clean, memory)`
pub fn the_opening(thing: &str, clean: &str, memory: &Memory) {
    crate::unported!("bot.memory.opening:the_opening")
}

