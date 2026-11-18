//! Port of `bot/modules/detail_question.py`: a detail the person adds, answered by the question it implies (`bot/compose/their_detail.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/detail_question.py:18 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The answer to the question the detail implies, or None for the floor to answer as before.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::compose::their_detail::reply_for(text, memory))
}
