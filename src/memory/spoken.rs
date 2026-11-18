//! Port of `bot/memory/spoken.py`.
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

/// Python: bot/memory/spoken.py:64 `_the_number(value)` (never called on the trace corpus)
pub fn _the_number(value: &Value) -> String {
    crate::unported!("bot.memory.spoken:_the_number")
}

/// Python: bot/memory/spoken.py:70 `_is_hours(unit)` (never called on the trace corpus)
pub fn _is_hours(unit: &str) -> bool {
    crate::unported!("bot.memory.spoken:_is_hours")
}

/// Python: bot/memory/spoken.py:74 `_in_digits(amount, unit)` (never called on the trace corpus)
pub fn _in_digits(amount: &Value, unit: &str) -> String {
    crate::unported!("bot.memory.spoken:_in_digits")
}

/// Python: bot/memory/spoken.py:87 `_spoken_as_numbers(text)`
pub fn _spoken_as_numbers(text: &str) -> String {
    crate::unported!("bot.memory.spoken:_spoken_as_numbers")
}

