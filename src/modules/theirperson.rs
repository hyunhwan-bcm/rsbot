//! Port of `bot/modules/theirperson.py`: a line about a person of theirs, where the composer's own reply may be replaced (`bot/compose/theirperson.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback_chain, theirperson};

/// Python: bot/modules/theirperson.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let reply = or_none(theirperson::reply_for(text, memory))?;
    let said = fallback_chain::reply_for(text, memory, true).val();
    if !ret_true(theirperson::may_replace(&s_or(Some(&said)), text)) {
        return None;
    }
    Some(reply)
}
