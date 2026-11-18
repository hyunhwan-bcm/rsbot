//! Port of `bot/memory/back_again.py`.
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

/// Python: bot/memory/back_again.py:112 `_the_earlier_line(memory, thing, text='')` (never called on the trace corpus)
pub fn _the_earlier_line(memory: &Memory, thing: &str, text: &str) -> Value {
    crate::unported!("bot.memory.back_again:_the_earlier_line")
}

/// Python: bot/memory/back_again.py:132 `_the_thing_went_away(memory, thing, text='')` (never called on the trace corpus)
pub fn _the_thing_went_away(memory: &Memory, thing: &str, text: &str) -> bool {
    crate::unported!("bot.memory.back_again:_the_thing_went_away")
}

/// Python: bot/memory/back_again.py:142 `_the_words(plural)` (never called on the trace corpus)
pub fn _the_words(plural: &Value) -> Value {
    crate::unported!("bot.memory.back_again:_the_words")
}

/// Python: bot/memory/back_again.py:149 `_filled(line, thing, plural)` (never called on the trace corpus)
pub fn _filled(line: &str, thing: &str, plural: &Value) -> Value {
    crate::unported!("bot.memory.back_again:_filled")
}

/// Python: bot/memory/back_again.py:167 `_an_earlier_topic(memory, thing)` (never called on the trace corpus)
pub fn _an_earlier_topic(memory: &Memory, thing: &str) -> bool {
    crate::unported!("bot.memory.back_again:_an_earlier_topic")
}

/// Python: bot/memory/back_again.py:182 `_from_this_line(record, memory)` (never called on the trace corpus)
pub fn _from_this_line(record: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.back_again:_from_this_line")
}

/// Python: bot/memory/back_again.py:187 `_the_thing(text, memory)`
pub fn _the_thing(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.back_again:_the_thing")
}

/// Python: bot/memory/back_again.py:247 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.back_again:reply_for")
}

