//! Port of `bot/knowledge/meat_in_a_card.py`.
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

/// Python: bot/knowledge/meat_in_a_card.py:50 `swapped_out(text, line='', avoided=None)`
pub fn swapped_out(text: &str, line: &str, avoided: &[String]) -> String {
    crate::unported!("bot.knowledge.meat_in_a_card:swapped_out")
}

