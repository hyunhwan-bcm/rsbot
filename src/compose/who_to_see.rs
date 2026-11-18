//! Port of `bot/compose/who_to_see.py`.
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

/// Python: bot/compose/who_to_see.py:91 `_clean(text)`
pub fn _clean(text: &str) -> String {
    crate::unported!("bot.compose.who_to_see:_clean")
}

/// Python: bot/compose/who_to_see.py:95 `_the_answer_in_hand(memory)` (never called on the trace corpus)
pub fn _the_answer_in_hand(memory: &Memory) -> Value {
    crate::unported!("bot.compose.who_to_see:_the_answer_in_hand")
}

/// Python: bot/compose/who_to_see.py:137 `_the_condition(said)` (never called on the trace corpus)
pub fn _the_condition(said: &str) -> Value {
    crate::unported!("bot.compose.who_to_see:_the_condition")
}

/// Python: bot/compose/who_to_see.py:160 `_the_question(said, clause='')` (never called on the trace corpus)
pub fn _the_question(said: &str, clause: &str) -> Value {
    crate::unported!("bot.compose.who_to_see:_the_question")
}

/// Python: bot/compose/who_to_see.py:171 `_the_condition_reply(said, memory)` (never called on the trace corpus)
pub fn _the_condition_reply(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.who_to_see:_the_condition_reply")
}

/// Python: bot/compose/who_to_see.py:196 `_a_sign_not_a_step(said, person)` (never called on the trace corpus)
pub fn _a_sign_not_a_step(said: &str, person: &Value) -> bool {
    crate::unported!("bot.compose.who_to_see:_a_sign_not_a_step")
}

/// Python: bot/compose/who_to_see.py:204 `_a_sentence_that_names_someone(said)` (never called on the trace corpus)
pub fn _a_sentence_that_names_someone(said: &str) -> Value {
    crate::unported!("bot.compose.who_to_see:_a_sentence_that_names_someone")
}

/// Python: bot/compose/who_to_see.py:218 `_the_call(said)` (never called on the trace corpus)
pub fn _the_call(said: &str) -> Value {
    crate::unported!("bot.compose.who_to_see:_the_call")
}

/// Python: bot/compose/who_to_see.py:256 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.who_to_see:reply_for")
}

/// Python: bot/compose/who_to_see.py:302 `replies_to_a_who_to_see(text, memory)`
pub fn replies_to_a_who_to_see(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.who_to_see:replies_to_a_who_to_see")
}

