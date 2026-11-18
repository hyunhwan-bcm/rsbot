//! Port of `bot/modules/the_trouble_named.py`: a thing named as the trouble is asked about (`bot/compose/the_trouble_named.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/the_trouble_named.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::compose::the_trouble_named::reply_for(text, memory))
}

/// Python: bot/modules/the_trouble_named.py `TAKES_AS_IT_COMES(text, memory=None)`
///
/// This module read the line whole as a thing named as the trouble, so it takes it as it comes.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    respond(text, &mut memory.clone()).is_some()
}
