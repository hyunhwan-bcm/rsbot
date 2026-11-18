//! Port of `bot/modules/confusing.py`.
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

/// Python: bot/modules/confusing.py:60 `_said_back(thing)` (never called on the trace corpus)
pub fn _said_back(thing: &str) -> String {
    crate::unported!("bot.modules.confusing:_said_back")
}

/// Python: bot/modules/confusing.py:65 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.confusing:respond")
}

