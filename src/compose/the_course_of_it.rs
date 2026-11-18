//! Port of `bot/compose/the_course_of_it.py`.
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

/// Python: bot/compose/the_course_of_it.py:284 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.the_course_of_it:_words")
}

/// Python: bot/compose/the_course_of_it.py:288 `_same_as_the_last_answer(written, memory)`
pub fn _same_as_the_last_answer(written: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.the_course_of_it:_same_as_the_last_answer")
}

/// Python: bot/compose/the_course_of_it.py:317 `_an_answer_worth_quoting(said)`
pub fn _an_answer_worth_quoting(said: &str) -> bool {
    crate::unported!("bot.compose.the_course_of_it:_an_answer_worth_quoting")
}

/// Python: bot/compose/the_course_of_it.py:323 `_the_card_in_the_chat(memory)`
pub fn _the_card_in_the_chat(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.compose.the_course_of_it:_the_card_in_the_chat")
}

/// Python: bot/compose/the_course_of_it.py:393 `_the_first_bit(card, limit=150, skip_openers=False)`
pub fn _the_first_bit(card: &str, limit: i64, skip_openers: bool) -> String {
    crate::unported!("bot.compose.the_course_of_it:_the_first_bit")
}

/// Python: bot/compose/the_course_of_it.py:418 `_leans_on(memory)`
pub fn _leans_on(memory: &Memory) -> String {
    crate::unported!("bot.compose.the_course_of_it:_leans_on")
}

/// Python: bot/compose/the_course_of_it.py:424 `_recent(memory, line='')`
pub fn _recent(memory: &Memory, line: &str) -> String {
    crate::unported!("bot.compose.the_course_of_it:_recent")
}

/// Python: bot/compose/the_course_of_it.py:454 `reads(text, memory=None)`
pub fn reads(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.the_course_of_it:reads")
}

/// Python: bot/compose/the_course_of_it.py:526 `reply_for(text, memory=None)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.the_course_of_it:reply_for")
}

