//! Port of `bot/modules/since_it_began.py`: a length the chat already gave, asked for (`bot/memory/when.py`, `how_long_answer`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/since_it_began.py `TAKES_AS_IT_COMES(text, memory)`
///
/// This line is the one to answer as it comes: it asks for a length the chat already gave.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    ret_true(crate::memory::when::how_long_answer(text, memory))
}

/// Python: bot/modules/since_it_began.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    or_none(crate::memory::when::how_long_answer(text, memory))
}
