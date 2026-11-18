//! Port of `bot/modules/a_habit_is_raised.py`: the sentence to raise somebody's habit with, when only a fallback line would answer (`bot/compose/habit.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain, habit};

/// Python: bot/modules/a_habit_is_raised.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The sentence to raise it with, or None when this is not that shape.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let raised = or_none(habit::answer_for(text, memory))?;
    let said = or_none(fallback_chain::reply_for(text, memory, true))?;
    let neutral = Value::Tuple(Arc::new(vec![data::get("bot.compose.fallback.NEUTRAL").clone()]));
    if !(ret_true(fallback::_is_a_hedge(&said)) || ret_true(fallback::_in_a_pool(&said, &neutral)) || ret_true(crate::memory::ack::reads_as_a_receipt(&said))) {
        // Something with an answer of its own is already speaking for this line, and the words
        // to raise a habit with are not it.
        return None;
    }
    Some(raised)
}
