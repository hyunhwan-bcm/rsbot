//! Port of `bot/memory/asked_about.py`.
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

/// Python: bot/memory/asked_about.py:64 `_sentences(answer)`
pub fn _sentences(answer: &str) -> Vec<String> {
    crate::unported!("bot.memory.asked_about:_sentences")
}

/// Python: bot/memory/asked_about.py:79 `_the_words_asked(text)`
pub fn _the_words_asked(text: &str) -> Vec<String> {
    crate::unported!("bot.memory.asked_about:_the_words_asked")
}

/// Python: bot/memory/asked_about.py:97 `_the_part_that_answers(answer, wanted, said_before)`
pub fn _the_part_that_answers(answer: &str, wanted: &[String], said_before: &str) -> String {
    crate::unported!("bot.memory.asked_about:_the_part_that_answers")
}

/// Python: bot/memory/asked_about.py:134 `the_answer_this_chat_gave(text, memory)`
pub fn the_answer_this_chat_gave(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.asked_about:the_answer_this_chat_gave")
}

