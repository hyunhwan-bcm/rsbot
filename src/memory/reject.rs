//! Port of `bot/memory/reject.py`.
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

/// Python: bot/memory/reject.py:150 `_words(text)`
pub fn _words(text: &str) -> Option<Vec<String>> {
    crate::unported!("bot.memory.reject:_words")
}

/// Python: bot/memory/reject.py:186 `refusal(text, offer='')`
pub fn refusal(text: &str, offer: &str) -> bool {
    crate::unported!("bot.memory.reject:refusal")
}

/// Python: bot/memory/reject.py:233 `_the_whole_name(option, reply)`
pub fn _the_whole_name(option: &str, reply: &str) -> String {
    crate::unported!("bot.memory.reject:_the_whole_name")
}

/// Python: bot/memory/reject.py:249 `_tidy(option)`
pub fn _tidy(option: &str) -> String {
    crate::unported!("bot.memory.reject:_tidy")
}

/// Python: bot/memory/reject.py:304 `_by_title(titles, options)`
pub fn _by_title(titles: &[String], options: &[String]) -> Vec<String> {
    crate::unported!("bot.memory.reject:_by_title")
}

/// Python: bot/memory/reject.py:339 `_a_choice_of_things(reply)`
pub fn _a_choice_of_things(reply: &str) -> bool {
    crate::unported!("bot.memory.reject:_a_choice_of_things")
}

/// Python: bot/memory/reject.py:364 `_the_offer_reply(memory)`
pub fn _the_offer_reply(memory: &Memory) -> String {
    crate::unported!("bot.memory.reject:_the_offer_reply")
}

/// Python: bot/memory/reject.py:378 `_options(reply)`
pub fn _options(reply: &str) -> Vec<String> {
    crate::unported!("bot.memory.reject:_options")
}

/// Python: bot/memory/reject.py:443 `_a_list_of_causes(reply, options)`
pub fn _a_list_of_causes(reply: &str, options: &[String]) -> bool {
    crate::unported!("bot.memory.reject:_a_list_of_causes")
}

/// Python: bot/memory/reject.py:465 `_a_noun_phrase(option)`
pub fn _a_noun_phrase(option: &str) -> bool {
    crate::unported!("bot.memory.reject:_a_noun_phrase")
}

/// Python: bot/memory/reject.py:479 `_said_by_them(option, text)`
pub fn _said_by_them(option: &str, text: &str) -> bool {
    crate::unported!("bot.memory.reject:_said_by_them")
}

/// Python: bot/memory/reject.py:506 `_offered(memory)`
pub fn _offered(memory: &Memory) {
    crate::unported!("bot.memory.reject:_offered")
}

/// Python: bot/memory/reject.py:560 `_the_fence(memory)` (never called on the trace corpus)
pub fn _the_fence(memory: &Memory) -> Value {
    crate::unported!("bot.memory.reject:_the_fence")
}

/// Python: bot/memory/reject.py:597 `_the_whole_lot(options)` (never called on the trace corpus)
pub fn _the_whole_lot(options: &Value) -> String {
    crate::unported!("bot.memory.reject:_the_whole_lot")
}

/// Python: bot/memory/reject.py:607 `offers_a_choice(memory)`
pub fn offers_a_choice(memory: &Memory) -> bool {
    crate::unported!("bot.memory.reject:offers_a_choice")
}

/// Python: bot/memory/reject.py:624 `_the_names(options)` (never called on the trace corpus)
pub fn _the_names(options: &Value) -> Value {
    crate::unported!("bot.memory.reject:_the_names")
}

/// Python: bot/memory/reject.py:632 `_the_lot_and_the_verb(lines)` (never called on the trace corpus)
pub fn _the_lot_and_the_verb(lines: &[String]) -> Value {
    crate::unported!("bot.memory.reject:_the_lot_and_the_verb")
}

/// Python: bot/memory/reject.py:644 `reply_for(text, memory)`
pub fn reply_for(text: &str, memory: &Memory) {
    crate::unported!("bot.memory.reject:reply_for")
}

