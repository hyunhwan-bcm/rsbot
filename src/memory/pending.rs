//! Port of `bot/memory/pending.py`.
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

/// Python: bot/memory/pending.py:76 `_clauses(text)`
pub fn _clauses(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.pending:_clauses")
}

/// Python: bot/memory/pending.py:85 `_content(text)`
pub fn _content(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.pending:_content")
}

/// Python: bot/memory/pending.py:89 `_what_is_missing(text)`
pub fn _what_is_missing(text: &str) -> Option<Vec<String>> {
    crate::unported!("bot.memory.pending:_what_is_missing")
}

/// Python: bot/memory/pending.py:105 `_the_occasion(text, missing)`
pub fn _the_occasion(text: &str, missing: &str) -> Option<String> {
    crate::unported!("bot.memory.pending:_the_occasion")
}

/// Python: bot/memory/pending.py:133 `as_question(text)`
pub fn as_question(text: &str) {
    crate::unported!("bot.memory.pending:as_question")
}

/// Python: bot/memory/pending.py:157 `answer(text, memory=None)`
pub fn answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.pending:answer")
}

