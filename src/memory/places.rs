//! Port of `bot/memory/places.py`: where somebody lives, read out of what they said - and said
//! back.
//!
//! These are the readers the store answers "where does my sister live" from: the town a line
//! gave ("my sister lives in cork"), the pronoun that stands for the person the chat has just been
//! about ("she moved to york"), the correction that moves somebody's town ("no, york"), and the
//! say-back line built from them.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::facts as extract;
use crate::memory::mem7::{self, inline_re, res, sets};
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::sync::LazyLock;

res!("places"; OTHER_PLACE, OTHER_PLACE_PRONOUN, OTHER_PERSON_NAMED, A_QUESTION_ASKED);
sets!("places"; PLACE_ENDS, NOT_A_PLACE_VALUE);
inline_re! {
    SPACE_OR_DASH = r"[ -]", 0;
    NOT_THIS = r"not\s+[a-z'-]+[, ]+", 0;
    A_PLACE_WORD = r"^(?:in|to|at|from|near)\s+", 0;
}

static WHERE_ASK: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.places.WHERE_ASK"));
static ASK_NAMED: LazyLock<Re> = LazyLock::new(|| {
    let w = &*WHERE_ASK;
    Re::new(&format!(r"\b{w} (?:does|do) my (\w+) (?:live|come from)\b|\b{w} is my (\w+) (?:living|based)\b"), 0)
});
static ASK_PRONOUN: LazyLock<Re> = LazyLock::new(|| {
    let w = &*WHERE_ASK;
    Re::new(&format!(r"\b{w} (?:does|do) (?:he|she|they) (?:live|come from)\b|\b{w} is (?:he|she|they) (?:living|based)\b"), 0)
});
static ASK_PLAIN: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"\b{} (?:is|are|was|were) (?:my|our) ([a-z]+)\b", *WHERE_ASK), 0));
static ASK_BY_NAME: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"\b{} (?:does|do) ([a-z][a-z'-]{{1,}}) (?:live|come from)\b", *WHERE_ASK), 0));

/// A word that ends a place, or is never one.
fn not_a_place_word(low: &str) -> bool {
    PLACE_ENDS.contains(low) || extract::NOT_PLACES.contains(low) || extract::HOME_STOP.contains(low)
}

/// Python: bot/memory/places.py:53 `_a_place(value)`
///
/// The place in a phrase of words, or None when the words are not one.
pub fn _a_place(value: &str) -> Option<String> {
    let mut words: Vec<String> = Vec::new();
    for word in SPACE_OR_DASH.split(value) {
        if not_a_place_word(&pt::lower(&word)) {
            break;
        }
        words.push(word);
        if words.len() == 2 {
            break;
        }
    }
    let first = pt::lower(words.first()?);
    if not_a_place_word(&first) {
        return None;
    }
    Some(words.iter().map(|w| if mem7::is_lower(w) { pt::upper_first(w) } else { w.clone() }).collect::<Vec<_>>().join(" "))
}

/// Python: bot/memory/places.py:82 `_the_place_alone(text)`
///
/// The place a line names with no frame around it: "no, york", "sorry, in york", "i meant york".
/// Only a line that opens with a take-back and carries a single place is read this way.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the place or None.
pub fn _the_place_alone(text: &str) -> Option<String> {
    let low = pt::squash(&pt::rstrip_chars(&pt::lower(pt::strip(text)), ".!,"));
    if low.is_empty() || low.ends_with('?') {
        return None;
    }
    // "not perth, york" names the value it takes back first, the same way the marked forms
    // name it last: the frame is still the line before's.
    let opener_end = match extract::CORRECTION_OPENER.match_(&low) {
        Some(opener) => opener.end(),
        None => NOT_THIS.match_(&low)?.end(),
    };
    let rest = pt::strip_chars(&low[opener_end..], " .,!-");
    let rest = A_PLACE_WORD.subn("", rest, 1);
    let words = pt::split_ref(&rest);
    if words.is_empty() || words.len() > 2 || words.iter().any(|word| NOT_A_PLACE_VALUE.contains(*word)) {
        return None;
    }
    let place = extract::place_value(&rest);
    if place.truthy() {
        Some(mem7::py_str(&place))
    } else {
        None
    }
}

/// Python: bot/memory/places.py:110 `_said_places(memory)`
///
/// Every place the user gave for somebody else, in the order they gave them, as
/// `(the person as said, the place)`. What the chat says last about somebody is what the store
/// says back; a bare correction ("no, york") moves the town of the person the line before it
/// gave one for, and only there.
pub fn _said_places(memory: &Memory) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut last = String::new();
    let mut fresh = false;
    for turn in &memory.turns {
        let text = turn.user.as_str();
        if text.is_empty() || pt::rstrip(text).ends_with('?') || A_QUESTION_ASKED.is_match(text) {
            // A question is not a place they gave: "does she live in leeds" asks, it does not say.
            fresh = false;
            continue;
        }
        let mut named = String::new();
        let mut gave = false;
        for m in OTHER_PLACE.finditer(text) {
            let place = _a_place(m.str(2));
            named = m.str(1).to_string();
            if let Some(place) = place {
                found.push((named.clone(), place));
                gave = true;
            }
        }
        if named.is_empty() {
            // "my sister is called priya" names her and gives her no place, and the next line's
            // pronoun is about her.
            if let Some(asked) = OTHER_PERSON_NAMED.search(pt::rstrip(text)) {
                named = asked.str(1).to_string();
            }
        }
        if !named.is_empty() {
            last = named;
        } else if !last.is_empty() && !OTHER_PLACE_PRONOUN.is_match(text) && fresh {
            // The line names no person and no verb, so the frame is the line it corrects, and
            // only that line: "my sister lives in perth" then "no, york" is York.
            if let Some(alone) = _the_place_alone(text) {
                found.push((last.clone(), alone));
                gave = true;
            }
        }
        if !last.is_empty() && !gave {
            for m in OTHER_PLACE_PRONOUN.finditer(text) {
                if let Some(place) = _a_place(m.str(1)) {
                    found.push((last.clone(), place));
                    gave = true;
                }
            }
        }
        fresh = gave;
    }
    found
}

/// "lives in" for one person, "live in" for a group ("my parents").
fn lives_in(who: &str) -> &'static str {
    if who.ends_with('s') {
        " live in "
    } else {
        " lives in "
    }
}

/// Python: bot/memory/places.py:163 `_a_town_moved(memory, text)`
///
/// The place a correction names for somebody else, said back as a line, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the line or None.
pub fn _a_town_moved(memory: &Memory, text: &str) -> Option<String> {
    let alone = match _the_place_alone(text) {
        Some(alone) => alone,
        None => {
            // "no, she lives in york" names the person only by the pronoun, and the line is a
            // correction, so it moves her town the same way.
            let low = pt::squash(&pt::lower(pt::strip(text)));
            if extract::CORRECTION_OPENER.match_(&low).is_none() {
                return None;
            }
            let frame = OTHER_PLACE_PRONOUN.search(&low)?;
            _a_place(frame.str(1))?
        }
    };
    let before = memory.turns.last().map_or("", |turn| turn.user.as_str());
    if before.is_empty() || !(OTHER_PLACE.is_match(before) || OTHER_PLACE_PRONOUN.is_match(before)) {
        // The frame is the line before this one, so the town they gave has to be on it.
        return None;
    }
    let said = _said_places(memory);
    let (who, place) = said.last()?;
    if pt::lower(place) == pt::lower(&alone) {
        return None;
    }
    let who = pt::lower(who);
    Some(format!("Got it — your {who}{}{alone}.", lives_in(&who)))
}

/// Python: bot/memory/places.py:194 `_other_place(memory, who)`
///
/// The last place the user gave for this person, said back as a line.
pub fn _other_place(memory: &Memory, who: &str) -> Option<String> {
    let want = pt::lower(who);
    let want = pt::rstrip_chars(&want, "s");
    let found: Vec<(String, String)> = _said_places(memory)
        .into_iter()
        .filter(|(said, _)| want.is_empty() || pt::rstrip_chars(&pt::lower(said), "s") == want)
        .collect();
    let (said, place) = found.last()?;
    let said = pt::lower(said);
    Some(format!("Your {said}{}{place}.", lives_in(&said)))
}

/// Python: bot/memory/places.py:206 `_asked_person(memory, low)`
///
/// Who a place ask is about, when it is about somebody other than the user, or "". "where does
/// my sister live", "and where does she live" and "where does jane live" all ask the same thing.
/// (`others(memory)` creates `memory["others"]` when it is missing, as in PyBot.)
pub fn _asked_person(memory: &mut Memory, low: &str) -> String {
    if let Some(named) = ASK_NAMED.search(low) {
        return named.get(1).filter(|g| !g.is_empty()).or(named.get(2)).unwrap_or("").to_string();
    }
    if ASK_PRONOUN.is_match(low) {
        let who = match memory.get("person") {
            Some(person) if person.is_dict() => person.get("who").cloned().unwrap_or(Value::None),
            _ => Value::from(""),
        };
        if who.truthy() {
            return pt::lower(&mem7::py_str(&who));
        }
        // "my parents live in cork" then "where do they live": the last person the chat gave a
        // place for is the one it is about.
        return _said_places(memory).last().map_or(String::new(), |(who, _)| pt::lower(who));
    }
    // "wheres my brother" is the same ask as "where does my brother live". Only a person is read
    // this way - "wheres my phone" is not a place ask about anybody.
    if let Some(plain) = ASK_PLAIN.search(low) {
        let mut word = pt::lower(plain.str(1));
        if word.ends_with('s') && extract::KIN_WORDS.contains(&word[..word.len() - 1].to_string()) {
            word.pop();
        }
        if extract::KIN_WORDS.contains(&word)
            || extract::RELATIONS.contains(&word)
            || mem7::others(memory).iter().any(|one| pt::lower(&mem7::text_of(Some(mem7::item_get(one, "who")))) == word)
        {
            return word;
        }
    }
    if let Some(by_name) = ASK_BY_NAME.search(low) {
        let word = pt::lower(by_name.str(1));
        if matches!(word.as_str(), "he" | "she" | "they" | "my" | "your" | "i" | "we" | "it" | "the" | "a") {
            return String::new();
        }
        for entry in mem7::others(memory) {
            if pt::lower(&mem7::text_of(Some(mem7::item_get(&entry, "name")))) == word {
                return pt::lower(&mem7::text_of(Some(mem7::item_get(&entry, "who"))));
            }
        }
    }
    String::new()
}
