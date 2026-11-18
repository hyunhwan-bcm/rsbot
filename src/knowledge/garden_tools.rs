//! Port of `bot/knowledge/garden_tools.py`.
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

/// Python: bot/knowledge/garden_tools.py:99 `_the_guard_question(words, line)` (never called on the trace corpus)
pub fn _the_guard_question(words: &[String], line: &str) -> bool {
    crate::unported!("bot.knowledge.garden_tools:_the_guard_question")
}

/// Python: bot/knowledge/garden_tools.py:110 `_garden_machine(memory, understood)` (never called on the trace corpus)
pub fn _garden_machine(memory: &Memory, understood: &Understood) -> String {
    crate::unported!("bot.knowledge.garden_tools:_garden_machine")
}

