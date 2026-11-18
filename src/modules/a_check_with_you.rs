//! Port of `bot/modules/a_check_with_you.py`: a check of a plan against the bot gets the agreement, not a search hedge or a receipt.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/a_check_with_you.py:17 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The agreement line, or None when this is not a check-with-you question.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // `check_with_you.answer_for(text, memory)`: `choose` is left at its default, None.
    as_reply(crate::compose::check_with_you::answer_for(text, memory, &Value::None))
}
