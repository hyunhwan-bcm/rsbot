//! Port of `bot/memory/guest.py`.
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

/// Python: bot/memory/guest.py:90 `_fold(word)`
pub fn _fold(word: &str) -> String {
    crate::unported!("bot.memory.guest:_fold")
}

/// Python: bot/memory/guest.py:104 `_known()`
pub fn _known() -> Vec<String> {
    crate::unported!("bot.memory.guest:_known")
}

/// Python: bot/memory/guest.py:140 `_clean(item)`
pub fn _clean(item: &str) -> String {
    crate::unported!("bot.memory.guest:_clean")
}

/// Python: bot/memory/guest.py:158 `_one_half(piece)` (never called on the trace corpus)
pub fn _one_half(piece: &str) -> bool {
    crate::unported!("bot.memory.guest:_one_half")
}

/// Python: bot/memory/guest.py:166 `_is_food(item)`
pub fn _is_food(item: &str) -> bool {
    crate::unported!("bot.memory.guest:_is_food")
}

/// Python: bot/memory/guest.py:176 `plain(item, a_pair=False)`
pub fn plain(item: &str, a_pair: bool) {
    crate::unported!("bot.memory.guest:plain")
}

/// Python: bot/memory/guest.py:198 `found(text)`
pub fn found(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.guest:found")
}

/// Python: bot/memory/guest.py:232 `turned_down_alone(text)`
pub fn turned_down_alone(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.guest:turned_down_alone")
}

/// Python: bot/memory/guest.py:244 `_contains(text, food)` (never called on the trace corpus)
pub fn _contains(text: &str, food: &Value) -> bool {
    crate::unported!("bot.memory.guest:_contains")
}

/// Python: bot/memory/guest.py:249 `_without_article(item)` (never called on the trace corpus)
pub fn _without_article(item: &'static Entry) -> String {
    crate::unported!("bot.memory.guest:_without_article")
}

/// Python: bot/memory/guest.py:253 `_split_list(rest)` (never called on the trace corpus)
pub fn _split_list(rest: &str) -> Value {
    crate::unported!("bot.memory.guest:_split_list")
}

/// Python: bot/memory/guest.py:267 `_kept_phrase(items)` (never called on the trace corpus)
pub fn _kept_phrase(items: &Value) -> String {
    crate::unported!("bot.memory.guest:_kept_phrase")
}

/// Python: bot/memory/guest.py:275 `revise(reply, foods)` (never called on the trace corpus)
pub fn revise(reply: &str, foods: &Value) -> Option<String> {
    crate::unported!("bot.memory.guest:revise")
}

/// Python: bot/memory/guest.py:304 `left_out(foods)` (never called on the trace corpus)
pub fn left_out(foods: &Value) -> Option<String> {
    crate::unported!("bot.memory.guest:left_out")
}

