//! Port of `bot/compose/a_count_of_packets.py`.
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

/// Python: bot/compose/a_count_of_packets.py:90 `is_a_dose(text)`
pub fn is_a_dose(text: &str) -> bool {
    crate::unported!("bot.compose.a_count_of_packets:is_a_dose")
}

