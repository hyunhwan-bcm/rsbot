//! Port of `bot/modules/safety_phrases.py`.
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

/// Python: bot/modules/safety_phrases.py:844 `_both_here()` (never called on the trace corpus)
pub fn _both_here(groups: &[Value]) -> String {
    crate::unported!("bot.modules.safety_phrases:_both_here")
}

