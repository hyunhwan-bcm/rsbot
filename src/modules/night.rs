//! Port of `bot/modules/night.py`: the hours between the two ends of the night they described (`bot/memory/asleep.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/night.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The hours between the two ends of the night they described, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // A sleep that names the telly, the fan or the lamp is a question about that thing, and
    // the chat module's bad-week pool must not take it. The row stands above `chat` (300), as
    // this module always has, and answers nothing else.
    if let Some(aid) = or_none(crate::knowledge::sleep_aids::answer_for(text, Some(&*memory))) {
        return Some(aid);
    }
    or_none(crate::memory::asleep::answer_for(text, memory))
}
