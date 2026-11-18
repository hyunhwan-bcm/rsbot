//! Port of `bot/compose/judgement.py`.
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

/// Python: bot/compose/judgement.py:22 `asks_for_a_judgement(text, memory=None)` (never called on the trace corpus)
pub fn asks_for_a_judgement(text: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.compose.judgement:asks_for_a_judgement")
}

