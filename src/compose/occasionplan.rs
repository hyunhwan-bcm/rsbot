//! Port of `bot/compose/occasionplan.py`.
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

/// Python: bot/compose/occasionplan.py:110 `_ordinal(age)` (never called on the trace corpus)
pub fn _ordinal(age: &Value) -> String {
    crate::unported!("bot.compose.occasionplan:_ordinal")
}

/// Python: bot/compose/occasionplan.py:119 `_whose(who)` (never called on the trace corpus)
pub fn _whose(who: &str) -> String {
    crate::unported!("bot.compose.occasionplan:_whose")
}

/// Python: bot/compose/occasionplan.py:152 `_the_day(when)`
pub fn _the_day(when: Option<&str>) -> String {
    crate::unported!("bot.compose.occasionplan:_the_day")
}

/// Python: bot/compose/occasionplan.py:173 `_the_age(text)`
pub fn _the_age(text: &str) {
    crate::unported!("bot.compose.occasionplan:_the_age")
}

/// Python: bot/compose/occasionplan.py:197 `_said_how_long_ago(memory, who=None, markers='')`
pub fn _said_how_long_ago(memory: &Memory, who: &str, markers: &Value) -> i64 {
    crate::unported!("bot.compose.occasionplan:_said_how_long_ago")
}

/// Python: bot/compose/occasionplan.py:267 `_the_person(said)`
pub fn _the_person(said: &str) {
    crate::unported!("bot.compose.occasionplan:_the_person")
}

/// Python: bot/compose/occasionplan.py:294 `_the_recent_age(memory, upto=6)`
pub fn _the_recent_age(memory: &Memory, upto: i64) {
    crate::unported!("bot.compose.occasionplan:_the_recent_age")
}

/// Python: bot/compose/occasionplan.py:320 `_the_occasion(memory)`
pub fn _the_occasion(memory: &Memory) {
    crate::unported!("bot.compose.occasionplan:_the_occasion")
}

/// Python: bot/compose/occasionplan.py:378 `_cap(match)` (never called on the trace corpus)
pub fn _cap(r#match: &Value) -> String {
    crate::unported!("bot.compose.occasionplan:_cap")
}

/// Python: bot/compose/occasionplan.py:397 `_reads_as_a_plan_ask(said, memory)`
pub fn _reads_as_a_plan_ask(said: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.occasionplan:_reads_as_a_plan_ask")
}

/// Python: bot/compose/occasionplan.py:423 `takes_as_it_comes(text, memory)`
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.occasionplan:takes_as_it_comes")
}

/// Python: bot/compose/occasionplan.py:434 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.occasionplan:reply_for")
}

