//! Port of `bot/memory/purse.py`.
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

/// Python: bot/memory/purse.py:74 `_a_number(word)` (never called on the trace corpus)
pub fn _a_number(word: &str) -> Value {
    crate::unported!("bot.memory.purse:_a_number")
}

/// Python: bot/memory/purse.py:82 `_money_in(text)`
pub fn _money_in(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.purse:_money_in")
}

/// Python: bot/memory/purse.py:92 `_the_money(value)` (never called on the trace corpus)
pub fn _the_money(value: &Value) -> String {
    crate::unported!("bot.memory.purse:_the_money")
}

/// Python: bot/memory/purse.py:99 `_the_words_after(answer, at)` (never called on the trace corpus)
pub fn _the_words_after(answer: &str, at: i64) -> Value {
    crate::unported!("bot.memory.purse:_the_words_after")
}

/// Python: bot/memory/purse.py:112 `_money_bands(answer)` (never called on the trace corpus)
pub fn _money_bands(answer: &str) -> Value {
    crate::unported!("bot.memory.purse:_money_bands")
}

/// Python: bot/memory/purse.py:127 `against_the_bands(text, turns)`
pub fn against_the_bands(text: &str, turns: &Value) {
    crate::unported!("bot.memory.purse:against_the_bands")
}

