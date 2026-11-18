//! Port of `bot/modules/group_ages.py`: the ages of a group the chat holds, kept and said back (`bot/memory/groupages.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::groupages;

/// Python: bot/modules/group_ages.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Keep the list a line gives (bot/memory/groupages.py).
pub fn observe(text: &str, memory: &mut Memory) {
    // an observer must never lose a turn
    let _ = py_try(|| {
        groupages::watch(text, memory);
    });
}

/// Python: bot/modules/group_ages.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The ages the chat holds, said back, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails answers the line as it did before
    or_on_raise(|| or_none(groupages::reply_for(text, memory)), None)
}

/// Python: bot/modules/group_ages.py `TAKES_AS_IT_COMES(text, memory)`
///
/// Only on an ask about a list the chat holds, and nowhere else. (The reader's own value is
/// handed back as it is: None stays None.)
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    match py_try(|| groupages::asks_for_the_list(text, memory).val()) {
        Ok(Value::None) => None,
        Ok(read) => Some(read.truthy()),
        Err(()) => Some(false),
    }
}
