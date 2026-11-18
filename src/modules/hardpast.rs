//! Port of `bot/modules/hardpast.py`.
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

/// Python: bot/modules/hardpast.py:148 `_hangs_on_the_person(clean, at)` (never called on the trace corpus)
pub fn _hangs_on_the_person(clean: &str, at: i64) -> bool {
    crate::unported!("bot.modules.hardpast:_hangs_on_the_person")
}

/// Python: bot/modules/hardpast.py:161 `_the_hard_thing(text, memory=None)`
pub fn _the_hard_thing(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.hardpast:_the_hard_thing")
}

/// Python: bot/modules/hardpast.py:186 `takes_as_it_comes(text, memory)` (never called on the trace corpus)
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.hardpast:takes_as_it_comes")
}

/// Python: bot/modules/hardpast.py:194 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.hardpast:respond")
}

