//! Port of `bot/modules/callback_about.py`.
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

/// Python: bot/modules/callback_about.py:33 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.callback_about:respond")
}

/// Python: bot/modules/callback_about.py:37 `CLAIMS_INSTEAD(text, memory)` (never called on the trace corpus)
pub fn CLAIMS_INSTEAD(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.callback_about:CLAIMS_INSTEAD")
}

