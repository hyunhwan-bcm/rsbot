//! Port of `bot/memory/asleep.py`.
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

/// Python: bot/memory/asleep.py:91 `_hour_from(atom)` (never called on the trace corpus)
pub fn _hour_from(atom: &Value) -> Value {
    crate::unported!("bot.memory.asleep:_hour_from")
}

/// Python: bot/memory/asleep.py:118 `_end(marker, text, after_bed, limit=26)`
pub fn _end(marker: &Value, text: &str, after_bed: bool, limit: i64) {
    crate::unported!("bot.memory.asleep:_end")
}

/// Python: bot/memory/asleep.py:151 `_ends(line, wake_only=False)`
pub fn _ends(line: &str, wake_only: bool) -> Value {
    crate::unported!("bot.memory.asleep:_ends")
}

/// Python: bot/memory/asleep.py:159 `_last_night(memory)` (never called on the trace corpus)
pub fn _last_night(memory: &Memory) -> Value {
    crate::unported!("bot.memory.asleep:_last_night")
}

/// Python: bot/memory/asleep.py:182 `_a_length(hours)` (never called on the trace corpus)
pub fn _a_length(hours: &Value) -> String {
    crate::unported!("bot.memory.asleep:_a_length")
}

/// Python: bot/memory/asleep.py:193 `_a_word(number)` (never called on the trace corpus)
pub fn _a_word(number: &Value) -> Value {
    crate::unported!("bot.memory.asleep:_a_word")
}

/// Python: bot/memory/asleep.py:219 `_seen(memory, reply)` (never called on the trace corpus)
pub fn _seen(memory: &mut Memory, reply: &str) -> Value {
    crate::unported!("bot.memory.asleep:_seen")
}

/// Python: bot/memory/asleep.py:226 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.asleep:answer_for")
}

