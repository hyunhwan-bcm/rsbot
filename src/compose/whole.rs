//! Port of `bot/compose/whole.py`.
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

/// Python: bot/compose/whole.py:33 `a_whole_cut(text, most)`
pub fn a_whole_cut(text: &str, most: i64) -> String {
    crate::unported!("bot.compose.whole:a_whole_cut")
}

