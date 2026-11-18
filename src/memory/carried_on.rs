//! Port of `bot/memory/carried_on.py`.
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

/// Python: bot/memory/carried_on.py:85 `_a_whole_step(said)` (never called on the trace corpus)
pub fn _a_whole_step(said: &str) -> String {
    crate::unported!("bot.memory.carried_on:_a_whole_step")
}

/// Python: bot/memory/carried_on.py:141 `_stem(word)` (never called on the trace corpus)
pub fn _stem(word: &str) -> Value {
    crate::unported!("bot.memory.carried_on:_stem")
}

/// Python: bot/memory/carried_on.py:152 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.carried_on:_words")
}

/// Python: bot/memory/carried_on.py:180 `_steps(answer)`
pub fn _steps(answer: &str) -> Vec<String> {
    crate::unported!("bot.memory.carried_on:_steps")
}

/// Python: bot/memory/carried_on.py:198 `_doing(word)` (never called on the trace corpus)
pub fn _doing(word: &str) -> Value {
    crate::unported!("bot.memory.carried_on:_doing")
}

/// Python: bot/memory/carried_on.py:223 `_the_action(line)` (never called on the trace corpus)
pub fn _the_action(line: &str) -> Value {
    crate::unported!("bot.memory.carried_on:_the_action")
}

/// Python: bot/memory/carried_on.py:243 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.carried_on:reply_for")
}

