//! Port of `bot/modules/taking_asked_again.py`: a bare "what can i take", about the thing the chat is on (`bot/memory/taking.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/taking_asked_again.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The answer to a bare "what can i take", about the thing the chat is on, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    or_none(crate::memory::taking::reply_for(text, memory))
}

/// Python: bot/modules/taking_asked_again.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// Only on the bare ask, and only where the chat has a complaint in it to answer about.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| ret_true(crate::memory::taking::the_ask_and_the_subject(text, memory)), false))
}
