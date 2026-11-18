//! Port of `bot/compose/rule.py`.
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

/// Python: bot/compose/rule.py:122 `_said(text)`
pub fn _said(text: &str) -> String {
    crate::unported!("bot.compose.rule:_said")
}

/// Python: bot/compose/rule.py:127 `_a_court_of_law(said)`
pub fn _a_court_of_law(said: &str) -> bool {
    crate::unported!("bot.compose.rule:_a_court_of_law")
}

/// Python: bot/compose/rule.py:139 `_the_thread_said(memory, pattern, look_back=2)` (never called on the trace corpus)
pub fn _the_thread_said(memory: &Memory, pattern: &Value, look_back: &Value) -> bool {
    crate::unported!("bot.compose.rule:_the_thread_said")
}

/// Python: bot/compose/rule.py:145 `_a_court_of_law_in_the_thread(memory, look_back=2)`
pub fn _a_court_of_law_in_the_thread(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.rule:_a_court_of_law_in_the_thread")
}

/// Python: bot/compose/rule.py:177 `_said_a_local_rule_recently(memory, look_back=2)`
pub fn _said_a_local_rule_recently(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.rule:_said_a_local_rule_recently")
}

/// Python: bot/compose/rule.py:183 `_a_rule_ask_in_the_thread(memory, look_back=2)` (never called on the trace corpus)
pub fn _a_rule_ask_in_the_thread(memory: &Memory, look_back: &Value) -> bool {
    crate::unported!("bot.compose.rule:_a_rule_ask_in_the_thread")
}

/// Python: bot/compose/rule.py:201 `pool_for(text, memory=None)`
pub fn pool_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.rule:pool_for")
}

/// Python: bot/compose/rule.py:218 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.rule:answer_for")
}

