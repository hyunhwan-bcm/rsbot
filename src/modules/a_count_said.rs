//! Port of `bot/modules/a_count_said.py`: where a count they reported stands against the answer above it (`bot/memory/amounts.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::amounts;

/// Python: bot/modules/a_count_said.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Nothing to keep: the report turn is read from the answer above it, every time.
pub fn observe(text: &str, memory: &mut Memory) {}

/// Python: bot/modules/a_count_said.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Where the count they reported stands against the answer above it, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let said = amounts::a_count_they_report(text).val();
    if said.is_none() {
        return None;
    }
    // `turns[-1].get("bot") if turns else ""`, then `last or ""`
    let last = memory.turns.last().map(|turn| turn.bot.clone()).unwrap_or_default();
    // CONTRACT NOTE: the skeleton types `said` as `&str`; the count goes over as Python's `str()`
    // of it until the owner of `bot/memory/amounts.py` types the value.
    let against = amounts::against_the_last_answer(&said, &last).val();
    if against.is_none() {
        return None;
    }
    as_reply(amounts::the_comparison(&against))
}
