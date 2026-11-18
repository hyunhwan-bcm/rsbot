//! Port of `bot/understanding/senses.py`: which sense of a word the line uses ("room" as space
//! or as a room of a house), read from the words around it.

#![allow(non_upper_case_globals)]

use crate::understanding::normalize::Line;
use crate::understanding::{pyuni, t};
use indexmap::IndexMap;
use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.understanding.senses.";

static CUES_AFTER_A_FIGURE: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "CUES_AFTER_A_FIGURE"));
static _A_FIGURE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_FIGURE"));
/// `CUES`: word -> sense -> cue words (Python's dict order).
static CUES: LazyLock<Vec<(String, Vec<(String, Vec<String>)>)>> = LazyLock::new(|| {
    t::value(M, "CUES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.pairs().iter().map(|(s, c)| (s.str().to_string(), c.strs())).collect())).collect()
});

/// `_a_figure_before(line, word)`: "3 degrees".
fn _a_figure_before(line: &Line, word: &str) -> bool {
    let forms = _forms(word);
    for (i, token) in line.norm.iter().enumerate() {
        if !forms.contains(&token.to_lowercase()) || i == 0 {
            continue;
        }
        let before = crate::py::text::strip_chars(&line.norm[i - 1].to_lowercase(), ".,!?;:").to_string();
        if pyuni::is_digit(&before) || _A_FIGURE.contains(&before) {
            return true;
        }
    }
    false
}

/// `_forms(word)`.
fn _forms(word: &str) -> HashSet<String> {
    let mut forms = HashSet::new();
    forms.insert(word.to_string());
    let n = crate::py::text::len(word);
    if n > 3 && word.ends_with('s') && !word.ends_with("ss") {
        forms.insert(word[..word.len() - 1].to_string());
    }
    if n > 5 && word.ends_with("es") {
        forms.insert(word[..word.len() - 2].to_string());
    }
    forms
}

/// `_the_words(line)`: every form of every word of the line.
fn _the_words(line: &Line) -> HashSet<String> {
    let mut words: Vec<String> = line.norm.iter().map(|w| w.to_lowercase()).collect();
    let clean = line.clean.to_lowercase();
    words.extend(crate::py::text::split(&clean));
    words.iter().flat_map(|w| _forms(w)).collect()
}

/// `_how_many_words(cues, words)`.
fn _how_many_words(cues: &[String], words: &HashSet<String>) -> usize {
    let found: HashSet<String> = cues.iter().flat_map(|c| _forms(c).into_iter().filter(|f| words.contains(f))).collect();
    found.len()
}

/// `senses(line)`: `{word: sense}` for each word whose sense the line settles.
pub fn senses(line: &Line) -> IndexMap<String, String> {
    let words = _the_words(line);
    let mut found = IndexMap::new();
    if words.is_empty() {
        return found;
    }
    for (word, by_sense) in CUES.iter() {
        if !_forms(word).iter().any(|f| words.contains(f)) {
            continue;
        }
        // sorted(((count, sense) ...), reverse=True): highest count, then the later sense name
        let mut scored: Vec<(usize, &String)> = by_sense.iter().map(|(sense, cues)| (_how_many_words(cues, &words), sense)).collect();
        scored.sort_by(|a, b| b.cmp(a));
        if scored.len() < 2 {
            continue;
        }
        if scored[0].0 == 0 {
            if let Some(sense) = CUES_AFTER_A_FIGURE.get(word) {
                if _a_figure_before(line, word) {
                    found.insert(word.clone(), sense.clone());
                }
            }
            continue;
        }
        if scored[0].0 == scored[1].0 {
            continue;
        }
        found.insert(word.clone(), scored[0].1.clone());
    }
    found
}

#[allow(dead_code)]
fn _unused(_: &BTreeSet<String>) {}
