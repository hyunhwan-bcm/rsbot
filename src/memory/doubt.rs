//! Port of `bot/memory/doubt.py`.
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

/// Python: bot/memory/doubt.py:97 `_the_answer_just_given(memory)`
pub fn _the_answer_just_given(memory: &Memory) -> String {
    crate::unported!("bot.memory.doubt:_the_answer_just_given")
}

/// Python: bot/memory/doubt.py:108 `the_doubt(text, memory)`
pub fn the_doubt(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.doubt:the_doubt")
}

/// Python: bot/memory/doubt.py:138 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.doubt:reply_for")
}

