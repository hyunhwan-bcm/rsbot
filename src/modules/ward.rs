//! Port of `bot/modules/ward.py`: a question about somebody's care is the ward's, not a search (`bot/compose/ward.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/ward.py:19 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::compose::ward::answer_for(text, memory))
}
