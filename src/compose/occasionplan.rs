//! Port of `bot/compose/occasionplan.py`: a plan ask ("any ideas") for the birthday the store
//! holds with a day still to come, answered with that day and age in it.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, bundle_vals};
use crate::data::Value;
use crate::py::random::Random;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.occasionplan"; _DAY_TO_COME, _ASKS_FOR_A_PLAN, _OWN_SUBJECT, _NOT_A_PLAN_ASK, _SOON,
    _A_MONTH_ON_ITS_OWN, _BIRTHDAY_IN_PLAN, _TURNS_AGE, _MINE_BEFORE_THE_DAY, _A_POSSESSIVE_BEFORE_THE_DAY,
    _A_NAME_BEFORE_THE_DAY, _SOMEONE_ELSES_DAY, _DAY_SAID, _ANOTHER_PERSON);
bundle_strs!("bot.compose.occasionplan"; PLANS, CHILD_PLANS, SOON_PLANS);
bundle_sets!("bot.compose.occasionplan"; RELATIONS);
bundle_vals!("bot.compose.occasionplan"; _SPOKEN_FOR, _STILL_TALKING_ABOUT);

/// One birthday the store holds: ("you" or "another", who, age, the day as the reply says it).
pub type Occasion = (String, Option<String>, Option<i64>, String);

/// `str(v or "")` for a value the store holds.
fn str_or_empty(v: Option<&Value>) -> String {
    match v {
        Some(v) if v.truthy() => match v {
            Value::Str(s) => s.to_string(),
            Value::Int(_) | Value::Float(_) | Value::Bool(_) => com4::py_str(v),
            other => format!("{other:?}"),
        },
        _ => String::new(),
    }
}

/// An int age in the store's range (`isinstance(age, int)`, not a bool, 1 < age < 120).
fn an_age(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Int(a)) if 1 < *a && *a < 120 => Some(*a),
        _ => None,
    }
}

/// Python: bot/compose/occasionplan.py:110 `_ordinal(age)`
///
/// 30 -> "30th", 21 -> "21st", 22 -> "22nd", 23 -> "23rd".
pub fn _ordinal(age: i64) -> String {
    let suffix = if (10..=20).contains(&age.rem_euclid(100)) {
        "th"
    } else {
        match age.rem_euclid(10) {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{age}{suffix}")
}

/// Python: bot/compose/occasionplan.py:119 `_whose(who)`
///
/// "sister" -> "your sister's"; a name is said as it is: "Jess" -> "Jess's".
pub fn _whose(who: &str) -> String {
    let who = text::strip(who);
    if who.is_empty() {
        return String::new();
    }
    let mut low = who.to_lowercase();
    if low.starts_with("your ") || low.starts_with("my ") {
        low = low.splitn(2, ' ').nth(1).unwrap_or("").to_string();
    }
    // A name is the person's own word for somebody, so it is said back as it was typed.
    if text::is_upper(&who.chars().take(1).collect::<String>()) {
        return format!("{who}'s");
    }
    format!("your {low}{}", if low.ends_with('s') { "'" } else { "'s" })
}

/// Python: bot/compose/occasionplan.py:152 `_the_day(when)`
///
/// Just the day phrase out of a birthday line, as the reply says it.
pub fn _the_day(when: Option<&str>) -> String {
    let when = text::squash(&when.unwrap_or("").to_lowercase());
    let Some(found) = _DAY_TO_COME.search(&when) else { return String::new() };
    let mut phrase = _DAY_SAID.sub_fn(text::strip(found.as_str()), |m| _cap(m));
    if _A_MONTH_ON_ITS_OWN.fullmatch(&phrase.to_lowercase()).is_some() {
        return format!("in {phrase}");
    }
    const OPENS: [&str; 15] = [
        "on ", "in ", "next ", "this ", "the ", "tomorrow", "tonight", "today", "monday", "tuesday", "wednesday", "thursday",
        "friday", "saturday", "sunday",
    ];
    if !text::starts_with_any(&phrase, &OPENS) {
        phrase = format!("on {phrase}");
    }
    phrase
}

/// The first group of a `_TURNS_AGE` match that matched, as a number.
fn age_of(found: &Match) -> Option<i64> {
    found.groups().into_iter().flatten().find(|g| !g.is_empty()).map(|g| g.parse::<i64>().expect("int(group)"))
}

/// Python: bot/compose/occasionplan.py:173 `_the_age(text)`
///
/// The age a birthday line states, "she turns 7" / "turning 40", or None.
pub fn _the_age(text_: &str) -> Option<i64> {
    let found = _TURNS_AGE.search(text_)?;
    for group in found.groups().into_iter().flatten() {
        if !group.is_empty() {
            let age: i64 = group.parse().expect("int(group)");
            if 1 < age && age < 120 {
                return Some(age);
            }
        }
    }
    None
}

/// Python: bot/compose/occasionplan.py:197 `_said_how_long_ago(memory, who=None, markers='')`
///
/// Turns since the line that filed this birthday, or None when nothing filed it.
pub fn _said_how_long_ago(memory: &Memory, who: Option<&str>, markers: Option<&Re>) -> Option<i64> {
    let now = memory.turns.len() as i64 - 1;
    let mut best: Option<i64> = None;
    let Some(log) = memory.get("facts_log") else { return None };
    for entry in log.items() {
        if !entry.is_dict() {
            continue;
        }
        let Some(turn) = entry.get("turn").and_then(|t| match t {
            Value::Int(i) => Some(*i),
            Value::Bool(b) => Some(*b as i64),
            _ => None,
        }) else {
            continue;
        };
        let key = entry.get("key").and_then(Value::as_str).unwrap_or("");
        let value = entry.get("value");
        let text_ = str_or_empty(entry.get("text"));
        if let Some(who) = who {
            if key != "other" || !value.is_some_and(Value::is_dict) {
                continue;
            }
            if str_or_empty(value.and_then(|v| v.get("who"))).to_lowercase() != who.to_lowercase() {
                continue;
            }
        } else {
            if !matches!(key, "plan" | "plans" | "other" | "people" | "birthday") {
                continue;
            }
            if let Some(markers) = markers {
                if markers.search(&text_.to_lowercase()).is_none() {
                    continue;
                }
            }
        }
        let gap = now - turn;
        best = Some(best.map_or(gap, |b| b.min(gap)));
    }
    best
}

/// Python: bot/compose/occasionplan.py:267 `_the_person(said)`
///
/// Who the birthday is for, read off the line: "son" for "my sons 5th birthday", else None. An
/// empty string means somebody is named and cannot be placed.
pub fn _the_person(said: &str) -> Option<String> {
    let low = text::squash(&said.to_lowercase().replace('\u{2019}', "'"));
    if _SOMEONE_ELSES_DAY.search(&low).is_some() {
        return Some(String::new());
    }
    let unplaced = || if _A_NAME_BEFORE_THE_DAY.search(&low).is_some() { Some(String::new()) } else { None };
    for shape in [&*_MINE_BEFORE_THE_DAY, &*_A_POSSESSIVE_BEFORE_THE_DAY] {
        let Some(found) = shape.search(&low) else { continue };
        let who = text::strip_chars(found.name_str("who"), "'");
        if let Some(spoken) = _SPOKEN_FOR.get(who) {
            return Some(spoken.str().to_string());
        }
        if RELATIONS.contains(who) {
            return Some(who.to_string());
        }
        if let Some(stem) = who.strip_suffix("'s").filter(|s| RELATIONS.contains(*s)) {
            return Some(stem.to_string());
        }
        if let Some(stem) = who.strip_suffix('s').filter(|s| RELATIONS.contains(*s)) {
            return Some(stem.to_string());
        }
        return unplaced();
    }
    unplaced()
}

/// Python: bot/compose/occasionplan.py:294 `_the_recent_age(memory, upto=6)`
///
/// The one age said over the last few turns, or None (two different ages are not one birthday).
pub fn _the_recent_age(memory: &Memory, upto: i64) -> Option<i64> {
    let mut ages: Vec<i64> = Vec::new();
    for (turns, last) in memory.turns.iter().rev().enumerate() {
        if turns as i64 + 1 > upto {
            break;
        }
        for found in _TURNS_AGE.finditer(&last.user.to_lowercase()) {
            if let Some(age) = age_of(&found) {
                if !ages.contains(&age) {
                    ages.push(age);
                }
            }
        }
    }
    if ages.len() == 1 && 1 < ages[0] && ages[0] < 120 {
        return Some(ages[0]);
    }
    None
}

/// Python: bot/compose/occasionplan.py:320 `_the_occasion(memory)`
///
/// The one birthday the store holds with a day still to come, else None.
pub fn _the_occasion(memory: &Memory) -> Option<Occasion> {
    let told = memory.turns.iter().rev().find(|t| !t.user.is_empty()).map(|t| t.user.clone()).unwrap_or_default();
    let still = _STILL_TALKING_ABOUT.as_int().expect("an int");
    let mut met: Vec<Occasion> = Vec::new();
    if let Some(others) = memory.get("others") {
        for one in others.items() {
            if !one.is_dict() || one.get("pronoun").is_some_and(Value::truthy) {
                continue;
            }
            let who = match one.get("who").filter(|w| w.truthy()) {
                Some(w) => Some(str_or_empty(Some(w))),
                None => _the_person(&told),
            };
            let Some(who) = who.filter(|w| !w.is_empty()) else { continue };
            let mut age = an_age(one.get("age"));
            let mut when = _the_day(Some(&str_or_empty(one.get("turns"))));
            if when.is_empty() {
                // The day and the age in one string: "birthday": "next month she turns 7".
                let birthday = str_or_empty(one.get("birthday"));
                when = _the_day(Some(&birthday));
                if !when.is_empty() {
                    age = _the_age(&birthday);
                }
            }
            if when.is_empty() {
                continue;
            }
            if let Some(gap) = _said_how_long_ago(memory, Some(&who), None) {
                if gap > still {
                    continue;
                }
            }
            met.push(("another".to_string(), Some(who), age, when));
        }
    }
    let mine = memory.get("user").filter(|u| u.is_dict());
    if let Some(mine) = mine.filter(|m| met.is_empty() && m.truthy()) {
        let plans: Vec<String> = match mine.get("plans") {
            Some(Value::List(l)) => l.iter().map(|p| str_or_empty(Some(p))).collect(),
            _ => Vec::new(),
        };
        let mut said = plans.last().cloned().unwrap_or_default();
        if said.is_empty() {
            said = str_or_empty(mine.get("plan"));
        }
        let when = if _BIRTHDAY_IN_PLAN.search(&said).is_some() { _the_day(Some(&said)) } else { String::new() };
        // The day comes from the plan and the person may be in either.
        let who = if when.is_empty() { None } else { _the_person(&format!("{said} {told}")) };
        if !when.is_empty() && who.as_deref() == Some("") {
            return None; // somebody is named and cannot be placed: not the person typing
        }
        if !when.is_empty() {
            let age = an_age(mine.get("age")).or_else(|| _the_age(&said)).or_else(|| _the_recent_age(memory, 6));
            let gap = _said_how_long_ago(memory, None, Some(&_BIRTHDAY_IN_PLAN));
            if gap.is_none_or(|g| g <= still) {
                met.push(("you".to_string(), who.filter(|w| !w.is_empty()), age, when));
            }
        }
    }
    if met.len() == 1 { met.pop() } else { None }
}

/// Python: bot/compose/occasionplan.py:378 `_cap(match)`
///
/// A day word said back: "on saturday" reads as a day and not as a stray lower case word.
pub fn _cap(found: &Match) -> String {
    text::upper_first(found.as_str())
}

/// Python: bot/compose/occasionplan.py:397 `_reads_as_a_plan_ask(said, memory)`
///
/// True when the line asks for a plan for the thing in hand and names nothing else.
pub fn _reads_as_a_plan_ask(said: &str, memory: &Memory) -> bool {
    let _ = memory;
    let said = text::squash(&said.to_lowercase());
    if said.is_empty() {
        return false;
    }
    let words = text::split_ref(&said.replace(['?', '!', '.'], " ")).len();
    if words > 10 || _ASKS_FOR_A_PLAN.search(&said).is_none() || _NOT_A_PLAN_ASK.search(&said).is_some() {
        return false;
    }
    if !_OWN_SUBJECT.findall(&said).is_empty() || _ANOTHER_PERSON.search(&said).is_some() {
        return false;
    }
    true
}

/// Python: bot/compose/occasionplan.py:423 `takes_as_it_comes(text, memory)`
///
/// The hook a module declares (`TAKES_AS_IT_COMES`): this line is the plan for the day on file.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> bool {
    reply_for(text_, memory).is_some()
}

/// Python: bot/compose/occasionplan.py:434 `reply_for(text, memory)`
///
/// A plan for the birthday the store holds, or None for every other line.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let (kind, who, age, when) = _the_occasion(memory)?;
    if !_reads_as_a_plan_ask(text_, memory) {
        return None;
    }
    let whose = if kind == "you" {
        match who.as_deref().filter(|w| !w.is_empty()) {
            Some(w) => _whose(w),
            None => "your".to_string(),
        }
    } else {
        let w = _whose(who.as_deref().unwrap_or(""));
        if w.is_empty() {
            return None;
        }
        w
    };
    let middle = match age {
        Some(a) => _ordinal(a),
        None => "birthday".to_string(),
    };
    let day = [whose.as_str(), middle.as_str(), when.as_str()].iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(" ");
    let pool: &Vec<String> = if age.is_some_and(|a| a < 16) {
        &CHILD_PLANS
    } else if _SOON.search(&when).is_some() {
        &SOON_PLANS
    } else {
        &PLANS
    };
    // The choice is made from the line and the turns before it, never from the clock.
    let seed = memory.turns.len() as i128 * 7919 + com4::ord_sum(&text::squash(&text_.to_lowercase()));
    let line = Random::new(seed).choice(pool).clone();
    Some(line.replace("{day}", &day))
}
