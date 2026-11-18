//! Port of `bot/memory/change.py`.
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

/// Python: bot/memory/change.py:88 `_names(line, subject)`
pub fn _names(line: &str, subject: &str) -> bool {
    crate::unported!("bot.memory.change:_names")
}

/// Python: bot/memory/change.py:97 `_left_over(line, subject)` (never called on the trace corpus)
pub fn _left_over(line: &str, subject: &str) -> Value {
    crate::unported!("bot.memory.change:_left_over")
}

/// Python: bot/memory/change.py:111 `reported(text, subject)`
pub fn reported(text: &str, subject: &str) {
    crate::unported!("bot.memory.change:reported")
}

/// Python: bot/memory/change.py:135 `reply(kind, subject, was=None)` (never called on the trace corpus)
pub fn reply(kind: &str, subject: &str, was: &str) -> Value {
    crate::unported!("bot.memory.change:reply")
}

