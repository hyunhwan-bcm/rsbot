//! Port of `bot/compose/childwords.py`.
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

/// Python: bot/compose/childwords.py:62 `_the_tail_after_the_child(text)` (never called on the trace corpus)
pub fn _the_tail_after_the_child(text: &str) -> Value {
    crate::unported!("bot.compose.childwords:_the_tail_after_the_child")
}

/// Python: bot/compose/childwords.py:72 `_the_tail_is_about_the_death(tail)` (never called on the trace corpus)
pub fn _the_tail_is_about_the_death(tail: &str) -> bool {
    crate::unported!("bot.compose.childwords:_the_tail_is_about_the_death")
}

/// Python: bot/compose/childwords.py:105 `a_death_in_the_thread(memory, look_back=3)` (never called on the trace corpus)
pub fn a_death_in_the_thread(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.childwords:a_death_in_the_thread")
}

/// Python: bot/compose/childwords.py:123 `asks_the_words_for_a_child(clean, memory)`
pub fn asks_the_words_for_a_child(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.childwords:asks_the_words_for_a_child")
}

/// Python: bot/compose/childwords.py:156 `_the_last_few_lines(memory, look_back=4)` (never called on the trace corpus)
pub fn _the_last_few_lines(memory: &Memory, look_back: &Value) -> String {
    crate::unported!("bot.compose.childwords:_the_last_few_lines")
}

