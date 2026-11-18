//! Port of `bot/memory/reminded.py`: "what was wrong with my boiler again" - a reminder about a
//! thing of theirs, so the chat is what it asks about rather than the world.

use super::mem8_py::{rx, strset};
use super::topics::{content, forms, stem};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;

const M: &str = "bot.memory.reminded.";

rx!(WITH_THE_THING, STATE_ASKS, STATE_SAID, IN_THE_PAST, AGAIN, REMINDING, THEIR_THING, WHAT_IT_DID);
strset!(TRAILING, EMPTY, THING_TAIL);

/// Python: bot/memory/reminded.py:96 `_trim(words)`
///
/// The named thing without the words that point at the remembering rather than the thing.
pub fn _trim(words: &str) -> String {
    let kept: Vec<&str> = text::split_ref(words).into_iter().filter(|w| !TRAILING.contains(&w.to_lowercase())).collect();
    text::strip_chars(&kept.join(" "), " .,!?").to_string()
}

/// `bot.memory.recent(memory, 12)`: the last twelve turns, oldest first.
fn recent12(memory: &Memory) -> &[Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(12)..]
}

/// Whether every word of the thing is named (as itself, a form of it or its stem) in `line`.
fn names_all(words: &[String], line: &str) -> bool {
    let said: HashSet<String> = content(line).into_iter().collect();
    words.iter().all(|word| {
        said.iter().any(|seen| {
            word == seen || !forms(word).is_disjoint(&forms(seen)) || stem(word) == stem(seen)
        })
    })
}

/// Python: bot/memory/reminded.py:102 `_said_before(thing, memory)`
///
/// True when one of their own earlier lines names this thing.
pub fn _said_before(thing: &str, memory: &Memory) -> bool {
    let words: Vec<String> = content(thing).into_iter().filter(|w| !EMPTY.contains(w)).collect();
    if words.is_empty() {
        return false;
    }
    recent12(memory).iter().any(|turn| !turn.user.is_empty() && names_all(&words, &turn.user))
}

/// Python: bot/memory/reminded.py:123 `the_thing(text, memory=None)`
///
/// The thing a reminder is about ("my boiler", "the car", "it"), or None when the line is not a
/// reminder about a thing they named.
pub fn the_thing(text: &str, memory: &Memory) -> Option<String> {
    let line = text::squash(&text.to_lowercase());
    if line.is_empty() {
        return None;
    }
    if !STATE_ASKS.is_match(&line) {
        return None;
    }
    if !(IN_THE_PAST.is_match(&line) || REMINDING.is_match(&line)) {
        return None;
    }
    // Without "again", "remind me" or "i can't remember" the line is the first question.
    if !AGAIN.is_match(&line) {
        return None;
    }
    let thing = if let Some(named) = WITH_THE_THING.search(&line) {
        match named.get(1).filter(|g| !g.is_empty()) {
            Some(group) => _trim(group),
            // A pronoun is the thing the understanding engine resolved.
            None => _resolved(named.str(2), memory),
        }
    } else if let Some(did) = WHAT_IT_DID.search(&line) {
        _trim(did.str(1))
    } else if let Some(theirs) = THEIR_THING.search(&line) {
        let phrase = theirs.get(1).filter(|g| !g.is_empty()).unwrap_or_else(|| theirs.str(2));
        _the_thing_itself(phrase)
    } else {
        String::new()
    };
    if thing.is_empty() || content(&thing).is_empty() {
        return None;
    }
    Some(thing)
}

/// Python: bot/memory/reminded.py:169 `_the_thing_itself(phrase)`
///
/// The thing out of "my bike problem": the phrase without the words that ask about its state.
pub fn _the_thing_itself(phrase: &str) -> String {
    let mut words = text::split_ref(phrase);
    while words.len() > 1 && THING_TAIL.contains(&text::strip_chars(words[words.len() - 1], ".,!?").to_lowercase()) {
        words.pop();
    }
    _trim(&words.join(" "))
}

/// Python: bot/memory/reminded.py:177 `_resolved(pronoun, memory)`
///
/// The thing a pronoun points at, as the understanding engine read it (`understood["ref"]`).
pub fn _resolved(pronoun: &str, memory: &Memory) -> String {
    let Some(read) = &memory.understood else { return String::new() };
    let r = &read.reference;
    let value = match pronoun.to_lowercase().as_str() {
        "it" => r.it.clone(),
        "that" => r.that.clone(),
        "they" => r.they.clone(),
        "there" => r.there.clone(),
        "he" => r.he.clone(),
        "she" => r.she.clone(),
        "kind" => r.kind.clone(),
        _ => None,
    };
    match value {
        Some(v) if !text::strip(&v).is_empty() => _trim(&v),
        _ => String::new(),
    }
}

/// Python: bot/memory/reminded.py:193 `_said_what_was_wrong(thing, memory)`
///
/// True when one of their own lines says something happened to this thing.
pub fn _said_what_was_wrong(thing: &str, memory: &Memory) -> bool {
    let words: Vec<String> = content(thing).into_iter().filter(|w| !EMPTY.contains(w)).collect();
    if words.is_empty() {
        return false;
    }
    recent12(memory)
        .iter()
        .any(|turn| !turn.user.is_empty() && STATE_SAID.is_match(&turn.user) && names_all(&words, &turn.user))
}

/// Python: bot/memory/reminded.py:217 `about_a_thing_they_said(text, memory=None)`
///
/// True when the line is a reminder about a thing of theirs. A reading that fails claims nothing.
pub fn about_a_thing_they_said(text: &str, memory: &Memory) -> bool {
    crate::runtime::guarded(|| match the_thing(text, memory) {
        Some(thing) => _said_before(&thing, memory) && _said_what_was_wrong(&thing, memory),
        None => false,
    })
    .unwrap_or(false)
}
