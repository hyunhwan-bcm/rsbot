//! Port of `bot/memory/people.py`.
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

/// Python: bot/memory/people.py:46 `_the_age_of_the_group(memory)` (never called on the trace corpus)
pub fn _the_age_of_the_group(memory: &Memory) -> Value {
    crate::unported!("bot.memory.people:_the_age_of_the_group")
}

/// Python: bot/memory/people.py:62 `_with_an_age(record)` (never called on the trace corpus)
pub fn _with_an_age(record: &Value) -> bool {
    crate::unported!("bot.memory.people:_with_an_age")
}

/// Python: bot/memory/people.py:71 `_age(record)` (never called on the trace corpus)
pub fn _age(record: &Value) -> Value {
    crate::unported!("bot.memory.people:_age")
}

/// Python: bot/memory/people.py:81 `a_person_with_an_age(memory, understood=None)`
pub fn a_person_with_an_age(memory: &mut Memory, understood: &Understood) {
    crate::unported!("bot.memory.people:a_person_with_an_age")
}

/// Python: bot/memory/people.py:126 `_the_group_record(memory, line)`
pub fn _the_group_record(memory: &Memory, line: &str) {
    crate::unported!("bot.memory.people:_the_group_record")
}

/// Python: bot/memory/people.py:136 `_the_ones_a_them_points_at(people)` (never called on the trace corpus)
pub fn _the_ones_a_them_points_at(people: &Value) -> Value {
    crate::unported!("bot.memory.people:_the_ones_a_them_points_at")
}

/// Python: bot/memory/people.py:159 `the_people(memory)` (never called on the trace corpus)
pub fn the_people(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.people:the_people")
}

/// Python: bot/memory/people.py:170 `a_child_the_chat_is_about(memory, understood=None)` (never called on the trace corpus)
pub fn a_child_the_chat_is_about(memory: &mut Memory, understood: &Understood) -> bool {
    crate::unported!("bot.memory.people:a_child_the_chat_is_about")
}

/// Python: bot/memory/people.py:232 `the_age_of_the_person_they_mean(memory, understood=None)`
pub fn the_age_of_the_person_they_mean(memory: &mut Memory, understood: &Understood) {
    crate::unported!("bot.memory.people:the_age_of_the_person_they_mean")
}

