//! Port of `bot/compose/thread_detail.py`.
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

/// Python: bot/compose/thread_detail.py:124 `_clean_line(text)`
pub fn _clean_line(text: &str) -> String {
    crate::unported!("bot.compose.thread_detail:_clean_line")
}

/// Python: bot/compose/thread_detail.py:137 `the_thread_is_serious(turns, line='')`
pub fn the_thread_is_serious(turns: &Value, line: &str) -> bool {
    crate::unported!("bot.compose.thread_detail:the_thread_is_serious")
}

/// Python: bot/compose/thread_detail.py:143 `fallback_heavy(turns)`
pub fn fallback_heavy(turns: &Value) -> bool {
    crate::unported!("bot.compose.thread_detail:fallback_heavy")
}

/// Python: bot/compose/thread_detail.py:152 `the_clause_said_back(text)`
pub fn the_clause_said_back(text: &str) -> String {
    crate::unported!("bot.compose.thread_detail:the_clause_said_back")
}

/// Python: bot/compose/thread_detail.py:167 `the_kind_of_detail(clean)`
pub fn the_kind_of_detail(clean: &str) -> Option<String> {
    crate::unported!("bot.compose.thread_detail:the_kind_of_detail")
}

/// Python: bot/compose/thread_detail.py:178 `detail_pool(clean, memory)`
pub fn detail_pool(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.thread_detail:detail_pool")
}

/// Python: bot/compose/thread_detail.py:210 `_second_person(clause)`
pub fn _second_person(clause: &str) -> String {
    crate::unported!("bot.compose.thread_detail:_second_person")
}

/// Python: bot/compose/thread_detail.py:224 `says_nothing_about_the_line(reply, text)`
pub fn says_nothing_about_the_line(reply: &str, text: &str) -> bool {
    crate::unported!("bot.compose.thread_detail:says_nothing_about_the_line")
}

