//! Port of `bot/memory/carry.py`.
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

/// Python: bot/memory/carry.py:96 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.carry:_flat")
}

/// Python: bot/memory/carry.py:100 `_the_thing_in_hand(text, memory)`
pub fn _the_thing_in_hand(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.carry:_the_thing_in_hand")
}

/// Python: bot/memory/carry.py:120 `_a_fresh_memory(memory)` (never called on the trace corpus)
pub fn _a_fresh_memory(memory: &Memory) -> Value {
    crate::unported!("bot.memory.carry:_a_fresh_memory")
}

/// Python: bot/memory/carry.py:136 `_the_answer_without_the_condition(subject_text, memory)` (never called on the trace corpus)
pub fn _the_answer_without_the_condition(subject_text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.carry:_the_answer_without_the_condition")
}

/// Python: bot/memory/carry.py:146 `_named_in(reply, thing, subject)` (never called on the trace corpus)
pub fn _named_in(reply: &str, thing: &str, subject: &str) -> bool {
    crate::unported!("bot.memory.carry:_named_in")
}

/// Python: bot/memory/carry.py:165 `_already_on_the_screen(reply, memory)` (never called on the trace corpus)
pub fn _already_on_the_screen(reply: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.carry:_already_on_the_screen")
}

/// Python: bot/memory/carry.py:206 `_a_verdict_on_the_other_thing(reply, condition, thing, subject)` (never called on the trace corpus)
pub fn _a_verdict_on_the_other_thing(reply: &str, condition: &Value, thing: &str, subject: &str) -> bool {
    crate::unported!("bot.memory.carry:_a_verdict_on_the_other_thing")
}

/// Python: bot/memory/carry.py:228 `_a_decision_in_the_chat(memory)`
pub fn _a_decision_in_the_chat(memory: &Memory) -> bool {
    crate::unported!("bot.memory.carry:_a_decision_in_the_chat")
}

/// Python: bot/memory/carry.py:269 `_a_verdict_asked(clean)`
pub fn _a_verdict_asked(clean: &str) -> bool {
    crate::unported!("bot.memory.carry:_a_verdict_asked")
}

/// Python: bot/memory/carry.py:283 `the_verdict_in_the_chat(text, memory)`
pub fn the_verdict_in_the_chat(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.carry:the_verdict_in_the_chat")
}

/// Python: bot/memory/carry.py:302 `_the_verdict_answer(memory, line, subject_text, thing, since)` (never called on the trace corpus)
pub fn _the_verdict_answer(memory: &Memory, line: &str, subject_text: &str, thing: &str, since: &Value) -> Value {
    crate::unported!("bot.memory.carry:_the_verdict_answer")
}

/// Python: bot/memory/carry.py:339 `_the_answer_just_given(reply, memory)` (never called on the trace corpus)
pub fn _the_answer_just_given(reply: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.carry:_the_answer_just_given")
}

/// Python: bot/memory/carry.py:351 `_the_things_the_verdict_is_about(memory, line)` (never called on the trace corpus)
pub fn _the_things_the_verdict_is_about(memory: &Memory, line: &str) -> Value {
    crate::unported!("bot.memory.carry:_the_things_the_verdict_is_about")
}

/// Python: bot/memory/carry.py:383 `_the_thing_they_named(line)` (never called on the trace corpus)
pub fn _the_thing_they_named(line: &str) -> Value {
    crate::unported!("bot.memory.carry:_the_thing_they_named")
}

/// Python: bot/memory/carry.py:404 `_the_lines_of_the_chat(memory)` (never called on the trace corpus)
pub fn _the_lines_of_the_chat(memory: &Memory) -> Value {
    crate::unported!("bot.memory.carry:_the_lines_of_the_chat")
}

/// Python: bot/memory/carry.py:418 `the_subject_in_the_condition(text, memory)`
pub fn the_subject_in_the_condition(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.carry:the_subject_in_the_condition")
}

/// Python: bot/memory/carry.py:520 `_the_quality(line)`
pub fn _the_quality(line: &str) {
    crate::unported!("bot.memory.carry:_the_quality")
}

/// Python: bot/memory/carry.py:546 `_the_course(memory, quality='')` (never called on the trace corpus)
pub fn _the_course(memory: &Memory, quality: &Value) -> Option<String> {
    crate::unported!("bot.memory.carry:_the_course")
}

/// Python: bot/memory/carry.py:569 `_is_a_hedge(reply)` (never called on the trace corpus)
pub fn _is_a_hedge(reply: &str) -> Option<bool> {
    crate::unported!("bot.memory.carry:_is_a_hedge")
}

/// Python: bot/memory/carry.py:577 `the_quality_in_the_chat(text, memory)`
pub fn the_quality_in_the_chat(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.carry:the_quality_in_the_chat")
}

/// Python: bot/memory/carry.py:628 `_the_last_ask(lines, said)` (never called on the trace corpus)
pub fn _the_last_ask(lines: &[String], said: &str) -> Value {
    crate::unported!("bot.memory.carry:_the_last_ask")
}

/// Python: bot/memory/carry.py:673 `_speaks_of(reply, things)` (never called on the trace corpus)
pub fn _speaks_of(reply: &str, things: &Value) -> bool {
    crate::unported!("bot.memory.carry:_speaks_of")
}

/// Python: bot/memory/carry.py:696 `_a_free_from(things)` (never called on the trace corpus)
pub fn _a_free_from(things: &Value) -> bool {
    crate::unported!("bot.memory.carry:_a_free_from")
}

/// Python: bot/memory/carry.py:708 `_the_subject_named(reply, base)` (never called on the trace corpus)
pub fn _the_subject_named(reply: &str, base: &Value) -> bool {
    crate::unported!("bot.memory.carry:_the_subject_named")
}

/// Python: bot/memory/carry.py:720 `the_limit_in_the_chat(text, memory)`
pub fn the_limit_in_the_chat(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.carry:the_limit_in_the_chat")
}

/// Python: bot/memory/carry.py:781 `_cooks_around_it(reply)` (never called on the trace corpus)
pub fn _cooks_around_it(reply: &str) -> bool {
    crate::unported!("bot.memory.carry:_cooks_around_it")
}

