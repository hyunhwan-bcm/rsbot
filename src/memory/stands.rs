//! Port of `bot/memory/stands.py`.
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

/// Python: bot/memory/stands.py:108 `_number(word)`
pub fn _number(word: &str) -> f64 {
    crate::unported!("bot.memory.stands:_number")
}

/// Python: bot/memory/stands.py:117 `_figures(text)`
pub fn _figures(text: &str) -> Value {
    crate::unported!("bot.memory.stands:_figures")
}

/// Python: bot/memory/stands.py:129 `_names_the_figure(text, value, unit)` (never called on the trace corpus)
pub fn _names_the_figure(text: &str, value: &Value, unit: &str) -> bool {
    crate::unported!("bot.memory.stands:_names_the_figure")
}

/// Python: bot/memory/stands.py:145 `_the_line_asks_about_a_figure_and_elsewhere_nothing(text)`
pub fn _the_line_asks_about_a_figure_and_elsewhere_nothing(text: &str) {
    crate::unported!("bot.memory.stands:_the_line_asks_about_a_figure_and_elsewhere_nothing")
}

/// Python: bot/memory/stands.py:172 `_the_chat_gave_it(memory, value, unit)` (never called on the trace corpus)
pub fn _the_chat_gave_it(memory: &Memory, value: &Value, unit: &str) -> bool {
    crate::unported!("bot.memory.stands:_the_chat_gave_it")
}

/// Python: bot/memory/stands.py:183 `_the_reason_behind(memory, value, unit)` (never called on the trace corpus)
pub fn _the_reason_behind(memory: &Memory, value: &Value, unit: &str) -> Value {
    crate::unported!("bot.memory.stands:_the_reason_behind")
}

/// Python: bot/memory/stands.py:208 `the_figure_asked_about(text, memory)`
pub fn the_figure_asked_about(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.stands:the_figure_asked_about")
}

/// Python: bot/memory/stands.py:300 `_in_minutes(number, unit)` (never called on the trace corpus)
pub fn _in_minutes(number: &Value, unit: &str) -> Value {
    crate::unported!("bot.memory.stands:_in_minutes")
}

/// Python: bot/memory/stands.py:317 `_the_best(candidates, family)` (never called on the trace corpus)
pub fn _the_best(candidates: &[Candidate], family: &Value) -> Value {
    crate::unported!("bot.memory.stands:_the_best")
}

/// Python: bot/memory/stands.py:330 `_figures_in_answer(said, family)` (never called on the trace corpus)
pub fn _figures_in_answer(said: &str, family: &Value) -> Value {
    crate::unported!("bot.memory.stands:_figures_in_answer")
}

/// Python: bot/memory/stands.py:353 `_the_same_figure(one, other)` (never called on the trace corpus)
pub fn _the_same_figure(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.stands:_the_same_figure")
}

/// Python: bot/memory/stands.py:367 `_subjects(said)` (never called on the trace corpus)
pub fn _subjects(said: &str) -> Value {
    crate::unported!("bot.memory.stands:_subjects")
}

/// Python: bot/memory/stands.py:372 `_about_the_same_thing(one, other)` (never called on the trace corpus)
pub fn _about_the_same_thing(one: &str, other: &str) -> bool {
    crate::unported!("bot.memory.stands:_about_the_same_thing")
}

/// Python: bot/memory/stands.py:386 `_a_figure_for(memory, family)` (never called on the trace corpus)
pub fn _a_figure_for(memory: &Memory, family: &Value) -> Value {
    crate::unported!("bot.memory.stands:_a_figure_for")
}

/// Python: bot/memory/stands.py:438 `_a_change_was_made(memory)`
pub fn _a_change_was_made(memory: &Memory) -> bool {
    crate::unported!("bot.memory.stands:_a_change_was_made")
}

/// Python: bot/memory/stands.py:463 `_a_change_of_size(memory)`
pub fn _a_change_of_size(memory: &Memory) -> bool {
    crate::unported!("bot.memory.stands:_a_change_of_size")
}

/// Python: bot/memory/stands.py:472 `the_change_asked_about(text, memory)`
pub fn the_change_asked_about(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.stands:the_change_asked_about")
}

/// Python: bot/memory/stands.py:515 `change_reply(text, memory)`
pub fn change_reply(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.stands:change_reply")
}

/// Python: bot/memory/stands.py:527 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.stands:reply_for")
}

