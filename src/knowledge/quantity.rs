//! Port of `bot/knowledge/quantity.py`.
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

/// Python: bot/knowledge/quantity.py:27 `a_figure_with_a_unit(understood=None, text='')`
pub fn a_figure_with_a_unit(understood: &Value, text: &str) -> bool {
    crate::unported!("bot.knowledge.quantity:a_figure_with_a_unit")
}

