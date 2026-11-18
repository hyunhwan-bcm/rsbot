//! Port of `bot/compose/urgent.py`.
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

/// Python: bot/compose/urgent.py:244 `_not_a_body(rest, before='')` (never called on the trace corpus)
pub fn _not_a_body(rest: &str, before: &str) -> bool {
    crate::unported!("bot.compose.urgent:_not_a_body")
}

/// Python: bot/compose/urgent.py:397 `_fed_and_unresponsive(before, rest)` (never called on the trace corpus)
pub fn _fed_and_unresponsive(before: &str, rest: &str) -> bool {
    crate::unported!("bot.compose.urgent:_fed_and_unresponsive")
}

/// Python: bot/compose/urgent.py:451 `reads_as_a_story(clean)`
pub fn reads_as_a_story(clean: &str) -> bool {
    crate::unported!("bot.compose.urgent:reads_as_a_story")
}

/// Python: bot/compose/urgent.py:522 `_named_a_gait(clean, start)` (never called on the trace corpus)
pub fn _named_a_gait(clean: &str, start: i64) -> bool {
    crate::unported!("bot.compose.urgent:_named_a_gait")
}

/// Python: bot/compose/urgent.py:535 `_walking(rest)` (never called on the trace corpus)
pub fn _walking(rest: &str) -> bool {
    crate::unported!("bot.compose.urgent:_walking")
}

/// Python: bot/compose/urgent.py:546 `_signs_a_body(clean, phrase, rest, start=0)` (never called on the trace corpus)
pub fn _signs_a_body(clean: &str, phrase: &str, rest: &str, start: i64) -> bool {
    crate::unported!("bot.compose.urgent:_signs_a_body")
}

/// Python: bot/compose/urgent.py:567 `_first(clean, phrases)`
pub fn _first(clean: &str, phrases: &[String]) {
    crate::unported!("bot.compose.urgent:_first")
}

/// Python: bot/compose/urgent.py:609 `_pet_is_the_subject(clean)`
pub fn _pet_is_the_subject(clean: &str) -> bool {
    crate::unported!("bot.compose.urgent:_pet_is_the_subject")
}

/// Python: bot/compose/urgent.py:619 `reply_for(text, memory=None)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.urgent:reply_for")
}

