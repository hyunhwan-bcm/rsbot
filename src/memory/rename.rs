//! Port of `bot/memory/rename.py`.
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

/// Python: bot/memory/rename.py:64 `moves(low)`
pub fn moves(low: &str) -> bool {
    crate::unported!("bot.memory.rename:moves")
}

/// Python: bot/memory/rename.py:69 `the_name_it_gave(low)` (never called on the trace corpus)
pub fn the_name_it_gave(low: &str) -> Value {
    crate::unported!("bot.memory.rename:the_name_it_gave")
}

/// Python: bot/memory/rename.py:91 `the_name_a_line_moved_on(low, pets, animal_named)` (never called on the trace corpus)
pub fn the_name_a_line_moved_on(low: &str, pets: &Value, animal_named: &Value) -> Value {
    crate::unported!("bot.memory.rename:the_name_a_line_moved_on")
}

