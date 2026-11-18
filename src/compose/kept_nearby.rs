//! Port of `bot/compose/kept_nearby.py`.
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

/// Python: bot/compose/kept_nearby.py:108 `_turns(memory)`
pub fn _turns(memory: &Memory) -> Value {
    crate::unported!("bot.compose.kept_nearby:_turns")
}

/// Python: bot/compose/kept_nearby.py:112 `_clean(text)`
pub fn _clean(text: &str) -> String {
    crate::unported!("bot.compose.kept_nearby:_clean")
}

/// Python: bot/compose/kept_nearby.py:116 `_without_plurals(line)`
pub fn _without_plurals(line: &str) -> String {
    crate::unported!("bot.compose.kept_nearby:_without_plurals")
}

/// Python: bot/compose/kept_nearby.py:133 `is_the_plain_hedge(said)` (never called on the trace corpus)
pub fn is_the_plain_hedge(said: &str) -> bool {
    crate::unported!("bot.compose.kept_nearby:is_the_plain_hedge")
}

/// Python: bot/compose/kept_nearby.py:138 `_inheritable(pool)` (never called on the trace corpus)
pub fn _inheritable(pool: &Value) -> bool {
    crate::unported!("bot.compose.kept_nearby:_inheritable")
}

/// Python: bot/compose/kept_nearby.py:150 `_the_kind(line, borrowable=False)`
pub fn _the_kind(line: &str, borrowable: bool) {
    crate::unported!("bot.compose.kept_nearby:_the_kind")
}

/// Python: bot/compose/kept_nearby.py:163 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &Memory) -> Option<Vec<String>> {
    crate::unported!("bot.compose.kept_nearby:answer_for")
}

