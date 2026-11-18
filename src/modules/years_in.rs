//! Port of `bot/modules/years_in.py`: a line that is only how long they have been at their job (`bot/memory/years_in.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::years_in;

/// Python: bot/modules/years_in.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply for a line that is only how long they have been at their job, else None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    or_none(years_in::the_years_in_the_doing(text, memory)).or_else(|| or_none(years_in::the_years_in_their_doing(text, memory)))
}

/// Python: bot/modules/years_in.py `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`).
///
/// This line is a length I read whole, so it is answered as it comes.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    ret_true(years_in::the_years_in_the_doing(text, memory)) || ret_true(years_in::the_years_in_their_doing(text, memory))
}
