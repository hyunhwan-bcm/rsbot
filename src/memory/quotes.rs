//! Port of `bot/memory/quotes.py`.
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

/// Python: bot/memory/quotes.py:27 `left_open(said)`
pub fn left_open(said: &str) -> bool {
    crate::unported!("bot.memory.quotes:left_open")
}

/// Python: bot/memory/quotes.py:39 `closed(said, rest, whole='')`
pub fn closed(said: &str, rest: &[String], whole: &str) -> String {
    crate::unported!("bot.memory.quotes:closed")
}

/// Python: bot/memory/quotes.py:66 `a_name(text, start, end)`
pub fn a_name(text: &str, start: i64, end: i64) -> bool {
    crate::unported!("bot.memory.quotes:a_name")
}

/// Python: bot/memory/quotes.py:77 `sentences(text, splitter)`
pub fn sentences(text: &str, splitter: &Value) -> Vec<String> {
    crate::unported!("bot.memory.quotes:sentences")
}

