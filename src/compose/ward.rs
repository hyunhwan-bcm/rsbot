//! Port of `bot/compose/ward.py`.
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

/// Python: bot/compose/ward.py:137 `_whom(said)` (never called on the trace corpus)
pub fn _whom(said: &str) -> String {
    crate::unported!("bot.compose.ward:_whom")
}

/// Python: bot/compose/ward.py:149 `_the_place_before(memory)`
pub fn _the_place_before(memory: &Memory) -> String {
    crate::unported!("bot.compose.ward:_the_place_before")
}

/// Python: bot/compose/ward.py:164 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.ward:answer_for")
}

