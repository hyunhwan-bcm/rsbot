//! Port of `bot/compose/not_knowing.py`.
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

/// Python: bot/compose/not_knowing.py:27 `_choose(options, memory, clean)`
pub fn _choose(options: &[String], memory: &Memory, clean: &str) -> String {
    crate::unported!("bot.compose.not_knowing:_choose")
}

/// Python: bot/compose/not_knowing.py:398 `_the_words_of(said)`
pub fn _the_words_of(said: &str) -> Vec<String> {
    crate::unported!("bot.compose.not_knowing:_the_words_of")
}

/// Python: bot/compose/not_knowing.py:408 `_says_nothing_of_its_own(clean)`
pub fn _says_nothing_of_its_own(clean: &str) -> bool {
    crate::unported!("bot.compose.not_knowing:_says_nothing_of_its_own")
}

/// Python: bot/compose/not_knowing.py:484 `_an_ask_with_no_subject(clean, memory=None)`
pub fn _an_ask_with_no_subject(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.not_knowing:_an_ask_with_no_subject")
}

/// Python: bot/compose/not_knowing.py:516 `_the_whole_exchange(line, memory)` (never called on the trace corpus)
pub fn _the_whole_exchange(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.not_knowing:_the_whole_exchange")
}

/// Python: bot/compose/not_knowing.py:529 `_the_routes_hold_this(line)`
pub fn _the_routes_hold_this(line: &str) -> bool {
    crate::unported!("bot.compose.not_knowing:_the_routes_hold_this")
}

/// Python: bot/compose/not_knowing.py:541 `_the_ok_lead(line)`
pub fn _the_ok_lead(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.not_knowing:_the_ok_lead")
}

/// Python: bot/compose/not_knowing.py:547 `_the_ok_route(line)`
pub fn _the_ok_route(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.not_knowing:_the_ok_route")
}

/// Python: bot/compose/not_knowing.py:557 `_the_rules_route(line)`
pub fn _the_rules_route(line: &str) -> Vec<String> {
    crate::unported!("bot.compose.not_knowing:_the_rules_route")
}

/// Python: bot/compose/not_knowing.py:562 `_the_lead_and_the_route(clean, memory=None)`
pub fn _the_lead_and_the_route(clean: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.not_knowing:_the_lead_and_the_route")
}

/// Python: bot/compose/not_knowing.py:654 `_asks_about_this_chat(clean)`
pub fn _asks_about_this_chat(clean: &str) -> bool {
    crate::unported!("bot.compose.not_knowing:_asks_about_this_chat")
}

