//! Port of `bot/modules/receiving_end.py`: a person in a role who keeps doing the same thing has a band of its own (`bot/compose/receiving_end.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/receiving_end.py:16 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply to a role that keeps doing it, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let understood = memory.understood.as_ref()?;
    let clean = if !understood.clean.is_empty() { understood.clean.clone() } else { fold(text) };
    let repaired = if !understood.plain.is_empty() { understood.plain.clone() } else { clean.clone() };
    as_reply(crate::compose::receiving_end::reply_for(&clean, &repaired, memory))
}
