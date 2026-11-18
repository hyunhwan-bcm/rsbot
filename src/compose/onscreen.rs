//! Port of `bot/compose/onscreen.py`.
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

/// Python: bot/compose/onscreen.py:215 `_stem(word)`
pub fn _stem(word: &str) -> String {
    crate::unported!("bot.compose.onscreen:_stem")
}

/// Python: bot/compose/onscreen.py:231 `_the_same_word(word, said)`
pub fn _the_same_word(word: &str, said: &[String]) -> bool {
    crate::unported!("bot.compose.onscreen:_the_same_word")
}

/// Python: bot/compose/onscreen.py:241 `_stems(words)`
pub fn _stems(words: &[String]) -> Vec<String> {
    crate::unported!("bot.compose.onscreen:_stems")
}

/// Python: bot/compose/onscreen.py:252 `_names_something_new(mine, said, in_the_chat)`
pub fn _names_something_new(mine: &[String], said: &[String], in_the_chat: &[String]) -> bool {
    crate::unported!("bot.compose.onscreen:_names_something_new")
}

/// Python: bot/compose/onscreen.py:277 `_the_words_in_the_chat(memory, answer)`
pub fn _the_words_in_the_chat(memory: &Memory, answer: &str) -> Vec<String> {
    crate::unported!("bot.compose.onscreen:_the_words_in_the_chat")
}

/// Python: bot/compose/onscreen.py:286 `_the_answer_to_hand_back(memory)`
pub fn _the_answer_to_hand_back(memory: &Memory) -> String {
    crate::unported!("bot.compose.onscreen:_the_answer_to_hand_back")
}

/// Python: bot/compose/onscreen.py:308 `_the_answer_is_a_card(memory)` (never called on the trace corpus)
pub fn _the_answer_is_a_card(memory: &Memory) -> bool {
    crate::unported!("bot.compose.onscreen:_the_answer_is_a_card")
}

/// Python: bot/compose/onscreen.py:328 `_the_parts(sentence)`
pub fn _the_parts(sentence: &str) -> Vec<String> {
    crate::unported!("bot.compose.onscreen:_the_parts")
}

/// Python: bot/compose/onscreen.py:338 `_the_sentence_to_hand_back(sentence, mine, said)`
pub fn _the_sentence_to_hand_back(sentence: &str, mine: &[String], said: &[String]) -> String {
    crate::unported!("bot.compose.onscreen:_the_sentence_to_hand_back")
}

/// Python: bot/compose/onscreen.py:384 `the_answer_on_screen(text, memory)`
pub fn the_answer_on_screen(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.compose.onscreen:the_answer_on_screen")
}

/// Python: bot/compose/onscreen.py:551 `earlier_turn(memory)`
pub fn earlier_turn(memory: &Memory) -> bool {
    crate::unported!("bot.compose.onscreen:earlier_turn")
}

/// Python: bot/compose/onscreen.py:561 `talks_about_the_conversation(reply)`
pub fn talks_about_the_conversation(reply: &str) -> bool {
    crate::unported!("bot.compose.onscreen:talks_about_the_conversation")
}

