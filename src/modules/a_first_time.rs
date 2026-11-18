//! Port of `bot/modules/a_first_time.py`: a first time is a task, not a first taste (the reading is `bot/compose/a_first_time.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/a_first_time.py:13 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// What helps a first time, or None for the floor to answer as before.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::compose::a_first_time::reply_for(text, memory))
}
