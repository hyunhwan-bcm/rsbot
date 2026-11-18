//! Port of `bot/modules/the_month_they_have.py`: a month said out loud, and the sum of it (`bot/memory/outgoings.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/the_month_they_have.py `TAKES_AS_IT_COMES(text, memory)`
///
/// This line is the one to answer as it comes: a month said out loud, and the sum of it.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &mut Memory) -> bool {
    ret_true(crate::memory::outgoings::reply_for(text, Some(&mut *memory)))
}

/// Python: bot/modules/the_month_they_have.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    or_none(crate::memory::outgoings::reply_for(text, Some(&mut *memory)))
}
