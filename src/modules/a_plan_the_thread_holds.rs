//! Port of `bot/modules/a_plan_the_thread_holds.py`: the plan the thread holds, laid out where the composer would only hedge (`bot/compose/plan_ask.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain, plan_ask};

/// Python: bot/modules/a_plan_the_thread_holds.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Lay out the plan the thread holds, or say nothing and let the composer answer as it did.
/// (`SEARCH_ONLY` is `fallback._is_a_hedge`.)
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let line = match memory.understood.as_ref() {
        Some(understood) if !understood.clean.is_empty() => understood.clean.clone(),
        _ => text.to_string(),
    };
    let clean = fallback::_matchable(&fallback::_plain(&line));
    let pool = plan_ask::plan_ask_pool(&clean, memory).val();
    if pool.is_none() {
        return None;
    }
    let said = fallback_chain::reply_for(text, memory, true).val();
    if !ret_true(fallback::_is_a_hedge(&s_or(Some(&said)))) {
        return None; // the composer has an answer of its own; it keeps the turn
    }
    as_reply(fallback_chain::_choose(&py_strs(&pool), memory, &fallback::_matchable(text), None))
}
