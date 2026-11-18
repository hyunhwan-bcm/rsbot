//! Port of `bot/modules/ruled_out.py`: the thing ruled out, said back against the answer (`bot/compose/ruledout.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/ruled_out.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The thing ruled out, said back against the answer, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails answers the line as it did before
    or_on_raise(|| or_none(crate::compose::ruledout::reply_for(text, memory)), None)
}

/// Python: bot/modules/ruled_out.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a line that is nothing but the rejection of something the answer never offered.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::compose::ruledout::takes_as_it_comes(text, memory)), false))
}
