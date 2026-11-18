//! Port of `bot/memory/portions.py`.
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

/// Python: bot/memory/portions.py:82 `_a_time_not_a_portion(said, found)` (never called on the trace corpus)
pub fn _a_time_not_a_portion(said: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.portions:_a_time_not_a_portion")
}

/// Python: bot/memory/portions.py:117 `_the_last_answer(memory)` (never called on the trace corpus)
pub fn _the_last_answer(memory: &Memory) -> String {
    crate::unported!("bot.memory.portions:_the_last_answer")
}

/// Python: bot/memory/portions.py:127 `_the_clause_with_the_count(said, start, end)` (never called on the trace corpus)
pub fn _the_clause_with_the_count(said: &str, start: i64, end: i64) -> String {
    crate::unported!("bot.memory.portions:_the_clause_with_the_count")
}

/// Python: bot/memory/portions.py:138 `_the_number(word)` (never called on the trace corpus)
pub fn _the_number(word: &str) -> String {
    crate::unported!("bot.memory.portions:_the_number")
}

/// Python: bot/memory/portions.py:151 `_as_a_count(word)` (never called on the trace corpus)
pub fn _as_a_count(word: &str) -> Value {
    crate::unported!("bot.memory.portions:_as_a_count")
}

/// Python: bot/memory/portions.py:159 `_in_grams(figure, unit)` (never called on the trace corpus)
pub fn _in_grams(figure: &Value, unit: &str) -> Value {
    crate::unported!("bot.memory.portions:_in_grams")
}

/// Python: bot/memory/portions.py:176 `_counts_people(said, found)` (never called on the trace corpus)
pub fn _counts_people(said: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.portions:_counts_people")
}

/// Python: bot/memory/portions.py:189 `_the_per_head_figure(said)` (never called on the trace corpus)
pub fn _the_per_head_figure(said: &str) -> Value {
    crate::unported!("bot.memory.portions:_the_per_head_figure")
}

/// Python: bot/memory/portions.py:213 `_a_total(grams)` (never called on the trace corpus)
pub fn _a_total(grams: &Value) -> String {
    crate::unported!("bot.memory.portions:_a_total")
}

/// Python: bot/memory/portions.py:222 `the_portion_answer(memory)` (never called on the trace corpus)
pub fn the_portion_answer(memory: &Memory) -> Value {
    crate::unported!("bot.memory.portions:the_portion_answer")
}

/// Python: bot/memory/portions.py:234 `an_ask(text, memory)`
pub fn an_ask(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.portions:an_ask")
}

/// Python: bot/memory/portions.py:259 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.portions:reply_for")
}

/// Python: bot/memory/portions.py:312 `_is_an_amount_not_a_count(text, found)` (never called on the trace corpus)
pub fn _is_an_amount_not_a_count(text: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.portions:_is_an_amount_not_a_count")
}

/// Python: bot/memory/portions.py:322 `_says_two_counts(text, found)` (never called on the trace corpus)
pub fn _says_two_counts(text: &str, found: &Value) -> bool {
    crate::unported!("bot.memory.portions:_says_two_counts")
}

/// Python: bot/memory/portions.py:333 `_the_new_count(text)`
pub fn _the_new_count(text: &str) {
    crate::unported!("bot.memory.portions:_the_new_count")
}

/// Python: bot/memory/portions.py:349 `_is_a_digit(word)` (never called on the trace corpus)
pub fn _is_a_digit(word: &str) -> bool {
    crate::unported!("bot.memory.portions:_is_a_digit")
}

/// Python: bot/memory/portions.py:353 `_the_count_they_gave(text)` (never called on the trace corpus)
pub fn _the_count_they_gave(text: &str) -> Value {
    crate::unported!("bot.memory.portions:_the_count_they_gave")
}

/// Python: bot/memory/portions.py:379 `_the_said_total(grams, count, count_word, old_word=None)` (never called on the trace corpus)
pub fn _the_said_total(grams: &Value, count: i64, count_word: &str, old_word: &str) -> String {
    crate::unported!("bot.memory.portions:_the_said_total")
}

/// Python: bot/memory/portions.py:392 `_a_total_for_a_bare_count(text, memory)`
pub fn _a_total_for_a_bare_count(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.portions:_a_total_for_a_bare_count")
}

/// Python: bot/memory/portions.py:417 `a_bare_count_to_fill(text, memory)`
pub fn a_bare_count_to_fill(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.portions:a_bare_count_to_fill")
}

/// Python: bot/memory/portions.py:433 `a_count_to_fill(text, memory)`
pub fn a_count_to_fill(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.portions:a_count_to_fill")
}

/// Python: bot/memory/portions.py:445 `the_total_for(text, memory)`
pub fn the_total_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.portions:the_total_for")
}

