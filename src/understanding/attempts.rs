//! Port of `bot/understanding/attempts.py`: what the person says they already tried.

#![allow(non_upper_case_globals)]

use crate::understanding::{pyuni, rx, t};
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.understanding.attempts.";

static ENDS_THE_ATTEMPT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ENDS_THE_ATTEMPT"));
static HANDS_OVER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HANDS_OVER"));
const LOOK_BACK: i64 = 4;
const MOST_WORDS: i64 = 24;
static NOT_A_CLAUSE_AFTER_AND: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_CLAUSE_AFTER_AND"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING"));
static ONE_MORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ONE_MORE"));
static REPORT_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "REPORT_SUBJECTS"));
static STARTS_A_CLAUSE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STARTS_A_CLAUSE"));
static STARTS_A_QUESTION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STARTS_A_QUESTION"));

/// `_tokens(norm)`.
fn _tokens(norm: &[String]) -> Vec<String> {
    norm.iter().map(|w| crate::py::text::strip(w).to_lowercase()).filter(|w| !w.is_empty()).collect()
}

/// `_a_report(words, at)`: "i tried", "we've tried".
fn _a_report(words: &[String], at: usize) -> bool {
    if at == 0 {
        return true;
    }
    words[at.saturating_sub(LOOK_BACK as usize)..at].iter().any(|w| REPORT_SUBJECTS.contains(w))
}

/// `_where_the_attempt_ends(words, at)`.
fn _where_the_attempt_ends(words: &[String], at: usize) -> usize {
    for index in at + 1..words.len() {
        let word = &words[index];
        if ENDS_THE_ATTEMPT.contains(word) || STARTS_A_CLAUSE.contains(word) {
            return index;
        }
        if word == "and" && (index + 1 == words.len() || NOT_A_CLAUSE_AFTER_AND.contains(&words[index + 1])) {
            return index;
        }
        let one_more_ahead = || words[(index + 1).min(words.len())..(index + 5).min(words.len())].iter().any(|o| ONE_MORE.contains(o));
        if STARTS_A_QUESTION.contains(word) {
            let before = if index > 0 { words[index - 1].as_str() } else { "" };
            if index == at + 1 || matches!(before, "and" | "but" | "so" | "then" | "now" | "everything" | "anything" | "something" | "that" | "this" | "it") || one_more_ahead() {
                return index;
            }
        }
        if HANDS_OVER.contains(word) && one_more_ahead() {
            return index;
        }
    }
    words.len()
}

/// `the_things_they_tried(norm)`: the words of what they tried.
pub fn the_things_they_tried(norm: &[String]) -> Vec<String> {
    let words = _tokens(norm);
    let mut tried: Vec<String> = Vec::new();
    for (at, word) in words.iter().enumerate() {
        if word != "tried" || !_a_report(&words, at) {
            continue;
        }
        for index in at + 1.._where_the_attempt_ends(&words, at) {
            let word = &words[index];
            if NOT_A_THING.contains(word) || crate::py::text::len(word) < 3 || !pyuni::is_alpha(word) {
                continue;
            }
            if !tried.contains(word) {
                tried.push(word.clone());
            }
        }
    }
    tried
}

/// `the_rest_after_the_try(norm)`.
pub fn the_rest_after_the_try(norm: &[String]) -> Vec<String> {
    let words = _tokens(norm);
    for (at, word) in words.iter().enumerate() {
        if word != "tried" || !_a_report(&words, at) {
            continue;
        }
        return words[_where_the_attempt_ends(&words, at)..].to_vec();
    }
    vec![]
}

/// `asks_for_one_more(norm)`: "i've tried x, can you suggest another".
pub fn asks_for_one_more(norm: &[String]) -> bool {
    let rest = the_rest_after_the_try(norm);
    if rest.is_empty() || !rest.iter().any(|w| ONE_MORE.contains(w)) {
        return false;
    }
    if rest.iter().any(|w| HANDS_OVER.contains(w)) {
        return true;
    }
    let joined = rest.join(" ");
    if rx(r"\b(?:can|could|would|will)\s+(?:you|u)\b").is_match(&joined) {
        return true;
    }
    rest.iter().any(|w| STARTS_A_QUESTION.contains(w))
}

/// `things_tried(text, norm=None)`: what the line says was tried already.
pub fn things_tried(text_: &str, norm: Option<&[String]>) -> Vec<String> {
    let words = match norm {
        Some(n) => _tokens(n),
        None => _tokens(&rx("[a-z']+").findall_str(&text_.to_lowercase().replace('\u{2019}', "'"))),
    };
    if words.is_empty() || words.len() as i64 > MOST_WORDS || !words.iter().any(|w| w == "tried") {
        return vec![];
    }
    the_things_they_tried(&words)
}

#[allow(dead_code)]
fn _unused(_: &HashSet<String>) {}
