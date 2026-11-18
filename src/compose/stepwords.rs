//! Port of `bot/compose/stepwords.py`.
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

/// Python: bot/compose/stepwords.py:55 `_ing(word)` (never called on the trace corpus)
pub fn _ing(word: &str) -> Value {
    crate::unported!("bot.compose.stepwords:_ing")
}

/// Python: bot/compose/stepwords.py:74 `a_step(word)` (never called on the trace corpus)
pub fn a_step(word: &str) -> bool {
    crate::unported!("bot.compose.stepwords:a_step")
}

/// Python: bot/compose/stepwords.py:79 `said_after_a_to(phrase, ing=None)` (never called on the trace corpus)
pub fn said_after_a_to(phrase: &str, ing: &Value) -> String {
    crate::unported!("bot.compose.stepwords:said_after_a_to")
}

