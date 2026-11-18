//! Port of `bot/memory/whose_body.py`.
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

/// Python: bot/memory/whose_body.py:47 `somebody_was_named(memory)` (never called on the trace corpus)
pub fn somebody_was_named(memory: &Memory) -> bool {
    crate::unported!("bot.memory.whose_body:somebody_was_named")
}

/// Python: bot/memory/whose_body.py:66 `a_body_of_a_pronoun(clean)`
pub fn a_body_of_a_pronoun(clean: &str) -> bool {
    crate::unported!("bot.memory.whose_body:a_body_of_a_pronoun")
}

/// Python: bot/memory/whose_body.py:74 `somebody_elses_body(clean, memory)`
pub fn somebody_elses_body(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.whose_body:somebody_elses_body")
}

