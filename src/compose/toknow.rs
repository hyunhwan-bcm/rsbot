//! Port of `bot/compose/toknow.py`.
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

/// Python: bot/compose/toknow.py:84 `_the_thing_asked_about(clean)` (never called on the trace corpus)
pub fn _the_thing_asked_about(clean: &str) -> Value {
    crate::unported!("bot.compose.toknow:_the_thing_asked_about")
}

/// Python: bot/compose/toknow.py:102 `_a_card_about(thing)` (never called on the trace corpus)
pub fn _a_card_about(thing: &str) -> Value {
    crate::unported!("bot.compose.toknow:_a_card_about")
}

/// Python: bot/compose/toknow.py:118 `ask_about_what_to_look_out_for(clean, memory=None)`
pub fn ask_about_what_to_look_out_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.toknow:ask_about_what_to_look_out_for")
}

