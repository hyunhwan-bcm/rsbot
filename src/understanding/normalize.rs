//! Port of `bot/understanding/normalize.py`: the line cleaned, respelled and cut into clauses.

use crate::understood::{Kept, Signals};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::data;
use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;

/// What `normalise(text)` returns (Python's dict, same keys).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Line {
    pub text: String,
    pub clean: String,
    pub words: Vec<String>,
    pub plain: String,
    pub norm: Vec<String>,
    pub norm_text: String,
    pub fixes: IndexMap<String, String>,
    pub intensifier: Vec<String>,
    pub intensifier_left: Vec<String>,
    pub clauses: Vec<Vec<String>>,
    /// None when there is no clause at all.
    pub focus: Option<Vec<String>>,
    pub asking: Vec<String>,
    pub signals: Signals,
    pub dropped_subject: bool,
    pub change_of_subject: bool,
    pub kept: Option<Kept>,
}

/// `normalise(text)`.
pub fn normalise(text: &str) -> Line {
    let _ = text;
    todo!("normalize::normalise")
}

/// `without_the_figure_of_speech(clean)`.
pub fn without_the_figure_of_speech(clean: &str) -> String {
    let _ = clean;
    todo!("normalize::without_the_figure_of_speech")
}

/// `_the_part_that_hurts(clean)`.
pub fn _the_part_that_hurts(clean: &str) -> String {
    let _ = clean;
    todo!("normalize::_the_part_that_hurts")
}

/// `_the_sugar_that_is_blood(clean)`.
pub fn _the_sugar_that_is_blood(clean: &str) -> String {
    let _ = clean;
    todo!("normalize::_the_sugar_that_is_blood")
}

const M: &str = "bot.understanding.normalize.";
const LEX: &str = "bot.understanding.lexicon.";

fn set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn lex_set(name: &str) -> HashSet<String> {
    data::set(&format!("{LEX}{name}"))
}

static LEADERS: LazyLock<HashSet<String>> = LazyLock::new(|| set("LEADERS"));
static REPORTING: LazyLock<HashSet<String>> = LazyLock::new(|| set("REPORTING"));
static REASON_STARTS: LazyLock<HashSet<String>> = LazyLock::new(|| set("REASON_STARTS"));
static _WH_QUESTIONS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_WH_QUESTIONS"));
static _PLACE_LEADS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PLACE_LEADS"));
static _PLACE_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PLACE_STOP"));
static AUX_OPENS: LazyLock<HashSet<String>> = LazyLock::new(|| set("AUX_OPENS"));
static _SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_SUBJECTS"));
static _PREDICATE_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PREDICATE_ONLY"));
static _SUBJECTLESS_AUX: LazyLock<HashSet<String>> = LazyLock::new(|| set("_SUBJECTLESS_AUX"));
static _STATE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| set("_STATE_FILLER"));
static _A_STATE_OF_THE_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_STATE_OF_THE_PERSON"));
static _STATE_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| set("_STATE_TAIL"));
static _HAVE_NEGATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_HAVE_NEGATIONS"));
static _HAVE_PARTICIPLES: LazyLock<HashSet<String>> = LazyLock::new(|| set("_HAVE_PARTICIPLES"));
static _HAVE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| set("_HAVE_THING"));
static _HAVE_ASKING: LazyLock<HashSet<String>> = LazyLock::new(|| set("_HAVE_ASKING"));
static TIME_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("TIME_WORDS"));
static _SUBJECTLESS_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| set("_SUBJECTLESS_ONLY"));
static _AFTER_PARTICIPLE: LazyLock<HashSet<String>> = LazyLock::new(|| set("_AFTER_PARTICIPLE"));
static CLAUSE_BREAKS: LazyLock<HashSet<String>> = LazyLock::new(|| lex_set("CLAUSE_BREAKS"));
static BACKGROUND_STARTS: LazyLock<HashSet<String>> = LazyLock::new(|| lex_set("BACKGROUND_STARTS"));
static PLACE_PREPS: LazyLock<HashSet<String>> = LazyLock::new(|| lex_set("PLACE_PREPS"));
static START_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| lex_set("START_VERBS"));

/// `strip_leaders(clause)`: the clause without its leading filler words ("so", "well", ...),
/// always keeping the last word.
pub fn strip_leaders(clause: &[String]) -> &[String] {
    let mut start = 0;
    while start + 1 < clause.len() && LEADERS.contains(&clause[start]) {
        start += 1;
    }
    &clause[start..]
}

/// `_clause_stop(norm, index)`: where the clause that starts at `index` ends.
fn _clause_stop(norm: &[String], index: usize) -> usize {
    let mut stop = index + 1;
    while stop < norm.len() && !CLAUSE_BREAKS.contains(&norm[stop]) && !_WH_QUESTIONS.contains(&norm[stop]) {
        stop += 1;
    }
    stop
}

/// `background_clause(norm)`: the positions of the first background clause ("when i was ...").
pub fn background_clause(norm: &[String]) -> BTreeSet<usize> {
    for (index, word) in norm.iter().enumerate() {
        if index == 0 || !BACKGROUND_STARTS.contains(word) {
            continue;
        }
        if REPORTING.contains(&norm[index - 1]) {
            continue;
        }
        return (index.._clause_stop(norm, index)).collect();
    }
    BTreeSet::new()
}

/// `reason_positions(norm)`: the positions of the first reason clause ("because ...").
pub fn reason_positions(norm: &[String]) -> BTreeSet<usize> {
    for (index, word) in norm.iter().enumerate() {
        if index == 0 || !REASON_STARTS.contains(word) {
            continue;
        }
        return (index.._clause_stop(norm, index)).collect();
    }
    BTreeSet::new()
}

/// `background_positions(norm)`: the place words inside the background of the line, which say
/// where it happened rather than what is asked.
pub fn background_positions(norm: &[String]) -> BTreeSet<usize> {
    let mut skip = BTreeSet::new();
    let mut spans: Vec<(usize, usize, bool)> = Vec::new();
    if norm.first().map(String::as_str) == Some("it") && norm.len() > 1 && START_VERBS.contains(&norm[1]) {
        spans.push((0, _clause_stop(norm, 1), true));
    }
    let clause = background_clause(norm);
    if let (Some(lo), Some(hi)) = (clause.first(), clause.last()) {
        spans.push((*lo, hi + 1, false));
    }
    for (start, stop, anywhere) in spans {
        let mut index = start;
        while index < stop {
            if !PLACE_PREPS.contains(&norm[index]) {
                index += 1;
                continue;
            }
            if !anywhere && index > 0 && !(_PLACE_LEADS.contains(&norm[index - 1]) || skip.contains(&(index - 1))) {
                index += 1;
                continue;
            }
            index += 1;
            let mut taken = 0;
            while index < stop
                && taken < 3
                && !CLAUSE_BREAKS.contains(&norm[index])
                && !_PLACE_STOP.contains(&norm[index])
                && !PLACE_PREPS.contains(&norm[index])
            {
                skip.insert(index);
                index += 1;
                taken += 1;
            }
        }
    }
    skip
}

/// `opens_without_subject(words, index)`: an auxiliary at `index` that opens a clause whose
/// subject was left out ("am going to the shops", "have been feeling low").
pub fn opens_without_subject(words: &[String], index: usize) -> bool {
    if index >= words.len() || words[index].is_empty() {
        return false;
    }
    let word = words[index].as_str();
    if !AUX_OPENS.contains(word) {
        return false;
    }
    let rest: Vec<&str> = words[index + 1..].iter().map(String::as_str).filter(|w| *w != "|").collect();
    if rest.is_empty() {
        return false;
    }
    let second = rest[0];
    if _SUBJECTS.contains(second) {
        return false;
    }
    if word == "am" {
        return true;
    }
    if _PREDICATE_ONLY.contains(second) {
        return true;
    }
    if _SUBJECTLESS_AUX.contains(word) {
        let clause: Vec<&str> = words[index + 1..].iter().map(String::as_str).take_while(|w| *w != "|").collect();
        let mut at = 0;
        while at < clause.len() && _STATE_FILLER.contains(clause[at]) {
            at += 1;
        }
        if at < clause.len()
            && _A_STATE_OF_THE_PERSON.contains(clause[at])
            && (at + 1 == clause.len() || _STATE_TAIL.contains(clause[at + 1]) || _STATE_FILLER.contains(clause[at + 1]))
        {
            return true;
        }
    }
    if _HAVE_NEGATIONS.contains(second) && word == "have" {
        return true;
    }
    if word == "have" && _HAVE_PARTICIPLES.contains(second) && rest.len() > 2 {
        return _HAVE_THING.contains(rest[1]) && !_HAVE_ASKING.contains(rest[2]) && !rest[1..].iter().any(|w| TIME_WORDS.contains(*w));
    }
    if crate::py::text::len(second) > 4 && (second.ends_with("ing") || second.ends_with("ed")) {
        if rest.len() < 2 {
            return true;
        }
        if _SUBJECTLESS_ONLY.contains(word) {
            return _AFTER_PARTICIPLE.contains(rest[1]);
        }
        let after = if rest.len() > 2 { rest[2] } else { "" };
        return _AFTER_PARTICIPLE.contains(rest[1]) && (after.ends_with("ing") || after.ends_with("ed"));
    }
    false
}
