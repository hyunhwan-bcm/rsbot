//! Port of `bot/modules/a_measure_asked_again.py`: a bare measure asked again is answered from the answer above (`bot/memory/measure.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/a_measure_asked_again.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The sentence of the answer above that answers the measure again, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a hook that raises answers nothing
    or_on_raise(|| as_reply(crate::memory::measure::reply_for(text, memory)), None)
}

/// Python: bot/modules/a_measure_asked_again.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a line that is nothing but a measure, and only when the answer above has it.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::memory::measure::reply_for(text, memory)), false))
}
