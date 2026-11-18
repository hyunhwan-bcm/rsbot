//! Port of `bot/memory/callback.py`.
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

/// Python: bot/memory/callback.py:49 `_the_person_held(memory)` (never called on the trace corpus)
pub fn _the_person_held(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.callback:_the_person_held")
}

/// Python: bot/memory/callback.py:79 `_the_news_about(memory, relation, name)` (never called on the trace corpus)
pub fn _the_news_about(memory: &Memory, relation: &Value, name: &str) -> String {
    crate::unported!("bot.memory.callback:_the_news_about")
}

/// Python: bot/memory/callback.py:109 `_the_pronouns(relation)` (never called on the trace corpus)
pub fn _the_pronouns(relation: &Value) -> Vec<String> {
    crate::unported!("bot.memory.callback:_the_pronouns")
}

/// Python: bot/memory/callback.py:117 `_the_ask(text, memory)`
pub fn _the_ask(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.callback:_the_ask")
}

/// Python: bot/memory/callback.py:167 `_a_medical_episode(memory)`
pub fn _a_medical_episode(memory: &Memory) -> bool {
    crate::unported!("bot.memory.callback:_a_medical_episode")
}

/// Python: bot/memory/callback.py:175 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.memory.callback:reply_for")
}

/// Python: bot/memory/callback.py:217 `claims(text, memory)` (never called on the trace corpus)
pub fn claims(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.callback:claims")
}

/// Python: bot/memory/callback.py:228 `_choose(pool, memory)` (never called on the trace corpus)
pub fn _choose(pool: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.callback:_choose")
}

