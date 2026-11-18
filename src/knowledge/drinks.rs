//! Port of `bot/knowledge/drinks.py`.
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

/// Python: bot/knowledge/drinks.py:76 `_named(text, words)` (never called on the trace corpus)
pub fn _named(text: &str, words: &[String]) -> bool {
    crate::unported!("bot.knowledge.drinks:_named")
}

/// Python: bot/knowledge/drinks.py:85 `_wine_pair(memory, understood)` (never called on the trace corpus)
pub fn _wine_pair(memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.knowledge.drinks:_wine_pair")
}

/// Python: bot/knowledge/drinks.py:625 `_how_strong(memory, reading)` (never called on the trace corpus)
pub fn _how_strong(memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.knowledge.drinks:_how_strong")
}

/// Python: bot/knowledge/drinks.py:688 `_a_drink(reading)`
pub fn _a_drink(reading: &Understood) -> Value {
    crate::unported!("bot.knowledge.drinks:_a_drink")
}

/// Python: bot/knowledge/drinks.py:706 `_the_drink_taste(memory, reading)` (never called on the trace corpus)
pub fn _the_drink_taste(memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.knowledge.drinks:_the_drink_taste")
}

/// Python: bot/knowledge/drinks.py:714 `_the_drink_itself(memory, reading)`
pub fn _the_drink_itself(memory: &Memory, reading: &Understood) -> String {
    crate::unported!("bot.knowledge.drinks:_the_drink_itself")
}

/// Python: bot/knowledge/drinks.py:726 `_what_it_goes_with(memory, reading)` (never called on the trace corpus)
pub fn _what_it_goes_with(memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.knowledge.drinks:_what_it_goes_with")
}

/// Python: bot/knowledge/drinks.py:791 `_the_difference(memory, reading)` (never called on the trace corpus)
pub fn _the_difference(memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.knowledge.drinks:_the_difference")
}

