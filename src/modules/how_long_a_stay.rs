//! Port of `bot/modules/how_long_a_stay.py`: a length asked of a stay the chat is on, answered from the length the person gave.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.how_long_a_stay.", stringify!($name))));
    )*};
}
macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.how_long_a_stay.", stringify!($name))));
    )*};
}
rx!(STAY, A_LENGTH, A_LENGTH_SAID, ASKS_A_LENGTH, ASKS_ADVICE);
pool!(A_LENGTH_REPLY, ADVICE, NO_LENGTH);
static GENERIC: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.how_long_a_stay.GENERIC"));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));
static A_SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"[.!?](?:\s|$)", 0));

/// Python: bot/modules/how_long_a_stay.py `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    WORDS.findall_str(&text.to_lowercase())
}

/// Python: bot/modules/how_long_a_stay.py `_the_stay(memory)`
///
/// The line that announced the stay, or "" when the chat has none.
pub fn _the_stay(memory: &Memory) -> String {
    memory.turns.iter().rev().map(|turn| &turn.user).find(|said| STAY.is_match(said)).cloned().unwrap_or_default()
}

/// Python: bot/modules/how_long_a_stay.py `_the_length_said(memory)`
///
/// The length the person gave for the stay ("four days", "the weekend"), or "".
pub fn _the_length_said(memory: &Memory) -> String {
    let stay = _the_stay(memory);
    let Some(found) = A_LENGTH.search(&stay) else { return String::new() };
    if !found.str(1).is_empty() {
        // `"%s %s" % (found.group(1), found.group(2))`
        return format!("{} {}", found.str(1), found.get(2).unwrap_or("None")).to_lowercase();
    }
    text::strip(found.as_str()).to_lowercase()
}

/// Python: bot/modules/how_long_a_stay.py `_the_opening(reply)`
///
/// The first sentence of a reply: the part that answers the question it was given.
pub fn _the_opening(reply: &str) -> String {
    match A_SENTENCE_END.search(reply) {
        Some(cut) => reply[..cut.end()].to_string(),
        None => reply.to_string(),
    }
}

/// Python: bot/modules/how_long_a_stay.py `_the_last_answer(memory)`
///
/// The bot's last reply and the module that gave it.
pub fn _the_last_answer(memory: &Memory) -> (String, String) {
    for turn in memory.turns.iter().rev() {
        if !turn.bot.is_empty() {
            return (turn.bot.clone(), turn.module.clone().unwrap_or_default());
        }
    }
    (String::new(), String::new())
}

/// Python: bot/modules/how_long_a_stay.py `_a_subject_of_its_own(text)`
///
/// Whether the length asked names something of its own ("the drive", "the crumble").
pub fn _a_subject_of_its_own(text: &str) -> bool {
    _words(text).iter().any(|word| !(GENERIC.contains(word) || text::len(word) <= 2) && !text::is_digit(word))
}

/// Python: bot/modules/how_long_a_stay.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply to a length asked of a stay in this chat, or None for every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if memory.turns.is_empty() {
        return None;
    }
    if !ASKS_A_LENGTH.is_match(text) {
        return None;
    }
    if _a_subject_of_its_own(text) {
        // "how long does the drive take" names the drive; that is a question about the drive.
        return None;
    }
    let stay = _the_stay(memory);
    if stay.is_empty() {
        return None;
    }
    let (last, module) = _the_last_answer(memory);
    if last.is_empty() || module != "answers" {
        // The turn before was small talk or a question, not a card about the visit: there is no
        // answer to be read out a second time, and the modules that own those turns keep them.
        return None;
    }
    // The length has to be the answer's own, not a passing word in it: the guest card says "a
    // week goes better when nobody is guessing", which is not a length given to this question,
    // while "Ten minutes." and "Lisbon three full days, Porto two" are and are left alone.
    if A_LENGTH_SAID.is_match(&_the_opening(&last)) {
        return None;
    }
    let length = _the_length_said(memory);
    if !length.is_empty() {
        let said = fmt_named(&pick(&A_LENGTH_REPLY, memory, &length), &[("length", length.as_str())]);
        return Some(cap1(&said));
    }
    if ASKS_ADVICE.is_match(text) {
        return Some(pick(&ADVICE, memory, &stay));
    }
    Some(pick(&NO_LENGTH, memory, &stay))
}
