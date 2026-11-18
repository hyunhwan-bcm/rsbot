//! Port of `bot/compose/aboutness.py`: what a line is about (its content words), whether a reply
//! names it, and which candidate answers the subject when the chosen reply does not.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::runtime::Candidate;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

const M: &str = "bot.compose.aboutness.";
static FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}FRAME")));
static HEDGE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}HEDGE")));
static STEM: LazyLock<usize> = LazyLock::new(|| data::get(&format!("{M}STEM")).as_int().unwrap() as usize);
static RESTATEMENT: LazyLock<f64> = LazyLock::new(|| data::get(&format!("{M}RESTATEMENT")).as_f64().unwrap());
static _WORDS: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_WORDS")));
static CHANGES_THE_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}CHANGES_THE_SUBJECT")));

/// `set(_WORDS.findall(text.lower()))`, first seen first.
fn word_set(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in _WORDS.findall_str(&text::lower(text)) {
        if !out.contains(&word) {
            out.push(word);
        }
    }
    out
}

/// `word[:STEM]` in characters.
fn stem(word: &str) -> String {
    word.chars().take(*STEM).collect()
}

fn set_value(words: Vec<String>) -> Value {
    Value::Set(Arc::new(words.into_iter().map(Value::from).collect()))
}

/// Python: bot/compose/aboutness.py:64 `after_a_change_of_subject(text)`
///
/// What the line is about after the marker that puts the earlier subject down, or "".
pub fn after_a_change_of_subject(text: &str) -> String {
    let mut tail = "";
    for found in CHANGES_THE_SUBJECT.finditer(text) {
        tail = &text[found.end()..];
    }
    let tail = text::strip_chars(tail, " \t,.;:!-\u{2014}-");
    if text::split_ref(tail).len() < 2 {
        return String::new();
    }
    tail.to_string()
}

/// Python: bot/compose/aboutness.py:80 `subject_words(text)`
///
/// The words the line is about, as a set (here a list, first seen first): content words minus
/// the frame and the light verbs.
pub fn subject_words(text: &str) -> Vec<String> {
    word_set(text).into_iter().filter(|word| text::len(word) >= 4 && !FRAME.contains(word)).collect()
}

/// Python: bot/compose/aboutness.py:86 `names_the_subject(reply, about)`
///
/// Which of `about` the reply actually says, by shared stem.
pub fn names_the_subject(reply: &str, about: &[String]) -> Vec<String> {
    let said = _WORDS.findall_str(&text::lower(reply));
    let mut found: Vec<String> = Vec::new();
    for word in about {
        let mine = stem(word);
        if said.iter().any(|other| text::len(other) >= 4 && stem(other) == mine) && !found.contains(word) {
            found.push(word.clone());
        }
    }
    found
}

/// Python: bot/compose/aboutness.py:98 `restates_the_line(reply, text)`
///
/// True when the reply is only the line given back.
pub fn restates_the_line(reply: &str, text: &str) -> bool {
    let said = word_set(reply);
    let mine = word_set(text);
    if mine.is_empty() {
        return false;
    }
    let shared = mine.iter().filter(|w| said.contains(w)).count();
    shared as f64 / mine.len() as f64 >= *RESTATEMENT
}

/// Python: bot/compose/aboutness.py:105 `is_a_hedge(reply)`
///
/// True when the reply is about the bot's knowledge rather than the line.
pub fn is_a_hedge(reply: &str) -> bool {
    HEDGE.is_match(reply)
}

/// `unlike_the_line` with its two sets as lists.
pub fn unlike_the_line_words(reply: &str, text: &str) -> (Vec<String>, Vec<String>) {
    let about = subject_words(text);
    if about.is_empty() {
        return (about, Vec::new());
    }
    if restates_the_line(reply, text) || is_a_hedge(reply) {
        return (about, Vec::new());
    }
    let said = names_the_subject(reply, &about);
    (about, said)
}

/// Python: bot/compose/aboutness.py:110 `unlike_the_line(reply, text)`
///
/// `(about, said)` as a tuple of two sets: the words the line is about, and the ones the reply
/// says. `unlike_the_line_words` gives the same as lists.
pub fn unlike_the_line(reply: &str, text: &str) -> Value {
    let (about, said) = unlike_the_line_words(reply, text);
    Value::Tuple(Arc::new(vec![set_value(about), set_value(said)]))
}

/// Python: bot/compose/aboutness.py:123 `the_better_answer(reply, candidates, text, skip=None, need=2)`
///
/// The index of the candidate that names `need` or more of the line's words when `reply` names
/// none, or None. `skip` is None or an index; `need` an int (callers pass 2 for the default).
pub fn the_better_answer(reply: &str, candidates: &[Candidate], text: &str, skip: &Value, need: &Value) -> Value {
    let (about, said) = unlike_the_line_words(reply, text);
    if about.is_empty() || !said.is_empty() {
        return Value::None;
    }
    let skip = skip.as_int();
    let need = need.as_int().unwrap_or(2);
    let mut best: Option<usize> = None;
    let mut best_named = need - 1;
    for (index, candidate) in candidates.iter().enumerate() {
        if skip == Some(index as i64) {
            continue;
        }
        let other = &candidate.reply;
        // `int(priority or 0) < 0`: the fallback band apologises for everything.
        if (candidate.priority.trunc() as i64) < 0 {
            continue;
        }
        if restates_the_line(other, text) || is_a_hedge(other) {
            continue;
        }
        let count = names_the_subject(other, &about).len() as i64;
        if count > best_named {
            best = Some(index);
            best_named = count;
        }
    }
    best.map(|i| Value::Int(i as i64)).unwrap_or(Value::None)
}
