//! Port of `bot/modules/a_fact_asked_again.py`: a fact asked for "again" - a date, a thing to avoid, a target - said back from what the store kept.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.a_fact_asked_again.", stringify!($name))));
    )*};
}
rx!(AGAIN, AN_ASK, NOT_A_RECALL, A_TIME, A_THING_TO_AVOID, A_TARGET);
static A_DATE: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.modules.a_fact_asked_again.A_DATE"));
static PLAN_OPENS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.a_fact_asked_again.PLAN_OPENS"));
static THE_WORD_ASKED: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.modules.a_fact_asked_again.THE_WORD_ASKED"));
static A_NAME_WORD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(?:vauxhall|ford|toyota|nissan|bmw|audi|honda|ikea|argos|january|february|march|april|may|june|july|august|september|october|november|december|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
        pyre::I,
    )
});

/// Python: bot/modules/a_fact_asked_again.py `_the_plans(memory)`
///
/// What the store kept as their plans, oldest first, newest last.
pub fn _the_plans(memory: &Memory) -> Vec<String> {
    let user = user_ro(memory);
    let mut plans: Vec<String> = vget(user, "plans").items().iter().filter_map(|plan| plan.as_str()).filter(|plan| !text::strip(plan).is_empty()).map(str::to_string).collect();
    if let Some(latest) = vget(user, "plan").as_str() {
        if !text::strip(latest).is_empty() && !plans.iter().any(|plan| plan == latest) {
            plans.push(latest.to_string());
        }
    }
    plans.iter().map(|plan| text::squash(plan)).collect()
}

/// Python: bot/modules/a_fact_asked_again.py `_the_allergies(memory)`
///
/// [(who, what)] the store filed for the people in the chat, newest last.
pub fn _the_allergies(memory: &Memory) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for one in mget(memory, "others").items() {
        if !one.is_dict() {
            continue;
        }
        let who = text::strip(&sget(one, "who")).to_string();
        for what in vget(one, "allergies").items() {
            let what = text::strip(&s_or(Some(what))).to_string();
            if !what.is_empty() {
                out.push((who.clone(), what));
            }
        }
    }
    out
}

/// Python: bot/modules/a_fact_asked_again.py `_the_date_in(plan)`
///
/// The date a plan carries, in the words the plan says it, or None.
pub fn _the_date_in(plan: &str) -> Option<String> {
    A_DATE.items().iter().find_map(|pattern| pattern.as_re().unwrap().search(plan).map(|m| text::squash(m.as_str())))
}

/// Python: bot/modules/a_fact_asked_again.py `_the_time_asked_again(text, memory)`
///
/// The one dated plan the chat holds, said back, or None.
pub fn _the_time_asked_again(text: &str, memory: &Memory) -> Option<String> {
    // The store can read the ask's own words - "when is my flight again" - and that reply is the
    // better one, so this module stays out of its way. (The store's reader must never lose this
    // line.)
    match py_try(|| ret_true(crate::memory::when::when_answer(text, memory))) {
        Ok(false) => {}
        _ => return None,
    }
    let dated: Vec<String> = _the_plans(memory).iter().filter_map(|plan| _the_date_in(plan)).filter(|date| !date.is_empty()).collect();
    if dated.len() != 1 {
        return None;
    }
    Some(format!("You said {}.", dated[0]))
}

/// Python: bot/modules/a_fact_asked_again.py `_the_thing_to_avoid_asked_again(text, memory)`
///
/// The allergy the store filed, said back, or None.
pub fn _the_thing_to_avoid_asked_again(text: &str, memory: &Memory) -> Option<String> {
    let allergies = _the_allergies(memory);
    let (who, what) = allergies.last()?;
    if !who.is_empty() {
        return Some(format!("{} - your {} is allergic to them.", text::capitalize(what), who));
    }
    Some(format!("{}.", text::capitalize(what)))
}

/// Python: bot/modules/a_fact_asked_again.py `_the_noun_asked(text)`
///
/// The person's own word for the thing they asked about again.
pub fn _the_noun_asked(text: &str) -> String {
    let low = fold(text);
    for pair in THE_WORD_ASKED.items() {
        let (word, noun) = (pair.items()[0].str(), pair.items()[1].str());
        if Re::new(&format!(r"\b{word}s?\b"), 0).is_match(&low) {
            return noun.to_string();
        }
    }
    "plan".to_string()
}

/// Python: bot/modules/a_fact_asked_again.py `_a_name(plan)`
///
/// Whether the word the plan opens on is somebody's or something's name, so it keeps its capital.
pub fn _a_name(plan: &str) -> bool {
    let first = text::split(plan).into_iter().next().unwrap_or_default();
    A_NAME_WORD.is_match(&first) && text::is_upper(&sl(&first, 0, Some(1)))
}

/// Python: bot/modules/a_fact_asked_again.py `_the_target_asked_again(text, memory)`
///
/// The plan they stated, said back under the word they used for it, or None.
pub fn _the_target_asked_again(text: &str, memory: &Memory) -> Option<String> {
    let plans = _the_plans(memory);
    let mut plan = plans.last()?.clone();
    let low = plan.to_lowercase();
    for opener in PLAN_OPENS.iter() {
        if low.starts_with(opener.as_str()) {
            plan = sl(&plan, text::len(opener) as isize, None);
            break;
        }
    }
    let noun = _the_noun_asked(text);
    // "Get to 12" keeps its capital when it opens a line and loses it inside one; a name keeps it.
    if text::is_upper(&sl(&plan, 0, Some(1))) && !text::is_upper(&sl(&plan, 0, Some(2))) && !_a_name(&plan) {
        plan = low1(&plan);
    }
    Some(format!("You said your {} is to {}.", noun, text::rstrip_chars(&plan, ".!?")))
}

/// Python: bot/modules/a_fact_asked_again.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The fact the store kept for the kind of thing the line asks for again, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let low = fold(text);
    if !AGAIN.is_match(&low) || !AN_ASK.is_match(&low) || NOT_A_RECALL.is_match(&low) {
        return None;
    }
    if A_TIME.is_match(&low) {
        return _the_time_asked_again(text, memory);
    }
    if A_THING_TO_AVOID.is_match(&low) {
        return _the_thing_to_avoid_asked_again(text, memory);
    }
    if A_TARGET.is_match(&low) {
        return _the_target_asked_again(text, memory);
    }
    None
}
