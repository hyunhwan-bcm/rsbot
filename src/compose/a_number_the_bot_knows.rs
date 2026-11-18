//! Port of `bot/compose/a_number_the_bot_knows.py`.
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

/// Python: bot/compose/a_number_the_bot_knows.py:112 `_said_before(memory)`
pub fn _said_before(memory: &Memory) -> String {
    crate::unported!("bot.compose.a_number_the_bot_knows:_said_before")
}

/// Python: bot/compose/a_number_the_bot_knows.py:124 `_carried_thing(text)` (never called on the trace corpus)
pub fn _carried_thing(text: &str) -> Value {
    crate::unported!("bot.compose.a_number_the_bot_knows:_carried_thing")
}

/// Python: bot/compose/a_number_the_bot_knows.py:143 `_asked_about(text)`
pub fn _asked_about(text: &str) -> String {
    crate::unported!("bot.compose.a_number_the_bot_knows:_asked_about")
}

/// Python: bot/compose/a_number_the_bot_knows.py:160 `_names(thing, names)` (never called on the trace corpus)
pub fn _names(thing: &str, names: &[String]) -> bool {
    crate::unported!("bot.compose.a_number_the_bot_knows:_names")
}

/// Python: bot/compose/a_number_the_bot_knows.py:172 `_replies(memory)`
pub fn _replies(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.a_number_the_bot_knows:_replies")
}

/// Python: bot/compose/a_number_the_bot_knows.py:190 `already_said(number, memory)` (never called on the trace corpus)
pub fn already_said(number: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.compose.a_number_the_bot_knows:already_said")
}

/// Python: bot/compose/a_number_the_bot_knows.py:199 `not_a_number(reply)` (never called on the trace corpus)
pub fn not_a_number(reply: &str) -> bool {
    crate::unported!("bot.compose.a_number_the_bot_knows:not_a_number")
}

/// Python: bot/compose/a_number_the_bot_knows.py:210 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.a_number_the_bot_knows:answer_for")
}

