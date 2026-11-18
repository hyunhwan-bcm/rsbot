//! Port of `bot/compose/onsubject.py`.
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

/// Python: bot/compose/onsubject.py:347 `_words(line)`
pub fn _words(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.onsubject:_words")
}

/// Python: bot/compose/onsubject.py:351 `_bare_follow_up(clean)`
pub fn _bare_follow_up(clean: &str) {
    crate::unported!("bot.compose.onsubject:_bare_follow_up")
}

/// Python: bot/compose/onsubject.py:372 `_stem(word)`
pub fn _stem(word: &str) -> String {
    crate::unported!("bot.compose.onsubject:_stem")
}

/// Python: bot/compose/onsubject.py:390 `_only_a_place(words, index)` (never called on the trace corpus)
pub fn _only_a_place(words: &[String], index: i64) -> bool {
    crate::unported!("bot.compose.onsubject:_only_a_place")
}

/// Python: bot/compose/onsubject.py:400 `_beside(words, index, allowed)` (never called on the trace corpus)
pub fn _beside(words: &[String], index: i64, allowed: &Value) -> bool {
    crate::unported!("bot.compose.onsubject:_beside")
}

/// Python: bot/compose/onsubject.py:407 `_named(line, skip_place_words=False, light=True)`
pub fn _named(line: &str, skip_place_words: bool, light: bool) -> Value {
    crate::unported!("bot.compose.onsubject:_named")
}

/// Python: bot/compose/onsubject.py:433 `_the_subject(memory, clean='')`
pub fn _the_subject(memory: &Memory, clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.onsubject:_the_subject")
}

/// Python: bot/compose/onsubject.py:492 `_already_the_who(word, kind)`
pub fn _already_the_who(word: &str, kind: &str) -> bool {
    crate::unported!("bot.compose.onsubject:_already_the_who")
}

/// Python: bot/compose/onsubject.py:503 `_beside_the_who(line, kind, subject)`
pub fn _beside_the_who(line: &str, kind: &str, subject: &str) -> String {
    crate::unported!("bot.compose.onsubject:_beside_the_who")
}

/// Python: bot/compose/onsubject.py:523 `_shape(text)`
pub fn _shape(text: &str) -> String {
    crate::unported!("bot.compose.onsubject:_shape")
}

/// Python: bot/compose/onsubject.py:538 `_content(text)` (never called on the trace corpus)
pub fn _content(text: &str) -> Value {
    crate::unported!("bot.compose.onsubject:_content")
}

/// Python: bot/compose/onsubject.py:543 `_already_said(sentence, memory)` (never called on the trace corpus)
pub fn _already_said(sentence: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.onsubject:_already_said")
}

/// Python: bot/compose/onsubject.py:558 `_one_sentence(text, limit=250)` (never called on the trace corpus)
pub fn _one_sentence(text: &str, limit: i64) -> Value {
    crate::unported!("bot.compose.onsubject:_one_sentence")
}

/// Python: bot/compose/onsubject.py:580 `_cap(clause)` (never called on the trace corpus)
pub fn _cap(clause: &str) -> Value {
    crate::unported!("bot.compose.onsubject:_cap")
}

/// Python: bot/compose/onsubject.py:617 `_the_place(line)` (never called on the trace corpus)
pub fn _the_place(line: &str) -> Value {
    crate::unported!("bot.compose.onsubject:_the_place")
}

/// Python: bot/compose/onsubject.py:626 `_the_answer_about_the_place(text, line, memory)`
pub fn _the_answer_about_the_place(text: &str, line: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.onsubject:_the_answer_about_the_place")
}

/// Python: bot/compose/onsubject.py:684 `_what_it_knows(line, memory)` (never called on the trace corpus)
pub fn _what_it_knows(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.onsubject:_what_it_knows")
}

/// Python: bot/compose/onsubject.py:721 `_the_line_before(chat, now)` (never called on the trace corpus)
pub fn _the_line_before(chat: &Value, now: &Value) -> Value {
    crate::unported!("bot.compose.onsubject:_the_line_before")
}

/// Python: bot/compose/onsubject.py:734 `place_pool(clean, understood=None, memory=None)`
pub fn place_pool(clean: &str, understood: &Understood, memory: &Memory) {
    crate::unported!("bot.compose.onsubject:place_pool")
}

/// Python: bot/compose/onsubject.py:774 `_food_above(memory)` (never called on the trace corpus)
pub fn _food_above(memory: &Memory) -> bool {
    crate::unported!("bot.compose.onsubject:_food_above")
}

/// Python: bot/compose/onsubject.py:786 `pool(clean, understood=None, memory=None)`
pub fn pool(clean: &str, understood: &Understood, memory: &Memory) {
    crate::unported!("bot.compose.onsubject:pool")
}

