//! Port of `bot/memory/already.py`.
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

/// Python: bot/memory/already.py:69 `_the_step_after_did(low)`
pub fn _the_step_after_did(low: &str) {
    crate::unported!("bot.memory.already:_the_step_after_did")
}

/// Python: bot/memory/already.py:232 `_a_crisis_line_said(memory, words)` (never called on the trace corpus)
pub fn _a_crisis_line_said(memory: &Memory, words: &[String]) -> bool {
    crate::unported!("bot.memory.already:_a_crisis_line_said")
}

/// Python: bot/memory/already.py:242 `_a_hard_thread(memory)` (never called on the trace corpus)
pub fn _a_hard_thread(memory: &Memory) -> bool {
    crate::unported!("bot.memory.already:_a_hard_thread")
}

/// Python: bot/memory/already.py:293 `_the_step(low)`
pub fn _the_step(low: &str) {
    crate::unported!("bot.memory.already:_the_step")
}

/// Python: bot/memory/already.py:342 `_nothing_more(low, match)` (never called on the trace corpus)
pub fn _nothing_more(low: &str, r#match: &Value) -> bool {
    crate::unported!("bot.memory.already:_nothing_more")
}

/// Python: bot/memory/already.py:348 `_a_job_with_an_end(answer)` (never called on the trace corpus)
pub fn _a_job_with_an_end(answer: &str) -> bool {
    crate::unported!("bot.memory.already:_a_job_with_an_end")
}

/// Python: bot/memory/already.py:354 `_the_finished_step(low, memory=None)`
pub fn _the_finished_step(low: &str, memory: Option<&Memory>) {
    crate::unported!("bot.memory.already:_the_finished_step")
}

/// Python: bot/memory/already.py:395 `the_step_they_took(text, memory=None)`
pub fn the_step_they_took(text: &str, memory: Option<&Memory>) {
    crate::unported!("bot.memory.already:the_step_they_took")
}

/// Python: bot/memory/already.py:541 `_the_thing_said(low, at)` (never called on the trace corpus)
pub fn _the_thing_said(low: &str, at: i64) -> Value {
    crate::unported!("bot.memory.already:_the_thing_said")
}

/// Python: bot/memory/already.py:583 `the_thing_that_is_mended(text)` (never called on the trace corpus)
pub fn the_thing_that_is_mended(text: &str) -> Value {
    crate::unported!("bot.memory.already:the_thing_that_is_mended")
}

/// Python: bot/memory/already.py:605 `_the_clause(low, at)` (never called on the trace corpus)
pub fn _the_clause(low: &str, at: i64) -> Value {
    crate::unported!("bot.memory.already:_the_clause")
}

/// Python: bot/memory/already.py:623 `_the_over_word(text)`
pub fn _the_over_word(text: &str) -> Value {
    crate::unported!("bot.memory.already:_the_over_word")
}

/// Python: bot/memory/already.py:660 `the_trouble_is_over(text)`
pub fn the_trouble_is_over(text: &str) {
    crate::unported!("bot.memory.already:the_trouble_is_over")
}

/// Python: bot/memory/already.py:670 `_gave_a_step(answer)` (never called on the trace corpus)
pub fn _gave_a_step(answer: &str) -> Option<bool> {
    crate::unported!("bot.memory.already:_gave_a_step")
}

/// Python: bot/memory/already.py:684 `_the_opening_of(answer)` (never called on the trace corpus)
pub fn _the_opening_of(answer: &str) -> Value {
    crate::unported!("bot.memory.already:_the_opening_of")
}

/// Python: bot/memory/already.py:699 `_says_the_step(step, answer)` (never called on the trace corpus)
pub fn _says_the_step(step: &Value, answer: &str) -> Value {
    crate::unported!("bot.memory.already:_says_the_step")
}

/// Python: bot/memory/already.py:718 `takes_as_it_comes(text, memory)` (never called on the trace corpus)
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.already:takes_as_it_comes")
}

/// Python: bot/memory/already.py:783 `_the_steps_after(step, answer, line='')` (never called on the trace corpus)
pub fn _the_steps_after(step: &Value, answer: &str, line: &str) -> Value {
    crate::unported!("bot.memory.already:_the_steps_after")
}

/// Python: bot/memory/already.py:843 `_the_instruction_of(step)` (never called on the trace corpus)
pub fn _the_instruction_of(step: &Value) -> Value {
    crate::unported!("bot.memory.already:_the_instruction_of")
}

/// Python: bot/memory/already.py:858 `_with_steps(after, memory)` (never called on the trace corpus)
pub fn _with_steps(after: &str, memory: &Memory) -> String {
    crate::unported!("bot.memory.already:_with_steps")
}

/// Python: bot/memory/already.py:871 `_the_last_answer(memory)` (never called on the trace corpus)
pub fn _the_last_answer(memory: &Memory) -> Value {
    crate::unported!("bot.memory.already:_the_last_answer")
}

/// Python: bot/memory/already.py:887 `tells_them_the_step_again(reply, text, memory)`
pub fn tells_them_the_step_again(reply: &str, text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.already:tells_them_the_step_again")
}

/// Python: bot/memory/already.py:918 `_of_a_thing(step, text)` (never called on the trace corpus)
pub fn _of_a_thing(step: &Value, text: &str) -> bool {
    crate::unported!("bot.memory.already:_of_a_thing")
}

/// Python: bot/memory/already.py:943 `_only_finished(text)` (never called on the trace corpus)
pub fn _only_finished(text: &str) -> bool {
    crate::unported!("bot.memory.already:_only_finished")
}

/// Python: bot/memory/already.py:951 `_the_part_after(answer, step_text)` (never called on the trace corpus)
pub fn _the_part_after(answer: &str, step_text: &str) -> Value {
    crate::unported!("bot.memory.already:_the_part_after")
}

/// Python: bot/memory/already.py:992 `after_the_step_handed_back(text, memory)`
pub fn after_the_step_handed_back(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.already:after_the_step_handed_back")
}

/// Python: bot/memory/already.py:1024 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.already:reply_for")
}

