//! Port of `bot/memory/household.py`.
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

/// Python: bot/memory/household.py:192 `_who_said(who)` (never called on the trace corpus)
pub fn _who_said(who: &str) -> String {
    crate::unported!("bot.memory.household:_who_said")
}

/// Python: bot/memory/household.py:200 `_who_asked(who, memory)` (never called on the trace corpus)
pub fn _who_asked(who: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.household:_who_asked")
}

/// Python: bot/memory/household.py:227 `_who_held(who, key)` (never called on the trace corpus)
pub fn _who_held(who: &str, key: &str) -> Value {
    crate::unported!("bot.memory.household:_who_held")
}

/// Python: bot/memory/household.py:239 `_the_ask_it_answers(said, memory)`
pub fn _the_ask_it_answers(said: &str, memory: &Memory) {
    crate::unported!("bot.memory.household:_the_ask_it_answers")
}

/// Python: bot/memory/household.py:257 `_their_households(memory)` (never called on the trace corpus)
pub fn _their_households(memory: &Memory) -> Value {
    crate::unported!("bot.memory.household:_their_households")
}

/// Python: bot/memory/household.py:263 `_held(memory)` (never called on the trace corpus)
pub fn _held(memory: &Memory) -> Value {
    crate::unported!("bot.memory.household:_held")
}

/// Python: bot/memory/household.py:274 `observe(text, memory)`
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.household:observe")
}

/// Python: bot/memory/household.py:306 `answer(text, memory)`
pub fn answer(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.household:answer")
}

/// Python: bot/memory/household.py:393 `takes_as_it_comes(text, memory)` (never called on the trace corpus)
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.household:takes_as_it_comes")
}

