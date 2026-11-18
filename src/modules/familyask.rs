//! Port of `bot/modules/familyask.py`.
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

/// Python: bot/modules/familyask.py:191 `the_relation(text)` (never called on the trace corpus)
pub fn the_relation(text: &str) -> Value {
    crate::unported!("bot.modules.familyask:the_relation")
}

/// Python: bot/modules/familyask.py:212 `the_class(text)` (never called on the trace corpus)
pub fn the_class(text: &str) -> Value {
    crate::unported!("bot.modules.familyask:the_class")
}

/// Python: bot/modules/familyask.py:237 `is_a_family_ask(text)`
pub fn is_a_family_ask(text: &str) -> bool {
    crate::unported!("bot.modules.familyask:is_a_family_ask")
}

/// Python: bot/modules/familyask.py:253 `reply_for(text, memory=None)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.familyask:reply_for")
}

/// Python: bot/modules/familyask.py:268 `TAKES_AS_IT_COMES(text, memory)` (never called on the trace corpus)
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.familyask:TAKES_AS_IT_COMES")
}

/// Python: bot/modules/familyask.py:279 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.familyask:respond")
}

