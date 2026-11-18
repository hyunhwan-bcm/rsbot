//! Port of `bot/modules/already.py`: a report that the step was already taken is answered with the next turn of the thread (`bot/memory/already.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/already.py:18 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// (`TAKES_AS_IT_COMES = already.takes_as_it_comes`: the arbiter reads the reader in
/// `bot/memory/already.py` directly.)
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::memory::already::reply_for(text, memory))
}
