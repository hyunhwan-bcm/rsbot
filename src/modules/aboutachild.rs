//! Port of `bot/modules/aboutachild.py`: a question about the child the store keeps, answered from the child's own row (`bot/memory/aboutthem.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/aboutachild.py `TAKES_AS_IT_COMES(text, memory)`
///
/// This line is the one to answer as it comes: it asks about the child the store keeps.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    ret_true(crate::memory::aboutthem::with_the_age(text, memory))
}

/// Python: bot/modules/aboutachild.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let written = or_none(crate::memory::aboutthem::with_the_age(text, memory))?;
    // `seen = copy.deepcopy(memory)`
    let mut seen = memory.clone();
    seen.understood = Some(crate::understanding::understand(&written, Some(&seen)));
    let reply = or_none(super::answers::_answer(&written, &mut seen))?; // its own reader, one answer
    nz(text::strip(&reply).to_string())
}
