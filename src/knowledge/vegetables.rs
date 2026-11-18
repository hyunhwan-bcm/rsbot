//! Port of `bot/knowledge/vegetables.py`.
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

/// Python: bot/knowledge/vegetables.py:25 `_phrase_in(text, phrase)` (never called on the trace corpus)
pub fn _phrase_in(text: &str, phrase: &str) -> bool {
    crate::unported!("bot.knowledge.vegetables:_phrase_in")
}

/// Python: bot/knowledge/vegetables.py:284 `_takes_to(words, position, spot)` (never called on the trace corpus)
pub fn _takes_to(words: &[String], position: &Value, spot: &Value) -> bool {
    crate::unported!("bot.knowledge.vegetables:_takes_to")
}

/// Python: bot/knowledge/vegetables.py:301 `_near(text, phrase)` (never called on the trace corpus)
pub fn _near(text: &str, phrase: &str) -> Value {
    crate::unported!("bot.knowledge.vegetables:_near")
}

/// Python: bot/knowledge/vegetables.py:314 `_a_word(text, word)` (never called on the trace corpus)
pub fn _a_word(text: &str, word: &str) -> bool {
    crate::unported!("bot.knowledge.vegetables:_a_word")
}

/// Python: bot/knowledge/vegetables.py:332 `_name_in(text, name)` (never called on the trace corpus)
pub fn _name_in(text: &str, name: &str) -> bool {
    crate::unported!("bot.knowledge.vegetables:_name_in")
}

/// Python: bot/knowledge/vegetables.py:360 `_ruled_out(text, names)` (never called on the trace corpus)
pub fn _ruled_out(text: &str, names: &[String]) -> bool {
    crate::unported!("bot.knowledge.vegetables:_ruled_out")
}

/// Python: bot/knowledge/vegetables.py:386 `_typed_line(memory, understood)` (never called on the trace corpus)
pub fn _typed_line(memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.vegetables:_typed_line")
}

/// Python: bot/knowledge/vegetables.py:403 `_reply_for(names, said, how=None)` (never called on the trace corpus)
pub fn _reply_for(names: &[String], said: &str, how: &str) -> Value {
    crate::unported!("bot.knowledge.vegetables:_reply_for")
}

/// Python: bot/knowledge/vegetables.py:582 `_roast_reply(memory, understood)` (never called on the trace corpus)
pub fn _roast_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    crate::unported!("bot.knowledge.vegetables:_roast_reply")
}

/// Python: bot/knowledge/vegetables.py:617 `_slug(name)` (never called on the trace corpus)
pub fn _slug(name: &str) -> String {
    crate::unported!("bot.knowledge.vegetables:_slug")
}

/// Python: bot/knowledge/vegetables.py:799 `_watery_veg(understood)` (never called on the trace corpus)
pub fn _watery_veg(understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.vegetables:_watery_veg")
}

/// Python: bot/knowledge/vegetables.py:877 `_soft_veg(text, understood=None)` (never called on the trace corpus)
pub fn _soft_veg(text: &str, understood: &Understood) -> Option<String> {
    crate::unported!("bot.knowledge.vegetables:_soft_veg")
}

