//! Port of `bot/compose/their_day.py`.
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

/// Python: bot/compose/their_day.py:202 `pool_for(text, care=False)`
pub fn pool_for(text: &str, care: bool) {
    crate::unported!("bot.compose.their_day:pool_for")
}

/// Python: bot/compose/their_day.py:222 `reads(clean, repaired=None)`
pub fn reads(clean: &str, repaired: &Value) {
    crate::unported!("bot.compose.their_day:reads")
}

/// Python: bot/compose/their_day.py:380 `own_fact(text, memory=None)`
pub fn own_fact(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.their_day:own_fact")
}

/// Python: bot/compose/their_day.py:423 `own_fact_for(text, memory=None)`
pub fn own_fact_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.their_day:own_fact_for")
}

