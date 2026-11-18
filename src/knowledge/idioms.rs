//! Port of `bot/knowledge/idioms.py`.
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

/// Python: bot/knowledge/idioms.py:422 `_slug(phrase)` (never called on the trace corpus)
pub fn _slug(phrase: &str) -> String {
    crate::unported!("bot.knowledge.idioms:_slug")
}

/// Python: bot/knowledge/idioms.py:451 `_idiom(row)` (never called on the trace corpus)
pub fn _idiom(row: &'static Entry) -> Value {
    crate::unported!("bot.knowledge.idioms:_idiom")
}

