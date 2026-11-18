//! Port of `bot/modules/nudge.py`: the honest end of the last answer, for a nudge that names nothing of its own (`bot/compose/still.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/nudge.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The honest end of the last answer, for a nudge that names nothing of its own. Else None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails answers the line as it did before
    or_on_raise(|| as_reply(crate::compose::still::reply_for(text, memory)), None)
}

/// Python: bot/modules/nudge.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on a nudge with an answer behind it: the line is read whole, and nowhere else.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::compose::still::takes_as_it_comes(text, memory)), false))
}
