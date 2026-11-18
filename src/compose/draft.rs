//! Port of `bot/compose/draft.py`.
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

/// Python: bot/compose/draft.py:261 `_wants_to_hurt(low)` (never called on the trace corpus)
pub fn _wants_to_hurt(low: &str) -> bool {
    crate::unported!("bot.compose.draft:_wants_to_hurt")
}

/// Python: bot/compose/draft.py:266 `_is_a_message_shape(low)` (never called on the trace corpus)
pub fn _is_a_message_shape(low: &str) -> bool {
    crate::unported!("bot.compose.draft:_is_a_message_shape")
}

/// Python: bot/compose/draft.py:298 `names_a_draft(reply)`
pub fn names_a_draft(reply: &str) -> bool {
    crate::unported!("bot.compose.draft:names_a_draft")
}

/// Python: bot/compose/draft.py:303 `offers_a_draft(reply)`
pub fn offers_a_draft(reply: &str) -> bool {
    crate::unported!("bot.compose.draft:offers_a_draft")
}

/// Python: bot/compose/draft.py:317 `hands_over_words(reply)` (never called on the trace corpus)
pub fn hands_over_words(reply: &str) -> bool {
    crate::unported!("bot.compose.draft:hands_over_words")
}

/// Python: bot/compose/draft.py:350 `hands_over_the_shape(reply)` (never called on the trace corpus)
pub fn hands_over_the_shape(reply: &str) -> bool {
    crate::unported!("bot.compose.draft:hands_over_the_shape")
}

/// Python: bot/compose/draft.py:387 `reads_as_a_writing_ask(line)` (never called on the trace corpus)
pub fn reads_as_a_writing_ask(line: &str) -> bool {
    crate::unported!("bot.compose.draft:reads_as_a_writing_ask")
}

/// Python: bot/compose/draft.py:408 `_tidy(text)` (never called on the trace corpus)
pub fn _tidy(text: &str) -> String {
    crate::unported!("bot.compose.draft:_tidy")
}

/// Python: bot/compose/draft.py:432 `_small(text)` (never called on the trace corpus)
pub fn _small(text: &str) -> Value {
    crate::unported!("bot.compose.draft:_small")
}

/// Python: bot/compose/draft.py:451 `_content(text)` (never called on the trace corpus)
pub fn _content(text: &str) -> Value {
    crate::unported!("bot.compose.draft:_content")
}

/// Python: bot/compose/draft.py:488 `_formal(memory, words)` (never called on the trace corpus)
pub fn _formal(memory: &Memory, words: &[String]) -> bool {
    crate::unported!("bot.compose.draft:_formal")
}

/// Python: bot/compose/draft.py:499 `_said_before(memory, line)`
pub fn _said_before(memory: &Memory, line: &str) {
    crate::unported!("bot.compose.draft:_said_before")
}

/// Python: bot/compose/draft.py:522 `_a_fault_they_were_asked_for(low)` (never called on the trace corpus)
pub fn _a_fault_they_were_asked_for(low: &str) -> bool {
    crate::unported!("bot.compose.draft:_a_fault_they_were_asked_for")
}

/// Python: bot/compose/draft.py:547 `is_theirs(memory, line)` (never called on the trace corpus)
pub fn is_theirs(memory: &Memory, line: &str) -> bool {
    crate::unported!("bot.compose.draft:is_theirs")
}

/// Python: bot/compose/draft.py:624 `_names_the_message(low)`
pub fn _names_the_message(low: &str) {
    crate::unported!("bot.compose.draft:_names_the_message")
}

/// Python: bot/compose/draft.py:632 `says_it_is_the_message(line)`
pub fn says_it_is_the_message(line: &str) -> bool {
    crate::unported!("bot.compose.draft:says_it_is_the_message")
}

/// Python: bot/compose/draft.py:646 `not_the_message(memory, line)`
pub fn not_the_message(memory: &Memory, line: &str) -> bool {
    crate::unported!("bot.compose.draft:not_the_message")
}

/// Python: bot/compose/draft.py:665 `drafter(memory, line, words=None)`
pub fn drafter(memory: &Memory, line: &str, words: &[String]) {
    crate::unported!("bot.compose.draft:drafter")
}

/// Python: bot/compose/draft.py:754 `example_subject(understood)` (never called on the trace corpus)
pub fn example_subject(understood: &Understood) -> Value {
    crate::unported!("bot.compose.draft:example_subject")
}

