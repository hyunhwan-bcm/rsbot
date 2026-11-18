//! Port of `bot/memory/not_a_trip.py`: a "going to ..." line that names a doing ("going to
//! wait till next year", "going to cry"), an everyday place ("going to uni"), or a person ("going
//! to my sisters on saturday") rather than a trip.

use super::mem8_py::{rx, strset};
use crate::py::re::Match;
use crate::py::text;

const M: &str = "bot.memory.not_a_trip.";

strset!(DETERMINER, BELONGS_TO_THE_PLACE, PARTICLE, DOING_AFTER_TO, ORDINARY, DOING);
rx!(VISIT_TO, THEIR_PERSON, GOING_BEFORE, FILLER_BEFORE, IN_A_CARING_PLACE);

const QUOTED: &str = " ,.;:!?'\u{2019}";
const PLAIN: &str = " ,.;:!?";

/// Python: bot/memory/not_a_trip.py:101 `a_doing_after_to(going_words)`
///
/// True when the words after "to" are a verb and its object ("start a small business").
pub fn a_doing_after_to(going_words: &[String]) -> bool {
    let mut kept: Vec<&String> = going_words.iter().filter(|w| !text::strip_chars(w, QUOTED).is_empty()).collect();
    if !kept.is_empty() && BELONGS_TO_THE_PLACE.contains(text::strip_chars(&kept[0].to_lowercase(), PLAIN)) {
        kept.remove(0);
    }
    if kept.is_empty() {
        return false;
    }
    let first = kept[0].to_lowercase();
    let first = text::strip_chars(&first, QUOTED);
    if DOING_AFTER_TO.contains(first) || DOING.contains(first) {
        // A verb is a doing whatever follows it. Nothing after it makes it a place.
        return true;
    }
    if kept.len() < 2 {
        return false;
    }
    let second = kept[1].to_lowercase();
    let second = text::strip_chars(&second, QUOTED);
    DETERMINER.contains(second) || PARTICLE.contains(second)
}

/// Python: bot/memory/not_a_trip.py:120 `a_place_of_ordinary_life(place)`
///
/// True when the place is where the person already goes, not somewhere they are going.
pub fn a_place_of_ordinary_life(place: &str) -> bool {
    let mut words: Vec<String> = text::split_on(place, " ")
        .iter()
        .map(|w| text::strip_chars(&w.to_lowercase(), QUOTED).to_string())
        .filter(|w| !w.is_empty())
        .collect();
    if !words.is_empty() && BELONGS_TO_THE_PLACE.contains(&words[0]) {
        words.remove(0);
    }
    !words.is_empty() && ORDINARY.contains(&words[0])
}

/// Python: bot/memory/not_a_trip.py:129 `not_a_destination(going_words, place)`
///
/// True when a "going to ..." line names a doing or an everyday place, not a trip.
pub fn not_a_destination(going_words: &[String], place: &str) -> bool {
    a_doing_after_to(going_words) || a_place_of_ordinary_life(place)
}

/// Python: bot/memory/not_a_trip.py:158 `_the_person_in(found)`
///
/// Who a match of VISIT_TO / THEIR_PERSON names, said the way the trip reader stores a place:
/// "my brothers", "my in-laws", "my parents house".
pub fn _the_person_in(found: &Match) -> String {
    let mut who = text::squash(&found.name_str("person").to_lowercase());
    if who.starts_with("in") {
        // "my in laws" is one relation, written as a hyphen.
        who = "in-laws".to_string();
    }
    let said = format!("{} {}", found.name_str("own").to_lowercase(), who);
    let thing = found.name_str("thing").to_lowercase();
    if thing.is_empty() { said } else { format!("{said} {thing}") }
}

/// Python: bot/memory/not_a_trip.py:172 `a_person(place)`
///
/// The person when a destination is only a person ("my aunts"), or "" when it is a place.
pub fn a_person(place: &str) -> String {
    THEIR_PERSON.match_(place).map(|found| _the_person_in(&found)).unwrap_or_default()
}

/// Python: bot/memory/not_a_trip.py:199 `the_visit(line)`
///
/// The person a line is going to see, or "" when the words after "to" name a place.
pub fn the_visit(line: &str) -> String {
    if IN_A_CARING_PLACE.is_match(line) {
        return String::new();
    }
    let Some(found) = VISIT_TO.search(line) else { return String::new() };
    let before = &line[..found.start()];
    if !(GOING_BEFORE.is_match(before) || FILLER_BEFORE.match_(before).is_some()) {
        return String::new();
    }
    _the_person_in(&found)
}
