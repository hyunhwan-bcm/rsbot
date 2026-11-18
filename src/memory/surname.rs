//! Port of `bot/memory/surname.py`.
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

/// Python: bot/memory/surname.py:94 `_vary(options, memory)` (never called on the trace corpus)
pub fn _vary(options: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.surname:_vary")
}

/// Python: bot/memory/surname.py:102 `_the_name_they_said(word)` (never called on the trace corpus)
pub fn _the_name_they_said(word: &str) -> String {
    crate::unported!("bot.memory.surname:_the_name_they_said")
}

/// Python: bot/memory/surname.py:121 `the_surname(memory)`
pub fn the_surname(memory: &Memory) -> String {
    crate::unported!("bot.memory.surname:the_surname")
}

/// Python: bot/memory/surname.py:130 `_given_one(text)`
pub fn _given_one(text: &str) -> String {
    crate::unported!("bot.memory.surname:_given_one")
}

/// Python: bot/memory/surname.py:151 `observe(text, memory)`
pub fn observe(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.surname:observe")
}

/// Python: bot/memory/surname.py:165 `_asks_for_it(text)`
pub fn _asks_for_it(text: &str) -> bool {
    crate::unported!("bot.memory.surname:_asks_for_it")
}

/// Python: bot/memory/surname.py:188 `_the_first_name(memory)` (never called on the trace corpus)
pub fn _the_first_name(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.surname:_the_first_name")
}

/// Python: bot/memory/surname.py:193 `a_note(text, memory)`
pub fn a_note(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.surname:a_note")
}

/// Python: bot/memory/surname.py:226 `_the_lines_before(memory, back=3)` (never called on the trace corpus)
pub fn _the_lines_before(memory: &Memory, back: &Value) -> String {
    crate::unported!("bot.memory.surname:_the_lines_before")
}

/// Python: bot/memory/surname.py:234 `a_follow_up(text, memory)`
pub fn a_follow_up(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.surname:a_follow_up")
}

/// Python: bot/memory/surname.py:254 `there_is_a_chat(memory)`
pub fn there_is_a_chat(memory: &Memory) -> bool {
    crate::unported!("bot.memory.surname:there_is_a_chat")
}

/// Python: bot/memory/surname.py:260 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.surname:reply_for")
}

/// Python: bot/memory/surname.py:295 `_plain(text)`
pub fn _plain(text: &str) -> String {
    crate::unported!("bot.memory.surname:_plain")
}

/// Python: bot/memory/surname.py:299 `_the_full_name(memory)` (never called on the trace corpus)
pub fn _the_full_name(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.surname:_the_full_name")
}

/// Python: bot/memory/surname.py:314 `_the_first_name_reply(memory)` (never called on the trace corpus)
pub fn _the_first_name_reply(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.surname:_the_first_name_reply")
}

