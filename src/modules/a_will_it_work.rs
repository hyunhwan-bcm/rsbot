//! Port of `bot/modules/a_will_it_work.py`: "will that work" about the card above: the yes with the card's own account of why (`bot/memory/will_it_work.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/a_will_it_work.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The yes with the card's own account of why, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a hook that raises answers nothing
    or_on_raise(|| as_reply(crate::memory::will_it_work::reply_for(text, memory)), None)
}

/// Python: bot/modules/a_will_it_work.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a line that is nothing but the check, and only when the card above says why.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::memory::will_it_work::reply_for(text, memory)), false))
}
