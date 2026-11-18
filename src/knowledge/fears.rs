//! Port of `bot/knowledge/fears.py`: a fear or a phobia of a named thing, and the exposure ladder.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;

globals!("bot.knowledge.fears",
    strs { _MARKERS },
    set { _STOP, _LINKS, _DETERMINERS, _TRAILING_FILLER, _AN_ING_NOUN, _PHOBIA_GERUNDS });

/// The feared thing as the person wrote it, or "" when the line names none.
pub fn _the_thing_they_fear(understood: &Understood) -> String {
    let said = text::squash(&understood.clean.to_lowercase());
    let mut found: &str = "";
    let mut at: isize = -1;
    for marker in _MARKERS.iter() {
        let mut pos = text::find(&said, &format!(" {marker} "));
        if pos < 0 {
            if said.starts_with(&format!("{marker} ")) {
                pos = -1;
            } else {
                continue;
            }
        }
        if at < 0 || pos < at || (pos == at && marker.chars().count() > found.chars().count()) {
            at = pos;
            found = marker;
        }
    }
    if found.is_empty() {
        return String::new();
    }
    let rest = text::slice(&said, at + found.chars().count() as isize + 1, None);
    let mut words: Vec<String> = Vec::new();
    let mut doing = false; // the thing so far is a doing, so the thing it is done to is part of it
    for word in text::split_ref(text::strip(&rest)) {
        let word = text::strip_chars(word, ",.!?;:\"'");
        if word.is_empty() {
            break;
        }
        if words.len() >= if doing { 5 } else { 3 } {
            break;
        }
        if _STOP.contains(word) {
            if doing && (_LINKS.contains(word) || _DETERMINERS.contains(word)) {
                // "going to the gym": the doing keeps the thing it is done to
                words.push(word.to_string());
                continue;
            }
            break;
        }
        words.push(word.to_string());
        if words.len() == 1 && word.ends_with("ing") && word.chars().count() > 4 {
            doing = true;
        }
    }
    if words.is_empty() {
        return String::new();
    }
    while words.last().is_some_and(|w| _TRAILING_FILLER.contains(w)) {
        words.pop();
    }
    if words.is_empty() {
        return String::new();
    }
    const LINK_ENDS: [&str; 12] = ["to", "on", "in", "at", "into", "about", "with", "of", "near", "around", "from", "inside"];
    while words.len() > 1 && LINK_ENDS.contains(&words[words.len() - 1].as_str()) {
        words.pop(); // a link with nothing after it is not part of a phrase
    }
    let thing = words.join(" ");
    if ["the", "a", "an", "my", "it", "them", "that", "this"].contains(&thing.as_str()) {
        return String::new();
    }
    thing
}

/// The exposure ladder, opened with the thing they fear when the line names it.
pub fn _the_fear_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let thing = _the_thing_they_fear(understood);
    let opening = if thing.is_empty() {
        "A fear like that is learned, so it can be unlearned".to_string()
    } else {
        format!("A fear of {thing} is learned, so it can be unlearned")
    };
    Some(opening
        + " in small steps: write a ladder from the easiest version of it to the hardest - a photo of it, then the smallest real version, then all of it - and only move up when the step you are on has stopped raising your heart. Pick the easiest rung and stay with it until the wave passes: leaving early is what teaches the fear that it was right. If it is limiting your life, ask the GP for CBT with exposure, which is the treatment with the best evidence.")
}

/// Only a line that names what is feared: a bare "what is a phobia" is the definition card's.
pub fn _names_the_thing(understood: &Understood) -> bool {
    let thing = _the_thing_they_fear(understood);
    if thing.is_empty() {
        return false;
    }
    let first = text::split_ref(&thing)[0];
    if first.ends_with("ing") && first.chars().count() > 4 && !_AN_ING_NOUN.contains(first) && !_PHOBIA_GERUNDS.contains(first) {
        return false;
    }
    true
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.fears:_names_the_thing:179", |_: &FnRef, u: &Understood| _names_the_thing(u))];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.fears:_the_fear_reply:148", |_: &FnRef, m: &mut Memory, u: &Understood| _the_fear_reply(m, u))];
