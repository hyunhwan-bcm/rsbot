//! Port of `bot/modules/what_they_said.py`: what somebody else said, asked back: the figure or the clause the person handed on, from their own line.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::lineback;

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.what_they_said.", stringify!($name))));
    )*};
}
rx!(_A_THIRD_PARTY_ASK, _SOMEBODY_SPOKE, _A_FIGURE, _QUESTION, _REPORTING_VERB, _A_MONEY, _A_UNIT_AFTER);
static _NOT_A_DETAIL: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.what_they_said._NOT_A_DETAIL"));
static _A_KIND: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.modules.what_they_said._A_KIND"));
static _THE_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.what_they_said._THE_PERSON"));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static A_PREPOSITION_BEFORE_A_FIGURE: LazyLock<Re> =
    LazyLock::new(|| Re::new("^(?:for|to|about|of|at)\\s+(?=(?:\u{a3}|\\$|\u{20ac})?\\s?\\d)", pyre::I));
static A_LETTER_OR_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9]", pyre::I));

/// Python: bot/modules/what_they_said.py `_details(text)`
///
/// The content words of a line, as flat words: what it says that is not scaffolding.
pub fn _details(text: &str) -> HashSet<String> {
    WORDS
        .findall_str(&text.to_lowercase())
        .iter()
        .map(|word| text::strip_chars(word, "'").to_string())
        .filter(|word| !_NOT_A_DETAIL.contains(word) && text::len(word) > 1)
        .collect()
}

/// Python: bot/modules/what_they_said.py `_the_kind_of_answer(said)`
///
/// The kind of answer the ask names - "figure" for a price - or "" when it names none.
pub fn _the_kind_of_answer(said: &str) -> String {
    for word in WORDS.findall_str(&said.to_lowercase()) {
        if let Some(kind) = _A_KIND.get(text::strip_chars(&word, "'")) {
            if !kind.is_empty() {
                return kind.clone();
            }
        }
    }
    String::new()
}

/// Python: bot/modules/what_they_said.py `_the_last_somebody_spoke(said, memory)`
///
/// The last earlier line where the person quoted somebody, or None.
pub fn _the_last_somebody_spoke(said: &str, memory: &Memory) -> Option<String> {
    let wanted = _details(said);
    let kind = _the_kind_of_answer(said);
    for line in py_strs(&lineback::the_lines(memory).val()).iter().rev() {
        if _QUESTION.is_match(line) || ret_true(lineback::is_a_recall_ask(line)) {
            continue; // an ask for the thing is not the thing
        }
        if !_SOMEBODY_SPOKE.is_match(line) {
            continue;
        }
        // An ask that names the speaker or the subject ("what did the vet say") shares that word
        // with the line it wants. An ask that names only the kind of answer ("what did they say
        // it would cost") does not have to: the figure in the line is the answer to it.
        if !kind.is_empty() || wanted.is_empty() || !wanted.is_disjoint(&_details(line)) {
            return Some(line.clone());
        }
    }
    None
}

/// Python: bot/modules/what_they_said.py `_the_reported_clause(line)`
///
/// What somebody else said, when the line hands it on in a clause, or "". Only a verb whose
/// subject is somebody other than the person reports somebody else's words.
pub fn _the_reported_clause(line: &str) -> String {
    let mut clause = String::new();
    for m in _REPORTING_VERB.finditer(line) {
        let who_words = text::split(m.name_str("who"));
        let who = text::strip_chars(&who_words[who_words.len() - 1], "',").to_string();
        if _THE_PERSON.contains(&who) {
            continue;
        }
        clause = text::strip(m.str(2)).to_string();
    }
    let clause = text::rstrip_chars(&clause, " .!?,").to_string();
    // "the vet asked for 60" hands on "for 60"; the figure is the thing said, not the preposition.
    let clause = A_PREPOSITION_BEFORE_A_FIGURE.sub("", &clause);
    let words = text::split(&clause);
    if words.is_empty() || words.len() > 14 {
        return String::new(); // a long tail is the rest of the line, not the thing that was said
    }
    if _QUESTION.is_match(&clause) || !A_LETTER_OR_DIGIT.is_match(&clause) {
        return String::new();
    }
    clause
}

/// Python: bot/modules/what_they_said.py `_the_figure_the_line_gave(line)`
///
/// The figure in the line that answers an ask for a price, or "". A bare number is a price only
/// when it is the only figure in the line; where the line holds more than one figure, the money
/// one is the answer, and a line with two of them says nothing for certain.
pub fn _the_figure_the_line_gave(line: &str) -> String {
    let figures: Vec<_> = _A_FIGURE.finditer(line).into_iter().filter(|found| _A_UNIT_AFTER.match_(&line[found.end()..]).is_none()).collect();
    let money: Vec<_> = figures.iter().filter(|found| _A_MONEY.is_match(found.as_str())).collect();
    if money.len() == 1 {
        return text::strip(money[0].as_str()).to_string();
    }
    if money.is_empty() && figures.len() == 1 {
        return text::strip(figures[0].as_str()).to_string();
    }
    String::new()
}

/// Python: bot/modules/what_they_said.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let said = ret_str(lineback::flat(text));
    if said.is_empty() || !_A_THIRD_PARTY_ASK.is_match(&said) {
        return None;
    }
    let line = _the_last_somebody_spoke(&said, memory).filter(|line| !line.is_empty())?;
    if _the_kind_of_answer(&said) == "figure" {
        let figure = _the_figure_the_line_gave(&line);
        if !figure.is_empty() {
            return Some(format!("You said {figure}."));
        }
    }
    let clause = _the_reported_clause(&line);
    if !clause.is_empty() {
        return Some(format!("You said {clause}."));
    }
    // Nothing was handed on in a clause and no figure was quoted: the line is the person's own act
    // ("i told my boss i was quitting") or a figure that is a length, so saying it back is the bot
    // repeating their line at them. The turn goes back to whoever answers the line itself.
    None
}
