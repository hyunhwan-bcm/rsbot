//! Port of `bot/memory/counts.py`.
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

/// Python: bot/memory/counts.py:370 `_many(n)` (never called on the trace corpus)
pub fn _many(n: i64) -> Value {
    crate::unported!("bot.memory.counts:_many")
}

/// Python: bot/memory/counts.py:378 `_head(after)`
pub fn _head(after: &str) -> Vec<String> {
    crate::unported!("bot.memory.counts:_head")
}

/// Python: bot/memory/counts.py:417 `_the_relation(phrase)`
pub fn _the_relation(phrase: &[String]) -> String {
    crate::unported!("bot.memory.counts:_the_relation")
}

/// Python: bot/memory/counts.py:432 `_thing_of(phrase)` (never called on the trace corpus)
pub fn _thing_of(phrase: &str) -> Value {
    crate::unported!("bot.memory.counts:_thing_of")
}

/// Python: bot/memory/counts.py:437 `_thing(word)`
pub fn _thing(word: &str) -> String {
    crate::unported!("bot.memory.counts:_thing")
}

/// Python: bot/memory/counts.py:454 `_beds(rest)`
pub fn _beds(rest: &str) -> i64 {
    crate::unported!("bot.memory.counts:_beds")
}

/// Python: bot/memory/counts.py:510 `_counted(rest)`
pub fn _counted(rest: &str) -> Value {
    crate::unported!("bot.memory.counts:_counted")
}

/// Python: bot/memory/counts.py:556 `_said_number(said)`
pub fn _said_number(said: &str) -> i64 {
    crate::unported!("bot.memory.counts:_said_number")
}

/// Python: bot/memory/counts.py:567 `_the_group(said)`
pub fn _the_group(said: &str) -> i64 {
    crate::unported!("bot.memory.counts:_the_group")
}

/// Python: bot/memory/counts.py:578 `_the_party(said)`
pub fn _the_party(said: &str) -> i64 {
    crate::unported!("bot.memory.counts:_the_party")
}

/// Python: bot/memory/counts.py:601 `_the_chat_lines(memory)`
pub fn _the_chat_lines(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.counts:_the_chat_lines")
}

/// Python: bot/memory/counts.py:609 `_the_chat_feeds_people(memory)`
pub fn _the_chat_feeds_people(memory: &Memory) -> bool {
    crate::unported!("bot.memory.counts:_the_chat_feeds_people")
}

/// Python: bot/memory/counts.py:624 `_the_chat_sets_a_table(memory)` (never called on the trace corpus)
pub fn _the_chat_sets_a_table(memory: &Memory) -> bool {
    crate::unported!("bot.memory.counts:_the_chat_sets_a_table")
}

/// Python: bot/memory/counts.py:631 `_the_bare_people(said)`
pub fn _the_bare_people(said: &str) -> i64 {
    crate::unported!("bot.memory.counts:_the_bare_people")
}

/// Python: bot/memory/counts.py:640 `_the_number_put_right(said, memory)`
pub fn _the_number_put_right(said: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.memory.counts:_the_number_put_right")
}

/// Python: bot/memory/counts.py:704 `_forms_of(thing)` (never called on the trace corpus)
pub fn _forms_of(thing: &str) -> Value {
    crate::unported!("bot.memory.counts:_forms_of")
}

/// Python: bot/memory/counts.py:716 `_the_figure_counts_the_thing(said, match, after_word, thing)` (never called on the trace corpus)
pub fn _the_figure_counts_the_thing(said: &str, r#match: &Value, after_word: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.counts:_the_figure_counts_the_thing")
}

/// Python: bot/memory/counts.py:745 `_the_lines(memory)` (never called on the trace corpus)
pub fn _the_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.counts:_the_lines")
}

/// Python: bot/memory/counts.py:756 `_the_thing_in_hand(memory, held)` (never called on the trace corpus)
pub fn _the_thing_in_hand(memory: &Memory, held: &Value) -> Value {
    crate::unported!("bot.memory.counts:_the_thing_in_hand")
}

/// Python: bot/memory/counts.py:772 `_the_figure_that_stands(said)` (never called on the trace corpus)
pub fn _the_figure_that_stands(said: &str) -> Value {
    crate::unported!("bot.memory.counts:_the_figure_that_stands")
}

/// Python: bot/memory/counts.py:783 `_a_figure_for_a_thing_the_chat_counts(said, memory, already=())`
pub fn _a_figure_for_a_thing_the_chat_counts(said: &str, memory: &Memory, already: &Value) -> Value {
    crate::unported!("bot.memory.counts:_a_figure_for_a_thing_the_chat_counts")
}

/// Python: bot/memory/counts.py:828 `observe(text, memory)`
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.counts:observe")
}

/// Python: bot/memory/counts.py:903 `answer(text, memory)`
pub fn answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.counts:answer")
}

/// Python: bot/memory/counts.py:988 `_the_chat_books(memory)` (never called on the trace corpus)
pub fn _the_chat_books(memory: &Memory) -> bool {
    crate::unported!("bot.memory.counts:_the_chat_books")
}

/// Python: bot/memory/counts.py:998 `_the_party_said_back(number, low, memory=None, of_the_group=False)` (never called on the trace corpus)
pub fn _the_party_said_back(number: &Value, low: &str, memory: &Memory, of_the_group: &Value) -> Value {
    crate::unported!("bot.memory.counts:_the_party_said_back")
}

/// Python: bot/memory/counts.py:1031 `_the_group_said_back(number)` (never called on the trace corpus)
pub fn _the_group_said_back(number: &Value) -> String {
    crate::unported!("bot.memory.counts:_the_group_said_back")
}

