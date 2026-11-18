//! Port of `bot/compose/moving.py`.
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

/// Python: bot/compose/moving.py:72 `reads_as_a_move(text)`
pub fn reads_as_a_move(text: &str) -> bool {
    crate::unported!("bot.compose.moving:reads_as_a_move")
}

/// Python: bot/compose/moving.py:93 `reads_as_a_move_cheered(text)`
pub fn reads_as_a_move_cheered(text: &str) -> bool {
    crate::unported!("bot.compose.moving:reads_as_a_move_cheered")
}

/// Python: bot/compose/moving.py:112 `reply_for(text, memory=None)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.moving:reply_for")
}

