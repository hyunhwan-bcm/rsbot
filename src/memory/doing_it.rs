//! Port of `bot/memory/doing_it.py`.
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

/// Python: bot/memory/doing_it.py:51 `_is_doing(word)`
pub fn _is_doing(word: &str) -> bool {
    crate::unported!("bot.memory.doing_it:_is_doing")
}

/// Python: bot/memory/doing_it.py:72 `doing_word(text)`
pub fn doing_word(text: &str) -> Option<String> {
    crate::unported!("bot.memory.doing_it:doing_word")
}

/// Python: bot/memory/doing_it.py:80 `asks_about_doing(text)`
pub fn asks_about_doing(text: &str) -> bool {
    crate::unported!("bot.memory.doing_it:asks_about_doing")
}

/// Python: bot/memory/doing_it.py:88 `_the_things(memory)` (never called on the trace corpus)
pub fn _the_things(memory: &Memory) -> Value {
    crate::unported!("bot.memory.doing_it:_the_things")
}

/// Python: bot/memory/doing_it.py:113 `written_out(text, memory)`
pub fn written_out(text: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.memory.doing_it:written_out")
}

/// Python: bot/memory/doing_it.py:156 `the_ask_behind_a_moan(text)`
pub fn the_ask_behind_a_moan(text: &str) -> String {
    crate::unported!("bot.memory.doing_it:the_ask_behind_a_moan")
}

