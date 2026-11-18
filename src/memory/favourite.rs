//! Port of `bot/memory/favourite.py`.
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

/// Python: bot/memory/favourite.py:165 `_plain(value)` (never called on the trace corpus)
pub fn _plain(value: &Value) -> String {
    crate::unported!("bot.memory.favourite:_plain")
}

/// Python: bot/memory/favourite.py:172 `_a_value(value)` (never called on the trace corpus)
pub fn _a_value(value: &Value) -> Value {
    crate::unported!("bot.memory.favourite:_a_value")
}

/// Python: bot/memory/favourite.py:200 `_a_thing(thing)`
pub fn _a_thing(thing: &str) -> String {
    crate::unported!("bot.memory.favourite:_a_thing")
}

/// Python: bot/memory/favourite.py:248 `_clauses(text)`
pub fn _clauses(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.favourite:_clauses")
}

/// Python: bot/memory/favourite.py:263 `read(line)`
pub fn read(line: &str) {
    crate::unported!("bot.memory.favourite:read")
}

/// Python: bot/memory/favourite.py:294 `asks_something_too(line)`
pub fn asks_something_too(line: &str) -> bool {
    crate::unported!("bot.memory.favourite:asks_something_too")
}

/// Python: bot/memory/favourite.py:320 `asked(line)`
pub fn asked(line: &str) {
    crate::unported!("bot.memory.favourite:asked")
}

/// Python: bot/memory/favourite.py:338 `_singular(thing)` (never called on the trace corpus)
pub fn _singular(thing: &str) -> Value {
    crate::unported!("bot.memory.favourite:_singular")
}

/// Python: bot/memory/favourite.py:346 `asks_all(line)`
pub fn asks_all(line: &str) -> bool {
    crate::unported!("bot.memory.favourite:asks_all")
}

/// Python: bot/memory/favourite.py:357 `prefers_now(line)` (never called on the trace corpus)
pub fn prefers_now(line: &str) -> Value {
    crate::unported!("bot.memory.favourite:prefers_now")
}

/// Python: bot/memory/favourite.py:368 `an_answer(line, thing)` (never called on the trace corpus)
pub fn an_answer(line: &str, thing: &str) -> Value {
    crate::unported!("bot.memory.favourite:an_answer")
}

/// Python: bot/memory/favourite.py:384 `corrected(line, thing)` (never called on the trace corpus)
pub fn corrected(line: &str, thing: &str) -> Value {
    crate::unported!("bot.memory.favourite:corrected")
}

/// Python: bot/memory/favourite.py:395 `kept(memory, thing)`
pub fn kept(memory: &Memory, thing: &str) -> String {
    crate::unported!("bot.memory.favourite:kept")
}

/// Python: bot/memory/favourite.py:408 `the_favourites(memory)` (never called on the trace corpus)
pub fn the_favourites(memory: &Memory) -> Value {
    crate::unported!("bot.memory.favourite:the_favourites")
}

/// Python: bot/memory/favourite.py:418 `store(memory, found)` (never called on the trace corpus)
pub fn store(memory: &mut Memory, found: &Value) {
    crate::unported!("bot.memory.favourite:store")
}

/// Python: bot/memory/favourite.py:452 `their_words(value)` (never called on the trace corpus)
pub fn their_words(value: &Value) -> Value {
    crate::unported!("bot.memory.favourite:their_words")
}

/// Python: bot/memory/favourite.py:467 `watch(text, memory)`
pub fn watch(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.favourite:watch")
}

