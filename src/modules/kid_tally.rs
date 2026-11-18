//! Port of `bot/modules/kid_tally.py`: the number of children, counted from the children the chat already holds (`bot/memory/kidcount.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/kid_tally.py:23 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The count said back, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails answers the line as it did before
    or_on_raise(|| or_none(crate::memory::kidcount::reply_for(text, memory)), None)
}
