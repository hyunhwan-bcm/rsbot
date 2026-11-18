//! Port of `bot/modules/hardpast.py`: a hard thing in the past, told flat, gets what a friend says to it.
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
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.hardpast.", stringify!($name))));
    )*};
}
rx!(A_HARD_PAST, A_THE_PAST, A_NEWS_ARRIVING, A_CLOSED, A_LOSS);
static A_OWNED: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.hardpast.A_OWNED"));
static A_BETWEEN: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.hardpast.A_BETWEEN"));
static A_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.hardpast.A_SUBJECT"));
static A_HARD_PAST_SAID: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.hardpast.A_HARD_PAST_SAID"));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Python: bot/modules/hardpast.py `_hangs_on_the_person(clean, at)`
///
/// True when the hard thing starting at `at` is the person's own rather than the world's. (`at`
/// is a byte offset here: the start of a match in `clean`.)
pub fn _hangs_on_the_person(clean: &str, at: usize) -> bool {
    let found = WORDS.findall_str(&clean[..at]);
    let before = &found[found.len().saturating_sub(6)..];
    for word in &before[before.len().saturating_sub(4)..] {
        if A_OWNED.contains(word) || (word.ends_with("'s") && A_OWNED.contains(&word[..word.len() - 2])) {
            return true;
        }
    }
    let low: Vec<String> = before.iter().map(|word| word.replace('\'', "")).collect();
    match low.iter().rposition(|word| A_SUBJECT.contains(word)) {
        Some(seen) => low[seen + 1..].iter().all(|word| A_BETWEEN.contains(word)),
        None => false,
    }
}

/// Python: bot/modules/hardpast.py `_the_hard_thing(text, memory=None)`
///
/// The tidied line when it is a hard thing in the past told flat, else "".
pub fn _the_hard_thing(text: &str, memory: &Memory) -> String {
    let clean = text::squash(&text.to_lowercase().replace('\u{2019}', "'"));
    if clean.is_empty() || clean.contains('?') {
        return String::new();
    }
    // A line that asks something is not a line that tells it. `kind` is the understanding
    // engine's reading of the whole line, so it catches the ask typed without a question mark
    // ("is my dad an alcoholic", "was your dad an alcoholic too") that a "?" test cannot.
    if memory.understood.as_ref().map_or(false, |understood| understood.kind == "question") {
        return String::new();
    }
    if !A_HARD_PAST.is_match(&clean) {
        return String::new();
    }
    if !A_HARD_PAST.finditer(&clean).iter().any(|found| _hangs_on_the_person(&clean, found.start())) {
        return String::new();
    }
    if A_NEWS_ARRIVING.is_match(&clean) || !A_THE_PAST.is_match(&clean) {
        return String::new();
    }
    if A_LOSS.is_match(&clean) {
        return String::new();
    }
    if A_CLOSED.is_match(&clean) {
        return String::new();
    }
    clean
}

/// Python: bot/modules/hardpast.py `takes_as_it_comes(text, memory)`
///
/// The claim is one reading of the whole line, so it is declared as well as banded.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| !_the_hard_thing(text, memory).is_empty(), false))
}

/// Python: bot/modules/hardpast.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// What a friend says to a hard thing from the past, or None on any other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a reading that fails answers the line as it did before
    or_on_raise(
        || {
            let clean = _the_hard_thing(text, memory);
            if clean.is_empty() {
                return None;
            }
            Some(pick(&A_HARD_PAST_SAID, memory, &clean))
        },
        None,
    )
}
