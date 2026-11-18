//! Port of `bot/modules/what_decides_it.py`: the test the answer above gave, asked for again (`bot/compose/what_decides_it.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/what_decides_it.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The test the answer above gave, or None on every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that raises answers nothing
    or_on_raise(|| as_reply(crate::compose::what_decides_it::answer_for(text, memory)), None)
}
