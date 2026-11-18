//! Port of `bot/compose/who.py`.
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

/// Python: bot/compose/who.py:134 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.compose.who:_flat")
}

/// Python: bot/compose/who.py:140 `asks_who(text)`
pub fn asks_who(text: &str) -> bool {
    crate::unported!("bot.compose.who:asks_who")
}

/// Python: bot/compose/who.py:145 `_last_module(memory)` (never called on the trace corpus)
pub fn _last_module(memory: &Memory) -> String {
    crate::unported!("bot.compose.who:_last_module")
}

/// Python: bot/compose/who.py:149 `_last_reply(memory)` (never called on the trace corpus)
pub fn _last_reply(memory: &Memory) -> Value {
    crate::unported!("bot.compose.who:_last_reply")
}

/// Python: bot/compose/who.py:153 `_names_no_subject(line)` (never called on the trace corpus)
pub fn _names_no_subject(line: &str) -> bool {
    crate::unported!("bot.compose.who:_names_no_subject")
}

/// Python: bot/compose/who.py:165 `_the_subject(memory)` (never called on the trace corpus)
pub fn _the_subject(memory: &Memory) -> Value {
    crate::unported!("bot.compose.who:_the_subject")
}

/// Python: bot/compose/who.py:188 `_thread_text(memory, answer='')` (never called on the trace corpus)
pub fn _thread_text(memory: &Memory, answer: &str) -> String {
    crate::unported!("bot.compose.who:_thread_text")
}

/// Python: bot/compose/who.py:207 `_sentence_at(text, at)` (never called on the trace corpus)
pub fn _sentence_at(text: &str, at: i64) -> Value {
    crate::unported!("bot.compose.who:_sentence_at")
}

/// Python: bot/compose/who.py:217 `_person_in(said)` (never called on the trace corpus)
pub fn _person_in(said: &str) -> Value {
    crate::unported!("bot.compose.who:_person_in")
}

/// Python: bot/compose/who.py:235 `_cut_the_preamble(sentence, at)` (never called on the trace corpus)
pub fn _cut_the_preamble(sentence: &str, at: i64) -> Value {
    crate::unported!("bot.compose.who:_cut_the_preamble")
}

/// Python: bot/compose/who.py:251 `_tidy(clause)` (never called on the trace corpus)
pub fn _tidy(clause: &str) -> Value {
    crate::unported!("bot.compose.who:_tidy")
}

/// Python: bot/compose/who.py:260 `_lead_of(name)` (never called on the trace corpus)
pub fn _lead_of(name: &str) -> Value {
    crate::unported!("bot.compose.who:_lead_of")
}

/// Python: bot/compose/who.py:264 `_the_way_in(name, memory, said, clause='')` (never called on the trace corpus)
pub fn _the_way_in(name: &str, memory: &Memory, said: &str, clause: &str) -> Value {
    crate::unported!("bot.compose.who:_the_way_in")
}

/// Python: bot/compose/who.py:281 `_says_it_again(way, said)` (never called on the trace corpus)
pub fn _says_it_again(way: &Value, said: &str) -> bool {
    crate::unported!("bot.compose.who:_says_it_again")
}

/// Python: bot/compose/who.py:290 `_person_for_the_topic(memory, said='', subject='')` (never called on the trace corpus)
pub fn _person_for_the_topic(memory: &Memory, said: &str, subject: &str) -> Value {
    crate::unported!("bot.compose.who:_person_for_the_topic")
}

/// Python: bot/compose/who.py:320 `_from_the_answer(name, sentence, at, memory, said, repeat=True)` (never called on the trace corpus)
pub fn _from_the_answer(name: &str, sentence: &str, at: i64, memory: &Memory, said: &str, repeat: &Value) -> Value {
    crate::unported!("bot.compose.who:_from_the_answer")
}

/// Python: bot/compose/who.py:343 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.who:reply_for")
}

/// Python: bot/compose/who.py:377 `takes_as_it_comes(text, memory)` (never called on the trace corpus)
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.compose.who:takes_as_it_comes")
}

