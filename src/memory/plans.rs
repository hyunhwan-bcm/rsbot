//! Port of `bot/memory/plans.py`.
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

/// Python: bot/memory/plans.py:61 `_a_first_day(text)`
pub fn _a_first_day(text: &str) -> bool {
    crate::unported!("bot.memory.plans:_a_first_day")
}

/// Python: bot/memory/plans.py:100 `as_question(text)` (never called on the trace corpus)
pub fn as_question(text: &str) -> Value {
    crate::unported!("bot.memory.plans:as_question")
}

/// Python: bot/memory/plans.py:106 `_implied(text)`
pub fn _implied(text: &str) -> Value {
    crate::unported!("bot.memory.plans:_implied")
}

/// Python: bot/memory/plans.py:135 `_the_reply(entry, question, memory)`
pub fn _the_reply(entry: &'static Entry, question: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.plans:_the_reply")
}

/// Python: bot/memory/plans.py:155 `answer(text, memory=None)`
pub fn answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.plans:answer")
}

/// Python: bot/memory/plans.py:191 `_about_planning(entry)`
pub fn _about_planning(entry: &'static Entry) -> bool {
    crate::unported!("bot.memory.plans:_about_planning")
}

/// Python: bot/memory/plans.py:232 `_the_clauses(detail)` (never called on the trace corpus)
pub fn _the_clauses(detail: &Value) -> Vec<String> {
    crate::unported!("bot.memory.plans:_the_clauses")
}

/// Python: bot/memory/plans.py:237 `_the_step(detail, wanted, first_only=False)` (never called on the trace corpus)
pub fn _the_step(detail: &Value, wanted: &Value, first_only: &Value) -> Value {
    crate::unported!("bot.memory.plans:_the_step")
}

/// Python: bot/memory/plans.py:254 `_the_first_step(sentence)` (never called on the trace corpus)
pub fn _the_first_step(sentence: &str) -> Option<String> {
    crate::unported!("bot.memory.plans:_the_first_step")
}

/// Python: bot/memory/plans.py:279 `_a_countdown_step(head, detail, wanted, about_the_start)` (never called on the trace corpus)
pub fn _a_countdown_step(head: &str, detail: &Value, wanted: &Value, about_the_start: &Value) -> Option<String> {
    crate::unported!("bot.memory.plans:_a_countdown_step")
}

/// Python: bot/memory/plans.py:299 `_another_thing(wanted, said_words)` (never called on the trace corpus)
pub fn _another_thing(wanted: &Value, said_words: &[String]) -> bool {
    crate::unported!("bot.memory.plans:_another_thing")
}

/// Python: bot/memory/plans.py:322 `_part_words(text)` (never called on the trace corpus)
pub fn _part_words(text: &str) -> Value {
    crate::unported!("bot.memory.plans:_part_words")
}

/// Python: bot/memory/plans.py:327 `_was_a_plan(line)`
pub fn _was_a_plan(line: &str) -> bool {
    crate::unported!("bot.memory.plans:_was_a_plan")
}

/// Python: bot/memory/plans.py:341 `follow_up(text, memory)`
pub fn follow_up(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.plans:follow_up")
}

