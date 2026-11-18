//! Port of `bot/compose/aboutness.py`.
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

/// Python: bot/compose/aboutness.py:63 `after_a_change_of_subject(text)`
pub fn after_a_change_of_subject(text: &str) -> String {
    crate::unported!("bot.compose.aboutness:after_a_change_of_subject")
}

/// Python: bot/compose/aboutness.py:80 `subject_words(text)`
pub fn subject_words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.aboutness:subject_words")
}

/// Python: bot/compose/aboutness.py:86 `names_the_subject(reply, about)`
pub fn names_the_subject(reply: &str, about: &[String]) -> Vec<String> {
    crate::unported!("bot.compose.aboutness:names_the_subject")
}

/// Python: bot/compose/aboutness.py:98 `restates_the_line(reply, text)`
pub fn restates_the_line(reply: &str, text: &str) -> bool {
    crate::unported!("bot.compose.aboutness:restates_the_line")
}

/// Python: bot/compose/aboutness.py:105 `is_a_hedge(reply)`
pub fn is_a_hedge(reply: &str) -> bool {
    crate::unported!("bot.compose.aboutness:is_a_hedge")
}

/// Python: bot/compose/aboutness.py:110 `unlike_the_line(reply, text)`
pub fn unlike_the_line(reply: &str, text: &str) -> Value {
    crate::unported!("bot.compose.aboutness:unlike_the_line")
}

/// Python: bot/compose/aboutness.py:124 `the_better_answer(reply, candidates, text, skip=None, need=2)` (never called on the trace corpus)
pub fn the_better_answer(reply: &str, candidates: &[Candidate], text: &str, skip: &Value, need: &Value) -> Value {
    crate::unported!("bot.compose.aboutness:the_better_answer")
}

