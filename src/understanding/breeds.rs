//! Port of `bot/understanding/breeds.py`: a breed named is the animal it is ("my labrador" is a
//! dog), so the dog cards answer it.

#![allow(non_upper_case_globals)]

use crate::py::re::Re;
use crate::understanding::{rx, t};
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.understanding.breeds.";

static BREED_KIND: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "BREED_KIND"));
static FRAME_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FRAME_BEFORE"));
const FRAME_REACH: i64 = 4;
static LOOSE_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LOOSE_FRAME"));
static MONEY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MONEY"));
static NOT_A_CREATURE_NEXT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_CREATURE_NEXT"));

/// `_singular(word)`: the table's spelling of a plural breed ("" when it is no breed).
fn _singular(word: &str) -> String {
    if BREED_KIND.contains_key(word) {
        return word.to_string();
    }
    for (end, back) in [("ies", "y"), ("es", ""), ("s", "")] {
        if let Some(stem) = word.strip_suffix(end) {
            let w = format!("{stem}{back}");
            if BREED_KIND.contains_key(&w) {
                return w;
            }
        }
    }
    String::new()
}

/// `the_animal_a_breed_is(words)`: the words with the animal's own word after each breed named
/// as an animal ("my labrador" -> "my labrador dog").
pub fn the_animal_a_breed_is(words: &[String]) -> Vec<String> {
    if words.is_empty() {
        return words.to_vec();
    }
    let low: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let reach = FRAME_REACH as usize;
    let mut filled: Vec<String> = Vec::new();
    let mut index = 0;
    while index < low.len() {
        let mut found = String::new();
        let mut span = 1;
        for length in [4usize, 3, 2] {
            let phrase = low[index..(index + length).min(low.len())].join(" ");
            if crate::py::text::split_ref(&phrase).len() == length {
                if let Some(kind) = BREED_KIND.get(&_singular(&phrase)).filter(|k| !k.is_empty()) {
                    found = kind.clone();
                    span = length;
                    break;
                }
            }
        }
        if found.is_empty() {
            found = BREED_KIND.get(&_singular(&low[index])).cloned().unwrap_or_default();
        }
        let after = low.get(index + span).map(String::as_str).unwrap_or("");
        let frame_words: Vec<&String> = low[index.saturating_sub(reach)..index].iter().filter(|w| FRAME_BEFORE.contains(*w)).collect();
        let mut framed = !frame_words.is_empty();
        if !found.is_empty() && framed {
            if NOT_A_CREATURE_NEXT.contains(after) {
                framed = false;
            } else if frame_words.iter().all(|w| LOOSE_FRAME.contains(*w)) && low.iter().any(|w| MONEY.contains(w)) {
                framed = false;
            }
        }
        if !found.is_empty() && framed {
            filled.extend(words[index..index + span].iter().cloned());
            filled.push(found);
            index += span;
            continue;
        }
        filled.push(words[index].clone());
        index += 1;
    }
    filled
}

/// `a_frame_before(text, start)`: a determiner or a verb of having in front of a breed.
pub fn a_frame_before(text_: &str, start: usize) -> bool {
    let before = rx("[a-z']+").findall_str(&text_[..start.min(text_.len())].to_lowercase());
    before[before.len().saturating_sub(FRAME_REACH as usize)..].iter().any(|w| FRAME_BEFORE.contains(w))
}

/// `the_breed_named(text)`.
pub fn the_breed_named(text_: &str) -> String {
    _singular(&crate::py::text::squash(&text_.to_lowercase()))
}

/// `the_kind_of(word)`: the animal a breed is ("" when none).
pub fn the_kind_of(word: &str) -> String {
    BREED_KIND.get(&the_breed_named(word)).cloned().unwrap_or_default()
}

/// `_alternation()`: every breed as one regex alternation, longest first.
pub fn _alternation() -> String {
    let mut names: Vec<&String> = BREED_KIND.keys().collect();
    names.sort_by_key(|n| (std::cmp::Reverse(crate::py::text::split_ref(n).len()), std::cmp::Reverse(crate::py::text::len(n))));
    names.iter().map(|n| n.replace(' ', r"\s+")).collect::<Vec<_>>().join("|")
}

#[allow(dead_code)]
fn _unused(_: &Re, _: &HashSet<String>, _: &IndexMap<String, String>) {}
