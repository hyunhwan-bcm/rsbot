//! Port of `bot/modules/an_age_ahead.py`: an age further ahead, for a child the chat holds (`bot/memory/aboutthem.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/an_age_ahead.py `TAKES_AS_IT_COMES(text, memory)`
///
/// This line is the one to answer as it comes: it asks for an age further ahead.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    ret_true(crate::memory::aboutthem::an_age_ahead(text, memory))
}

/// Python: bot/modules/an_age_ahead.py `CLAIMS_INSTEAD(text, memory)`
///
/// The store claims this line as about the chat; the answer here is about the child.
pub fn CLAIMS_INSTEAD(text: &str, memory: &Memory) -> bool {
    ret_true(crate::memory::aboutthem::an_age_ahead(text, memory))
}

/// Python: bot/modules/an_age_ahead.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    or_none(crate::memory::aboutthem::an_age_ahead(text, memory))
}
