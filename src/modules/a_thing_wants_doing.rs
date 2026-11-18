//! Port of `bot/modules/a_thing_wants_doing.py`.
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

/// Python: bot/modules/a_thing_wants_doing.py:31 `_the_thing_and_its_state(text, memory)`
pub fn _the_thing_and_its_state(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.a_thing_wants_doing:_the_thing_and_its_state")
}

/// Python: bot/modules/a_thing_wants_doing.py:38 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.a_thing_wants_doing:respond")
}

