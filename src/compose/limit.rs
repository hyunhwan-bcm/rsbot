//! Port of `bot/compose/limit.py`.
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

/// Python: bot/compose/limit.py:81 `restates_the_answer(bot_line)`
pub fn restates_the_answer(bot_line: &str) -> bool {
    crate::unported!("bot.compose.limit:restates_the_answer")
}

/// Python: bot/compose/limit.py:98 `forms_of(thing)` (never called on the trace corpus)
pub fn forms_of(thing: &str) -> Value {
    crate::unported!("bot.compose.limit:forms_of")
}

/// Python: bot/compose/limit.py:128 `_carries(part, forms)` (never called on the trace corpus)
pub fn _carries(part: &str, forms: &[String]) -> bool {
    crate::unported!("bot.compose.limit:_carries")
}

/// Python: bot/compose/limit.py:134 `_last_answer(memory)` (never called on the trace corpus)
pub fn _last_answer(memory: &Memory) -> Value {
    crate::unported!("bot.compose.limit:_last_answer")
}

/// Python: bot/compose/limit.py:150 `_pick(pool, memory)` (never called on the trace corpus)
pub fn _pick(pool: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.compose.limit:_pick")
}

/// Python: bot/compose/limit.py:161 `_take_out(said, forms)` (never called on the trace corpus)
pub fn _take_out(said: &str, forms: &[String]) -> Value {
    crate::unported!("bot.compose.limit:_take_out")
}

/// Python: bot/compose/limit.py:172 `steps_left(understood, memory)` (never called on the trace corpus)
pub fn steps_left(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.compose.limit:steps_left")
}

/// Python: bot/compose/limit.py:191 `_the_parts_left(said, forms)` (never called on the trace corpus)
pub fn _the_parts_left(said: &str, forms: &[String]) -> Value {
    crate::unported!("bot.compose.limit:_the_parts_left")
}

/// Python: bot/compose/limit.py:226 `_joined(kept)` (never called on the trace corpus)
pub fn _joined(kept: &Value) -> Value {
    crate::unported!("bot.compose.limit:_joined")
}

/// Python: bot/compose/limit.py:250 `_a_fact_put_right(memory)` (never called on the trace corpus)
pub fn _a_fact_put_right(memory: &Memory) -> bool {
    crate::unported!("bot.compose.limit:_a_fact_put_right")
}

/// Python: bot/compose/limit.py:291 `a_limit_on_the_answer(text)`
pub fn a_limit_on_the_answer(text: &str) -> bool {
    crate::unported!("bot.compose.limit:a_limit_on_the_answer")
}

/// Python: bot/compose/limit.py:316 `a_limit_the_reading_missed(clean, memory)`
pub fn a_limit_the_reading_missed(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.limit:a_limit_the_reading_missed")
}

/// Python: bot/compose/limit.py:330 `reply_for(understood, memory)` (never called on the trace corpus)
pub fn reply_for(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.compose.limit:reply_for")
}

