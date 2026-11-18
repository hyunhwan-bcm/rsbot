//! Port of `bot/compose/onmybehalf.py`.
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

/// Python: bot/compose/onmybehalf.py:174 `_facing_them(phrase)` (never called on the trace corpus)
pub fn _facing_them(phrase: &str) -> Value {
    crate::unported!("bot.compose.onmybehalf:_facing_them")
}

/// Python: bot/compose/onmybehalf.py:179 `_what_was_asked_for(clean)` (never called on the trace corpus)
pub fn _what_was_asked_for(clean: &str) -> String {
    crate::unported!("bot.compose.onmybehalf:_what_was_asked_for")
}

/// Python: bot/compose/onmybehalf.py:195 `_is_a_flat_read(reply)` (never called on the trace corpus)
pub fn _is_a_flat_read(reply: &str) -> bool {
    crate::unported!("bot.compose.onmybehalf:_is_a_flat_read")
}

/// Python: bot/compose/onmybehalf.py:200 `_is_a_hedge(reply)` (never called on the trace corpus)
pub fn _is_a_hedge(reply: &str) -> bool {
    crate::unported!("bot.compose.onmybehalf:_is_a_hedge")
}

/// Python: bot/compose/onmybehalf.py:222 `_about_an_errand(clean)`
pub fn _about_an_errand(clean: &str) {
    crate::unported!("bot.compose.onmybehalf:_about_an_errand")
}

/// Python: bot/compose/onmybehalf.py:249 `answer_for(text, memory, reply)`
pub fn answer_for(text: &str, memory: &Memory, reply: &str) {
    crate::unported!("bot.compose.onmybehalf:answer_for")
}

