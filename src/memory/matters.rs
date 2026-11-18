//! Port of `bot/memory/matters.py`: a follow-up that says a fact about the person the answer was
//! about, written out with them in it.
//!
//! "my mum is 78 and keeps falling" / "she lives alone": the store holds the person as "mum", so
//! "she lives alone" is written out as "my mum lives alone" and handed on, and the answer is the
//! answer to the fact rather than a receipt. Only the narrow shape is taken. The file also holds
//! the reply to "she says she's fine" in a chat about a care worry, and the corrected age of one
//! of a group of children.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, lists, res, sets, texts, values};
use crate::py::text as pt;
use crate::state::Memory;

res!("matters"; SO, A_REPORT, THEIR_DIET, ON_THEIR_OWN, THEIR_WORD, CARE_MATTER, _ONE_AGE, _NOT_THE_OLD_ONE, _OLDEST, _YOUNGEST, _A_CORRECTION);
sets!("matters"; FACT_VERB);
texts!("matters"; MINE, LIVES_ALONE);
lists!("matters"; FINE_REPLY);
values!("matters"; MOST_WORDS);
sets!("people"; HIM, HER);


/// Python: bot/memory/matters.py:94 `_the_person(memory)`
///
/// The one person the chat knows: (relation, their record, sex - "him", "her" or ""), or None.
/// Two people is nobody in particular.
pub fn _the_person(memory: &Memory) -> Option<(String, Value, String)> {
    let Some(Value::List(others)) = memory.get("others") else {
        return None;
    };
    let mut seen: Vec<(String, Value)> = Vec::new();
    for record in others.iter() {
        if !record.is_dict() {
            continue;
        }
        let who = pt::lower(pt::strip(&mem7::text_of(record.get("who"))));
        if !who.is_empty() && !seen.iter().any(|(one, _)| *one == who) {
            seen.push((who, record.clone()));
        }
    }
    if seen.len() != 1 {
        return None;
    }
    let (who, record) = seen.remove(0);
    let sex = if HIM.contains(&who) {
        "him"
    } else if HER.contains(&who) {
        "her"
    } else {
        ""
    };
    Some((who, record, sex.to_string()))
}

/// Python: bot/memory/matters.py:117 `about_them(text, memory)`
///
/// The line written with the person in it: "she lives alone" -> "my mum lives alone", or "".
pub fn about_them(text: &str, memory: &Memory) -> String {
    let Some(found) = SO.match_(text) else {
        return String::new();
    };
    let mut rest = pt::squash(found.name_str("rest"));
    if mem7::word_count(&rest) as i64 > MOST_WORDS.as_int().unwrap_or(8) {
        return String::new();
    }
    if A_REPORT.match_(&rest).is_some() || THEIR_DIET.match_(&rest).is_some() {
        return String::new();
    }
    let alone = ON_THEIR_OWN.match_(&rest).map(|m| m.end());
    if let Some(end) = alone {
        // "on her own at home" and "by herself" are "lives alone" about a person.
        rest = pt::strip(&format!("{} {}", *LIVES_ALONE, &rest[end..])).to_string();
    } else {
        // `rest.split()[0]` (the rest has two characters at least, so a word)
        let first = pt::split_ref(&rest).first().copied().unwrap_or("");
        if !FACT_VERB.contains(&pt::lower(pt::strip_chars(first, ",."))) {
            return String::new();
        }
    }
    let Some((who, _record, sex)) = _the_person(memory) else {
        return String::new();
    };
    let said = pt::lower(found.name_str("who"));
    if !sex.is_empty() && ((said == "she" && sex != "her") || (said == "he" && sex != "him")) {
        // "she" and the store's one person is a son: the pronoun is about somebody else.
        return String::new();
    }
    if pt::split(&pt::lower(&rest)).contains(&who) {
        return String::new(); // the line names a relation already
    }
    if alone.is_some() {
        // "she's on her own at home" is "she lives alone", verb and all.
        return format!("{}{} {}", *MINE, who, rest);
    }
    if found.name("is").is_some() {
        // "she's on blood thinners" is "she is on blood thinners".
        return format!("{}{} is {}", *MINE, who, rest);
    }
    format!("{}{} {}", *MINE, who, rest)
}

/// Python: bot/memory/matters.py:181 `_a_worry_in_the_chat(memory)`
///
/// True when the chat has named a care worry about the person, and not just a mood.
pub fn _a_worry_in_the_chat(memory: &Memory) -> bool {
    let said: Vec<&str> = mem7::tail(&memory.turns, 5).iter().map(|turn| turn.user.as_str()).collect();
    CARE_MATTER.is_match(&said.join(" "))
}

/// Python: bot/memory/matters.py:197 `what_they_say(text, memory)`
///
/// The reply for "she says she's fine" about the person on screen, or "" to pass.
pub fn what_they_say(text: &str, memory: &mut Memory) -> String {
    let Some(found) = THEIR_WORD.match_(text) else {
        return String::new();
    };
    let Some((who, _record, sex)) = _the_person(memory) else {
        return String::new();
    };
    let said = pt::lower(found.name_str("who"));
    if !sex.is_empty() && ((matches!(said.as_str(), "she" | "her") && sex != "her") || (said == "he" && sex != "him")) {
        return String::new(); // "she says she's fine" and the one person in the store is a son
    }
    if !_a_worry_in_the_chat(memory) {
        return String::new();
    }
    // `isinstance(at, int) and at >= 0` (a bool is an int)
    let at = mem7::as_int(memory.get("fine_said")).filter(|at| *at >= 0).unwrap_or(0);
    memory.set("fine_said", at + 1);
    mem7::fmt(&FINE_REPLY[(at as usize) % FINE_REPLY.len()], &[("who", &who)])
}

/// Python: bot/memory/matters.py:243 `_the_group_ages(memory)`
///
/// The single group of ages the store holds, as (its index in `memory["others"]`, the ages), or
/// None.
pub fn _the_group_ages(memory: &Memory) -> Option<(usize, Vec<i64>)> {
    let mut found = Vec::new();
    for (index, entry) in mem7::iterate_or_empty(memory.get("others")).iter().enumerate() {
        if !entry.is_dict() {
            continue;
        }
        let ages: Vec<i64> = mem7::iterate_or_empty(entry.get("ages")).iter().filter_map(Value::as_int).collect();
        if ages.len() > 1 {
            found.push((index, ages));
        }
    }
    if found.len() != 1 {
        return None;
    }
    found.pop()
}

/// Python: bot/memory/matters.py:258 `_say_the_ages(ages)`
pub fn _say_the_ages(ages: &[i64]) -> String {
    let mut ages = ages.to_vec();
    ages.sort();
    if ages.len() == 2 {
        return format!("{} and {}", ages[0], ages[1]);
    }
    let (last, rest) = ages.split_last().expect("IndexError: no ages");
    format!("{} and {}", rest.iter().map(i64::to_string).collect::<Vec<_>>().join(", "), last)
}

/// `entry["ages"] = sorted(ages)` on the group in memory, and the reply that says them.
fn put_the_ages(memory: &mut Memory, index: usize, mut ages: Vec<i64>) -> String {
    ages.sort();
    if let Some(Value::List(others)) = memory.get_mut("others") {
        std::sync::Arc::make_mut(others)[index].set_key("ages", Value::from(ages.iter().map(|a| Value::Int(*a)).collect::<Vec<_>>()));
    }
    format!("Right \u{2014} {}, then.", _say_the_ages(&ages))
}

/// Python: bot/memory/matters.py:265 `a_corrected_age(text, memory)`
///
/// The reply for a corrected age in a group of children, or "" to pass. Only a correction that
/// says which child it is about is applied; one that could be either asks which.
///
/// CONTRACT: memory is `&mut Memory` (the skeleton had `&Memory`): the correction is written onto
/// the group's record. Its caller (greeting.rs) holds a `&mut Memory` already.
pub fn a_corrected_age(text: &str, memory: &mut Memory) -> String {
    let low = pt::squash(&pt::lower(text));
    if low.is_empty() || mem7::word_count(&low) > 10 {
        return String::new();
    }
    let Some((index, mut ages)) = _the_group_ages(memory) else {
        return String::new();
    };
    let numbers: Vec<i64> = _ONE_AGE
        .finditer(&low)
        .iter()
        .map(|m| mem7::int_str(m.name_str("age")).unwrap_or(0))
        .filter(|age| 0 < *age && *age <= 120)
        .collect();
    if numbers.is_empty() {
        return String::new();
    }
    let correcting = _A_CORRECTION.is_match(&low);
    // "ones 7 not 6" names the age being replaced, so there is nothing to guess.
    if let Some(replaced) = _NOT_THE_OLD_ONE.search(&low) {
        let was = mem7::int_str(replaced.name_str("old")).unwrap_or(0);
        let new: Vec<i64> = numbers.iter().copied().filter(|age| *age != was).collect();
        if let (Some(at), Some(first)) = (ages.iter().position(|age| *age == was), new.first()) {
            ages[at] = *first;
            return put_the_ages(memory, index, ages);
        }
        return String::new();
    }
    // "the oldest is 7 now" and "the little one is 2": the word says which end of the group moves.
    let oldest = _OLDEST.is_match(&low);
    let youngest = _YOUNGEST.is_match(&low);
    if oldest && !youngest {
        let max = *ages.iter().max().expect("two ages at least");
        let at = ages.iter().position(|age| *age == max).expect("the max is there");
        ages[at] = numbers[0];
        return put_the_ages(memory, index, ages);
    }
    if youngest && !oldest {
        let min = *ages.iter().min().expect("two ages at least");
        let at = ages.iter().position(|age| *age == min).expect("the min is there");
        ages[at] = numbers[0];
        return put_the_ages(memory, index, ages);
    }
    // "sorry ones 7": nothing saying which child. Asking is the honest reply.
    if correcting && !ages.contains(&numbers[0]) {
        let mut sorted = ages.clone();
        sorted.sort();
        let which = sorted.iter().map(i64::to_string).collect::<Vec<_>>().join(" or the ");
        return format!("Which one is {} \u{2014} the {}?", numbers[0], which);
    }
    String::new()
}
