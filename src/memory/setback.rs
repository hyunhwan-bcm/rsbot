//! Port of `bot/memory/setback.py`: a vague setback ("the cake went wrong", "work is killing
//! me", "i messed it up") answered with the question it leaves open, in the person's own words
//! for the thing.

use super::mem8_py::{int, rx, strs, strset};
use crate::py::re::Re;
use crate::py::text;
use std::sync::LazyLock;

const M: &str = "bot.memory.setback.";

rx!(VAGUE, NOW, HELPLESS, BLAME, OWNER);
strset!(MODIFIER, YOURS, NOT_A_THING, PEOPLE, GOES_ON, JOINERS, NOT_A_DOER);
strs!(ABOUT_THING, ABOUT_OCCASION, ONGOING, WHOLE, WHOLE_NOW, _NOT_A_PLURAL_ENDING);
int!(MORE_THAN);

static NAMED_AFTER: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:the|my|our|this|that)\s+(?:whole\s+)?([a-z][a-z'-]{1,18})(?:\s+([a-z][a-z'-]{1,18}))?", 0)
});

/// Python: bot/memory/setback.py:188 `_flat(text)`
pub fn _flat(text: &str) -> String {
    text::squash(&text.to_lowercase().replace('\u{2019}', "'"))
}

/// Python: bot/memory/setback.py:192 `_thing(text)`
///
/// The person's own words for the thing that went wrong, or "" for a day or nothing.
pub fn _thing(text: &str) -> String {
    if let Some(found) = OWNER.finditer(text).into_iter().last() {
        let qualifier = found.str(1);
        let mut word = found.str(2).to_string();
        let mut after = found.str(3).to_string();
        if (word == "whole" || word == "entire") && !after.is_empty() {
            // "my whole kitchen" is the kitchen.
            word = std::mem::take(&mut after);
        } else if MODIFIER.contains(&word) && !after.is_empty() {
            word = format!("{word} {after}");
            after.clear();
        } else if JOINERS.contains(&after) {
            // The word after it starts the rest of the sentence.
            after.clear();
        }
        if NOT_A_THING.contains(&word) {
            return String::new();
        }
        let owner = if YOURS.contains(qualifier) { "your" } else { "the" };
        let said = if after.is_empty() { format!("{owner} {word}") } else { format!("{owner} {word} {after}") };
        return text::strip(&said).to_string();
    }
    String::new()
}

/// Python: bot/memory/setback.py:212 `_named_after(text)`
///
/// The thing after a blaming verb: "i messed up the cake" -> "your cake".
pub fn _named_after(text: &str) -> String {
    let Some(found) = NAMED_AFTER.search(text) else { return String::new() };
    let mut word = found.str(1).to_string();
    let mut after = found.str(2).to_string();
    if MODIFIER.contains(&word) && !after.is_empty() {
        word = format!("{word} {after}");
        after.clear();
    } else if JOINERS.contains(&after) {
        after.clear();
    }
    if NOT_A_THING.contains(&word) {
        return String::new();
    }
    if after.is_empty() { format!("your {word}") } else { format!("your {word} {after}") }
}

/// Python: bot/memory/setback.py:228 `_bare_subject(head)`
///
/// The thing a line names with no "my"/"the" in front of it: "work is killing me".
pub fn _bare_subject(head: &str) -> String {
    let mut words = text::split_ref(head);
    while words.last().is_some_and(|w| ["is", "are", "was", "were", "been", "has", "have", "had", "really", "just"].contains(w)) {
        words.pop();
    }
    let Some(word) = words.last() else { return String::new() };
    if NOT_A_THING.contains(*word) || ["i", "im", "it", "that", "this", "there", "he", "she", "they", "we", "you"].contains(word) {
        return String::new();
    }
    word.to_string()
}

/// Python: bot/memory/setback.py:245 `_refused(name)`
///
/// Whether the thing the line names is a person, not a thing to ask about.
pub fn _refused(name: &str) -> bool {
    text::split_ref(name).last().is_some_and(|last| PEOPLE.contains(*last))
}

/// Python: bot/memory/setback.py:250 `reading(text)`
///
/// (name, "past") | (name, "now") for a setback or a thing wearing on them, or None.
pub fn reading(text: &str) -> Option<(String, String)> {
    let clean = _flat(text);
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    for (pattern, tense, which) in [(*VAGUE, "past", 0), (*NOW, "now", 1), (*BLAME, "past", 2)] {
        let Some(found) = pattern.search(&clean) else { continue };
        let tail = &clean[found.end()..];
        let rest: Vec<String> = if text::split_ref(tail).is_empty() { Vec::new() } else { text::split(&HELPLESS.sub("", tail)) };
        if rest.len() as i64 > *MORE_THAN || rest.iter().any(|w| GOES_ON.contains(text::strip_chars(w, ".,!"))) {
            // The line has gone on to say what happened, or to name the fault.
            return None;
        }
        let mut name = _thing(&clean[..found.start()]);
        if name.is_empty() && which == 2 {
            name = _named_after(tail);
        }
        if name.is_empty() && which == 1 {
            name = _bare_subject(&clean[..found.start()]);
        }
        if _refused(&name) {
            return None;
        }
        return Some((name, tense.to_string()));
    }
    None
}

/// Python: bot/memory/setback.py:288 `_a_plural_thing(name)`
///
/// Whether the phrase for the thing reads as a plural ("the lessons", "my feet").
pub fn _a_plural_thing(name: &str) -> bool {
    let Some(last) = text::split_ref(name).last().map(|w| text::strip_chars(&w.to_lowercase(), ".,!?;:").to_string()) else {
        return false;
    };
    last.ends_with('s') && !text::ends_with_any(&last, &_NOT_A_PLURAL_ENDING)
}

/// Python: bot/memory/setback.py:299 `_agrees_with_the_thing(line, name)`
///
/// The pool line with its verb made to agree with the thing that fills it.
pub fn _agrees_with_the_thing(line: &str, name: &str) -> String {
    if _a_plural_thing(name) {
        return line.replace(" sounds ", " sound ").replace(" it sounds ", " they sound ");
    }
    line.to_string()
}

/// Python: bot/memory/setback.py:306 `pool(text)`
///
/// The replies for a setback, in the person's own thing where the line named one; empty for a
/// line that is not a setback.
pub fn pool(text: &str) -> Vec<String> {
    let Some((name, tense)) = reading(text) else { return Vec::new() };
    if name.is_empty() {
        return if tense == "now" { WHOLE_NOW.clone() } else { WHOLE.clone() };
    }
    if tense == "now" {
        return ONGOING.iter().map(|line| _agrees_with_the_thing(&line.replace("{thing}", &name), &name)).collect();
    }
    let last = text::split_ref(&name).last().map(|s| s.to_string()).unwrap_or_default();
    let lines = if NOT_A_DOER.contains(&last) { &*ABOUT_OCCASION } else { &*ABOUT_THING };
    lines.iter().map(|line| line.replace("{thing}", &name)).collect()
}
