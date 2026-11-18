//! Port of `bot/memory/back_again.py`: a trouble the chat already had, said to be back.
//!
//! "i have a headache" / "its back", "how do i get rid of slugs" / "the slugs are back": the
//! person reports that the thing the chat was working on has returned, and the reply has to
//! reach the thing and say so. Left alone: a line that asks, a person or an animal said to be
//! back, a thing the chat never had, and "im back".
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, lists, res};
use crate::memory::{history, topics};
use crate::py::random;
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::sync::LazyLock;

res!("back_again"; AFTER_BACK, PRONOUN_BACK, VERB_BACK, BARE_BACK, A_PERSON_OR_PET, AWAY_FROM_HOME, A_SYMPTOM);
lists!("back_again"; DETERMINERS, NAMED, UNNAMED);

/// `topics.current(memory)`'s words: `(record or {}).get("words") or []`.
fn record_words(record: Option<&Value>) -> Vec<String> {
    record.map(|record| mem7::iterate_or_empty(record.get("words")).iter().map(mem7::py_str).collect()).unwrap_or_default()
}

/// Python: bot/memory/back_again.py:110 `_the_earlier_line(memory, thing, text="")`
///
/// The person's own earlier line about the thing, or "" when there is none.
pub fn _the_earlier_line(memory: &Memory, thing: &str, text: &str) -> String {
    let wanted = if thing.is_empty() { Vec::new() } else { topics::keywords(thing) };
    let words = if wanted.is_empty() { record_words(topics::current(memory)) } else { wanted };
    if words.is_empty() {
        return String::new();
    }
    let now = pt::squash(&pt::lower(text));
    for turn in memory.turns.iter().rev() {
        let line = turn.user.as_str();
        if pt::squash(&pt::lower(line)) == now {
            continue; // the line being answered is not an earlier turn
        }
        let have = topics::keywords(line);
        if words.iter().any(|word| have.iter().any(|other| topics::alike(word, other))) {
            return line.to_string();
        }
    }
    String::new()
}

/// Python: bot/memory/back_again.py:131 `_the_thing_went_away(memory, thing, text="")`
///
/// True when the earlier line about the thing puts it somewhere else, or a caller came out.
pub fn _the_thing_went_away(memory: &Memory, thing: &str, text: &str) -> bool {
    let earlier = _the_earlier_line(memory, thing, text);
    if earlier.is_empty() || A_SYMPTOM.is_match(&earlier) {
        return false;
    }
    AWAY_FROM_HOME.is_match(&earlier)
}

/// Python: bot/memory/back_again.py:141 `_the_words(plural)`
///
/// The pronouns a reply needs for one thing or for several, in Python's key order.
pub fn _the_words(plural: bool) -> Vec<(String, String)> {
    let pairs: [(&str, &str); 3] = if plural {
        [("they", "they"), ("them", "them"), ("has", "have")]
    } else {
        [("they", "it"), ("them", "it"), ("has", "has")]
    };
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Python: bot/memory/back_again.py:148 `_filled(line, thing, plural)`
///
/// The reply line with its fields filled. `thing` is "" when the line named none.
pub fn _filled(line: &str, thing: &str, plural: bool) -> String {
    let mut words = _the_words(plural);
    words.push(("the thing".into(), if thing.is_empty() { "It".into() } else { pt::upper_first(thing) }));
    let mut out = line.to_string();
    for (key, value) in &words {
        out = out.replace(&format!("{{{key}}}"), value);
        out = out.replace(&format!("{{{}}}", pt::upper_first(key)), &pt::upper_first(value));
    }
    if !thing.is_empty() {
        return out;
    }
    // The first word of the unnamed line is the thing itself; every unnamed line is written to
    // stand without one.
    pt::upper_first(&out)
}

/// Python: bot/memory/back_again.py:166 `_an_earlier_topic(memory, thing)`
///
/// Whether a turn before this one was about the thing: "the slugs" after the slug question.
pub fn _an_earlier_topic(memory: &Memory, thing: &str) -> bool {
    let wanted = topics::keywords(thing);
    if wanted.is_empty() {
        return false;
    }
    let here = Value::from(mem7::turn_count(memory));
    for record in mem7::iterate_or_empty(memory.get("topics")).iter().rev() {
        if *mem7::item_get(record, "turn") == here {
            continue;
        }
        let have: Vec<String> = mem7::iterate_or_empty(record.get("words")).iter().map(mem7::py_str).collect();
        if wanted.iter().any(|word| have.iter().any(|other| topics::alike(word, other))) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/back_again.py:181 `_from_this_line(record, memory)`
///
/// True when the topic record was written for the line being answered right now.
pub fn _from_this_line(record: &Value, memory: &Memory) -> bool {
    *mem7::item_get(record, "turn") == Value::from(mem7::turn_count(memory))
}

/// Python: bot/memory/back_again.py:186 `_the_thing(text, memory)`
///
/// The thing said to be back: (the words for it - "" for "it" -, is it plural), or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns a pair or None.
pub fn _the_thing(text: &str, memory: &Memory) -> Option<(String, bool)> {
    let line = pt::squash(&pt::lower(text));
    if line.is_empty() || line.contains('?') {
        return None;
    }
    let mut words = pt::split_ref(&line);
    if words.first().is_some_and(|w| matches!(*w, "and" | "so" | "well" | "oh" | "ok" | "okay" | "right")) {
        words.remove(0);
    }
    if words.first().is_some_and(|w| DETERMINERS.iter().any(|d| d == w)) {
        let mut found: Vec<(i64, i64, String)> = Vec::new();
        let mut numbers: Vec<(String, bool)> = Vec::new();
        for count in (1..=3.min(words.len() - 1)).rev() {
            let thing = words[..count].join(" ");
            let rest = words[count..].join(" ");
            let Some(said) = VERB_BACK.match_(&rest) else {
                continue;
            };
            if AFTER_BACK.match_(said.name_str("tail")).is_none() {
                continue;
            }
            if A_PERSON_OR_PET.is_match(&thing) {
                return None;
            }
            // A thing the chat never had cannot be back: there has to be an earlier turn about it.
            if !_an_earlier_topic(memory, &thing) {
                continue;
            }
            let cop = said.name("cop").filter(|cop| !cop.is_empty());
            found.push((if cop.is_some() { 0 } else { 1 }, -(count as i64), thing.clone()));
            let number = match cop {
                Some(cop) => matches!(pt::lower(cop).as_str(), "are" | "were" | "have"),
                None => pt::split_ref(&thing).last().is_some_and(|last| last.ends_with('s')),
            };
            match numbers.iter_mut().find(|(key, _)| *key == thing) {
                Some(slot) => slot.1 = number,
                None => numbers.push((thing, number)),
            }
        }
        if found.is_empty() {
            return None;
        }
        found.sort();
        let thing = found[0].2.clone();
        let plural = numbers.iter().find(|(key, _)| *key == thing).map_or(false, |(_, n)| *n);
        return Some((thing, plural));
    }
    if let Some(spoken) = PRONOUN_BACK.match_(&line) {
        if AFTER_BACK.match_(spoken.name_str("tail")).is_none() {
            return None;
        }
        let record = topics::current(memory).filter(|record| record.truthy())?;
        if _from_this_line(record, memory) {
            return None;
        }
        if A_PERSON_OR_PET.is_match(&record_words(Some(record)).join(" ")) {
            return None;
        }
        let plural = ["theyre", "they're", "they are", "these", "those"].iter().any(|start| line.starts_with(start));
        return Some((String::new(), plural));
    }
    if BARE_BACK.match_(&line).is_some() {
        let record = topics::current(memory).filter(|record| record.truthy())?;
        if _from_this_line(record, memory) {
            return None;
        }
        return Some((String::new(), false));
    }
    None
}

/// Python: bot/memory/back_again.py:247 `reply_for(text, memory)`
///
/// The reply to a trouble said to be back, or None when the line is something else.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    if !history::an_earlier_turn(memory, text) {
        return None;
    }
    let (thing, plural) = _the_thing(text, memory)?;
    if _the_thing_went_away(memory, &thing, text) {
        // "my car is at the garage" / "my car is back" is good news about a thing that was away.
        return None;
    }
    let line = random::choice(if thing.is_empty() { &*UNNAMED } else { &*NAMED });
    Some(_filled(&line, &thing, plural))
}
