//! Port of `bot/compose/lead.py`.
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

/// Python: bot/compose/lead.py:80 `_sentences(reply)`
pub fn _sentences(reply: &str) -> Vec<String> {
    crate::unported!("bot.compose.lead:_sentences")
}

/// Python: bot/compose/lead.py:103 `_wants_the_method(understood)`
pub fn _wants_the_method(understood: Option<&Understood>) -> bool {
    crate::unported!("bot.compose.lead:_wants_the_method")
}

/// Python: bot/compose/lead.py:127 `rate_first(reply, understood, line=None)`
pub fn rate_first(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    crate::unported!("bot.compose.lead:rate_first")
}

/// Python: bot/compose/lead.py:157 `answer_first(reply, understood, line=None)`
pub fn answer_first(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    crate::unported!("bot.compose.lead:answer_first")
}

/// Python: bot/compose/lead.py:213 `verdict_first(reply, understood, line=None)`
pub fn verdict_first(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    crate::unported!("bot.compose.lead:verdict_first")
}

/// Python: bot/compose/lead.py:256 `method_first(reply, understood)`
pub fn method_first(reply: &str, understood: &Understood) -> String {
    crate::unported!("bot.compose.lead:method_first")
}

/// Python: bot/compose/lead.py:290 `_wants_the_cause(understood)`
pub fn _wants_the_cause(understood: Option<&Understood>) -> bool {
    crate::unported!("bot.compose.lead:_wants_the_cause")
}

/// Python: bot/compose/lead.py:299 `cause_first(reply, understood)`
pub fn cause_first(reply: &str, understood: Option<&Understood>) -> String {
    crate::unported!("bot.compose.lead:cause_first")
}

/// Python: bot/compose/lead.py:339 `_the_want(understood)`
pub fn _the_want(understood: Option<&Understood>) -> String {
    crate::unported!("bot.compose.lead:_the_want")
}

/// Python: bot/compose/lead.py:348 `_the_opening(reply)`
pub fn _the_opening(reply: &str) -> String {
    crate::unported!("bot.compose.lead:_the_opening")
}

/// Python: bot/compose/lead.py:352 `_already_said(lead_said, opening)`
pub fn _already_said(lead_said: &str, opening: &str) -> bool {
    crate::unported!("bot.compose.lead:_already_said")
}

/// Python: bot/compose/lead.py:361 `_the_own_sentence_holding(parts, bare)`
pub fn _the_own_sentence_holding(parts: &[String], bare: &str) {
    crate::unported!("bot.compose.lead:_the_own_sentence_holding")
}

/// Python: bot/compose/lead.py:379 `the_sentence_owed(reply, understood, declared=None)`
pub fn the_sentence_owed(reply: &str, understood: Option<&Understood>, declared: &Value) -> String {
    crate::unported!("bot.compose.lead:the_sentence_owed")
}

/// Python: bot/compose/lead.py:493 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.lead:_words")
}

/// Python: bot/compose/lead.py:498 `_names_the_line(sentence, line)`
pub fn _names_the_line(sentence: &str, line: &str) -> bool {
    crate::unported!("bot.compose.lead:_names_the_line")
}

/// Python: bot/compose/lead.py:506 `_sentence_stands_alone(sentence)`
pub fn _sentence_stands_alone(sentence: &str) -> bool {
    crate::unported!("bot.compose.lead:_sentence_stands_alone")
}

/// Python: bot/compose/lead.py:516 `shape(reply, understood, line=None)`
pub fn shape(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    crate::unported!("bot.compose.lead:shape")
}

