//! Port of `bot/modules/on_record.py`.
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

/// Python: bot/modules/on_record.py:47 `gives(reply, want, line=None)`
pub fn gives(reply: &str, want: &str, line: &str) -> Option<bool> {
    crate::unported!("bot.modules.on_record:gives")
}

