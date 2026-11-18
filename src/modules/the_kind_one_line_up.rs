//! Port of `bot/modules/the_kind_one_line_up.py`: the kind of answer the earlier line named, when the fallback answered the line above (`bot/compose/kept_nearby.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

static FALLBACK_MODULES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.the_kind_one_line_up.FALLBACK_MODULES"));

/// Python: bot/modules/the_kind_one_line_up.py `_answered_by_the_fallback(memory)`
pub fn _answered_by_the_fallback(memory: &Memory) -> bool {
    match memory.turns.last() {
        None => false,
        Some(turn) => FALLBACK_MODULES.contains(turn.module.as_deref().unwrap_or("")),
    }
}

/// Python: bot/modules/the_kind_one_line_up.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The kind of answer the earlier line named, or None on every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that raises answers nothing
    or_on_raise(
        || {
            if !_answered_by_the_fallback(memory) {
                return None;
            }
            let nearby = crate::compose::kept_nearby::answer_for(text, memory).val();
            if !nearby.truthy() {
                return None;
            }
            as_reply(crate::compose::fallback_chain::_choose(&py_strs(&nearby), memory, text, None))
        },
        None,
    )
}
