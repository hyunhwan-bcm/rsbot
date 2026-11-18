//! Port of `bot/memory/places.py`.
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

/// Python: bot/memory/places.py:53 `_a_place(value)` (never called on the trace corpus)
pub fn _a_place(value: &Value) -> Option<String> {
    crate::unported!("bot.memory.places:_a_place")
}

/// Python: bot/memory/places.py:82 `_the_place_alone(text)`
pub fn _the_place_alone(text: &str) {
    crate::unported!("bot.memory.places:_the_place_alone")
}

/// Python: bot/memory/places.py:110 `_said_places(memory)` (never called on the trace corpus)
pub fn _said_places(memory: &Memory) -> Value {
    crate::unported!("bot.memory.places:_said_places")
}

/// Python: bot/memory/places.py:163 `_a_town_moved(memory, text)`
pub fn _a_town_moved(memory: &Memory, text: &str) {
    crate::unported!("bot.memory.places:_a_town_moved")
}

/// Python: bot/memory/places.py:194 `_other_place(memory, who)` (never called on the trace corpus)
pub fn _other_place(memory: &Memory, who: &str) -> Option<String> {
    crate::unported!("bot.memory.places:_other_place")
}

/// Python: bot/memory/places.py:206 `_asked_person(memory, low)`
pub fn _asked_person(memory: &mut Memory, low: &str) -> String {
    crate::unported!("bot.memory.places:_asked_person")
}

