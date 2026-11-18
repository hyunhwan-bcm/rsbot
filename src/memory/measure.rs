//! Port of `bot/memory/measure.py`.
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

/// Python: bot/memory/measure.py:69 `the_measure_asked(line)`
pub fn the_measure_asked(line: &str) -> String {
    crate::unported!("bot.memory.measure:the_measure_asked")
}

/// Python: bot/memory/measure.py:93 `_the_card_said_last(memory)` (never called on the trace corpus)
pub fn _the_card_said_last(memory: &Memory) -> Value {
    crate::unported!("bot.memory.measure:_the_card_said_last")
}

/// Python: bot/memory/measure.py:136 `_the_clause_that_carries(sentence, pattern=None)` (never called on the trace corpus)
pub fn _the_clause_that_carries(sentence: &str, pattern: &Value) -> Value {
    crate::unported!("bot.memory.measure:_the_clause_that_carries")
}

/// Python: bot/memory/measure.py:148 `_the_sentence_with(text, measure)` (never called on the trace corpus)
pub fn _the_sentence_with(text: &str, measure: &Value) -> Value {
    crate::unported!("bot.memory.measure:_the_sentence_with")
}

/// Python: bot/memory/measure.py:171 `_the_thing_the_line_named(memory)` (never called on the trace corpus)
pub fn _the_thing_the_line_named(memory: &Memory) -> Value {
    crate::unported!("bot.memory.measure:_the_thing_the_line_named")
}

/// Python: bot/memory/measure.py:212 `_with_the_thing(sentence, thing)` (never called on the trace corpus)
pub fn _with_the_thing(sentence: &str, thing: &str) -> String {
    crate::unported!("bot.memory.measure:_with_the_thing")
}

/// Python: bot/memory/measure.py:222 `_as_a_headline(thing, sentence)` (never called on the trace corpus)
pub fn _as_a_headline(thing: &str, sentence: &str) -> String {
    crate::unported!("bot.memory.measure:_as_a_headline")
}

/// Python: bot/memory/measure.py:231 `_as_a_sentence(text)` (never called on the trace corpus)
pub fn _as_a_sentence(text: &str) -> Value {
    crate::unported!("bot.memory.measure:_as_a_sentence")
}

/// Python: bot/memory/measure.py:240 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.measure:reply_for")
}

