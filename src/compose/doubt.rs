//! Port of `bot/compose/doubt.py`.
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

/// Python: bot/compose/doubt.py:85 `_last_bot(memory)` (never called on the trace corpus)
pub fn _last_bot(memory: &Memory) -> Value {
    crate::unported!("bot.compose.doubt:_last_bot")
}

/// Python: bot/compose/doubt.py:99 `reads_as_a_doubt(clean)`
pub fn reads_as_a_doubt(clean: &str) -> bool {
    crate::unported!("bot.compose.doubt:reads_as_a_doubt")
}

/// Python: bot/compose/doubt.py:106 `_the_line_behind(memory)` (never called on the trace corpus)
pub fn _the_line_behind(memory: &Memory) -> String {
    crate::unported!("bot.compose.doubt:_the_line_behind")
}

/// Python: bot/compose/doubt.py:119 `reply_for(clean, memory)`
pub fn reply_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.doubt:reply_for")
}

