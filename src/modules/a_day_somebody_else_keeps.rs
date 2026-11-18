//! Port of `bot/modules/a_day_somebody_else_keeps.py`: a start or a finish of a day somebody else keeps - a school day, a class - is theirs to say.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::py::random::Random;

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.a_day_somebody_else_keeps.", stringify!($name))));
    )*};
}
rx!(MY_CARE_ASK, MY_CARE_NOT, THEIR_CLASS, A_CHILD);
static SCHOOL_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.a_day_somebody_else_keeps.SCHOOL_LINES"));
static CLASS_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.a_day_somebody_else_keeps.CLASS_LINES"));

/// Python: bot/modules/a_day_somebody_else_keeps.py `_clean(text)`
pub fn _clean(text: &str) -> String {
    fold(text)
}

/// Python: bot/modules/a_day_somebody_else_keeps.py `_choose(pool, memory, said="")`
///
/// A line from the pool, never the same one twice running, and the same one every run: the seed
/// is the turn count and the line itself.
pub fn _choose(pool: &[String], memory: &Memory, said: &str) -> String {
    let last = text::strip(&last_bot(memory)).to_string();
    let fresh: Vec<String> = pool.iter().filter(|line| **line != last).cloned().collect();
    let options = if fresh.is_empty() { pool.to_vec() } else { fresh };
    let seed = memory.turns.len() as i128 * 7919 + crate::py::random::ord_sum(&said.to_lowercase());
    Random::new(seed).choice(&options).clone()
}

/// Python: bot/modules/a_day_somebody_else_keeps.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let said = _clean(text);
    if said.is_empty() {
        return None;
    }
    if MY_CARE_NOT.is_match(&said) {
        return None;
    }
    if !MY_CARE_ASK.is_match(&said) {
        return None;
    }
    let pool = if THEIR_CLASS.is_match(&said) && !A_CHILD.is_match(&said) { &*CLASS_LINES } else { &*SCHOOL_LINES };
    Some(_choose(pool, memory, &said))
}
