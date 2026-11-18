//! Port of `bot/modules/a_number_the_bot_knows.py`: a number the bot already knows (`bot/compose/a_number_the_bot_knows.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/a_number_the_bot_knows.py `respond(text, memory=None)` - a runtime hook; its signature is the runtime's.
///
/// The number the bot already knows, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    as_reply(crate::compose::a_number_the_bot_knows::answer_for(text, memory))
}
