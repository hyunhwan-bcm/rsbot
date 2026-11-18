//! Port of `bot/compose/said_about_me.py`.
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

/// Python: bot/compose/said_about_me.py:189 `reads_as_somebody_elses_words(text)`
pub fn reads_as_somebody_elses_words(text: &str) -> bool {
    crate::unported!("bot.compose.said_about_me:reads_as_somebody_elses_words")
}

/// Python: bot/compose/said_about_me.py:195 `_kind(text)`
pub fn _kind(text: &str) -> String {
    crate::unported!("bot.compose.said_about_me:_kind")
}

/// Python: bot/compose/said_about_me.py:210 `reads_as_reported_judgement(text)`
pub fn reads_as_reported_judgement(text: &str) -> bool {
    crate::unported!("bot.compose.said_about_me:reads_as_reported_judgement")
}

/// Python: bot/compose/said_about_me.py:233 `_user_lines(memory, how_many=3)`
pub fn _user_lines(memory: &Memory, how_many: i64) -> Vec<String> {
    crate::unported!("bot.compose.said_about_me:_user_lines")
}

/// Python: bot/compose/said_about_me.py:249 `_on_the_thread(memory)`
pub fn _on_the_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.said_about_me:_on_the_thread")
}

/// Python: bot/compose/said_about_me.py:264 `answer_followup(text, memory=None)`
pub fn answer_followup(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.said_about_me:answer_followup")
}

/// Python: bot/compose/said_about_me.py:286 `speaks_to_a_self_judgement(reply)` (never called on the trace corpus)
pub fn speaks_to_a_self_judgement(reply: &str) -> bool {
    crate::unported!("bot.compose.said_about_me:speaks_to_a_self_judgement")
}

/// Python: bot/compose/said_about_me.py:292 `answer_for(text, memory=None)`
pub fn answer_for(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.said_about_me:answer_for")
}

