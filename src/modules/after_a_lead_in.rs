//! Port of `bot/modules/after_a_lead_in.py`: the ask behind a lead-in: "and is b&q open on sunday" is the same question as without it (`bot/compose/lead_in.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/after_a_lead_in.py:27 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::compose::lead_in::the_answer_the_lead_in_hid(text, memory))
}
