//! Port of `bot/compose/facts.py`.
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

/// Python: bot/compose/facts.py:52 `asks_for_a_when(text)`
pub fn asks_for_a_when(text: &str) -> bool {
    crate::unported!("bot.compose.facts:asks_for_a_when")
}

/// Python: bot/compose/facts.py:59 `holds_a_date(text)` (never called on the trace corpus)
pub fn holds_a_date(text: &str) -> bool {
    crate::unported!("bot.compose.facts:holds_a_date")
}

/// Python: bot/compose/facts.py:80 `_date_words(text)` (never called on the trace corpus)
pub fn _date_words(text: &str) -> Value {
    crate::unported!("bot.compose.facts:_date_words")
}

/// Python: bot/compose/facts.py:84 `the_same_date(one, two)` (never called on the trace corpus)
pub fn the_same_date(one: &str, two: &Value) -> bool {
    crate::unported!("bot.compose.facts:the_same_date")
}

/// Python: bot/compose/facts.py:90 `the_fact_asked_for(text, reply)` (never called on the trace corpus)
pub fn the_fact_asked_for(text: &str, reply: &str) -> bool {
    crate::unported!("bot.compose.facts:the_fact_asked_for")
}

