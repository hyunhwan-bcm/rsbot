//! Port of `bot/memory/too_late.py`.
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

/// Python: bot/memory/too_late.py:109 `_the_sentence(said)` (never called on the trace corpus)
pub fn _the_sentence(said: &str) -> Value {
    crate::unported!("bot.memory.too_late:_the_sentence")
}

/// Python: bot/memory/too_late.py:131 `_two_clauses(sentence)` (never called on the trace corpus)
pub fn _two_clauses(sentence: &str) -> String {
    crate::unported!("bot.memory.too_late:_two_clauses")
}

/// Python: bot/memory/too_late.py:145 `_the_reply(sentence, memory, clean)` (never called on the trace corpus)
pub fn _the_reply(sentence: &str, memory: &Memory, clean: &str) -> String {
    crate::unported!("bot.memory.too_late:_the_reply")
}

/// Python: bot/memory/too_late.py:154 `answer_for(text, memory)`
pub fn answer_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.too_late:answer_for")
}

