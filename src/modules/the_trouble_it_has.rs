//! Port of `bot/modules/the_trouble_it_has.py`: one question back about the thing that is not working, instead of a stall (`bot/compose/a_trouble.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{a_trouble, fallback, fallback_chain};

/// Python: bot/modules/the_trouble_it_has.py:18 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Ask about the trouble the person named, or say nothing and let the composer answer.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // `(memory.get("understood") or {}).get("clean") or text`
    let line = match memory.understood.as_ref() {
        Some(understood) if !understood.clean.is_empty() => understood.clean.clone(),
        _ => text.to_string(),
    };
    let clean = fallback::_matchable(&fallback::_plain(&line));
    let pool = a_trouble::trouble_pool(&clean, memory).val();
    if !pool.truthy() {
        return None;
    }
    let said = fallback_chain::reply_for(text, memory, true).val();
    if !ret_true(a_trouble::says_nothing_about_the_line(&s_or(Some(&said)), &fallback::_matchable(text))) {
        return None; // the composer has an answer of its own; it keeps the turn
    }
    as_reply(fallback_chain::_choose(&py_strs(&pool), memory, &fallback::_matchable(text), None))
}
