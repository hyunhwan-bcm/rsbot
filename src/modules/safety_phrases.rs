//! Port of `bot/modules/safety_phrases.py`: the phrase tables `bot/modules/safety.py` reads (the tables themselves are in the exported bundle).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Python: bot/modules/safety_phrases.py `_both_here(*groups)`
///
/// One thing from each group, all in the line, in whichever order: a lookahead per group.
pub fn _both_here(groups: &[Value]) -> String {
    groups.iter().map(|group| format!("(?=.*(?:{}))", py_strs(group).join("|"))).collect()
}
