//! Port of `bot/modules/first_line_open.py`: an opener for a first line the greeting would only have given a receipt of nothing.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/first_line_open.py `_has_a_turn_behind_it(memory)`
pub fn _has_a_turn_behind_it(memory: &Memory) -> bool {
    memory.turns.iter().any(|one| !text::strip(&one.bot).is_empty())
}

/// Python: bot/modules/first_line_open.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if _has_a_turn_behind_it(memory) {
        return None;
    }
    // a reading that fails simply adds no opener
    let reply = py_try(|| super::greeting::respond(text, memory)).ok()??;
    if reply.is_empty() || !data::re("bot.compose.fallback_data.A_RECEIPT_OF_NOTHING").is_match(&reply) {
        return None;
    }
    let clean = text::squash(text);
    as_reply(crate::compose::fallback_chain::_choose(&data::strs("bot.compose.fallback_data.COLD_OPEN"), memory, &clean, None))
}
