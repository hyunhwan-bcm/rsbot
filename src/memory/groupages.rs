//! Port of `bot/memory/groupages.py`.
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

/// Python: bot/memory/groupages.py:92 `_the_word(word)` (never called on the trace corpus)
pub fn _the_word(word: &str) -> Value {
    crate::unported!("bot.memory.groupages:_the_word")
}

/// Python: bot/memory/groupages.py:99 `_ages_in(value)` (never called on the trace corpus)
pub fn _ages_in(value: &Value) -> Value {
    crate::unported!("bot.memory.groupages:_ages_in")
}

/// Python: bot/memory/groupages.py:104 `the_list_given(text)`
pub fn the_list_given(text: &str) {
    crate::unported!("bot.memory.groupages:the_list_given")
}

/// Python: bot/memory/groupages.py:122 `watch(text, memory)`
pub fn watch(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.groupages:watch")
}

/// Python: bot/memory/groupages.py:133 `_the_kept(memory)`
pub fn _the_kept(memory: &Memory) -> Value {
    crate::unported!("bot.memory.groupages:_the_kept")
}

/// Python: bot/memory/groupages.py:147 `_a_list(ages)` (never called on the trace corpus)
pub fn _a_list(ages: &Value) -> Value {
    crate::unported!("bot.memory.groupages:_a_list")
}

/// Python: bot/memory/groupages.py:170 `_the_singulars(word)` (never called on the trace corpus)
pub fn _the_singulars(word: &str) -> Value {
    crate::unported!("bot.memory.groupages:_the_singulars")
}

/// Python: bot/memory/groupages.py:179 `_the_end_one_is_the_group(text, end_one, word)` (never called on the trace corpus)
pub fn _the_end_one_is_the_group(text: &str, end_one: &Value, word: &str) -> bool {
    crate::unported!("bot.memory.groupages:_the_end_one_is_the_group")
}

/// Python: bot/memory/groupages.py:197 `_the_group_asked(text, word)` (never called on the trace corpus)
pub fn _the_group_asked(text: &str, word: &str) -> Value {
    crate::unported!("bot.memory.groupages:_the_group_asked")
}

/// Python: bot/memory/groupages.py:211 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.groupages:reply_for")
}

/// Python: bot/memory/groupages.py:240 `asks_for_the_list(text, memory)` (never called on the trace corpus)
pub fn asks_for_the_list(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.groupages:asks_for_the_list")
}

