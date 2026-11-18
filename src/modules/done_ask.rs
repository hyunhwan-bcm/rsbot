//! Port of `bot/modules/done_ask.py`.
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

/// Python: bot/modules/done_ask.py:60 `_entry(text)`
pub fn _entry(text: &str) -> &'static Entry {
    crate::unported!("bot.modules.done_ask:_entry")
}

/// Python: bot/modules/done_ask.py:69 `_the_answer_to_lean_on(memory)`
pub fn _the_answer_to_lean_on(memory: &Memory) -> String {
    crate::unported!("bot.modules.done_ask:_the_answer_to_lean_on")
}

/// Python: bot/modules/done_ask.py:78 `_the_topics_on_the_chat(memory)`
pub fn _the_topics_on_the_chat(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.done_ask:_the_topics_on_the_chat")
}

/// Python: bot/modules/done_ask.py:94 `_key_groups(item)`
pub fn _key_groups(item: &'static Entry) -> Value {
    crate::unported!("bot.modules.done_ask:_key_groups")
}

/// Python: bot/modules/done_ask.py:105 `_keys_the_word(phrases, word)`
pub fn _keys_the_word(phrases: &[String], word: &str) -> bool {
    crate::unported!("bot.modules.done_ask:_keys_the_word")
}

/// Python: bot/modules/done_ask.py:124 `_about_the_thing(item, word)`
pub fn _about_the_thing(item: &'static Entry, word: &str) -> bool {
    crate::unported!("bot.modules.done_ask:_about_the_thing")
}

/// Python: bot/modules/done_ask.py:136 `_says_how_it_turns_out(item)`
pub fn _says_how_it_turns_out(item: &'static Entry) -> bool {
    crate::unported!("bot.modules.done_ask:_says_how_it_turns_out")
}

/// Python: bot/modules/done_ask.py:153 `_the_thing_words(memory, said_words)`
pub fn _the_thing_words(memory: &Memory, said_words: &[String]) -> Vec<String> {
    crate::unported!("bot.modules.done_ask:_the_thing_words")
}

/// Python: bot/modules/done_ask.py:164 `_the_chat_has_moved_on(found, memory)` (never called on the trace corpus)
pub fn _the_chat_has_moved_on(found: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.modules.done_ask:_the_chat_has_moved_on")
}

/// Python: bot/modules/done_ask.py:193 `the_done_state_of_the_thing_in_hand(line, memory)`
pub fn the_done_state_of_the_thing_in_hand(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.done_ask:the_done_state_of_the_thing_in_hand")
}

/// Python: bot/modules/done_ask.py:224 `observe(line, memory)` - a runtime hook; its signature is the runtime's.
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.done_ask:observe")
}

