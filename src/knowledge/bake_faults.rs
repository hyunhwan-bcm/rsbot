//! Port of `bot/knowledge/bake_faults.py`.
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

/// Python: bot/knowledge/bake_faults.py:13 `_same(word, wanted)` (never called on the trace corpus)
pub fn _same(word: &str, wanted: &Value) -> bool {
    crate::unported!("bot.knowledge.bake_faults:_same")
}

/// Python: bot/knowledge/bake_faults.py:39 `_bake_fault(memory, understood)` (never called on the trace corpus)
pub fn _bake_fault(memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.bake_faults:_bake_fault")
}

/// Python: bot/knowledge/bake_faults.py:213 `_bake_always(memory, understood)` (never called on the trace corpus)
pub fn _bake_always(memory: &Memory, understood: &Understood) -> String {
    crate::unported!("bot.knowledge.bake_faults:_bake_always")
}

