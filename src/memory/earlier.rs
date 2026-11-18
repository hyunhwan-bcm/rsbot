//! Port of `bot/memory/earlier.py`.
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

/// Python: bot/memory/earlier.py:130 `_a_word(word)`
pub fn _a_word(word: &str) -> bool {
    crate::unported!("bot.memory.earlier:_a_word")
}

/// Python: bot/memory/earlier.py:145 `_turns(memory)`
pub fn _turns(memory: &Memory) -> Value {
    crate::unported!("bot.memory.earlier:_turns")
}

/// Python: bot/memory/earlier.py:152 `said_lines(memory)`
pub fn said_lines(memory: &Memory) -> Value {
    crate::unported!("bot.memory.earlier:said_lines")
}

/// Python: bot/memory/earlier.py:172 `kind_of(text)`
pub fn kind_of(text: &str) -> Option<String> {
    crate::unported!("bot.memory.earlier:kind_of")
}

/// Python: bot/memory/earlier.py:196 `carries(kind, line)`
pub fn carries(kind: &str, line: &str) -> bool {
    crate::unported!("bot.memory.earlier:carries")
}

/// Python: bot/memory/earlier.py:207 `_about(text)`
pub fn _about(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.earlier:_about")
}

/// Python: bot/memory/earlier.py:213 `_said_words(text)` (never called on the trace corpus)
pub fn _said_words(text: &str) -> Value {
    crate::unported!("bot.memory.earlier:_said_words")
}

/// Python: bot/memory/earlier.py:224 `_mentions(line, words)`
pub fn _mentions(line: &str, words: &[String]) -> bool {
    crate::unported!("bot.memory.earlier:_mentions")
}

/// Python: bot/memory/earlier.py:238 `kinds_for(kind, ask)` (never called on the trace corpus)
pub fn kinds_for(kind: &str, ask: &str) -> Value {
    crate::unported!("bot.memory.earlier:kinds_for")
}

/// Python: bot/memory/earlier.py:251 `carried(memory, kind, about=(), window=None, prefer=(), after=0)`
pub fn carried(memory: &Memory, kind: &str, about: &[String], window: &Value, prefer: &[String], after: i64) {
    crate::unported!("bot.memory.earlier:carried")
}

/// Python: bot/memory/earlier.py:287 `_pointing_back(memory, about)` (never called on the trace corpus)
pub fn _pointing_back(memory: &Memory, about: &Value) -> bool {
    crate::unported!("bot.memory.earlier:_pointing_back")
}

/// Python: bot/memory/earlier.py:299 `carried_alone(memory, kind)` (never called on the trace corpus)
pub fn carried_alone(memory: &Memory, kind: &str) -> Value {
    crate::unported!("bot.memory.earlier:carried_alone")
}

/// Python: bot/memory/earlier.py:311 `hand_back(line, memory=None, text='')` (never called on the trace corpus)
pub fn hand_back(line: &str, memory: &Memory, text: &str) -> Option<String> {
    crate::unported!("bot.memory.earlier:hand_back")
}

/// Python: bot/memory/earlier.py:319 `a_claim(reply)`
pub fn a_claim(reply: &str) -> Option<String> {
    crate::unported!("bot.memory.earlier:a_claim")
}

/// Python: bot/memory/earlier.py:336 `_values(kind, text)` (never called on the trace corpus)
pub fn _values(kind: &str, text: &str) -> Value {
    crate::unported!("bot.memory.earlier:_values")
}

/// Python: bot/memory/earlier.py:350 `_the_line_the_ask_names(text, memory)`
pub fn _the_line_the_ask_names(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.earlier:_the_line_the_ask_names")
}

/// Python: bot/memory/earlier.py:367 `gives_the_named_value(text, memory, reply)` (never called on the trace corpus)
pub fn gives_the_named_value(text: &str, memory: &Memory, reply: &str) -> bool {
    crate::unported!("bot.memory.earlier:gives_the_named_value")
}

/// Python: bot/memory/earlier.py:375 `another_line_was_asked_for(text, memory, reply)`
pub fn another_line_was_asked_for(text: &str, memory: &Memory, reply: &str) -> bool {
    crate::unported!("bot.memory.earlier:another_line_was_asked_for")
}

/// Python: bot/memory/earlier.py:396 `a_misquoted_line(reply, memory, ask='')`
pub fn a_misquoted_line(reply: &str, memory: &Memory, ask: &str) {
    crate::unported!("bot.memory.earlier:a_misquoted_line")
}

/// Python: bot/memory/earlier.py:422 `answer_for(text, memory)`
pub fn answer_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.earlier:answer_for")
}

/// Python: bot/memory/earlier.py:477 `their_own_words(text)`
pub fn their_own_words(text: &str) -> bool {
    crate::unported!("bot.memory.earlier:their_own_words")
}

/// Python: bot/memory/earlier.py:498 `from_the_line(text, memory)`
pub fn from_the_line(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.earlier:from_the_line")
}

/// Python: bot/memory/earlier.py:537 `put_right(reply, memory, ask='')`
pub fn put_right(reply: &str, memory: &Memory, ask: &str) {
    crate::unported!("bot.memory.earlier:put_right")
}

