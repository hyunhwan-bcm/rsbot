//! Port of `bot/memory/cause.py`.
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

/// Python: bot/memory/cause.py:234 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.memory.cause:_flat")
}

/// Python: bot/memory/cause.py:238 `_turns(memory)`
pub fn _turns(memory: &Memory) -> Value {
    crate::unported!("bot.memory.cause:_turns")
}

/// Python: bot/memory/cause.py:242 `_the_state_they_reported(memory, back=4)`
pub fn _the_state_they_reported(memory: &Memory, back: i64) -> Value {
    crate::unported!("bot.memory.cause:_the_state_they_reported")
}

/// Python: bot/memory/cause.py:257 `_the_bot_just_spoke(memory)`
pub fn _the_bot_just_spoke(memory: &Memory) -> String {
    crate::unported!("bot.memory.cause:_the_bot_just_spoke")
}

/// Python: bot/memory/cause.py:266 `_already_has_it(said, thing)`
pub fn _already_has_it(said: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.cause:_already_has_it")
}

/// Python: bot/memory/cause.py:292 `_only_a_nod(text)`
pub fn _only_a_nod(text: &str) -> bool {
    crate::unported!("bot.memory.cause:_only_a_nod")
}

/// Python: bot/memory/cause.py:301 `_a_thing(text)`
pub fn _a_thing(text: &str) {
    crate::unported!("bot.memory.cause:_a_thing")
}

/// Python: bot/memory/cause.py:343 `expand(text, memory)`
pub fn expand(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.cause:expand")
}

/// Python: bot/memory/cause.py:368 `_the_bot_would_only_shrug(line, memory)` (never called on the trace corpus)
pub fn _the_bot_would_only_shrug(line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.cause:_the_bot_would_only_shrug")
}

/// Python: bot/memory/cause.py:396 `_asked(line, memory, names=('answers', 'chat'))` (never called on the trace corpus)
pub fn _asked(line: &str, memory: &Memory, names: &[String]) -> Option<String> {
    crate::unported!("bot.memory.cause:_asked")
}

/// Python: bot/memory/cause.py:452 `_a_stock_low_line(said)` (never called on the trace corpus)
pub fn _a_stock_low_line(said: &str) -> bool {
    crate::unported!("bot.memory.cause:_a_stock_low_line")
}

/// Python: bot/memory/cause.py:466 `_says_the_reason(said, thing)` (never called on the trace corpus)
pub fn _says_the_reason(said: &str, thing: &str) -> bool {
    crate::unported!("bot.memory.cause:_says_the_reason")
}

/// Python: bot/memory/cause.py:481 `_names_nothing(reason)` (never called on the trace corpus)
pub fn _names_nothing(reason: &Value) -> bool {
    crate::unported!("bot.memory.cause:_names_nothing")
}

/// Python: bot/memory/cause.py:491 `_the_persons_own_reason(reason)` (never called on the trace corpus)
pub fn _the_persons_own_reason(reason: &Value) -> Value {
    crate::unported!("bot.memory.cause:_the_persons_own_reason")
}

/// Python: bot/memory/cause.py:499 `_asks_for_the_rest(text)` (never called on the trace corpus)
pub fn _asks_for_the_rest(text: &str) -> Option<bool> {
    crate::unported!("bot.memory.cause:_asks_for_the_rest")
}

/// Python: bot/memory/cause.py:508 `_a_reason_for_how_they_are(thing, memory)` (never called on the trace corpus)
pub fn _a_reason_for_how_they_are(thing: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.cause:_a_reason_for_how_they_are")
}

/// Python: bot/memory/cause.py:543 `_without_what_was_just_said(said, last)` (never called on the trace corpus)
pub fn _without_what_was_just_said(said: &str, last: &str) -> String {
    crate::unported!("bot.memory.cause:_without_what_was_just_said")
}

/// Python: bot/memory/cause.py:553 `_a_new_answer(said, memory)` (never called on the trace corpus)
pub fn _a_new_answer(said: &str, memory: &Memory) -> Value {
    crate::unported!("bot.memory.cause:_a_new_answer")
}

/// Python: bot/memory/cause.py:581 `_the_feeling_without_a_reason(memory)` (never called on the trace corpus)
pub fn _the_feeling_without_a_reason(memory: &Memory) -> Value {
    crate::unported!("bot.memory.cause:_the_feeling_without_a_reason")
}

/// Python: bot/memory/cause.py:608 `_the_reason_nobody_has(memory)` (never called on the trace corpus)
pub fn _the_reason_nobody_has(memory: &Memory) -> Value {
    crate::unported!("bot.memory.cause:_the_reason_nobody_has")
}

/// Python: bot/memory/cause.py:628 `_the_knowledge_base_answers_it(thing, memory)` (never called on the trace corpus)
pub fn _the_knowledge_base_answers_it(thing: &str, memory: &Memory) -> bool {
    crate::unported!("bot.memory.cause:_the_knowledge_base_answers_it")
}

/// Python: bot/memory/cause.py:649 `_asks_about_the_bot(text, memory)`
pub fn _asks_about_the_bot(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.memory.cause:_asks_about_the_bot")
}

/// Python: bot/memory/cause.py:675 `answer_for(text, memory)`
pub fn answer_for(text: &str, memory: &mut Memory) {
    crate::unported!("bot.memory.cause:answer_for")
}

