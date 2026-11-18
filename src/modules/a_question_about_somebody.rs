//! Port of `bot/modules/a_question_about_somebody.py`.
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

/// Python: bot/modules/a_question_about_somebody.py:82 `_there_is_a_chat(memory)`
pub fn _there_is_a_chat(memory: &Memory) -> bool {
    crate::unported!("bot.modules.a_question_about_somebody:_there_is_a_chat")
}

/// Python: bot/modules/a_question_about_somebody.py:96 `_flat(text)`
pub fn _flat(text: &str) -> String {
    crate::unported!("bot.modules.a_question_about_somebody:_flat")
}

/// Python: bot/modules/a_question_about_somebody.py:101 `about_somebody_unnamed(text, memory)`
pub fn about_somebody_unnamed(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.a_question_about_somebody:about_somebody_unnamed")
}

/// Python: bot/modules/a_question_about_somebody.py:115 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.a_question_about_somebody:respond")
}

