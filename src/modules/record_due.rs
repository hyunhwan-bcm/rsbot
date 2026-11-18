//! Port of `bot/modules/record_due.py`.
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

/// Python: bot/modules/record_due.py:63 `_the_record(clean)`
pub fn _the_record(clean: &str) {
    crate::unported!("bot.modules.record_due:_the_record")
}

/// Python: bot/modules/record_due.py:130 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.record_due:respond")
}

/// Python: bot/modules/record_due.py:162 `_pick(lines, memory)` (never called on the trace corpus)
pub fn _pick(lines: &[String], memory: &Memory) -> Value {
    crate::unported!("bot.modules.record_due:_pick")
}

