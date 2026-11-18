//! Port of `bot/memory/their_feeling.py`: "my son hates it", "she loves the new school" - how
//! somebody of theirs feels about the thing the chat is on, answered with the one question a
//! friend asks: what is worst, or best, about it.

use super::mem8_py::{pick, rx, strset, val};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.memory.their_feeling.";

rx!(FEELING, A_THING, POINTS_BACK, ASKING, DEFINITE);
strset!(WORDS);
val!(WITH_THING, WITHOUT_THING);
static DISLIKED: LazyLock<Re> = LazyLock::new(|| {
    Re::new(&format!(r"(?:^|\s)(?:{})(?:\s|$)", crate::data::string("bot.memory.their_feeling.DISLIKE")), 0)
});
static A_POSSESSIVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:my|our|your|his|her|their)\b", 0));
static NOT_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z']+", 0));

/// Python: bot/memory/their_feeling.py:109 `_the_last_person(text, memory)`
///
/// (the person as the reply says them, the person as the line said them), or ("", "").
pub fn _the_last_person(text: &str, memory: &mut Memory) -> (String, String) {
    let low = text.to_lowercase();
    let Some(found) = FEELING.match_(&low) else { return (String::new(), String::new()) };
    let who = found.name_str("who").to_string();
    let last = text::split_ref(&who).last().map(|s| s.to_string()).unwrap_or_default();
    if who.starts_with("my ") || who.starts_with("our ") {
        // The store words a person by the relation alone, and a reply says "your son".
        return (format!("your {last}"), last);
    }
    if who.starts_with("his ") || who.starts_with("her ") || who.starts_with("their ") {
        return (String::new(), who);
    }
    // `except Exception: person = ""`.
    let mut person = crate::runtime::guarded(|| super::relations::_person_from_pronoun(&low, memory, &[])).unwrap_or_default();
    if !person.is_empty() && !A_POSSESSIVE.is_match(&person) {
        person = format!("your {person}");
    }
    (person, who)
}

/// Python: bot/memory/their_feeling.py:136 `reply_for(text, memory)`
///
/// The question about how somebody of theirs feels about the thing, or None.
pub fn reply_for(text: &str, memory: &mut Memory) -> Option<String> {
    let low = text::squash(&text.to_lowercase());
    if low.is_empty() || ASKING.is_match(&low) {
        return None;
    }
    let found = FEELING.match_(&low)?;
    let how_said = found.name_str("how").to_string();
    let thing = text::strip_chars(found.name_str("thing"), " .,!;:").to_string();
    let (person, _) = _the_last_person(&low, memory);
    if person.is_empty() || thing.is_empty() {
        return None;
    }
    let back = POINTS_BACK.match_(&thing).is_some();
    let definite = DEFINITE.match_(&thing).is_some();
    if !back {
        if A_THING.match_(&thing).is_none() {
            return None;
        }
        if !definite && !_an_earlier_turn(memory, &low) {
            // A bare subject on the first line is the knowledge base's.
            return None;
        }
        if !definite && !_says_something(&thing) {
            return None;
        }
    }
    if !_an_earlier_turn(memory, &low) && !definite {
        return None;
    }
    let how = if DISLIKED.is_match(&how_said) { "no" } else { "yes" };
    let pool = if back { &*WITHOUT_THING } else { &*WITH_THING };
    let frames = pool.get(how).expect("their_feeling pool").strs();
    let filled: Vec<String> = frames.iter().map(|frame| _fill(frame, &person, if back { "" } else { &thing })).collect();
    Some(_vary(&filled, memory, &low))
}

/// Python: bot/memory/their_feeling.py:171 `_says_something(thing)`
pub fn _says_something(thing: &str) -> bool {
    let low = thing.to_lowercase();
    let words: Vec<String> = NOT_LETTERS.split(&low).into_iter().filter(|w| !w.is_empty()).collect();
    !words.is_empty() && words.iter().any(|w| !WORDS.contains(w))
}

/// Python: bot/memory/their_feeling.py:177 `_an_earlier_turn(memory, low)`
///
/// `history.an_earlier_turn`, False where it raises.
pub fn _an_earlier_turn(memory: &Memory, low: &str) -> bool {
    crate::runtime::guarded(|| super::history::an_earlier_turn(memory, low)).unwrap_or(false)
}

/// Python: bot/memory/their_feeling.py:185 `_fill(frame, person, thing)`
pub fn _fill(frame: &str, person: &str, thing: &str) -> String {
    let pronouns = crate::compose::remark::person_pronouns(person);
    let they = pronouns.items().first().map(|v| v.str().to_string()).unwrap_or_default();
    let mut said = frame.replace("{person}", person).replace("{they}", &they);
    if frame.contains("{thing") || frame.contains("{Thing}") {
        let cap = text::upper_first(thing);
        said = said.replace("{thingcap}", &cap).replace("{Thing}", &cap).replace("{thing}", thing);
    }
    // The frame may open on the thing, and a sentence opens on a capital.
    text::upper_first(&said)
}

/// Python: bot/memory/their_feeling.py:197 `_vary(options, memory, line='')`
pub fn _vary(options: &[String], memory: &Memory, line: &str) -> String {
    pick(options, memory, line)
}
