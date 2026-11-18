//! Port of `bot/modules/household.py`: who lives with them, filed and said back (`bot/memory/household.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/household.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// File who lives with them, on the chat's own memory.
pub fn observe(text: &str, memory: &mut Memory) {
    // a reading that fails files nothing
    let _ = py_try(|| {
        crate::memory::household::observe(text, memory);
    });
}

/// Python: bot/modules/household.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The household said back, or the ask answered, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails leaves the line as it was
    or_on_raise(|| or_none(crate::memory::household::answer(text, memory)), None)
}

/// Python: bot/modules/household.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a line that is nothing but the household.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::memory::household::takes_as_it_comes(text, memory)), false))
}
