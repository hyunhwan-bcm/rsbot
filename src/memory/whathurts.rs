//! Port of `bot/memory/whathurts.py`.
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

/// Python: bot/memory/whathurts.py:37 `_vary(options, memory)` (never called on the trace corpus)
pub fn _vary(options: &Value, memory: &Memory) -> Value {
    crate::unported!("bot.memory.whathurts:_vary")
}

/// Python: bot/memory/whathurts.py:46 `which_one_is_a_side(low)`
pub fn which_one_is_a_side(low: &str) -> bool {
    crate::unported!("bot.memory.whathurts:which_one_is_a_side")
}

/// Python: bot/memory/whathurts.py:55 `before(memory)`
pub fn before(memory: &Memory) -> String {
    crate::unported!("bot.memory.whathurts:before")
}

/// Python: bot/memory/whathurts.py:61 `after(memory, held_before)`
pub fn after(memory: &Memory, held_before: &str) {
    crate::unported!("bot.memory.whathurts:after")
}

/// Python: bot/memory/whathurts.py:77 `_a_question_about_a_part(line)` (never called on the trace corpus)
pub fn _a_question_about_a_part(line: &str) -> bool {
    crate::unported!("bot.memory.whathurts:_a_question_about_a_part")
}

/// Python: bot/memory/whathurts.py:81 `what_is_hurting_reply(memory, low)` (never called on the trace corpus)
pub fn what_is_hurting_reply(memory: &Memory, low: &str) -> Value {
    crate::unported!("bot.memory.whathurts:what_is_hurting_reply")
}

/// Python: bot/memory/whathurts.py:124 `the_part_asked(memory, low)`
pub fn the_part_asked(memory: &Memory, low: &str) {
    crate::unported!("bot.memory.whathurts:the_part_asked")
}

