//! Port of `bot/modules/how_long_a_stay.py`.
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

/// Python: bot/modules/how_long_a_stay.py:123 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.how_long_a_stay:_words")
}

/// Python: bot/modules/how_long_a_stay.py:127 `_the_stay(memory)` (never called on the trace corpus)
pub fn _the_stay(memory: &Memory) -> Value {
    crate::unported!("bot.modules.how_long_a_stay:_the_stay")
}

/// Python: bot/modules/how_long_a_stay.py:138 `_the_length_said(memory)` (never called on the trace corpus)
pub fn _the_length_said(memory: &Memory) -> String {
    crate::unported!("bot.modules.how_long_a_stay:_the_length_said")
}

/// Python: bot/modules/how_long_a_stay.py:148 `_the_opening(reply)` (never called on the trace corpus)
pub fn _the_opening(reply: &str) -> Value {
    crate::unported!("bot.modules.how_long_a_stay:_the_opening")
}

/// Python: bot/modules/how_long_a_stay.py:155 `_the_last_answer(memory)` (never called on the trace corpus)
pub fn _the_last_answer(memory: &Memory) -> Value {
    crate::unported!("bot.modules.how_long_a_stay:_the_last_answer")
}

/// Python: bot/modules/how_long_a_stay.py:166 `_a_subject_of_its_own(text)`
pub fn _a_subject_of_its_own(text: &str) -> bool {
    crate::unported!("bot.modules.how_long_a_stay:_a_subject_of_its_own")
}

/// Python: bot/modules/how_long_a_stay.py:177 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.how_long_a_stay:respond")
}

