//! Port of `bot/compose/breakup.py`.
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

/// Python: bot/compose/breakup.py:38 `_thread_said(memory, line)` (never called on the trace corpus)
pub fn _thread_said(memory: &Memory, line: &str) -> String {
    crate::unported!("bot.compose.breakup:_thread_said")
}

/// Python: bot/compose/breakup.py:47 `answer_for(text, reply, memory)`
pub fn answer_for(text: &str, reply: &str, memory: &Memory) {
    crate::unported!("bot.compose.breakup:answer_for")
}

