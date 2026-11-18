//! Port of `bot/modules/doubt.py`: a doubt about the bot's own last answer, and the ask for which thing a word means (`bot/memory/doubt.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

static SUBJECT_UNCLEAR: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.doubt.SUBJECT_UNCLEAR"));

/// Python: bot/modules/doubt.py `_the_question(text, word)`
///
/// One of the lines above, chosen by the line so the same chat does not repeat one.
pub fn _the_question(text: &str, word: &str) -> String {
    let at = (text.chars().map(|letter| letter as u64).sum::<u64>() % SUBJECT_UNCLEAR.len() as u64) as usize;
    fmt_named(&SUBJECT_UNCLEAR[at], &[("word", word)])
}

/// Python: bot/modules/doubt.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply to a doubt about the bot's own last answer, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if let Some(word) = memory.understood.as_ref().and_then(|understood| understood.subject_unclear.clone()) {
        if !word.is_empty() {
            return Some(_the_question(text, &word));
        }
    }
    as_reply(crate::memory::doubt::reply_for(text, memory))
}
