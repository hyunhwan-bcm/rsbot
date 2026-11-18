//! Port of `bot/modules/onsubject.py`: the honest line for a follow-up whose subject is only in the chat (`bot/compose/onsubject.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain, onsubject};
use crate::understood::Understood;

/// Python: bot/modules/onsubject.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // The reading is the cheap half and it passes on almost every line, so it is asked first and
    // the composer is only run for a line this module could answer. (`understood` is left at its
    // default, None: the readers take the argument and do not read it.)
    let nothing_read = Understood::default();
    let mut reading = or_none(onsubject::pool(text, &nothing_read, memory));
    if reading.is_none() {
        // A there-ask about the place the chat named needs no subject of its own: "do i need an
        // adapter there" after a trip to Orlando is a question about Orlando.
        reading = or_none(onsubject::place_pool(text, &nothing_read, memory));
    }
    let reading = reading?;
    // a broken composer keeps its own turn
    let reply = py_try(|| fallback_chain::reply_for(text, memory, true).val()).ok()?;
    let unknown = Value::Tuple(Arc::new(vec![data::get("bot.compose.fallback.UNKNOWN").clone()]));
    match reply.as_str() {
        Some(reply) if ret_true(fallback::_in_a_pool(reply, &unknown)) => Some(reading),
        _ => None,
    }
}
