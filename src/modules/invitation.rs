//! Port of `bot/modules/invitation.py`: the line an invitation to the bot gets (`bot/compose/invitation.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/invitation.py `DEFERS_TO(reply, text)`
///
/// Yes for every answer at a real priority: the invitation is the floor of its shape.
pub fn DEFERS_TO(reply: &str, text: &str) -> bool {
    true
}

/// Python: bot/modules/invitation.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The line an invitation to the bot gets; None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::compose::invitation::reply_for(text, memory))
}
