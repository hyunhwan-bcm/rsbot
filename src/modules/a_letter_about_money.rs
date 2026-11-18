//! Port of `bot/modules/a_letter_about_money.py`.
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

/// Python: bot/modules/a_letter_about_money.py:121 `_ask_for_what_is_missing(line, clean, thing)` (never called on the trace corpus)
pub fn _ask_for_what_is_missing(line: &str, clean: &str, thing: &str) -> Value {
    crate::unported!("bot.modules.a_letter_about_money:_ask_for_what_is_missing")
}

/// Python: bot/modules/a_letter_about_money.py:157 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.a_letter_about_money:respond")
}

/// Python: bot/modules/a_letter_about_money.py:187 `_pick(lines, memory)` (never called on the trace corpus)
pub fn _pick(lines: &[String], memory: &Memory) -> Value {
    crate::unported!("bot.modules.a_letter_about_money:_pick")
}

