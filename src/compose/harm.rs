//! Port of `bot/compose/harm.py`.
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

/// Python: bot/compose/harm.py:171 `_person_said(clean)` (never called on the trace corpus)
pub fn _person_said(clean: &str) -> Value {
    crate::unported!("bot.compose.harm:_person_said")
}

/// Python: bot/compose/harm.py:191 `_said_back(text)` (never called on the trace corpus)
pub fn _said_back(text: &str) -> Value {
    crate::unported!("bot.compose.harm:_said_back")
}

/// Python: bot/compose/harm.py:225 `reads_as_hurting_someone(clean)`
pub fn reads_as_hurting_someone(clean: &str) -> bool {
    crate::unported!("bot.compose.harm:reads_as_hurting_someone")
}

/// Python: bot/compose/harm.py:271 `reads_as_our_reply(reply)` (never called on the trace corpus)
pub fn reads_as_our_reply(reply: &str) -> bool {
    crate::unported!("bot.compose.harm:reads_as_our_reply")
}

/// Python: bot/compose/harm.py:288 `_child_at_the_hurt(flat)` (never called on the trace corpus)
pub fn _child_at_the_hurt(flat: &Value) -> String {
    crate::unported!("bot.compose.harm:_child_at_the_hurt")
}

/// Python: bot/compose/harm.py:313 `_stands_on_its_own(text, found)` (never called on the trace corpus)
pub fn _stands_on_its_own(text: &str, found: &Value) -> bool {
    crate::unported!("bot.compose.harm:_stands_on_its_own")
}

/// Python: bot/compose/harm.py:322 `reply_for(clean, memory=None)` (never called on the trace corpus)
pub fn reply_for(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.harm:reply_for")
}

