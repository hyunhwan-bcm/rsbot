//! Port of `bot/memory/returns.py`.
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

/// Python: bot/memory/returns.py:65 `_the_chat_has(words, memory)` (never called on the trace corpus)
pub fn _the_chat_has(words: &[String], memory: &Memory) -> bool {
    crate::unported!("bot.memory.returns:_the_chat_has")
}

/// Python: bot/memory/returns.py:77 `_the_parts(text)`
pub fn _the_parts(text: &str) {
    crate::unported!("bot.memory.returns:_the_parts")
}

/// Python: bot/memory/returns.py:113 `_the_lines(named, ask)` (never called on the trace corpus)
pub fn _the_lines(named: &Value, ask: &str) -> Value {
    crate::unported!("bot.memory.returns:_the_lines")
}

/// Python: bot/memory/returns.py:135 `_the_domain_of_the_chat(named, memory)` (never called on the trace corpus)
pub fn _the_domain_of_the_chat(named: &Value, memory: &Memory) -> String {
    crate::unported!("bot.memory.returns:_the_domain_of_the_chat")
}

/// Python: bot/memory/returns.py:163 `a_return_to_the_topic(text, memory)`
pub fn a_return_to_the_topic(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.returns:a_return_to_the_topic")
}

/// Python: bot/memory/returns.py:204 `the_thing_named(text)` (never called on the trace corpus)
pub fn the_thing_named(text: &str) -> String {
    crate::unported!("bot.memory.returns:the_thing_named")
}

/// Python: bot/memory/returns.py:225 `_the_sentence_with_the_amount(answer)` (never called on the trace corpus)
pub fn _the_sentence_with_the_amount(answer: &str) -> String {
    crate::unported!("bot.memory.returns:_the_sentence_with_the_amount")
}

/// Python: bot/memory/returns.py:233 `as_the_same_answer(reply, text, short=None)` (never called on the trace corpus)
pub fn as_the_same_answer(reply: &str, text: &str, short: &Value) -> String {
    crate::unported!("bot.memory.returns:as_the_same_answer")
}

/// Python: bot/memory/returns.py:272 `a_bare_return(text, memory=None)`
pub fn a_bare_return(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.returns:a_bare_return")
}

