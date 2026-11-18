//! Port of `bot/knowledge/dish_verdicts.py`.
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

/// Python: bot/knowledge/dish_verdicts.py:370 `_fenced(said)`
pub fn _fenced(said: &str) -> bool {
    crate::unported!("bot.knowledge.dish_verdicts:_fenced")
}

/// Python: bot/knowledge/dish_verdicts.py:384 `_the_dish(memory, understood)`
pub fn _the_dish(memory: &Memory, understood: &Understood) -> String {
    crate::unported!("bot.knowledge.dish_verdicts:_the_dish")
}

