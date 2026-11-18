//! Port of `bot/memory/condition.py`.
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

/// Python: bot/memory/condition.py:66 `_places(words)` (never called on the trace corpus)
pub fn _places(words: &[String]) -> Value {
    crate::unported!("bot.memory.condition:_places")
}

/// Python: bot/memory/condition.py:70 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.condition:_words")
}

/// Python: bot/memory/condition.py:75 `_pairs(words)`
pub fn _pairs(words: &[String]) -> Vec<String> {
    crate::unported!("bot.memory.condition:_pairs")
}

/// Python: bot/memory/condition.py:107 `about_someone_else(text)` (never called on the trace corpus)
pub fn about_someone_else(text: &str) -> bool {
    crate::unported!("bot.memory.condition:about_someone_else")
}

/// Python: bot/memory/condition.py:119 `_sentences(said)`
pub fn _sentences(said: &str) -> Vec<String> {
    crate::unported!("bot.memory.condition:_sentences")
}

/// Python: bot/memory/condition.py:123 `_one_move(consequence)`
pub fn _one_move(consequence: &str) -> String {
    crate::unported!("bot.memory.condition:_one_move")
}

/// Python: bot/memory/condition.py:150 `conditions(said)`
pub fn conditions(said: &str) -> Value {
    crate::unported!("bot.memory.condition:conditions")
}

/// Python: bot/memory/condition.py:219 `_number(word)` (never called on the trace corpus)
pub fn _number(word: &str) -> Value {
    crate::unported!("bot.memory.condition:_number")
}

/// Python: bot/memory/condition.py:226 `_measures(text)` (never called on the trace corpus)
pub fn _measures(text: &str) -> Value {
    crate::unported!("bot.memory.condition:_measures")
}

/// Python: bot/memory/condition.py:242 `_tests(case)` (never called on the trace corpus)
pub fn _tests(case: &Value) -> Value {
    crate::unported!("bot.memory.condition:_tests")
}

/// Python: bot/memory/condition.py:259 `_meets(case, text)` (never called on the trace corpus)
pub fn _meets(case: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.condition:_meets")
}

/// Python: bot/memory/condition.py:280 `chosen(text, said)`
pub fn chosen(text: &str, said: &str) {
    crate::unported!("bot.memory.condition:chosen")
}

/// Python: bot/memory/condition.py:295 `refused(text, said)`
pub fn refused(text: &str, said: &str) -> bool {
    crate::unported!("bot.memory.condition:refused")
}

/// Python: bot/memory/condition.py:306 `_carried(case_words, words)`
pub fn _carried(case_words: &[String], words: &[String]) -> bool {
    crate::unported!("bot.memory.condition:_carried")
}

/// Python: bot/memory/condition.py:324 `_about_a_person(case_words)` (never called on the trace corpus)
pub fn _about_a_person(case_words: &[String]) -> bool {
    crate::unported!("bot.memory.condition:_about_a_person")
}

/// Python: bot/memory/condition.py:329 `_candidates(text, said)`
pub fn _candidates(text: &str, said: &str) -> Vec<String> {
    crate::unported!("bot.memory.condition:_candidates")
}

