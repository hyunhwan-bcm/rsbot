//! Port of `bot/modules/near_me.py`: a near-me ask with the thing left out is answered with the thing put back in (`bot/compose/nearby_one.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/near_me.py:20 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The map-app answer to a pronoun near-me ask, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails answers the line as it did before
    or_on_raise(|| or_none(crate::compose::nearby_one::reply_for(text, memory)), None)
}
