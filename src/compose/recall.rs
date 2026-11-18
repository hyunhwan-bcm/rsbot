//! Port of `bot/compose/recall.py`.
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

/// Python: bot/compose/recall.py:68 `_is_a_death(said)` (never called on the trace corpus)
pub fn _is_a_death(said: &str) -> bool {
    crate::unported!("bot.compose.recall:_is_a_death")
}

/// Python: bot/compose/recall.py:78 `_said_on_the_last_turn(said, memory)` (never called on the trace corpus)
pub fn _said_on_the_last_turn(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.recall:_said_on_the_last_turn")
}

/// Python: bot/compose/recall.py:93 `_names_a_thing(said)` (never called on the trace corpus)
pub fn _names_a_thing(said: &str) -> bool {
    crate::unported!("bot.compose.recall:_names_a_thing")
}

/// Python: bot/compose/recall.py:99 `with_a_tail(said, memory)` (never called on the trace corpus)
pub fn with_a_tail(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.recall:with_a_tail")
}

