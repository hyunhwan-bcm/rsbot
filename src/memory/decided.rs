//! Port of `bot/memory/decided.py`.
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

/// Python: bot/memory/decided.py:178 `_the_thing(phrase)`
pub fn _the_thing(phrase: &str) -> Option<String> {
    crate::unported!("bot.memory.decided:_the_thing")
}

/// Python: bot/memory/decided.py:224 `_two_things(said)` (never called on the trace corpus)
pub fn _two_things(said: &str) -> Option<String> {
    crate::unported!("bot.memory.decided:_two_things")
}

/// Python: bot/memory/decided.py:255 `_options_since(memory, line)` (never called on the trace corpus)
pub fn _options_since(memory: &Memory, line: &str) -> Value {
    crate::unported!("bot.memory.decided:_options_since")
}

/// Python: bot/memory/decided.py:281 `_the_pair_left_open(memory)` (never called on the trace corpus)
pub fn _the_pair_left_open(memory: &Memory) -> Value {
    crate::unported!("bot.memory.decided:_the_pair_left_open")
}

/// Python: bot/memory/decided.py:293 `_as_typed(text, thing)`
pub fn _as_typed(text: &str, thing: &str) -> String {
    crate::unported!("bot.memory.decided:_as_typed")
}

/// Python: bot/memory/decided.py:303 `read(text)`
pub fn read(text: &str) {
    crate::unported!("bot.memory.decided:read")
}

/// Python: bot/memory/decided.py:350 `extract(text)` (never called on the trace corpus)
pub fn extract(text: &str) -> Value {
    crate::unported!("bot.memory.decided:extract")
}

/// Python: bot/memory/decided.py:361 `_an_activity(thing)` (never called on the trace corpus)
pub fn _an_activity(thing: &str) -> bool {
    crate::unported!("bot.memory.decided:_an_activity")
}

/// Python: bot/memory/decided.py:369 `_a_bare_thing(said)`
pub fn _a_bare_thing(said: &str) -> bool {
    crate::unported!("bot.memory.decided:_a_bare_thing")
}

/// Python: bot/memory/decided.py:380 `_options_on_the_table(memory)`
pub fn _options_on_the_table(memory: &Memory) -> bool {
    crate::unported!("bot.memory.decided:_options_on_the_table")
}

/// Python: bot/memory/decided.py:423 `_a_choice_reached_for(memory, kept_line=None)` (never called on the trace corpus)
pub fn _a_choice_reached_for(memory: &Memory, kept_line: &str) -> bool {
    crate::unported!("bot.memory.decided:_a_choice_reached_for")
}

/// Python: bot/memory/decided.py:445 `_a_plan_since(memory)` (never called on the trace corpus)
pub fn _a_plan_since(memory: &Memory) -> bool {
    crate::unported!("bot.memory.decided:_a_plan_since")
}

/// Python: bot/memory/decided.py:465 `observe(text, memory)`
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.decided:observe")
}

/// Python: bot/memory/decided.py:490 `_the_plan_on_record(memory)` (never called on the trace corpus)
pub fn _the_plan_on_record(memory: &Memory) -> Value {
    crate::unported!("bot.memory.decided:_the_plan_on_record")
}

/// Python: bot/memory/decided.py:512 `said_back(memory, words)` (never called on the trace corpus)
pub fn said_back(memory: &Memory, words: &[String]) -> bool {
    crate::unported!("bot.memory.decided:said_back")
}

/// Python: bot/memory/decided.py:521 `the_decision(memory)` (never called on the trace corpus)
pub fn the_decision(memory: &Memory) -> Value {
    crate::unported!("bot.memory.decided:the_decision")
}

/// Python: bot/memory/decided.py:529 `asks(text)`
pub fn asks(text: &str) -> bool {
    crate::unported!("bot.memory.decided:asks")
}

/// Python: bot/memory/decided.py:535 `the_guess(text)` (never called on the trace corpus)
pub fn the_guess(text: &str) -> Value {
    crate::unported!("bot.memory.decided:the_guess")
}

/// Python: bot/memory/decided.py:544 `_the_same_thing(said, held)` (never called on the trace corpus)
pub fn _the_same_thing(said: &str, held: &Value) -> bool {
    crate::unported!("bot.memory.decided:_the_same_thing")
}

/// Python: bot/memory/decided.py:564 `_said_to_you(thing)` (never called on the trace corpus)
pub fn _said_to_you(thing: &str) -> String {
    crate::unported!("bot.memory.decided:_said_to_you")
}

/// Python: bot/memory/decided.py:574 `_a_verb_phrase(thing)` (never called on the trace corpus)
pub fn _a_verb_phrase(thing: &str) -> bool {
    crate::unported!("bot.memory.decided:_a_verb_phrase")
}

/// Python: bot/memory/decided.py:580 `_the_choice_said(thing)` (never called on the trace corpus)
pub fn _the_choice_said(thing: &str) -> String {
    crate::unported!("bot.memory.decided:_the_choice_said")
}

/// Python: bot/memory/decided.py:589 `_at_the_start(thing)` (never called on the trace corpus)
pub fn _at_the_start(thing: &str) -> Value {
    crate::unported!("bot.memory.decided:_at_the_start")
}

/// Python: bot/memory/decided.py:635 `_asks_about_another_thing(text, held)` (never called on the trace corpus)
pub fn _asks_about_another_thing(text: &str, held: &Value) -> bool {
    crate::unported!("bot.memory.decided:_asks_about_another_thing")
}

/// Python: bot/memory/decided.py:657 `answer(text, memory)`
pub fn answer(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.decided:answer")
}

