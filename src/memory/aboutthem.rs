//! Port of `bot/memory/aboutthem.py`: an ask about a person in the chat, written out with the
//! age the store keeps for them.
//!
//! "my son is 5" / "how much sleep does he need" is written out as "how much sleep does a 5 year
//! old need", which the sleep entries are keyed on. Only a child, and only when the store holds
//! the age. The file also answers "how old will he be next year" by arithmetic on the age kept.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::facts::{HE, SHE};
use crate::memory::mem7::{self, res, values};
use crate::memory::topics;
use crate::py::re::Match;
use crate::py::text as pt;
use crate::state::Memory;
use chrono::Datelike;
use std::collections::HashSet;

res!("aboutthem"; PRONOUN, THEIR_AGE, ASKS, GIVEN, AHEAD_ASK, AHEAD_TIME, KIN);
values!("aboutthem"; CHILD_AGE, BABY_MONTHS, SPOKEN, COUNT, KIN_SAME);

/// `SPOKEN.get(n, str(n))`.
fn spoken(n: i64) -> String {
    SPOKEN.pairs().iter().find(|(k, _)| k.as_int() == Some(n)).map_or_else(|| n.to_string(), |(_, v)| v.str().to_string())
}

/// The relations a pronoun fits: SHE, HE, or both for "their".
fn fits_of(word: &str) -> HashSet<String> {
    match word {
        "she" | "her" => SHE.clone(),
        "he" | "his" | "him" => HE.clone(),
        _ => SHE.union(&HE).cloned().collect(),
    }
}

/// `(found.group("who") or found.group("own") or found.group("obj") or "").lower()`.
fn the_pronoun(found: &Match) -> String {
    ["who", "own", "obj"].iter().filter_map(|g| found.name(g)).find(|w| !w.is_empty()).map(pt::lower).unwrap_or_default()
}

/// Python: bot/memory/aboutthem.py:60 `_the_child(text, memory)`
///
/// (how old the child is, written out, and the person), or None.
pub fn _the_child(text: &str, memory: &Memory) -> Option<(String, String)> {
    if text.is_empty() {
        return None;
    }
    if THEIR_AGE.is_match(text) || GIVEN.is_match(text) || !ASKS.is_match(text) {
        return None;
    }
    if !text.contains('?') && !topics::_leads_a_question(text) {
        return None;
    }
    let found = PRONOUN.search(text)?;
    let word = the_pronoun(&found);
    // "his" fits HE, "her" fits SHE; "him"/"them"/"their" as the Python reads them.
    let fits = match word.as_str() {
        "she" | "her" => SHE.clone(),
        "he" | "his" => HE.clone(),
        _ => SHE.union(&HE).cloned().collect(),
    };
    for entry in _people(memory) {
        let who = pt::lower(pt::strip(&mem7::text_of(entry.get("who"))));
        if who.is_empty() || !fits.contains(&who) {
            continue;
        }
        if let Some(months) = mem7::as_int(entry.get("months")).filter(|m| 0 < *m && *m < BABY_MONTHS.as_int().unwrap_or(24)) {
            // A baby is kept in months.
            return Some((format!("{} month old", spoken(months)), who));
        }
        if let Some(age) = mem7::as_int(entry.get("age")).filter(|a| 0 < *a && *a < CHILD_AGE.as_int().unwrap_or(18)) {
            return Some((format!("{} year old", spoken(age)), who));
        }
    }
    None
}

/// Python: bot/memory/aboutthem.py:96 `_people(memory)`
///
/// The person the chat was last about first, then everybody the store keeps a fact about.
pub fn _people(memory: &Memory) -> Vec<Value> {
    let mut found = Vec::new();
    if let Some(person) = memory.get("person") {
        if person.is_dict()
            && mem7::get(person, "who").truthy()
            && (!mem7::get(person, "age").is_none() || !mem7::get(person, "months").is_none())
        {
            found.push(person.clone());
        }
    }
    for entry in mem7::iterate_or_empty(memory.get("others")) {
        if entry.is_dict() {
            found.push(entry);
        }
    }
    found
}

/// Python: bot/memory/aboutthem.py:110 `with_the_age(text, memory)`
///
/// The line asked about a child of that age, or "" when there is no child in it.
pub fn with_the_age(text: &str, memory: &Memory) -> String {
    let Some((how_old, _who)) = _the_child(text, memory) else {
        return String::new();
    };
    let Some(found) = PRONOUN.search(text) else {
        return String::new();
    };
    let written = if found.name("own").is_some() {
        // "his bedtime" is "a 5 year old's bedtime".
        let thing = pt::strip(found.name_str("thing"));
        format!("{}a {}'s {}{}", &text[..found.start()], how_old, thing, &text[found.end()..])
    } else {
        format!("{}a {}{}", &text[..found.start()], how_old, &text[found.end()..])
    };
    let written = pt::squash(&written);
    if written != text {
        written
    } else {
        String::new()
    }
}

/// Python: bot/memory/aboutthem.py:150 `an_age_ahead(text, memory)`
///
/// The age the child will be at a day further ahead, said back, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the line or None.
pub fn an_age_ahead(text: &str, memory: &Memory) -> Option<String> {
    let said = pt::squash(text);
    if said.is_empty() || !AHEAD_ASK.is_match(&said) {
        return None;
    }
    let when = AHEAD_TIME.search(&said)?;
    let (age, words) = _the_one_asked_about(&said, memory)?;
    let years = _the_years_ahead(&when).filter(|y| *y >= 0)?;
    let day = _the_day_named(&when, years);
    if day.is_empty() {
        return None;
    }
    let subject = _the_subject(&words);
    let verb = format!("{}{}", if words.starts_with("your ") { " " } else { "" }, _the_future(&words));
    Some(format!("{subject}{verb} {} {day}.", _an_age_said(age + years)))
}

/// Python: bot/memory/aboutthem.py:175 `_the_years_ahead(when)`
///
/// How many years ahead the ask is, or None when the day it names is one to leave alone.
pub fn _the_years_ahead(when: &Match) -> Option<i64> {
    if let Some(next) = when.name("next") {
        // A birthday is a year, a summer is not.
        return if pt::lower(next) == "year" { Some(1) } else { None };
    }
    if let Some(year) = when.name("year") {
        return mem7::int_str(year).map(|y| y - crate::py::clock::today().year() as i64);
    }
    let count = when.name("years")?;
    if pt::is_digit(count) {
        return mem7::int_str(count);
    }
    let low = pt::lower(count);
    COUNT.pairs().iter().find(|(k, _)| k.as_str() == Some(low.as_str())).and_then(|(_, v)| v.as_int())
}

/// Python: bot/memory/aboutthem.py:192 `_the_day_named(when, years)`
///
/// The day the ask names, said the way a person says it.
pub fn _the_day_named(when: &Match, years: i64) -> String {
    let year = when.name("year");
    if years == 0 {
        return match year {
            None => "now".into(),
            Some(year) => format!("in {year}"),
        };
    }
    if let Some(year) = year {
        return format!("in {year}");
    }
    if years == 1 {
        return "next year".into();
    }
    format!("in {} years", spoken(years))
}

/// The child age the store keeps for an entry, when it is a child's whole years.
fn a_child_age(entry: &Value) -> Option<i64> {
    mem7::as_int(entry.get("age")).filter(|a| 0 < *a && *a < CHILD_AGE.as_int().unwrap_or(18))
}

/// Python: bot/memory/aboutthem.py:203 `_the_one_asked_about(said, memory)`
///
/// (the age the store keeps, the words the reply uses for that person), or None.
pub fn _the_one_asked_about(said: &str, memory: &Memory) -> Option<(i64, String)> {
    let people = _people(memory);
    if let Some(named) = KIN.search(said) {
        let who_said = pt::lower(named.name_str("who"));
        let fits: Vec<String> = match KIN_SAME.get(&who_said) {
            Some(same) => same.strs(),
            None => vec![who_said.clone()],
        };
        for entry in &people {
            let who = pt::lower(pt::strip(&mem7::text_of(entry.get("who"))));
            if who.is_empty() || !fits.contains(&who) {
                continue;
            }
            if let Some(age) = a_child_age(entry) {
                return Some((age, format!("your {who_said}")));
            }
        }
        return None;
    }
    let found = PRONOUN.search(said)?;
    let word = the_pronoun(&found);
    let said_who = match word.as_str() {
        "she" | "her" => "she",
        "he" | "his" | "him" => "he",
        _ => "they",
    };
    let fits = fits_of(&word);
    for entry in &people {
        let who = pt::lower(pt::strip(&mem7::text_of(entry.get("who"))));
        if who.is_empty() || !fits.contains(&who) {
            continue;
        }
        if let Some(age) = a_child_age(entry) {
            return Some((age, said_who.to_string()));
        }
    }
    None
}

/// Python: bot/memory/aboutthem.py:239 `_an_age_said(age)`
///
/// An age said the way a person says it: "seven" while a child, "34" after that.
pub fn _an_age_said(age: i64) -> String {
    if 0 < age && age <= 12 {
        return spoken(age);
    }
    age.to_string()
}

/// Python: bot/memory/aboutthem.py:246 `_the_subject(words)`
///
/// The words a reply about that person opens with.
pub fn _the_subject(words: &str) -> String {
    let words = pt::strip(words);
    if words.is_empty() {
        "They".into()
    } else {
        pt::upper_first(words)
    }
}

/// Python: bot/memory/aboutthem.py:252 `_the_future(words)`
///
/// The verb, joined to the person: "He'll be", "Your son will be".
pub fn _the_future(words: &str) -> String {
    if words.starts_with("your ") { "will be".into() } else { "'ll be".into() }
}
