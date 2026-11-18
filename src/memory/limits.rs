//! Port of `bot/memory/limits.py`.
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

/// Python: bot/memory/limits.py:162 `_the_trip_is_on(clean, memory)` (never called on the trace corpus)
pub fn _the_trip_is_on(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.limits:_the_trip_is_on")
}

/// Python: bot/memory/limits.py:218 `_kind(clean)`
pub fn _kind(clean: &str) {
    crate::unported!("bot.memory.limits:_kind")
}

/// Python: bot/memory/limits.py:245 `names_a_limit(text)`
pub fn names_a_limit(text: &str) {
    crate::unported!("bot.memory.limits:names_a_limit")
}

/// Python: bot/memory/limits.py:255 `reply(text, memory)`
pub fn reply(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.limits:reply")
}

/// Python: bot/memory/limits.py:288 `_turns(memory)` (never called on the trace corpus)
pub fn _turns(memory: &Memory) -> Value {
    crate::unported!("bot.memory.limits:_turns")
}

/// Python: bot/memory/limits.py:293 `said(memory)` (never called on the trace corpus)
pub fn said(memory: &Memory) -> Value {
    crate::unported!("bot.memory.limits:said")
}

