//! Port of `bot/modules/what_they_said.py`.
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

/// Python: bot/modules/what_they_said.py:125 `_details(text)`
pub fn _details(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.what_they_said:_details")
}

/// Python: bot/modules/what_they_said.py:131 `_the_kind_of_answer(said)`
pub fn _the_kind_of_answer(said: &str) -> String {
    crate::unported!("bot.modules.what_they_said:_the_kind_of_answer")
}

/// Python: bot/modules/what_they_said.py:140 `_the_last_somebody_spoke(said, memory)`
pub fn _the_last_somebody_spoke(said: &str, memory: &Memory) {
    crate::unported!("bot.modules.what_they_said:_the_last_somebody_spoke")
}

/// Python: bot/modules/what_they_said.py:157 `_the_reported_clause(line)` (never called on the trace corpus)
pub fn _the_reported_clause(line: &str) -> Value {
    crate::unported!("bot.modules.what_they_said:_the_reported_clause")
}

/// Python: bot/modules/what_they_said.py:181 `_the_figure_the_line_gave(line)` (never called on the trace corpus)
pub fn _the_figure_the_line_gave(line: &str) -> String {
    crate::unported!("bot.modules.what_they_said:_the_figure_the_line_gave")
}

/// Python: bot/modules/what_they_said.py:199 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.what_they_said:respond")
}

