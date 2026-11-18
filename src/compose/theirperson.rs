//! Port of `bot/compose/theirperson.py`.
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

/// Python: bot/compose/theirperson.py:194 `_said(text)`
pub fn _said(text: &str) -> String {
    crate::unported!("bot.compose.theirperson:_said")
}

/// Python: bot/compose/theirperson.py:199 `_who(person, said)` (never called on the trace corpus)
pub fn _who(person: &Value, said: &str) -> String {
    crate::unported!("bot.compose.theirperson:_who")
}

/// Python: bot/compose/theirperson.py:206 `reply_for(text, memory=None)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.theirperson:reply_for")
}

/// Python: bot/compose/theirperson.py:266 `may_replace(reply, text='')` (never called on the trace corpus)
pub fn may_replace(reply: &str, text: &str) -> bool {
    crate::unported!("bot.compose.theirperson:may_replace")
}

