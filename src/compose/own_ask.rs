//! Port of `bot/compose/own_ask.py`.
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

/// Python: bot/compose/own_ask.py:112 `_words(clean)`
pub fn _words(clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.own_ask:_words")
}

/// Python: bot/compose/own_ask.py:116 `_named_thing(clean)`
pub fn _named_thing(clean: &str) -> String {
    crate::unported!("bot.compose.own_ask:_named_thing")
}

/// Python: bot/compose/own_ask.py:213 `_family(clean)` (never called on the trace corpus)
pub fn _family(clean: &str) -> Value {
    crate::unported!("bot.compose.own_ask:_family")
}

/// Python: bot/compose/own_ask.py:234 `answer(clean, memory=None)`
pub fn answer(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.own_ask:answer")
}

