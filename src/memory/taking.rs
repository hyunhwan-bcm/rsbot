//! Port of `bot/memory/taking.py`.
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

/// Python: bot/memory/taking.py:111 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.taking:_flat")
}

/// Python: bot/memory/taking.py:115 `_asks_what_to_take(text)`
pub fn _asks_what_to_take(text: &str) -> bool {
    crate::unported!("bot.memory.taking:_asks_what_to_take")
}

/// Python: bot/memory/taking.py:123 `_complaint_words()` (never called on the trace corpus)
pub fn _complaint_words() -> Value {
    crate::unported!("bot.memory.taking:_complaint_words")
}

/// Python: bot/memory/taking.py:157 `_someone_else()` (never called on the trace corpus)
pub fn _someone_else() -> Value {
    crate::unported!("bot.memory.taking:_someone_else")
}

/// Python: bot/memory/taking.py:168 `_a_complaint(subject)` (never called on the trace corpus)
pub fn _a_complaint(subject: &str) -> bool {
    crate::unported!("bot.memory.taking:_a_complaint")
}

/// Python: bot/memory/taking.py:182 `_subjects(memory)` (never called on the trace corpus)
pub fn _subjects(memory: &Memory) -> Value {
    crate::unported!("bot.memory.taking:_subjects")
}

/// Python: bot/memory/taking.py:200 `_the_things_to_take_for(subject)` (never called on the trace corpus)
pub fn _the_things_to_take_for(subject: &str) -> Value {
    crate::unported!("bot.memory.taking:_the_things_to_take_for")
}

/// Python: bot/memory/taking.py:219 `_a_trip_above(memory, text)` (never called on the trace corpus)
pub fn _a_trip_above(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.memory.taking:_a_trip_above")
}

/// Python: bot/memory/taking.py:229 `_the_answer(text, memory)` (never called on the trace corpus)
pub fn _the_answer(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.taking:_the_answer")
}

/// Python: bot/memory/taking.py:243 `_the_said_in_the_chat(memory)` (never called on the trace corpus)
pub fn _the_said_in_the_chat(memory: &Memory) -> String {
    crate::unported!("bot.memory.taking:_the_said_in_the_chat")
}

/// Python: bot/memory/taking.py:251 `_the_last_reply(memory)` (never called on the trace corpus)
pub fn _the_last_reply(memory: &Memory) -> String {
    crate::unported!("bot.memory.taking:_the_last_reply")
}

/// Python: bot/memory/taking.py:258 `_the_situation_this_chat_is_in(reply, memory)` (never called on the trace corpus)
pub fn _the_situation_this_chat_is_in(reply: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.taking:_the_situation_this_chat_is_in")
}

/// Python: bot/memory/taking.py:274 `_the_answer_today(text, memory)` (never called on the trace corpus)
pub fn _the_answer_today(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.taking:_the_answer_today")
}

/// Python: bot/memory/taking.py:284 `_the_same_answer(reply, text, memory, today)` (never called on the trace corpus)
pub fn _the_same_answer(reply: &str, text: &str, memory: &Memory, today: &Value) -> bool {
    crate::unported!("bot.memory.taking:_the_same_answer")
}

/// Python: bot/memory/taking.py:301 `_names(reply, word)` (never called on the trace corpus)
pub fn _names(reply: &str, word: &str) -> bool {
    crate::unported!("bot.memory.taking:_names")
}

/// Python: bot/memory/taking.py:312 `_the_answer_above(reply, memory)` (never called on the trace corpus)
pub fn _the_answer_above(reply: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.taking:_the_answer_above")
}

/// Python: bot/memory/taking.py:329 `_the_subjects_to_answer(memory, text)` (never called on the trace corpus)
pub fn _the_subjects_to_answer(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.taking:_the_subjects_to_answer")
}

/// Python: bot/memory/taking.py:348 `the_ask_and_the_subject(text, memory)` (never called on the trace corpus)
pub fn the_ask_and_the_subject(text: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.memory.taking:the_ask_and_the_subject")
}

/// Python: bot/memory/taking.py:364 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.taking:reply_for")
}

