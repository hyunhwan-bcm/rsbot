//! Port of `bot/compose/receiving_end.py`.
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

/// Python: bot/compose/receiving_end.py:126 `_the_role_said_back(found)` (never called on the trace corpus)
pub fn _the_role_said_back(found: &Value) -> Value {
    crate::unported!("bot.compose.receiving_end:_the_role_said_back")
}

/// Python: bot/compose/receiving_end.py:155 `_a_body_line_above(memory)` (never called on the trace corpus)
pub fn _a_body_line_above(memory: &Memory) -> bool {
    crate::unported!("bot.compose.receiving_end:_a_body_line_above")
}

/// Python: bot/compose/receiving_end.py:164 `_a_person_who_keeps(clean, repaired, memory)`
pub fn _a_person_who_keeps(clean: &str, repaired: &str, memory: &Memory) {
    crate::unported!("bot.compose.receiving_end:_a_person_who_keeps")
}

/// Python: bot/compose/receiving_end.py:205 `reply_for(clean, repaired, memory)`
pub fn reply_for(clean: &str, repaired: &str, memory: &Memory) {
    crate::unported!("bot.compose.receiving_end:reply_for")
}

