//! Port of `bot/compose/is_it_worth_it.py`.
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

/// Python: bot/compose/is_it_worth_it.py:285 `_a_destination(said)`
pub fn _a_destination(said: &str) -> String {
    crate::unported!("bot.compose.is_it_worth_it:_a_destination")
}

/// Python: bot/compose/is_it_worth_it.py:293 `_a_trip_in_the_thread(memory)`
pub fn _a_trip_in_the_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.is_it_worth_it:_a_trip_in_the_thread")
}

/// Python: bot/compose/is_it_worth_it.py:299 `_the_thread(memory, back=2)`
pub fn _the_thread(memory: &Memory, back: i64) -> String {
    crate::unported!("bot.compose.is_it_worth_it:_the_thread")
}

/// Python: bot/compose/is_it_worth_it.py:310 `_is_it_worth_it(text, memory, reply)`
pub fn _is_it_worth_it(text: &str, memory: &Memory, reply: &str) {
    crate::unported!("bot.compose.is_it_worth_it:_is_it_worth_it")
}

/// Python: bot/compose/is_it_worth_it.py:394 `_with_what_they_said(reply, memory, said)`
pub fn _with_what_they_said(reply: &str, memory: &Memory, said: &str) {
    crate::unported!("bot.compose.is_it_worth_it:_with_what_they_said")
}

/// Python: bot/compose/is_it_worth_it.py:407 `is_it_worth_it(text, memory, reply)`
pub fn is_it_worth_it(text: &str, memory: &Memory, reply: &str) {
    crate::unported!("bot.compose.is_it_worth_it:is_it_worth_it")
}

