//! Port of `bot/compose/colour_on_the_body.py`.
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

/// Python: bot/compose/colour_on_the_body.py:165 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.colour_on_the_body:_words")
}

/// Python: bot/compose/colour_on_the_body.py:169 `_last_reply(memory)` (never called on the trace corpus)
pub fn _last_reply(memory: &Memory) -> String {
    crate::unported!("bot.compose.colour_on_the_body:_last_reply")
}

/// Python: bot/compose/colour_on_the_body.py:178 `_pick(pool, line, memory=None)` (never called on the trace corpus)
pub fn _pick(pool: &Value, line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.colour_on_the_body:_pick")
}

/// Python: bot/compose/colour_on_the_body.py:198 `_someone_elses(words)` (never called on the trace corpus)
pub fn _someone_elses(words: &[String]) -> bool {
    crate::unported!("bot.compose.colour_on_the_body:_someone_elses")
}

/// Python: bot/compose/colour_on_the_body.py:217 `_thread_names_a_pet(memory)` (never called on the trace corpus)
pub fn _thread_names_a_pet(memory: &Memory) -> bool {
    crate::unported!("bot.compose.colour_on_the_body:_thread_names_a_pet")
}

/// Python: bot/compose/colour_on_the_body.py:235 `reply_for(text, memory=None)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.colour_on_the_body:reply_for")
}

