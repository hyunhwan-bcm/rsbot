//! Port of `bot/knowledge/cooking_fats.py`.
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

/// Python: bot/knowledge/cooking_fats.py:68 `_asks_between(line)` (never called on the trace corpus)
pub fn _asks_between(line: &str) -> bool {
    crate::unported!("bot.knowledge.cooking_fats:_asks_between")
}

/// Python: bot/knowledge/cooking_fats.py:84 `_fold(text)` (never called on the trace corpus)
pub fn _fold(text: &str) -> String {
    crate::unported!("bot.knowledge.cooking_fats:_fold")
}

/// Python: bot/knowledge/cooking_fats.py:111 `_last_reply(memory)` (never called on the trace corpus)
pub fn _last_reply(memory: &Memory) -> String {
    crate::unported!("bot.knowledge.cooking_fats:_last_reply")
}

/// Python: bot/knowledge/cooking_fats.py:119 `_fat_for_the_pan(memory, reading)` (never called on the trace corpus)
pub fn _fat_for_the_pan(memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.knowledge.cooking_fats:_fat_for_the_pan")
}

