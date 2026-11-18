//! Port of `bot/memory/obstacle.py`.
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

/// Python: bot/memory/obstacle.py:210 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.obstacle:_flat")
}

/// Python: bot/memory/obstacle.py:214 `_words(text)` (never called on the trace corpus)
pub fn _words(text: &str) -> Value {
    crate::unported!("bot.memory.obstacle:_words")
}

/// Python: bot/memory/obstacle.py:218 `the_thing(text)`
pub fn the_thing(text: &str) {
    crate::unported!("bot.memory.obstacle:the_thing")
}

/// Python: bot/memory/obstacle.py:270 `the_thing_they_cannot_use(text)`
pub fn the_thing_they_cannot_use(text: &str) {
    crate::unported!("bot.memory.obstacle:the_thing_they_cannot_use")
}

/// Python: bot/memory/obstacle.py:298 `the_move(text)`
pub fn the_move(text: &str) {
    crate::unported!("bot.memory.obstacle:the_move")
}

/// Python: bot/memory/obstacle.py:329 `_the_family(move)` (never called on the trace corpus)
pub fn _the_family(r#move: &Value) -> Value {
    crate::unported!("bot.memory.obstacle:_the_family")
}

/// Python: bot/memory/obstacle.py:345 `_has_family(words, family)` (never called on the trace corpus)
pub fn _has_family(words: &[String], family: &Value) -> bool {
    crate::unported!("bot.memory.obstacle:_has_family")
}

/// Python: bot/memory/obstacle.py:354 `_carries_a_move(sentence, family)` (never called on the trace corpus)
pub fn _carries_a_move(sentence: &str, family: &Value) -> bool {
    crate::unported!("bot.memory.obstacle:_carries_a_move")
}

/// Python: bot/memory/obstacle.py:398 `_opens_on_a_verb(move)` (never called on the trace corpus)
pub fn _opens_on_a_verb(r#move: &Value) -> bool {
    crate::unported!("bot.memory.obstacle:_opens_on_a_verb")
}

/// Python: bot/memory/obstacle.py:413 `_the_noun(move)` (never called on the trace corpus)
pub fn _the_noun(r#move: &Value) -> String {
    crate::unported!("bot.memory.obstacle:_the_noun")
}

/// Python: bot/memory/obstacle.py:418 `_a_list_under_a_frame(one)` (never called on the trace corpus)
pub fn _a_list_under_a_frame(one: &str) -> bool {
    crate::unported!("bot.memory.obstacle:_a_list_under_a_frame")
}

/// Python: bot/memory/obstacle.py:427 `_a_doing(one)` (never called on the trace corpus)
pub fn _a_doing(one: &str) -> bool {
    crate::unported!("bot.memory.obstacle:_a_doing")
}

/// Python: bot/memory/obstacle.py:448 `_a_whole_step(sentence)` (never called on the trace corpus)
pub fn _a_whole_step(sentence: &str) -> Value {
    crate::unported!("bot.memory.obstacle:_a_whole_step")
}

/// Python: bot/memory/obstacle.py:472 `a_move_refused(text, memory)`
pub fn a_move_refused(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.obstacle:a_move_refused")
}

/// Python: bot/memory/obstacle.py:551 `_under_its_frame(one)` (never called on the trace corpus)
pub fn _under_its_frame(one: &str) -> Value {
    crate::unported!("bot.memory.obstacle:_under_its_frame")
}

/// Python: bot/memory/obstacle.py:575 `_the_frame(one)` (never called on the trace corpus)
pub fn _the_frame(one: &str) -> String {
    crate::unported!("bot.memory.obstacle:_the_frame")
}

/// Python: bot/memory/obstacle.py:581 `_steps(said)` (never called on the trace corpus)
pub fn _steps(said: &str) -> Value {
    crate::unported!("bot.memory.obstacle:_steps")
}

/// Python: bot/memory/obstacle.py:606 `_needs_it(step, thing)` (never called on the trace corpus)
pub fn _needs_it(step: &Value, thing: &str) -> bool {
    crate::unported!("bot.memory.obstacle:_needs_it")
}

/// Python: bot/memory/obstacle.py:612 `_leans_on_it(said, thing, asked=None)` (never called on the trace corpus)
pub fn _leans_on_it(said: &str, thing: &str, asked: &str) -> bool {
    crate::unported!("bot.memory.obstacle:_leans_on_it")
}

/// Python: bot/memory/obstacle.py:635 `_before_this_turn(memory, text)` (never called on the trace corpus)
pub fn _before_this_turn(memory: &Memory, text: &str) -> Value {
    crate::unported!("bot.memory.obstacle:_before_this_turn")
}

/// Python: bot/memory/obstacle.py:655 `_the_reply_for(text, memory, thing, limited)` (never called on the trace corpus)
pub fn _the_reply_for(text: &str, memory: &mut Memory, thing: &str, limited: &Value) -> Value {
    crate::unported!("bot.memory.obstacle:_the_reply_for")
}

/// Python: bot/memory/obstacle.py:724 `answer_for(text, memory)`
pub fn answer_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.obstacle:answer_for")
}

/// Python: bot/memory/obstacle.py:734 `answer_for_a_thing_they_cannot_use(text, memory)`
pub fn answer_for_a_thing_they_cannot_use(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.obstacle:answer_for_a_thing_they_cannot_use")
}

