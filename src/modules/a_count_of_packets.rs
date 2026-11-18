//! Port of `bot/modules/a_count_of_packets.py`: the poisons card for a count of packets (`bot/compose/a_count_of_packets.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/a_count_of_packets.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The poisons card for a count of packets, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if !ret_true(crate::compose::a_count_of_packets::is_a_dose(text)) {
        return None;
    }
    // Safety's own reading comes first: every arm it has is better than this one, and a count it
    // already reads - "i took a whole packet", "i took 20 paracetamol" - is its turn. `urgent` is
    // asked as well, because a child who has swallowed a packet is its card and not this one.
    if or_none(super::safety::respond(text, memory)).is_some() || or_none(super::urgent::respond(text, memory)).is_some() {
        return None;
    }
    memory.set("safety_case", "poison");
    memory.set("safety_safe", false);
    let pool = data::get("bot.knowledge.crisis_replies.REPLIES").get("poison").map(|v| v.strs()).unwrap_or_else(|| panic!("KeyError: 'poison'"));
    as_reply(super::safety::_fit(&pick(&pool, memory, ""), text, "poison"))
}
