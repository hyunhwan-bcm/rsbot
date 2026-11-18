//! Port of `bot/compose/fallback_pools.py`.
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

/// Python: bot/compose/fallback_pools.py:64 `_the_repeat_answer(memory)`
pub fn _the_repeat_answer(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback_pools:_the_repeat_answer")
}

/// Python: bot/compose/fallback_pools.py:83 `_drafting_pool(clean)` (never called on the trace corpus)
pub fn _drafting_pool(clean: &str) -> Value {
    crate::unported!("bot.compose.fallback_pools:_drafting_pool")
}

/// Python: bot/compose/fallback_pools.py:96 `_a_care_thread(memory, look_back=3)`
pub fn _a_care_thread(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback_pools:_a_care_thread")
}

/// Python: bot/compose/fallback_pools.py:113 `_a_money_thread(memory, look_back=3)`
pub fn _a_money_thread(memory: &Memory, look_back: i64) -> bool {
    crate::unported!("bot.compose.fallback_pools:_a_money_thread")
}

/// Python: bot/compose/fallback_pools.py:122 `_the_rule_for_it_for_the_turn(clean, repaired, memory)`
pub fn _the_rule_for_it_for_the_turn(clean: &str, repaired: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback_pools:_the_rule_for_it_for_the_turn")
}

/// Python: bot/compose/fallback_pools.py:173 `_a_missed_step(clean, repaired, memory, words)`
pub fn _a_missed_step(clean: &str, repaired: &str, memory: &Memory, words: &[String]) {
    crate::unported!("bot.compose.fallback_pools:_a_missed_step")
}

/// Python: bot/compose/fallback_pools.py:192 `_unknown(understood, memory, clean)`
pub fn _unknown(understood: &Understood, memory: &Memory, clean: &str) -> String {
    crate::unported!("bot.compose.fallback_pools:_unknown")
}

/// Python: bot/compose/fallback_pools.py:686 `_a_countdown_in_a_care_thread_(memory, clean, repaired, understood)`
pub fn _a_countdown_in_a_care_thread_(memory: &Memory, clean: &str, repaired: &str, understood: &Understood) {
    crate::unported!("bot.compose.fallback_pools:_a_countdown_in_a_care_thread_")
}

/// Python: bot/compose/fallback_pools.py:692 `_the_view_asked_for(clean, memory)`
pub fn _the_view_asked_for(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.fallback_pools:_the_view_asked_for")
}

