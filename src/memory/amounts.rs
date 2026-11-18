//! Port of `bot/memory/amounts.py`.
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

/// Python: bot/memory/amounts.py:67 `_a_value(word)` (never called on the trace corpus)
pub fn _a_value(word: &str) -> Value {
    crate::unported!("bot.memory.amounts:_a_value")
}

/// Python: bot/memory/amounts.py:78 `_a_number(value)` (never called on the trace corpus)
pub fn _a_number(value: &Value) -> Value {
    crate::unported!("bot.memory.amounts:_a_number")
}

/// Python: bot/memory/amounts.py:85 `_counts(text)`
pub fn _counts(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.amounts:_counts")
}

/// Python: bot/memory/amounts.py:101 `_the_period_at(text, at)` (never called on the trace corpus)
pub fn _the_period_at(text: &str, at: i64) -> Option<String> {
    crate::unported!("bot.memory.amounts:_the_period_at")
}

/// Python: bot/memory/amounts.py:110 `a_count_they_report(text)`
pub fn a_count_they_report(text: &str) {
    crate::unported!("bot.memory.amounts:a_count_they_report")
}

/// Python: bot/memory/amounts.py:126 `_the_figure_named(answer, thing, mine)` (never called on the trace corpus)
pub fn _the_figure_named(answer: &str, thing: &str, mine: &Memory) -> Value {
    crate::unported!("bot.memory.amounts:_the_figure_named")
}

/// Python: bot/memory/amounts.py:170 `_the_periods_disagree(mine, theirs)` (never called on the trace corpus)
pub fn _the_periods_disagree(mine: &Memory, theirs: &Value) -> bool {
    crate::unported!("bot.memory.amounts:_the_periods_disagree")
}

/// Python: bot/memory/amounts.py:177 `against_the_last_answer(said, answer)` (never called on the trace corpus)
pub fn against_the_last_answer(said: &str, answer: &str) -> Value {
    crate::unported!("bot.memory.amounts:against_the_last_answer")
}

/// Python: bot/memory/amounts.py:194 `_the_figure_said(against)` (never called on the trace corpus)
pub fn _the_figure_said(against: &Value) -> Value {
    crate::unported!("bot.memory.amounts:_the_figure_said")
}

/// Python: bot/memory/amounts.py:202 `_a_mid_sentence(figure)` (never called on the trace corpus)
pub fn _a_mid_sentence(figure: &Value) -> Value {
    crate::unported!("bot.memory.amounts:_a_mid_sentence")
}

/// Python: bot/memory/amounts.py:219 `_the_aim_said(against)` (never called on the trace corpus)
pub fn _the_aim_said(against: &Value) -> Value {
    crate::unported!("bot.memory.amounts:_the_aim_said")
}

/// Python: bot/memory/amounts.py:244 `the_comparison(against)` (never called on the trace corpus)
pub fn the_comparison(against: &Value) -> String {
    crate::unported!("bot.memory.amounts:the_comparison")
}

