//! Port of `bot/memory/whereabout.py`.
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

/// Python: bot/memory/whereabout.py:81 `the_place(line)`
pub fn the_place(line: &str) -> String {
    crate::unported!("bot.memory.whereabout:the_place")
}

/// Python: bot/memory/whereabout.py:113 `the_question_with_the_place(text, memory)`
pub fn the_question_with_the_place(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.whereabout:the_question_with_the_place")
}

