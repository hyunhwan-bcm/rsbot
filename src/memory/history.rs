//! Port of `bot/memory/history.py`.
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

/// Python: bot/memory/history.py:123 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.history:_flat")
}

/// Python: bot/memory/history.py:128 `the_thing_named(text)`
pub fn the_thing_named(text: &str) -> String {
    crate::unported!("bot.memory.history:the_thing_named")
}

/// Python: bot/memory/history.py:155 `the_thing_they_named(text)` (never called on the trace corpus)
pub fn the_thing_they_named(text: &str) -> Value {
    crate::unported!("bot.memory.history:the_thing_they_named")
}

/// Python: bot/memory/history.py:174 `asked(text, memory=None)`
pub fn asked(text: &str, memory: Option<&Memory>) {
    crate::unported!("bot.memory.history:asked")
}

/// Python: bot/memory/history.py:217 `_nothing_said(text)`
pub fn _nothing_said(text: &str) -> bool {
    crate::unported!("bot.memory.history:_nothing_said")
}

/// Python: bot/memory/history.py:223 `about_the_chat(text)`
pub fn about_the_chat(text: &str) -> bool {
    crate::unported!("bot.memory.history:about_the_chat")
}

/// Python: bot/memory/history.py:233 `not_a_subject(text)`
pub fn not_a_subject(text: &str) -> bool {
    crate::unported!("bot.memory.history:not_a_subject")
}

/// Python: bot/memory/history.py:312 `brings_nothing_of_its_own(text)`
pub fn brings_nothing_of_its_own(text: &str) -> bool {
    crate::unported!("bot.memory.history:brings_nothing_of_its_own")
}

/// Python: bot/memory/history.py:320 `an_earlier_turn(memory, text='')`
pub fn an_earlier_turn(memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.memory.history:an_earlier_turn")
}

/// Python: bot/memory/history.py:332 `a_past_that_is_not_there(reply, memory, said=None)`
pub fn a_past_that_is_not_there(reply: &str, memory: &Value, said: Option<&str>) -> bool {
    crate::unported!("bot.memory.history:a_past_that_is_not_there")
}

/// Python: bot/memory/history.py:356 `first_bot_line(memory)` (never called on the trace corpus)
pub fn first_bot_line(memory: &Memory) -> String {
    crate::unported!("bot.memory.history:first_bot_line")
}

/// Python: bot/memory/history.py:366 `_earlier_turns(memory, text='')`
pub fn _earlier_turns(memory: &Memory, text: &str) -> Vec<String> {
    crate::unported!("bot.memory.history:_earlier_turns")
}

/// Python: bot/memory/history.py:393 `_a_line_they_asked(row)` (never called on the trace corpus)
pub fn _a_line_they_asked(row: &'static Entry) -> bool {
    crate::unported!("bot.memory.history:_a_line_they_asked")
}

/// Python: bot/memory/history.py:398 `_the_forms(words)` (never called on the trace corpus)
pub fn _the_forms(words: &[String]) -> Value {
    crate::unported!("bot.memory.history:_the_forms")
}

/// Python: bot/memory/history.py:411 `the_earlier_turn(memory, text='', about='')` (never called on the trace corpus)
pub fn the_earlier_turn(memory: &Memory, text: &str, about: &Value) -> Value {
    crate::unported!("bot.memory.history:the_earlier_turn")
}

/// Python: bot/memory/history.py:439 `first_line(memory)` (never called on the trace corpus)
pub fn first_line(memory: &Memory) -> String {
    crate::unported!("bot.memory.history:first_line")
}

/// Python: bot/memory/history.py:449 `topic_lines(memory, limit=3)` (never called on the trace corpus)
pub fn topic_lines(memory: &Memory, limit: i64) -> Value {
    crate::unported!("bot.memory.history:topic_lines")
}

/// Python: bot/memory/history.py:507 `turns_before(memory)`
pub fn turns_before(memory: &Memory) -> Value {
    crate::unported!("bot.memory.history:turns_before")
}

/// Python: bot/memory/history.py:517 `the_turn_before(memory)` (never called on the trace corpus)
pub fn the_turn_before(memory: &Memory) -> Value {
    crate::unported!("bot.memory.history:the_turn_before")
}

/// Python: bot/memory/history.py:527 `nothing_said_before(memory)` (never called on the trace corpus)
pub fn nothing_said_before(memory: &Memory) -> bool {
    crate::unported!("bot.memory.history:nothing_said_before")
}

/// Python: bot/memory/history.py:536 `the_lines_before(memory, look_back=LOOK_BACK)` (never called on the trace corpus)
pub fn the_lines_before(memory: &Memory, look_back: &Value) -> Value {
    crate::unported!("bot.memory.history:the_lines_before")
}

/// Python: bot/memory/history.py:542 `the_line_before(memory, look_back=LOOK_BACK)` (never called on the trace corpus)
pub fn the_line_before(memory: &Memory, look_back: &Value) -> Value {
    crate::unported!("bot.memory.history:the_line_before")
}

/// Python: bot/memory/history.py:548 `is_their_news(line, words=3)` (never called on the trace corpus)
pub fn is_their_news(line: &str, words: &[String]) -> bool {
    crate::unported!("bot.memory.history:is_their_news")
}

/// Python: bot/memory/history.py:566 `the_news_turn_before(memory, words=3)` (never called on the trace corpus)
pub fn the_news_turn_before(memory: &Memory, words: &[String]) -> Value {
    crate::unported!("bot.memory.history:the_news_turn_before")
}

/// Python: bot/memory/history.py:579 `the_earlier_line(memory, pattern, look_back=LOOK_BACK)` (never called on the trace corpus)
pub fn the_earlier_line(memory: &Memory, pattern: &Value, look_back: &Value) -> Value {
    crate::unported!("bot.memory.history:the_earlier_line")
}

/// Python: bot/memory/history.py:594 `the_earlier_lines_that(memory, pattern, look_back=LOOK_BACK)` (never called on the trace corpus)
pub fn the_earlier_lines_that(memory: &Memory, pattern: &Value, look_back: &Value) -> Value {
    crate::unported!("bot.memory.history:the_earlier_lines_that")
}

/// Python: bot/memory/history.py:601 `only_with_an_earlier_turn(reader)` (never called on the trace corpus)
pub fn only_with_an_earlier_turn(reader: &Value) -> Value {
    crate::unported!("bot.memory.history:only_with_an_earlier_turn")
}

/// Python: bot/memory/history.py:631 `the_last_answer(memory)` (never called on the trace corpus)
pub fn the_last_answer(memory: &Memory) -> String {
    crate::unported!("bot.memory.history:the_last_answer")
}

/// Python: bot/memory/history.py:644 `_a_note_not_an_answer(said)`
pub fn _a_note_not_an_answer(said: &str) -> bool {
    crate::unported!("bot.memory.history:_a_note_not_an_answer")
}

/// Python: bot/memory/history.py:650 `_a_hedge(said)`
pub fn _a_hedge(said: &str) -> bool {
    crate::unported!("bot.memory.history:_a_hedge")
}

/// Python: bot/memory/history.py:657 `the_answer_on_screen(memory, within=3, modules=('answers', 'safety'))`
pub fn the_answer_on_screen(memory: &Memory, within: i64, modules: &[String]) -> String {
    crate::unported!("bot.memory.history:the_answer_on_screen")
}

/// Python: bot/memory/history.py:683 `the_turn_the_answer_came_from(memory, within=3)` (never called on the trace corpus)
pub fn the_turn_the_answer_came_from(memory: &Memory, within: &Value) -> Value {
    crate::unported!("bot.memory.history:the_turn_the_answer_came_from")
}

/// Python: bot/memory/history.py:694 `the_answer_is_a_card(memory, within=3)` (never called on the trace corpus)
pub fn the_answer_is_a_card(memory: &Memory, within: &Value) -> bool {
    crate::unported!("bot.memory.history:the_answer_is_a_card")
}

