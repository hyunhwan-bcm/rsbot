//! Port of `bot/memory/not_a_trip.py`.
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

/// Python: bot/memory/not_a_trip.py:101 `a_doing_after_to(going_words)`
pub fn a_doing_after_to(going_words: &[String]) -> bool {
    crate::unported!("bot.memory.not_a_trip:a_doing_after_to")
}

/// Python: bot/memory/not_a_trip.py:120 `a_place_of_ordinary_life(place)`
pub fn a_place_of_ordinary_life(place: &str) -> bool {
    crate::unported!("bot.memory.not_a_trip:a_place_of_ordinary_life")
}

/// Python: bot/memory/not_a_trip.py:129 `not_a_destination(going_words, place)`
pub fn not_a_destination(going_words: &[String], place: &str) -> bool {
    crate::unported!("bot.memory.not_a_trip:not_a_destination")
}

/// Python: bot/memory/not_a_trip.py:158 `_the_person_in(found)` (never called on the trace corpus)
pub fn _the_person_in(found: &Value) -> Value {
    crate::unported!("bot.memory.not_a_trip:_the_person_in")
}

/// Python: bot/memory/not_a_trip.py:172 `a_person(place)` (never called on the trace corpus)
pub fn a_person(place: &str) -> Value {
    crate::unported!("bot.memory.not_a_trip:a_person")
}

/// Python: bot/memory/not_a_trip.py:199 `the_visit(line)`
pub fn the_visit(line: &str) -> String {
    crate::unported!("bot.memory.not_a_trip:the_visit")
}

