//! Port of `bot/modules/who_to_see.py`: the person to see, when that is what was asked for (`bot/compose/who.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/who_to_see.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The person to see and the condition on them, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails answers the line as it did before
    or_on_raise(|| or_none(crate::compose::who::reply_for(text, memory)), None)
}

/// Python: bot/modules/who_to_see.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a line that is nothing but the ask, over a thread that has a person to name.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::compose::who::takes_as_it_comes(text, memory)), false))
}
