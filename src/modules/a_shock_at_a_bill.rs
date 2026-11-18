//! Port of `bot/modules/a_shock_at_a_bill.py`: "i nearly died when i saw the bill": a body ending said of a page is about the page, and gets the bill's answer.
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
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.a_shock_at_a_bill.", stringify!($name))));
    )*};
}
macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.a_shock_at_a_bill.", stringify!($name))));
    )*};
}
macro_rules! wordset {
    ($($name:ident),*) => {$(
        static $name: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(concat!("bot.modules.a_shock_at_a_bill.", stringify!($name))));
    )*};
}
rx!(A_BODY_ENDING, A_SHOCK_AT_A_PAGE, A_DOCUMENT, _NOT_YET, A_PAGE_VERB, _ONLY_THE_PAGE, _A_WENT_OUT_COLD, _A_STATE_OF_A_BODY, _A_FIGURE, _A_SOMEBODY_ELSE, A_MONEY_PROBLEM, A_MONEY_IN_THE_ANSWER);
pool!(_BILL_LINES, _WITHOUT_A_FIGURE, _BEFORE_IT_COMES, A_BILL_WITH_ITS_OWN_CARD);
wordset!(_A_SYMPTOM, _A_BODY_WORD, _ALSO_A_COMPLAINT, _A_ACCOUNT_WORD, _A_WORD_ABOUT_ANY_BILL, _A_WORD_BEFORE_A_KIND);
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// The last `n` characters of `s` (`s[-n:]`).
fn tail_chars(s: &str, n: usize) -> &str {
    &s[text::byte_at(s, s.chars().count().saturating_sub(n))..]
}

/// Python: bot/modules/a_shock_at_a_bill.py `_a_complaint_added_after_the_ending(text)`
///
/// True when a real complaint is said after the figure of speech, which is the answer needed.
/// The words are read only after the ending, so the ending's own words are not the complaint.
pub fn _a_complaint_added_after_the_ending(text: &str) -> bool {
    let line = fold(text);
    let Some(said) = A_BODY_ENDING.search(&line) else { return false };
    let tail = &line[said.end()..];
    let after: HashSet<String> = WORDS.findall_str(tail).into_iter().collect();
    if after.iter().any(|word| _A_SYMPTOM.contains(word) || _ALSO_A_COMPLAINT.contains(word)) {
        return true;
    }
    if _A_WENT_OUT_COLD.is_match(tail) {
        return true;
    }
    after.iter().any(|word| _A_BODY_WORD.contains(word)) && _A_STATE_OF_A_BODY.is_match(tail)
}

/// Python: bot/modules/a_shock_at_a_bill.py `_the_page_in_hand(line)`
///
/// True when the line is only somebody holding or looking at a bill, statement or invoice.
pub fn _the_page_in_hand(line: &str) -> bool {
    _ONLY_THE_PAGE.match_(line).is_some() && !_NOT_YET.is_match(line)
}

/// Python: bot/modules/a_shock_at_a_bill.py `_the_ending_and_the_page(text)`
///
/// True when a body ending and a document are said of each other. Two orders count: the ending,
/// then the page; and the page, then the ending, with a page verb in front of the document.
pub fn _the_ending_and_the_page(text: &str) -> bool {
    let line = text::squash(text);
    if line.is_empty() {
        return false;
    }
    if _the_page_in_hand(&line) {
        return true;
    }
    if _NOT_YET.is_match(&line) && A_DOCUMENT.is_match(&line) && A_BODY_ENDING.is_match(&line) {
        return true;
    }
    if A_SHOCK_AT_A_PAGE.is_match(&line) {
        return A_DOCUMENT.is_match(&line);
    }
    let Some(said) = A_BODY_ENDING.search(&line) else { return false };
    if A_DOCUMENT.search_at(&line, said.end()).is_some() {
        return true;
    }
    let Some(document) = A_DOCUMENT.search(&line) else { return false };
    let before = &line[..document.start()];
    A_PAGE_VERB.is_match(tail_chars(before, 24))
}

/// Python: bot/modules/a_shock_at_a_bill.py `_pick(lines, memory)`
///
/// Two wordings, so a second bill ask does not repeat the first reply word for word.
pub fn _pick(lines: &[String], memory: &Memory) -> String {
    let last = last_bot(memory);
    lines.iter().find(|line| **line != last).unwrap_or(&lines[0]).clone()
}

/// Python: bot/modules/a_shock_at_a_bill.py `_the_ending_is_somebody_elses(text)`
///
/// True when the body ending is said of another person, whose news is the thing to answer.
pub fn _the_ending_is_somebody_elses(text: &str) -> bool {
    let line = fold(text);
    let Some(said) = A_BODY_ENDING.search(&line) else { return false };
    _A_SOMEBODY_ELSE.is_match(&line[..said.start()])
}

/// Python: bot/modules/a_shock_at_a_bill.py `_a_kind_word_in_front_of_the_page(text)`
///
/// The kind word the person put in front of a bill/statement/invoice, or "".
pub fn _a_kind_word_in_front_of_the_page(text: &str) -> String {
    let line = fold(text);
    for document in A_DOCUMENT.finditer(&line) {
        // `line[max(0, document.start() - 40):document.start()]`, in characters
        let before = WORDS.findall_str(tail_chars(&line[..document.start()], 40));
        for word in before.iter().rev() {
            if _A_WORD_ABOUT_ANY_BILL.contains(word) {
                continue;
            }
            if _A_WORD_BEFORE_A_KIND.contains(word) || _A_ACCOUNT_WORD.contains(word) {
                return String::new();
            }
            return word.clone();
        }
    }
    String::new()
}

/// Python: bot/modules/a_shock_at_a_bill.py `_a_bill_of_another_kind(text)`
///
/// True when the line names a kind of bill whose own card keeps the turn.
pub fn _a_bill_of_another_kind(text: &str) -> bool {
    let low = fold(text);
    !_a_kind_word_in_front_of_the_page(text).is_empty() || A_BILL_WITH_ITS_OWN_CARD.iter().any(|kind| low.contains(kind.as_str()))
}

/// Python: bot/modules/a_shock_at_a_bill.py `TAKES_AS_IT_COMES(text, memory=None)`
///
/// This line, because the module read it whole: a body ending and a document after it. The
/// module says which of its own proposals is this one, on the line in front of it.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    if !_the_ending_and_the_page(text) {
        return false;
    }
    if _a_complaint_added_after_the_ending(text) {
        return false;
    }
    if _the_ending_is_somebody_elses(text) {
        return false;
    }
    if _a_money_problem_in_the_line(text, &mut memory.clone()) {
        return false;
    }
    !_a_bill_of_another_kind(text)
}

/// Python: bot/modules/a_shock_at_a_bill.py `_ask_for_what_is_missing(memory, text)`
///
/// The page-check, asking for the thing the line has not already said.
pub fn _ask_for_what_is_missing(memory: &Memory, text: &str) -> String {
    let line = text::squash(text);
    if _NOT_YET.is_match(&line) {
        return _pick(&_BEFORE_IT_COMES, memory);
    }
    let lines: &[String] = if _A_FIGURE.is_match(&line) { &_WITHOUT_A_FIGURE } else { &_BILL_LINES };
    _pick(lines, memory)
}

/// Python: bot/modules/a_shock_at_a_bill.py `_a_money_problem_in_the_line(text, memory=None)`
///
/// True when the line names a money trouble *and* the knowledge base answers about the money.
/// Both halves are needed.
pub fn _a_money_problem_in_the_line(text: &str, memory: &mut Memory) -> bool {
    if !A_MONEY_PROBLEM.is_match(&fold(text)) {
        return false;
    }
    // a knowledge base that cannot be read adds no stand-down
    match py_try(|| super::answers::respond(text, memory)) {
        Ok(Some(reply)) => !reply.is_empty() && A_MONEY_IN_THE_ANSWER.is_match(&reply),
        _ => false,
    }
}

/// Python: bot/modules/a_shock_at_a_bill.py `respond(text, memory=None)` - a runtime hook; its signature is the runtime's.
///
/// The bill's answer to a line that says a shock at a bill, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    if !_the_ending_and_the_page(text) {
        return None;
    }
    if _a_complaint_added_after_the_ending(text) {
        return None;
    }
    if _the_ending_is_somebody_elses(text) {
        return None;
    }
    if _a_money_problem_in_the_line(text, memory) {
        return None;
    }
    if _a_bill_of_another_kind(text) {
        return None;
    }
    // a module that cannot answer adds nothing
    or_on_raise(|| Some(_ask_for_what_is_missing(memory, text)), None)
}
