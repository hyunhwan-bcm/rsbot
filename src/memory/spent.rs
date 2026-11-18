//! Port of `bot/memory/spent.py`.
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

/// Python: bot/memory/spent.py:99 `_amount(number)` (never called on the trace corpus)
pub fn _amount(number: &Value) -> String {
    crate::unported!("bot.memory.spent:_amount")
}

/// Python: bot/memory/spent.py:104 `_the_thing(rest)` (never called on the trace corpus)
pub fn _the_thing(rest: &str) -> Value {
    crate::unported!("bot.memory.spent:_the_thing")
}

/// Python: bot/memory/spent.py:166 `_a_bill(text)` (never called on the trace corpus)
pub fn _a_bill(text: &str) -> Value {
    crate::unported!("bot.memory.spent:_a_bill")
}

/// Python: bot/memory/spent.py:182 `_an_age_not_a_bill(match, thing, pattern)` (never called on the trace corpus)
pub fn _an_age_not_a_bill(r#match: &Value, thing: &str, pattern: &Value) -> bool {
    crate::unported!("bot.memory.spent:_an_age_not_a_bill")
}

/// Python: bot/memory/spent.py:191 `_one(text)` (never called on the trace corpus)
pub fn _one(text: &str) -> Value {
    crate::unported!("bot.memory.spent:_one")
}

/// Python: bot/memory/spent.py:217 `_plain(text)` (never called on the trace corpus)
pub fn _plain(text: &str) -> Value {
    crate::unported!("bot.memory.spent:_plain")
}

/// Python: bot/memory/spent.py:222 `readings(line)` (never called on the trace corpus)
pub fn readings(line: &str) -> Value {
    crate::unported!("bot.memory.spent:readings")
}

/// Python: bot/memory/spent.py:235 `reading(line)` (never called on the trace corpus)
pub fn reading(line: &str) -> Value {
    crate::unported!("bot.memory.spent:reading")
}

/// Python: bot/memory/spent.py:244 `the_lines(memory, text=None)` (never called on the trace corpus)
pub fn the_lines(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.spent:the_lines")
}

/// Python: bot/memory/spent.py:257 `the_spends(memory, text=None)` (never called on the trace corpus)
pub fn the_spends(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.spent:the_spends")
}

/// Python: bot/memory/spent.py:290 `_named(thing, wanted)` (never called on the trace corpus)
pub fn _named(thing: &str, wanted: &Value) -> bool {
    crate::unported!("bot.memory.spent:_named")
}

/// Python: bot/memory/spent.py:304 `_about(thing, wanted)` (never called on the trace corpus)
pub fn _about(thing: &str, wanted: &Value) -> bool {
    crate::unported!("bot.memory.spent:_about")
}

/// Python: bot/memory/spent.py:322 `_family(thing)` (never called on the trace corpus)
pub fn _family(thing: &str) -> Value {
    crate::unported!("bot.memory.spent:_family")
}

/// Python: bot/memory/spent.py:333 `the_total(spends)` (never called on the trace corpus)
pub fn the_total(spends: &Value) -> String {
    crate::unported!("bot.memory.spent:the_total")
}

/// Python: bot/memory/spent.py:348 `_listed(spends)` (never called on the trace corpus)
pub fn _listed(spends: &Value) -> Value {
    crate::unported!("bot.memory.spent:_listed")
}

/// Python: bot/memory/spent.py:359 `answer(text, memory)`
pub fn answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.spent:answer")
}

/// Python: bot/memory/spent.py:488 `_a_count_of_a_thing(text)` (never called on the trace corpus)
pub fn _a_count_of_a_thing(text: &str) -> bool {
    crate::unported!("bot.memory.spent:_a_count_of_a_thing")
}

/// Python: bot/memory/spent.py:538 `_figure(value)` (never called on the trace corpus)
pub fn _figure(value: &Value) -> String {
    crate::unported!("bot.memory.spent:_figure")
}

/// Python: bot/memory/spent.py:546 `_with_scale(number, word)` (never called on the trace corpus)
pub fn _with_scale(number: &Value, word: &str) -> Value {
    crate::unported!("bot.memory.spent:_with_scale")
}

/// Python: bot/memory/spent.py:562 `_held_in(line)` (never called on the trace corpus)
pub fn _held_in(line: &str) -> Value {
    crate::unported!("bot.memory.spent:_held_in")
}

/// Python: bot/memory/spent.py:584 `the_held(memory, text=None)` (never called on the trace corpus)
pub fn the_held(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.spent:the_held")
}

/// Python: bot/memory/spent.py:594 `_a_thing_to_buy(said)` (never called on the trace corpus)
pub fn _a_thing_to_buy(said: &str) -> Value {
    crate::unported!("bot.memory.spent:_a_thing_to_buy")
}

/// Python: bot/memory/spent.py:616 `_the_price(thing)` (never called on the trace corpus)
pub fn _the_price(thing: &str) -> Value {
    crate::unported!("bot.memory.spent:_the_price")
}

/// Python: bot/memory/spent.py:636 `_bare_thing(thing)` (never called on the trace corpus)
pub fn _bare_thing(thing: &str) -> String {
    crate::unported!("bot.memory.spent:_bare_thing")
}

/// Python: bot/memory/spent.py:649 `_only_the_ask(said)` (never called on the trace corpus)
pub fn _only_the_ask(said: &str) -> bool {
    crate::unported!("bot.memory.spent:_only_the_ask")
}

/// Python: bot/memory/spent.py:655 `_nothing_before(memory, text)` (never called on the trace corpus)
pub fn _nothing_before(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.memory.spent:_nothing_before")
}

/// Python: bot/memory/spent.py:674 `_the_what_if(match, memory)` (never called on the trace corpus)
pub fn _the_what_if(r#match: &Value, memory: &Memory) -> Option<String> {
    crate::unported!("bot.memory.spent:_the_what_if")
}

/// Python: bot/memory/spent.py:700 `what_is_left(text, memory)`
pub fn what_is_left(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.spent:what_is_left")
}

