//! Port of `bot/modules/a_thing_being_weighed.py`: a thing being weighed up, answered over the seed fallback's receipt (`bot/compose/a_thing_being_weighed.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/a_thing_being_weighed.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // The reading is the cheap half and it passes on almost every line, so it is asked first and
    // the seed fallback is only run for a line this module could answer.
    if !ret_true(crate::compose::a_thing_being_weighed::pool(text)) {
        return None;
    }
    // a broken seed keeps its own turn
    let reply = py_try(|| super::greeting::respond(text, memory)).ok()??;
    if text::strip(&reply).is_empty() {
        return None;
    }
    as_reply(crate::compose::a_thing_being_weighed::answer_for(text, memory, &reply))
}
