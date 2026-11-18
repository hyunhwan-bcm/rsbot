//! Port of `bot/modules/pace.py`: the pace of the answer just given (`bot/compose/pace.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/pace.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The pace of the answer just given, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a module that raises claims nothing
    or_on_raise(|| as_reply(crate::compose::pace::reply_for(text, memory)), None)
}

/// Python: bot/modules/pace.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a rate asked about the plan just given, and nowhere else.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::compose::pace::the_pace_line(text, memory)) || ret_true(crate::compose::pace::the_keep_up_line(text, memory)), false))
}
