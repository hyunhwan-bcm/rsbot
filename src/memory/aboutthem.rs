//! Port of `bot/memory/aboutthem.py`.
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

/// Python: bot/memory/aboutthem.py:56 `_the_child(text, memory)`
pub fn _the_child(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.aboutthem:_the_child")
}

/// Python: bot/memory/aboutthem.py:91 `_people(memory)`
pub fn _people(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.aboutthem:_people")
}

/// Python: bot/memory/aboutthem.py:104 `with_the_age(text, memory)`
pub fn with_the_age(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.aboutthem:with_the_age")
}

/// Python: bot/memory/aboutthem.py:149 `an_age_ahead(text, memory)`
pub fn an_age_ahead(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.aboutthem:an_age_ahead")
}

/// Python: bot/memory/aboutthem.py:175 `_the_years_ahead(when)` (never called on the trace corpus)
pub fn _the_years_ahead(when: &str) -> Value {
    crate::unported!("bot.memory.aboutthem:_the_years_ahead")
}

/// Python: bot/memory/aboutthem.py:191 `_the_day_named(when, years)` (never called on the trace corpus)
pub fn _the_day_named(when: &str, years: &Value) -> String {
    crate::unported!("bot.memory.aboutthem:_the_day_named")
}

/// Python: bot/memory/aboutthem.py:202 `_the_one_asked_about(said, memory)` (never called on the trace corpus)
pub fn _the_one_asked_about(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.aboutthem:_the_one_asked_about")
}

/// Python: bot/memory/aboutthem.py:239 `_an_age_said(age)` (never called on the trace corpus)
pub fn _an_age_said(age: &Value) -> Value {
    crate::unported!("bot.memory.aboutthem:_an_age_said")
}

/// Python: bot/memory/aboutthem.py:246 `_the_subject(words)` (never called on the trace corpus)
pub fn _the_subject(words: &[String]) -> String {
    crate::unported!("bot.memory.aboutthem:_the_subject")
}

/// Python: bot/memory/aboutthem.py:252 `_the_future(words)` (never called on the trace corpus)
pub fn _the_future(words: &[String]) -> String {
    crate::unported!("bot.memory.aboutthem:_the_future")
}

