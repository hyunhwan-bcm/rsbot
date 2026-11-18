//! Port of `bot/modules/a_thing_wants_doing.py`: a thing in a state that wants doing gets a question about the thing, not a receipt (`bot/compose/a_thing_wants_doing.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain};

/// Python: bot/modules/a_thing_wants_doing.py `_the_thing_and_its_state(text, memory)`: the
/// folded line and the pool that reads it, as Python's (clean, pool) pair.
pub fn _the_thing_and_its_state(text: &str, memory: &Memory) -> (String, Value) {
    let clean = match memory.understood.as_ref() {
        Some(understood) if !understood.clean.is_empty() => understood.clean.clone(),
        _ => fold(text),
    };
    let pool = crate::compose::a_thing_wants_doing::pool(&clean, memory).val();
    (clean, pool)
}

/// Python: bot/modules/a_thing_wants_doing.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// A question about the thing, or None where the bot already said something about it.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let (clean, pool) = _the_thing_and_its_state(text, memory);
    if !pool.truthy() {
        return None;
    }
    // Every reader of the composer's own is asked first: the line is only this module's where the
    // answer that came back is the receipt and nothing else.
    let said = match or_none(super::greeting::respond(text, memory)) {
        Some(said) => said,
        None => s_or(Some(&fallback_chain::reply_for(text, memory, true).val())),
    };
    let neutral = Value::Tuple(Arc::new(vec![data::get("bot.compose.fallback.NEUTRAL").clone()]));
    if !ret_true(fallback::_in_a_pool(&said, &neutral)) {
        return None;
    }
    as_reply(fallback_chain::_choose(&py_strs(&pool), memory, &clean, None))
}
