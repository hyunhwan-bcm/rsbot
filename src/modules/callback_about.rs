//! Port of `bot/modules/callback_about.py`: the ask after a person the chat holds (`bot/memory/callback.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/callback_about.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    or_none(crate::memory::callback::reply_for(text, memory))
}

/// Python: bot/modules/callback_about.py `CLAIMS_INSTEAD(text, memory)`
///
/// The ask after a person the chat holds, where a module below claims the follow-up.
pub fn CLAIMS_INSTEAD(text: &str, memory: &mut Memory) -> bool {
    ret_true(crate::memory::callback::claims(text, memory))
}
