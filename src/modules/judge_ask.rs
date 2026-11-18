//! Port of `bot/modules/judge_ask.py`: the bot asked what it makes of people ("do you judge people"), where the composer says nothing.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.judge_ask.", stringify!($name))));
    )*};
}
rx!(JUDGE_PEOPLE_ASK, JUDGE_A_THING_NOT_PEOPLE, JUDGE_BY_A_STANDARD, JUDGE_SOMEONE_ELSE);
static PEOPLE_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.judge_ask.PEOPLE_LINES"));
static EMPTY_HEADS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.judge_ask.EMPTY_HEADS"));

/// Python: bot/modules/judge_ask.py `_says_nothing(said)`
///
/// True when a reply is one of the pools that tell the person nothing.
pub fn _says_nothing(said: &str) -> bool {
    if said.is_empty() {
        return true;
    }
    if ret_true(fallback::_is_a_hedge(said)) {
        return true;
    }
    let lowered = said.to_lowercase();
    EMPTY_HEADS.iter().any(|head| lowered.starts_with(head.as_str()))
}

/// Python: bot/modules/judge_ask.py `_the_composer_says_nothing(text, memory)`
///
/// True when the composer's own answer for this line is a nod, a receipt, a hedge or an echo.
pub fn _the_composer_says_nothing(text: &str, memory: &mut Memory) -> bool {
    // Both are asked before either is read: the tuple is built first.
    let floor = or_none(super::greeting::respond(text, memory));
    let composer = or_none(fallback_chain::reply_for(text, memory, true));
    for reply in [floor, composer].into_iter().flatten() {
        if !_says_nothing(&reply) {
            return false;
        }
    }
    true
}

/// Python: bot/modules/judge_ask.py `reads_it(text)`
///
/// True for the lines this module reads whole: the bot asked what it makes of people.
pub fn reads_it(text: &str) -> bool {
    let said = fold(text);
    if said.is_empty() {
        return false;
    }
    if !JUDGE_PEOPLE_ASK.is_match(&said) {
        return false;
    }
    !(JUDGE_A_THING_NOT_PEOPLE.is_match(&said) || JUDGE_BY_A_STANDARD.is_match(&said) || JUDGE_SOMEONE_ELSE.is_match(&said))
}

/// Python: bot/modules/judge_ask.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// One of the people lines, never the one the bot has just said, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // The reading is asked first and it is cheap: the composer is only run for a line this module
    // would answer, not on every turn. (@quick-iguana)
    if !reads_it(text) {
        return None;
    }
    if !_the_composer_says_nothing(text, memory) {
        return None;
    }
    let last = last_bot(memory);
    let start = crate::py::random::ord_sum(&fold(text)) as usize;
    for step in 0..PEOPLE_LINES.len() {
        let line = &PEOPLE_LINES[(start + step) % PEOPLE_LINES.len()];
        if last.is_empty() || !last.contains(&sl(line, 0, Some(24))) {
            return Some(line.clone());
        }
    }
    Some(PEOPLE_LINES[start % PEOPLE_LINES.len()].clone())
}
