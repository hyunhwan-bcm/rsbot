//! Port of `bot/modules/where_it_answers.py`.
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

/// Python: bot/modules/where_it_answers.py:89 `_content(text, skip=())` (never called on the trace corpus)
pub fn _content(text: &str, skip: &Value) -> Value {
    crate::unported!("bot.modules.where_it_answers:_content")
}

/// Python: bot/modules/where_it_answers.py:98 `_would_be_said(text, memory)`
pub fn _would_be_said(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.where_it_answers:_would_be_said")
}

/// Python: bot/modules/where_it_answers.py:117 `_answers_the_ask(route, clean, said)`
pub fn _answers_the_ask(route: &[String], clean: &str, said: &str) -> bool {
    crate::unported!("bot.modules.where_it_answers:_answers_the_ask")
}

/// Python: bot/modules/where_it_answers.py:141 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.where_it_answers:respond")
}

