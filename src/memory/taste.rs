//! Port of `bot/memory/taste.py`.
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

/// Python: bot/memory/taste.py:117 `_kept(thing)`
pub fn _kept(thing: &str) -> Value {
    crate::unported!("bot.memory.taste:_kept")
}

/// Python: bot/memory/taste.py:139 `_clean(thing)`
pub fn _clean(thing: &str) -> Option<String> {
    crate::unported!("bot.memory.taste:_clean")
}

/// Python: bot/memory/taste.py:152 `_thing_end(text, match)`
pub fn _thing_end(text: &str, r#match: &Value) -> i64 {
    crate::unported!("bot.memory.taste:_thing_end")
}

/// Python: bot/memory/taste.py:176 `_read_clause(text)`
pub fn _read_clause(text: &str) -> Value {
    crate::unported!("bot.memory.taste:_read_clause")
}

/// Python: bot/memory/taste.py:215 `read(line)`
pub fn read(line: &str) -> Value {
    crate::unported!("bot.memory.taste:read")
}

/// Python: bot/memory/taste.py:276 `read_all(line)`
pub fn read_all(line: &str) -> Value {
    crate::unported!("bot.memory.taste:read_all")
}

/// Python: bot/memory/taste.py:298 `only_a_taste(text)`
pub fn only_a_taste(text: &str) -> bool {
    crate::unported!("bot.memory.taste:only_a_taste")
}

/// Python: bot/memory/taste.py:342 `said_about(text, position)`
pub fn said_about(text: &str, position: i64) -> bool {
    crate::unported!("bot.memory.taste:said_about")
}

/// Python: bot/memory/taste.py:371 `asks_about_a_taste(line)`
pub fn asks_about_a_taste(line: &str) -> bool {
    crate::unported!("bot.memory.taste:asks_about_a_taste")
}

/// Python: bot/memory/taste.py:442 `read_other(line)`
pub fn read_other(line: &str) -> Vec<String> {
    crate::unported!("bot.memory.taste:read_other")
}

/// Python: bot/memory/taste.py:479 `_their_taste_in(rest, said)`
pub fn _their_taste_in(rest: &str, said: &Value) -> Value {
    crate::unported!("bot.memory.taste:_their_taste_in")
}

