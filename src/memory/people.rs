//! Port of `bot/memory/people.py`: the person a line is about, when the chat already knows one -
//! who, and how old they are.
//!
//! A line that names the relation ("my daughter") is the sure case. A line that says only `him`
//! or `her` is read by the gender of the relation the store holds, and only when the store holds
//! exactly one person of it. Everyone else is None, so a module that asks falls back to the
//! answer it had before.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, res, sets};
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;

res!("people"; HIM_SAID, HER_SAID, THEM_SAID);
sets!("people"; HIM, HER, CHILD_OF_THE_FAMILY);

/// `str(reading.get("clean") or reading.get("text") or "")`.
fn the_line(understood: Option<&Understood>) -> String {
    match understood {
        Some(reading) if !reading.clean.is_empty() => reading.clean.clone(),
        Some(reading) => reading.text.clone(),
        None => String::new(),
    }
}

/// Python: bot/memory/people.py:46 `_the_age_of_the_group(memory)`
///
/// The age the chat holds for the people a "them" points back at, or None.
pub fn _the_age_of_the_group(memory: &Memory) -> Option<i64> {
    let theirs = memory.get("their_age").filter(|theirs| theirs.is_dict())?;
    mem7::as_int(theirs.get("age")).filter(|age| 0 < *age && *age < 120)
}

/// Python: bot/memory/people.py:62 `_with_an_age(record)`
///
/// True for a person the store knows the age of, whether it was said as one number or a list.
pub fn _with_an_age(record: &Value) -> bool {
    if !record.is_dict() {
        return false;
    }
    if mem7::as_int(record.get("age")).is_some_and(|age| age > 0) {
        return true;
    }
    mem7::iterate_or_empty(record.get("ages")).iter().any(|age| age.as_int().is_some_and(|age| age > 0))
}

/// Python: bot/memory/people.py:71 `_age(record)`
///
/// The age the store holds for a person: the one number, or the first of a list.
pub fn _age(record: &Value) -> Option<i64> {
    if let Some(age) = mem7::as_int(mem7::item_get(record, "age").into()) {
        return Some(age);
    }
    mem7::iterate_or_empty(record.get("ages")).iter().find_map(Value::as_int)
}

/// The people kept one by one: `[one for one in others(memory) if isinstance(one, dict) and
/// one.get("who")]`.
fn kept_people(memory: &mut Memory) -> Vec<Value> {
    mem7::others(memory).into_iter().filter(|one| one.is_dict() && mem7::get(one, "who").truthy()).collect()
}

/// Python: bot/memory/people.py:81 `a_person_with_an_age(memory, understood=None)`
///
/// The person the line is about, taken from the store, as `{"who", "age", ...}` or None.
pub fn a_person_with_an_age(memory: &mut Memory, understood: Option<&Understood>) -> Option<Value> {
    let line = the_line(understood);
    let people = kept_people(memory);
    if people.is_empty() {
        // Nobody is kept one by one, but the chat may have been told the age of a group.
        return _the_group_record(memory, &line);
    }
    // The relation named in this line: "what should i get my son" is about him.
    let named = crate::memory::topics::person(&line).filter(|named| !named.is_empty());
    if let Some(named) = &named {
        for one in &people {
            if mem7::lower_of(one, "who") == *named && _with_an_age(one) {
                return Some(one.clone());
            }
        }
        // A line that names a relation of nobody in the store is about somebody new.
        return None;
    }
    let said_him = HIM_SAID.is_match(&line);
    let said_her = HER_SAID.is_match(&line);
    if said_him == said_her && THEM_SAID.is_match(&line) {
        return _the_ones_a_them_points_at(&people);
    }
    if said_him == said_her {
        // No pronoun, or both: it may still point at the group the chat was told an age for.
        return _the_group_record(memory, &line);
    }
    let wanted = if said_him { &*HIM } else { &*HER };
    let found: Vec<&Value> = people.iter().filter(|one| wanted.contains(&mem7::lower_of(one, "who")) && _with_an_age(one)).collect();
    if found.len() != 1 {
        // Two sons and a "him": nobody in particular, so the generic answer stays.
        return None;
    }
    Some(found[0].clone())
}

/// Python: bot/memory/people.py:126 `_the_group_record(memory, line)`
///
/// The people a "them" points back at, as a person record with their age, or None.
pub fn _the_group_record(memory: &Memory, line: &str) -> Option<Value> {
    if !THEM_SAID.is_match(line) {
        return None;
    }
    let age = _the_age_of_the_group(memory).filter(|age| *age != 0)?;
    Some(mem7::dict(vec![("who", Value::from("them")), ("age", Value::from(age))]))
}

/// Python: bot/memory/people.py:136 `_the_ones_a_them_points_at(people)`
///
/// The person a plural pronoun points at: the one with an age, when there is only one age.
pub fn _the_ones_a_them_points_at(people: &[Value]) -> Option<Value> {
    let with_an_age: Vec<&Value> = people.iter().filter(|one| _with_an_age(one)).collect();
    let first = *with_an_age.first()?;
    let ages: HashSet<Option<i64>> = with_an_age.iter().map(|one| _age(one)).collect();
    if ages.len() != 1 {
        return None;
    }
    Some(first.clone())
}

/// Python: bot/memory/people.py:159 `the_people(memory)`
///
/// Every person the store keeps for this chat: `memory["others"]` plus the one in hand.
pub fn the_people(memory: &mut Memory) -> Vec<Value> {
    let mut found = kept_people(memory);
    if let Some(one) = memory.get("person") {
        if one.is_dict() && mem7::get(one, "who").truthy() && !found.contains(one) {
            found.push(one.clone());
        }
    }
    found
}

/// Python: bot/memory/people.py:170 `a_child_the_chat_is_about(memory, understood=None)`
///
/// True when the person this chat is about is a child of the family, with or without an age.
pub fn a_child_the_chat_is_about(memory: &mut Memory, understood: Option<&Understood>) -> bool {
    let line = the_line(understood);
    let people = the_people(memory);
    if let Some(named) = crate::memory::topics::person(&line).filter(|named| !named.is_empty()) {
        // The line names the person it is about. An adult relation is never a child's present.
        if !CHILD_OF_THE_FAMILY.contains(&named) {
            return false;
        }
        for one in &people {
            if mem7::lower_of(one, "who") != named {
                continue;
            }
            if _age(one).is_some_and(|age| age >= 18) {
                return false; // a grown son is a grown-up's present
            }
        }
        // A child relation of its own, with nobody in the store to go on: a child's present.
        return true;
    }
    let said_him = HIM_SAID.is_match(&line);
    let said_her = HER_SAID.is_match(&line);
    let wanted = if said_him && !said_her {
        Some(&*HIM)
    } else if said_her && !said_him {
        Some(&*HER)
    } else {
        None
    };
    // One person can be held twice, so the age is read per person, not per record.
    let mut ages: Vec<(String, i64)> = Vec::new();
    for one in &people {
        let who = mem7::lower_of(one, "who");
        if let Some(age) = _age(one) {
            match ages.iter_mut().find(|(key, _)| *key == who) {
                Some(slot) => slot.1 = age,
                None => ages.push((who, age)),
            }
        }
    }
    if let Some(wanted) = wanted {
        // `him` with a son and a father in the store is nobody in particular.
        let of_that_kind: HashSet<String> = people.iter().map(|one| mem7::lower_of(one, "who")).filter(|who| wanted.contains(who)).collect();
        if of_that_kind.len() > 1 {
            return false;
        }
    }
    for one in &people {
        let who = mem7::lower_of(one, "who");
        if !CHILD_OF_THE_FAMILY.contains(&who) {
            continue;
        }
        if wanted.is_some_and(|wanted| !wanted.contains(&who)) {
            continue; // "what should i get her" is not about the son
        }
        if ages.iter().find(|(key, _)| *key == who).is_some_and(|(_, age)| *age >= 18) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/memory/people.py:232 `the_age_of_the_person_they_mean(memory, understood=None)`
///
/// How old the person this line is about is, from the store, or None.
pub fn the_age_of_the_person_they_mean(memory: &mut Memory, understood: Option<&Understood>) -> Option<i64> {
    if let Some(found) = a_person_with_an_age(memory, understood).filter(Value::truthy) {
        return _age(&found);
    }
    let line = the_line(understood);
    if !THEM_SAID.is_match(&line) || HIM_SAID.is_match(&line) || HER_SAID.is_match(&line) {
        return None;
    }
    let held = memory.get("their_age").filter(|held| held.is_dict())?;
    if _with_an_age(held) {
        return _age(held);
    }
    None
}
