//! Port of `bot/modules/saving_followup.py`.
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

/// Python: bot/modules/saving_followup.py:57 `_lower_first(answer)` (never called on the trace corpus)
pub fn _lower_first(answer: &str) -> Value {
    crate::unported!("bot.modules.saving_followup:_lower_first")
}

/// Python: bot/modules/saving_followup.py:65 `_the_money_answer(question, memory)` (never called on the trace corpus)
pub fn _the_money_answer(question: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.saving_followup:_the_money_answer")
}

/// Python: bot/modules/saving_followup.py:94 `_the_reply(text, memory)`
pub fn _the_reply(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.saving_followup:_the_reply")
}

/// Python: bot/modules/saving_followup.py:114 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.saving_followup:respond")
}

/// Python: bot/modules/saving_followup.py:121 `TAKES_AS_IT_COMES(text, memory)` (never called on the trace corpus)
pub fn TAKES_AS_IT_COMES(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.saving_followup:TAKES_AS_IT_COMES")
}

/// Python: bot/modules/saving_followup.py:134 `_read(text, memory)`
pub fn _read(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.saving_followup:_read")
}

