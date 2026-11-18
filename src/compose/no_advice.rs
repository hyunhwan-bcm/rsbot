//! Port of `bot/compose/no_advice.py`.
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

/// Python: bot/compose/no_advice.py:137 `_what_is_left_after_the_decline(line)`
pub fn _what_is_left_after_the_decline(line: &str) -> String {
    crate::unported!("bot.compose.no_advice:_what_is_left_after_the_decline")
}

/// Python: bot/compose/no_advice.py:145 `asks_something_of_its_own(line)`
pub fn asks_something_of_its_own(line: &str) -> bool {
    crate::unported!("bot.compose.no_advice:asks_something_of_its_own")
}

/// Python: bot/compose/no_advice.py:164 `_plain(text)`
pub fn _plain(text: &str) -> String {
    crate::unported!("bot.compose.no_advice:_plain")
}

/// Python: bot/compose/no_advice.py:168 `says_a_death(text)`
pub fn says_a_death(text: &str) -> bool {
    crate::unported!("bot.compose.no_advice:says_a_death")
}

/// Python: bot/compose/no_advice.py:173 `asks_for_a_view(text)`
pub fn asks_for_a_view(text: &str) -> bool {
    crate::unported!("bot.compose.no_advice:asks_for_a_view")
}

/// Python: bot/compose/no_advice.py:178 `says_no_advice(text)`
pub fn says_no_advice(text: &str) -> bool {
    crate::unported!("bot.compose.no_advice:says_no_advice")
}

/// Python: bot/compose/no_advice.py:209 `pool_name(text)`
pub fn pool_name(text: &str) {
    crate::unported!("bot.compose.no_advice:pool_name")
}

/// Python: bot/compose/no_advice.py:240 `lines_for(pool)`
pub fn lines_for(pool: &str) -> Vec<String> {
    crate::unported!("bot.compose.no_advice:lines_for")
}

/// Python: bot/compose/no_advice.py:245 `reply_for(pool, text, memory=None)`
pub fn reply_for(pool: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.no_advice:reply_for")
}

