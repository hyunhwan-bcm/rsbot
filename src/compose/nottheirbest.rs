//! Port of `bot/compose/nottheirbest.py`.
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

/// Python: bot/compose/nottheirbest.py:83 `reads_as_a_low_day(text)`
pub fn reads_as_a_low_day(text: &str) -> bool {
    crate::unported!("bot.compose.nottheirbest:reads_as_a_low_day")
}

/// Python: bot/compose/nottheirbest.py:95 `_may_replace(reply)` (never called on the trace corpus)
pub fn _may_replace(reply: &str) -> bool {
    crate::unported!("bot.compose.nottheirbest:_may_replace")
}

/// Python: bot/compose/nottheirbest.py:111 `answer_for(text, reply, memory=None)`
pub fn answer_for(text: &str, reply: &str, memory: &Memory) {
    crate::unported!("bot.compose.nottheirbest:answer_for")
}

