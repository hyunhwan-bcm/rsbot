//! Port of `bot/compose/missing_half.py`: an age comparison ("am i older than my brother") where
//! one of the two ages was never given, said plainly instead of hedged.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, format_named, inline_re};
use crate::data::Value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.missing_half"; A_COMPARISON, A_GAP, A_GROUP, A_PERSON_NAMED, A_THING_NAMED, A_SOMEONE,
    A_THROUGH_SOMEONE, A_MYSELF);
bundle_strs!("bot.compose.missing_half"; KIN, WITH_MINE, WITHOUT_MINE, BOTH_MISSING, NOBODY_ELSE, TWO_THINGS, THEIR_AGE_ONLY,
    HIM_OR_HER, THEY_ARE_OLDER, I_AM_OLDER, SAME_AGE);
bundle_sets!("bot.compose.missing_half"; FEMALE, MALE);
inline_re!(THEY, r"\b(?:them|they)\b");
inline_re!(SHE, r"\bher\b|\bshe\b");
inline_re!(HE, r"\bhim\b|\bhis\b|\bhe\b");
inline_re!(A_YEAR, r"\b\d{4}\b|\b(?:19|20)\d\d\b");

/// The ages the store holds for other people, by who, in the order they were given.
pub type Others = Vec<(String, i64)>;

/// Python: bot/compose/missing_half.py:138 `_choose(pool, text)`
///
/// The same line for the same words, so a chat is not a slot machine.
pub fn _choose(pool: &[String], text_: &str) -> String {
    com4::by_ord(pool, text_).to_string()
}

/// Python: bot/compose/missing_half.py:143 `_a_sentence(line)`
pub fn _a_sentence(line: &str) -> String {
    text::upper_first(line)
}

/// Python: bot/compose/missing_half.py:154 `_fits(low, who)`
///
/// Whether a pronoun in the line can only be the person the store holds under `who`.
pub fn _fits(low: &str, who: &str) -> bool {
    if THEY.search(low).is_some() {
        return true;
    }
    let she = SHE.search(low).is_some();
    let he = HE.search(low).is_some();
    if she && !he {
        return FEMALE.contains(who);
    }
    if he && !she {
        return MALE.contains(who);
    }
    true
}

/// Python: bot/compose/missing_half.py:167 `_given_beside(low, who)`
///
/// Whether the line puts an age on this person itself ("my brother who is 40").
pub fn _given_beside(low: &str, who: &str) -> bool {
    Re::new(&format!(r"\bmy\s+{who}\b[^.!?,]{{0,16}}?\b\d{{1,3}}\b"), 0).search(low).is_some()
}

/// Python: bot/compose/missing_half.py:176 `_the_ages(memory)`
///
/// The ages the store holds: the person's own, and the ones they gave for other people.
pub fn _the_ages(memory: &Memory) -> (Option<i64>, Others) {
    let mine = match memory.get("user").and_then(|u| u.get("age")) {
        Some(age) if !age.is_none() => com4::py_int(age),
        _ => None,
    };
    let mut others: Others = Vec::new();
    if let Some(list) = memory.get("others") {
        for entry in list.items() {
            if !entry.is_dict() {
                continue;
            }
            let Some(age) = entry.get("age").filter(|a| !a.is_none()) else { continue };
            let who = entry.get("who").map(|w| match w {
                Value::Str(s) => s.to_string(),
                Value::None => String::new(),
                other => if other.truthy() { crate::compose::com4::py_str(other) } else { String::new() },
            });
            let who = text::strip(&who.unwrap_or_default()).to_lowercase();
            if who.is_empty() {
                continue;
            }
            let Some(age) = com4::py_int(age) else { continue };
            match others.iter_mut().find(|(w, _)| *w == who) {
                Some(slot) => slot.1 = age,
                None => others.push((who, age)),
            }
        }
    }
    (mine, others)
}

/// Python: bot/compose/missing_half.py:199 `_the_known_list(others)`
///
/// "your wife at 32" out of the ages the store holds, in the order they were given.
pub fn _the_known_list(others: &[(String, i64)]) -> String {
    let parts: Vec<String> = others.iter().take(2).map(|(who, age)| format!("your {who} at {age}")).collect();
    match parts.len() {
        2 => format!("{} and {}", parts[0], parts[1]),
        0 => "nothing".to_string(),
        _ => parts[0].clone(),
    }
}

/// Python: bot/compose/missing_half.py:212 `the_missing_half(text, memory)`
///
/// The age comparison said plainly when one of its two ages was never given, else "".
pub fn the_missing_half(text_: &str, memory: &Memory) -> String {
    let low = text::squash(&text_.to_lowercase());
    if low.is_empty() || A_COMPARISON.search(&low).is_none() {
        return String::new();
    }
    if A_GAP.search(&low).is_some() || A_GROUP.search(&low).is_some() || A_THROUGH_SOMEONE.search(&low).is_some() {
        // A gap, a group, or a half reached only through another person.
        return String::new();
    }
    let (mine, others) = _the_ages(memory);
    let held = |who: &str| others.iter().find(|(w, _)| w == who).map(|(_, a)| *a);
    let mut named: Vec<String> = Vec::new();
    for one in A_PERSON_NAMED.findall_str(&low) {
        let one = one.to_lowercase();
        if !named.contains(&one) {
            named.push(one);
        }
    }
    let mut things: Vec<String> = Vec::new();
    for one in A_THING_NAMED.findall_str(&low) {
        let one = one.to_lowercase();
        if !KIN.contains(&one) && !things.contains(&one) {
            things.push(one);
        }
    }
    if !named.is_empty() {
        let unknown: Vec<&String> = named.iter().filter(|one| held(one).is_none() && !_given_beside(&low, one)).collect();
        let Some(first) = unknown.first() else {
            // Everyone the line names has an age the store holds.
            return String::new();
        };
        let theirs = format!("your {first}");
        if !things.is_empty() {
            // A made thing beside a person is not two ages of the same kind.
            return String::new();
        }
        if let Some(mine) = mine {
            return format_named(&_choose(&WITH_MINE, &low), &[("mine", &mine.to_string()), ("theirs", &theirs)]);
        }
        if !others.is_empty() {
            return format_named(&_choose(&BOTH_MISSING, &low), &[("known", &_the_known_list(&others)), ("theirs", &theirs)]);
        }
        return _a_sentence(&format_named(&_choose(&WITHOUT_MINE, &low), &[("theirs", &theirs)]));
    }
    if !things.is_empty() {
        // Made things only when the line names two of them and no person at all.
        if things.len() >= 2 && A_SOMEONE.search(&low).is_none() {
            if A_YEAR.search(&low).is_some() {
                // The line hands over a year itself.
                return String::new();
            }
            return _a_sentence(&_choose(&TWO_THINGS, &low));
        }
        return String::new();
    }
    if A_SOMEONE.search(&low).is_none() {
        if !others.is_empty() && mine.is_some() {
            return String::new();
        }
        if let Some(mine) = mine {
            return _a_sentence(&format_named(&_choose(&NOBODY_ELSE, &low), &[("mine", &mine.to_string())]));
        }
        return String::new();
    }
    // A pronoun standing for a person whose age is not held.
    if others.is_empty() {
        return String::new();
    }
    let fits: Vec<&(String, i64)> = others.iter().filter(|(who, _)| _fits(&low, who)).collect();
    if mine.is_none() && fits.len() == 1 {
        let (who, age) = fits[0];
        return _a_sentence(&format_named(
            &_choose(&THEIR_AGE_ONLY, &low),
            &[("theirs", &format!("your {who}")), ("their_age", &age.to_string())],
        ));
    }
    if let (Some(mine), 1) = (mine, fits.len()) {
        let (who, their_age) = fits[0];
        let theirs = format!("your {who}");
        let (m, a) = (mine.to_string(), their_age.to_string());
        if *their_age == mine {
            return format_named(&_choose(&SAME_AGE, &low), &[("mine", &m)]);
        }
        if *their_age > mine {
            return format_named(
                &_choose(&THEY_ARE_OLDER, &low),
                &[("Theirs", &text::upper_first(&theirs)), ("their_age", &a), ("mine", &m)],
            );
        }
        return _a_sentence(&format_named(&_choose(&I_AM_OLDER, &low), &[("theirs", &theirs), ("their_age", &a), ("mine", &m)]));
    }
    _a_sentence(&format_named(&_choose(&HIM_OR_HER, &low), &[("known", &_the_known_list(&others))]))
}

#[allow(dead_code)]
fn _unused() -> &'static Re {
    &A_MYSELF
}
