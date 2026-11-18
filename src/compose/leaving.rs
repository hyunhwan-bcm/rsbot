//! Port of `bot/compose/leaving.py`.
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

/// Python: bot/compose/leaving.py:338 `_clauses(clean)`
pub fn _clauses(clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.leaving:_clauses")
}

/// Python: bot/compose/leaving.py:375 `_is_sign_off(clause)`
pub fn _is_sign_off(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_is_sign_off")
}

/// Python: bot/compose/leaving.py:399 `_ends_in_a_sign_off(clause)`
pub fn _ends_in_a_sign_off(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_ends_in_a_sign_off")
}

/// Python: bot/compose/leaving.py:409 `_is_thanks_clause(clause)`
pub fn _is_thanks_clause(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_is_thanks_clause")
}

/// Python: bot/compose/leaving.py:442 `_reads_as_a_body_complaint(clause)` (never called on the trace corpus)
pub fn _reads_as_a_body_complaint(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_reads_as_a_body_complaint")
}

/// Python: bot/compose/leaving.py:447 `_reads_as_unwell(clause)` (never called on the trace corpus)
pub fn _reads_as_unwell(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_reads_as_unwell")
}

/// Python: bot/compose/leaving.py:493 `_a_trouble_behind_a_contrast(clean)`
pub fn _a_trouble_behind_a_contrast(clean: &str) -> bool {
    crate::unported!("bot.compose.leaving:_a_trouble_behind_a_contrast")
}

/// Python: bot/compose/leaving.py:515 `_names_their_own_trouble(text)`
pub fn _names_their_own_trouble(text: &str) -> bool {
    crate::unported!("bot.compose.leaving:_names_their_own_trouble")
}

/// Python: bot/compose/leaving.py:542 `reads_as_a_sign_off(reply)` (never called on the trace corpus)
pub fn reads_as_a_sign_off(reply: &str) -> bool {
    crate::unported!("bot.compose.leaving:reads_as_a_sign_off")
}

/// Python: bot/compose/leaving.py:552 `_is_wish_clause(clause)`
pub fn _is_wish_clause(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_is_wish_clause")
}

/// Python: bot/compose/leaving.py:572 `_is_leave_clause(clause)`
pub fn _is_leave_clause(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_is_leave_clause")
}

/// Python: bot/compose/leaving.py:606 `reads_as_leaving(clean, words=None)`
pub fn reads_as_leaving(clean: &str, words: Option<&[String]>) -> bool {
    crate::unported!("bot.compose.leaving:reads_as_leaving")
}

/// Python: bot/compose/leaving.py:617 `reads_as_a_leave_with_a_body_in_it(clean, words=None)`
pub fn reads_as_a_leave_with_a_body_in_it(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.leaving:reads_as_a_leave_with_a_body_in_it")
}

/// Python: bot/compose/leaving.py:629 `reads_as_a_leave_with_a_trouble_in_it(clean, words=None)`
pub fn reads_as_a_leave_with_a_trouble_in_it(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.leaving:reads_as_a_leave_with_a_trouble_in_it")
}

/// Python: bot/compose/leaving.py:643 `_reads_as_leaving(clean, words=None, unwell_blocks=True, trouble_blocks=True)`
pub fn _reads_as_leaving(clean: &str, words: Option<&[String]>, unwell_blocks: bool, trouble_blocks: bool) -> bool {
    crate::unported!("bot.compose.leaving:_reads_as_leaving")
}

/// Python: bot/compose/leaving.py:719 `_asks_something(clause)`
pub fn _asks_something(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_asks_something")
}

/// Python: bot/compose/leaving.py:734 `_is_filler(clause)` (never called on the trace corpus)
pub fn _is_filler(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_is_filler")
}

/// Python: bot/compose/leaving.py:814 `_thread_is_an_emergency(memory)` (never called on the trace corpus)
pub fn _thread_is_an_emergency(memory: &Memory) -> bool {
    crate::unported!("bot.compose.leaving:_thread_is_an_emergency")
}

/// Python: bot/compose/leaving.py:838 `_thread_is_unwell(memory)` (never called on the trace corpus)
pub fn _thread_is_unwell(memory: &Memory) -> bool {
    crate::unported!("bot.compose.leaving:_thread_is_unwell")
}

/// Python: bot/compose/leaving.py:853 `_wish_pool(clean, memory=None)` (never called on the trace corpus)
pub fn _wish_pool(clean: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.leaving:_wish_pool")
}

/// Python: bot/compose/leaving.py:867 `pool(clean, words=None, unwell_blocks=True, memory=None, trouble_blocks=True)`
pub fn pool(clean: &str, words: &[String], unwell_blocks: bool, memory: &Memory, trouble_blocks: bool) {
    crate::unported!("bot.compose.leaving:pool")
}

/// Python: bot/compose/leaving.py:960 `_a_state_not_an_errand(clauses)` (never called on the trace corpus)
pub fn _a_state_not_an_errand(clauses: &Value) -> bool {
    crate::unported!("bot.compose.leaving:_a_state_not_an_errand")
}

/// Python: bot/compose/leaving.py:980 `_is_a_bedtime(clause)` (never called on the trace corpus)
pub fn _is_a_bedtime(clause: &str) -> bool {
    crate::unported!("bot.compose.leaving:_is_a_bedtime")
}

/// Python: bot/compose/leaving.py:987 `lines()` (never called on the trace corpus)
pub fn lines() -> Value {
    crate::unported!("bot.compose.leaving:lines")
}

