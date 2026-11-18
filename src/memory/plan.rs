//! Port of `bot/memory/plan.py`.
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

/// Python: bot/memory/plan.py:145 `_the_thing_named(text)` (never called on the trace corpus)
pub fn _the_thing_named(text: &str) -> Value {
    crate::unported!("bot.memory.plan:_the_thing_named")
}

/// Python: bot/memory/plan.py:151 `_tried(step, memory)` (never called on the trace corpus)
pub fn _tried(step: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.plan:_tried")
}

/// Python: bot/memory/plan.py:173 `the_last_answer(memory)` (never called on the trace corpus)
pub fn the_last_answer(memory: &Memory) -> String {
    crate::unported!("bot.memory.plan:the_last_answer")
}

/// Python: bot/memory/plan.py:181 `the_answer_in_hand(memory)` (never called on the trace corpus)
pub fn the_answer_in_hand(memory: &Memory) -> Value {
    crate::unported!("bot.memory.plan:the_answer_in_hand")
}

/// Python: bot/memory/plan.py:206 `_module_of_last(memory)` (never called on the trace corpus)
pub fn _module_of_last(memory: &Memory) -> String {
    crate::unported!("bot.memory.plan:_module_of_last")
}

/// Python: bot/memory/plan.py:214 `_last_user(memory)` (never called on the trace corpus)
pub fn _last_user(memory: &Memory) -> String {
    crate::unported!("bot.memory.plan:_last_user")
}

/// Python: bot/memory/plan.py:222 `_a_cut_at_a_colon(step)` (never called on the trace corpus)
pub fn _a_cut_at_a_colon(step: &Value) -> Value {
    crate::unported!("bot.memory.plan:_a_cut_at_a_colon")
}

/// Python: bot/memory/plan.py:237 `steps_of(answer)` (never called on the trace corpus)
pub fn steps_of(answer: &str) -> Value {
    crate::unported!("bot.memory.plan:steps_of")
}

/// Python: bot/memory/plan.py:257 `_the_amounts_first(answer, steps)` (never called on the trace corpus)
pub fn _the_amounts_first(answer: &str, steps: &Value) -> Value {
    crate::unported!("bot.memory.plan:_the_amounts_first")
}

/// Python: bot/memory/plan.py:277 `_words(text)` (never called on the trace corpus)
pub fn _words(text: &str) -> Value {
    crate::unported!("bot.memory.plan:_words")
}

/// Python: bot/memory/plan.py:282 `_stems(text)` (never called on the trace corpus)
pub fn _stems(text: &str) -> Value {
    crate::unported!("bot.memory.plan:_stems")
}

/// Python: bot/memory/plan.py:293 `_the_same_job(answer, plan)` (never called on the trace corpus)
pub fn _the_same_job(answer: &str, plan: &str) -> bool {
    crate::unported!("bot.memory.plan:_the_same_job")
}

/// Python: bot/memory/plan.py:311 `_the_same_topic(memory, plan)` (never called on the trace corpus)
pub fn _the_same_topic(memory: &Memory, plan: &str) -> bool {
    crate::unported!("bot.memory.plan:_the_same_topic")
}

/// Python: bot/memory/plan.py:331 `_said_in(answer, plan)` (never called on the trace corpus)
pub fn _said_in(answer: &str, plan: &str) -> Value {
    crate::unported!("bot.memory.plan:_said_in")
}

/// Python: bot/memory/plan.py:343 `_our_own_line(answer, plan)` (never called on the trace corpus)
pub fn _our_own_line(answer: &str, plan: &str) -> bool {
    crate::unported!("bot.memory.plan:_our_own_line")
}

/// Python: bot/memory/plan.py:361 `_plan_for(memory)` (never called on the trace corpus)
pub fn _plan_for(memory: &Memory) -> Value {
    crate::unported!("bot.memory.plan:_plan_for")
}

/// Python: bot/memory/plan.py:400 `carry_on(text, memory)`
pub fn carry_on(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.plan:carry_on")
}

/// Python: bot/memory/plan.py:458 `_record(memory, answer, steps, at, said='')` (never called on the trace corpus)
pub fn _record(memory: &mut Memory, answer: &str, steps: &Value, at: i64, said: &str) {
    crate::unported!("bot.memory.plan:_record")
}

