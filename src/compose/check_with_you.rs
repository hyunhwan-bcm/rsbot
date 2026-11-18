//! Port of `bot/compose/check_with_you.py`.
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

/// Python: bot/compose/check_with_you.py:91 `reads_as_a_check(text)`
pub fn reads_as_a_check(text: &str) -> bool {
    crate::unported!("bot.compose.check_with_you:reads_as_a_check")
}

/// Python: bot/compose/check_with_you.py:105 `answer_for(text, memory=None, choose=None)`
pub fn answer_for(text: &str, memory: &Memory, choose: &Value) {
    crate::unported!("bot.compose.check_with_you:answer_for")
}

