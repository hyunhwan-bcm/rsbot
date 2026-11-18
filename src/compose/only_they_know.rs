//! Port of `bot/compose/only_they_know.py`.
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

/// Python: bot/compose/only_they_know.py:207 `_a_judgement_i_am_asked_for(clean, words)`
pub fn _a_judgement_i_am_asked_for(clean: &str, words: &[String]) {
    crate::unported!("bot.compose.only_they_know:_a_judgement_i_am_asked_for")
}

/// Python: bot/compose/only_they_know.py:470 `_said_person(clean)`
pub fn _said_person(clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.only_they_know:_said_person")
}

/// Python: bot/compose/only_they_know.py:487 `_a_feeling_toward_me(clean)`
pub fn _a_feeling_toward_me(clean: &str) {
    crate::unported!("bot.compose.only_they_know:_a_feeling_toward_me")
}

/// Python: bot/compose/only_they_know.py:507 `_feeling_subject(clean)`
pub fn _feeling_subject(clean: &str) -> String {
    crate::unported!("bot.compose.only_they_know:_feeling_subject")
}

/// Python: bot/compose/only_they_know.py:514 `_a_fact_about_their_person(clean)`
pub fn _a_fact_about_their_person(clean: &str) {
    crate::unported!("bot.compose.only_they_know:_a_fact_about_their_person")
}

/// Python: bot/compose/only_they_know.py:555 `answer_for(text, memory)`
pub fn answer_for(text: &str, memory: &Memory) {
    crate::unported!("bot.compose.only_they_know:answer_for")
}

