//! Port of `bot/memory/matters.py`.
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

/// Python: bot/memory/matters.py:77 `_the_person(memory)` (never called on the trace corpus)
pub fn _the_person(memory: &Memory) -> Value {
    crate::unported!("bot.memory.matters:_the_person")
}

/// Python: bot/memory/matters.py:100 `about_them(text, memory)`
pub fn about_them(text: &str, memory: &Value) -> String {
    crate::unported!("bot.memory.matters:about_them")
}

/// Python: bot/memory/matters.py:180 `_a_worry_in_the_chat(memory)` (never called on the trace corpus)
pub fn _a_worry_in_the_chat(memory: &Memory) -> bool {
    crate::unported!("bot.memory.matters:_a_worry_in_the_chat")
}

/// Python: bot/memory/matters.py:196 `what_they_say(text, memory)`
pub fn what_they_say(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.matters:what_they_say")
}

/// Python: bot/memory/matters.py:241 `_the_group_ages(memory)`
pub fn _the_group_ages(memory: &Memory) -> Value {
    crate::unported!("bot.memory.matters:_the_group_ages")
}

/// Python: bot/memory/matters.py:257 `_say_the_ages(ages)` (never called on the trace corpus)
pub fn _say_the_ages(ages: &Value) -> String {
    crate::unported!("bot.memory.matters:_say_the_ages")
}

/// Python: bot/memory/matters.py:264 `a_corrected_age(text, memory)`
pub fn a_corrected_age(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.matters:a_corrected_age")
}

