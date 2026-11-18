//! Port of `bot/memory/company.py`: who is going with them, and the ask that wants it back.
//!
//! "im travelling with my mum" is kept as the companion, and "who am i going with" is answered
//! from it. A line that puts the companion right ("actually its my sister not my wife") replaces
//! the one held.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, inline_re, res};
use crate::py::re::{self, Re};
use crate::py::text as pt;
use crate::state::Memory;

res!("company"; SAID_WITH, OFF_TO_WITH, TAKING, COMING_WITH, _NOT_COMING_ALONG, _STOP, _A_PERSON, EITHER_SIDE, ASK, IS_COMING_TOO, ASKS_FOR_A_DETAIL, _ABOUT_SOMEONE_ELSE, PUT_RIGHT, ON_THEIR_OWN, _ASKS, _THE_MONTHS, _TO_THE_TRIP);
inline_re! {
    IN_LAW = r"\b([a-z]+)\s+in[\s-]+(laws?)\b", re::I;
}

/// `_A_PERSON.match(who) or _A_PERSON.match("my " + who)`.
fn a_person(who: &str) -> bool {
    _A_PERSON.match_(who).is_some() || _A_PERSON.match_(&format!("my {who}")).is_some()
}

/// Python: bot/memory/company.py:97 `_bare(who)`
///
/// The companion without the possessive in front: "my wife" -> "wife".
pub fn _bare(who: &str) -> String {
    let low = pt::lower(who);
    let mut words = pt::split_ref(&low);
    while words.first().is_some_and(|w| matches!(*w, "my" | "our" | "the" | "a" | "an" | "your")) {
        words.remove(0);
    }
    words.join(" ")
}

/// Python: bot/memory/company.py:105 `_the_companion(text)`
///
/// The companion a line names, as they said it, or "" when the line names nobody.
pub fn _the_companion(text: &str) -> String {
    let said = IN_LAW.sub(r"\1-in-\2", &pt::squash(text));
    if said.is_empty() || pt::rstrip(&said).ends_with('?') {
        return String::new();
    }
    let patterns: [(&Re, bool); 5] = [(*SAID_WITH, false), (*OFF_TO_WITH, false), (*TAKING, false), (*COMING_WITH, true), (*EITHER_SIDE, false)];
    for (pattern, coming_with) in patterns {
        let Some(m) = pattern.search(&said) else {
            continue;
        };
        let mut who = m.name("who").filter(|w| !w.is_empty()).or(m.name("other").filter(|w| !w.is_empty())).unwrap_or("").to_string();
        if let Some(cut) = _STOP.search(&who) {
            who = who[..cut.start()].to_string();
        }
        let mut who = pt::strip_chars(&who, " ,.;:-'\u{2019}").to_string();
        if who.is_empty() {
            continue;
        }
        // "and her kids" after the first person keeps them.
        if who.contains(" and ") {
            let (first, _, rest) = pt::partition(&who, " and ");
            who = if a_person(rest) { format!("{first} and {rest}") } else { first.to_string() };
        }
        if mem7::word_count(&who) > 5 {
            continue;
        }
        if coming_with && _NOT_COMING_ALONG.is_match(&said) {
            continue;
        }
        if !a_person(&who) {
            continue;
        }
        return who;
    }
    String::new()
}

/// Python: bot/memory/company.py:143 `_as_you_would_say_it(who)`
///
/// The companion said back to the person: "my mum" -> "your mum".
pub fn _as_you_would_say_it(who: &str) -> String {
    let said = pt::strip(who);
    for (mine, yours) in [("my ", "your "), ("our ", "your "), ("i ", "you "), ("me", "you"), ("we ", "you "), ("myself", "yourself")] {
        if let Some(rest) = said.strip_prefix(mine) {
            return format!("{yours}{rest}");
        }
    }
    said.to_string()
}

/// Python: bot/memory/company.py:153 `_more_than_one(who)`
///
/// Whether the companion is more than one person, so the verb is "are".
pub fn _more_than_one(who: &str) -> bool {
    let low = pt::lower(who);
    low.contains(" and ") || pt::split_ref(&low).last().expect("IndexError: no companion").ends_with('s')
}

/// `memory["company"]["who"]` when a companion is held.
fn the_held(memory: &Memory) -> Option<String> {
    let held = memory.get("company").filter(|h| h.is_dict())?;
    let who = held.get("who").filter(|w| w.truthy())?;
    Some(mem7::py_str(who))
}

/// Python: bot/memory/company.py:159 `_the_correction(text, memory)`
///
/// The companion the line puts right, or "" when the line corrects something else.
pub fn _the_correction(text: &str, memory: &Memory) -> String {
    let Some(held) = the_held(memory) else {
        return String::new();
    };
    let low = pt::squash(&pt::lower(text));
    let Some(m) = PUT_RIGHT.search(&low) else {
        return String::new();
    };
    let old = _bare(m.name_str("old"));
    if old.is_empty() || old != _bare(&held) {
        return String::new();
    }
    let new = pt::strip_chars(m.name_str("new"), " ,.;:-");
    if a_person(new) {
        new.to_string()
    } else {
        String::new()
    }
}

/// `(memory.get("user") or {}).get(key)`.
fn user_field<'a>(memory: &'a Memory, key: &str) -> Option<&'a Value> {
    memory.get("user").filter(|u| u.truthy()).and_then(|u| u.get(key))
}

/// The plans the store keeps: `(user or {}).get("plans") or []`.
fn plans(memory: &Memory) -> Vec<String> {
    mem7::iterate_or_empty(user_field(memory, "plans")).iter().map(|p| mem7::text_of(Some(p))).collect()
}

/// Python: bot/memory/company.py:180 `_a_trip_is_here(memory)`
///
/// Whether the chat holds a plan, which is what the ask for company is about.
pub fn _a_trip_is_here(memory: &Memory) -> bool {
    plans(memory).iter().any(|plan| !pt::strip(plan).is_empty())
}

/// Python: bot/memory/company.py:187 `_on_their_own(memory)`
///
/// Whether the plan they gave says they are going alone.
pub fn _on_their_own(memory: &Memory) -> bool {
    plans(memory).iter().any(|plan| ON_THEIR_OWN.is_match(plan))
}

/// Python: bot/memory/company.py:211 `_the_phrase_a_frame_can_say(plan)`
///
/// The trip as a phrase, from a plan that may be the whole line it was said in.
pub fn _the_phrase_a_frame_can_say(plan: &str) -> String {
    let mut plan = pt::strip_chars(&pt::squash(plan), " .").to_string();
    if let Some(found) = _TO_THE_TRIP.search(&plan) {
        let rest = pt::strip_chars(found.name_str("rest"), " .");
        if !rest.is_empty() {
            plan = rest.to_string();
        }
    }
    if plan.is_empty() {
        return String::new();
    }
    // A month is a name in the middle of the phrase too.
    let plan = _THE_MONTHS.sub_fn(&plan, |m| pt::capitalize(m.as_str()));
    pt::upper_first(&plan)
}

/// Python: bot/memory/company.py:233 `_the_trip_on_the_table(memory)`
///
/// The trip the chat is on, in as few words as the plan was said: "Cornwall in july".
pub fn _the_trip_on_the_table(memory: &Memory) -> String {
    let places = mem7::iterate_or_empty(user_field(memory, "places"));
    let Some(last) = places.last() else {
        return String::new();
    };
    let place = pt::strip_chars(&mem7::py_str(last), " .").to_string();
    let plan = pt::strip_chars(&mem7::text_of(user_field(memory, "plan")), " .").to_string();
    if place.is_empty() || (!plan.is_empty() && !pt::lower(&plan).contains(&pt::lower(&place))) {
        return String::new();
    }
    let phrase = _the_phrase_a_frame_can_say(&plan);
    if phrase.is_empty() {
        place
    } else {
        phrase
    }
}

/// Python: bot/memory/company.py:252 `_new_to_the_chat(who, memory)`
///
/// Whether this is the first line of the chat that names them.
pub fn _new_to_the_chat(who: &str, memory: &Memory) -> bool {
    let bare = _bare(who);
    if bare.is_empty() {
        return false;
    }
    let said = Re::new(&format!(r"\b{}\b", re::escape(&bare)), re::I);
    let mut turns: Vec<&str> = memory.turns.iter().map(|t| t.user.as_str()).collect();
    let now = memory.understood.as_ref().map_or("", |u| u.text.as_str());
    if turns.last().is_some_and(|last| pt::strip(last) == pt::strip(now)) {
        turns.pop();
    }
    !turns.iter().any(|line| said.is_match(line))
}

/// `pick(options, memory, seed_text=low) or None`.
fn pick_or_none(options: &[String], memory: &Memory, low: &str) -> Option<String> {
    Some(mem7::pick(options, memory, low)).filter(|reply| !reply.is_empty())
}

/// Python: bot/memory/company.py:271 `_the_one_person_asked_about(text, memory)`
///
/// Whether that very person is coming, or None when the line does not ask about one.
pub fn _the_one_person_asked_about(text: &str, memory: &Memory) -> Option<String> {
    let low = pt::squash(&pt::lower(text));
    if low.is_empty() || _NOT_COMING_ALONG.is_match(&low) || ASKS_FOR_A_DETAIL.is_match(&low) {
        return None;
    }
    let found = IS_COMING_TOO.search(&low)?;
    let named = _bare(found.name_str("who"));
    if named.is_empty() || !a_person(&named) {
        return None;
    }
    let Some(held) = the_held(memory) else {
        // Nobody was ever named, but the chat is on a trip: that side of it is the answer.
        if !_a_trip_is_here(memory) {
            return None;
        }
        return pick_or_none(
            &[
                format!("Nothing said about your {named} - you have not said who is coming with you."),
                format!("You have not told me who is coming, so I cannot say about your {named}."),
            ],
            memory,
            &low,
        );
    };
    let who = _as_you_would_say_it(&held);
    if named == _bare(&held) {
        return pick_or_none(
            &[
                format!("Yes - you said {who} is coming with you."),
                format!("You said {who} is coming, so yes."),
                format!("{} - that is who you said was coming.", pt::capitalize(&who)),
            ],
            memory,
            &low,
        );
    }
    pick_or_none(
        &[
            format!("You only said {who} was coming - nothing about your {named}."),
            format!("Nothing said about your {named} - it was {who} you told me about."),
        ],
        memory,
        &low,
    )
}

/// Python: bot/memory/company.py:310 `_the_companion_joins(text, memory)`
///
/// The line puts somebody on the plan the chat already has, or None.
pub fn _the_companion_joins(text: &str, memory: &Memory) -> Option<String> {
    let low = pt::squash(&pt::lower(text));
    if low.is_empty() || pt::rstrip(&low).ends_with('?') || _ASKS.match_(&low).is_some() {
        return None;
    }
    let place = _the_trip_on_the_table(memory);
    if place.is_empty() {
        return None;
    }
    let who = _the_companion(text);
    if who.is_empty() || !_new_to_the_chat(&who, memory) {
        return None;
    }
    let said = _as_you_would_say_it(&who);
    if _the_plan_is_this_line(text, memory) {
        // The person named the people and the trip in one breath: the news is the trip with them.
        return pick_or_none(
            &[format!("Nice - {place} with {said}."), format!("Lovely - {place} with {said}."), format!("{place} with {said} - good.")],
            memory,
            &low,
        );
    }
    pick_or_none(
        &[
            format!("{} as well - good. {place} is better with company.", pt::capitalize(&said)),
            format!("Nice, {said} coming too. {place} goes better shared."),
            format!("Good - {said} along for {place}, then."),
        ],
        memory,
        &low,
    )
}

/// Python: bot/memory/company.py:344 `_the_plan_is_this_line(text, memory)`
///
/// Whether the plan the chat holds is the line being answered, said in one breath.
pub fn _the_plan_is_this_line(text: &str, memory: &Memory) -> bool {
    let line = pt::strip_chars(&pt::squash(&pt::lower(text)), " .").to_string();
    let plan = pt::strip_chars(&pt::squash(&pt::lower(&mem7::text_of(user_field(memory, "plan")))), " .").to_string();
    !line.is_empty() && !plan.is_empty() && (plan == line || plan.contains(&line) || line.contains(&plan))
}

/// Python: bot/memory/company.py:357 `observe(text, memory)`
///
/// Keep the companion the line names, or the one it puts right.
pub fn observe(text: &str, memory: &mut Memory) {
    let turns = mem7::turn_count(memory);
    let fixed = _the_correction(text, memory);
    if !fixed.is_empty() {
        memory.set("company", mem7::dict(vec![("who", Value::from(fixed)), ("turns", Value::from(turns)), ("corrected", Value::Bool(true))]));
        return;
    }
    let who = _the_companion(text);
    if who.is_empty() {
        return;
    }
    memory.set("company", mem7::dict(vec![("who", Value::from(who)), ("turns", Value::from(turns))]));
}

/// Python: bot/memory/company.py:370 `answer(text, memory)`
///
/// The companion, said back, or the addition taken up. None when the line does neither.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer(text: &str, memory: &Memory) -> Option<String> {
    let low = pt::squash(&pt::lower(text));
    if !ASK.is_match(&low) || _ABOUT_SOMEONE_ELSE.is_match(&low) {
        if let Some(one) = _the_one_person_asked_about(text, memory) {
            return Some(one);
        }
        return _the_companion_joins(text, memory);
    }
    let Some(held) = the_held(memory) else {
        // No companion was ever named: the ask is still about the trip.
        if !_a_trip_is_here(memory) {
            return None;
        }
        if _on_their_own(memory) {
            return pick_or_none(&["You said you were going on your own.".into(), "On your own, you said - nobody else was named.".into()], memory, &low);
        }
        return pick_or_none(&["You haven't said who's going with you.".into(), "Nobody yet - you haven't said who is going with you.".into()], memory, &low);
    };
    let who = _as_you_would_say_it(&held);
    let verb = if _more_than_one(&who) { "are" } else { "is" };
    let trimmed = pt::rstrip(&low);
    if trimmed.ends_with("alone") || trimmed.ends_with("on my own") {
        return Some(format!("Not on your own - {who} {verb} coming with you."));
    }
    let mut options = vec![
        format!("You said {who} {verb} coming with you."),
        format!("{} - that is who you said you were going with.", pt::capitalize(&who)),
        format!("You said {who} {verb} coming with you, and that is all I know."),
    ];
    if who.starts_with("your ") {
        options.insert(1, format!("Your {}, you told me.", pt::partition(&who, " ").2));
    }
    pick_or_none(&options, memory, &low)
}
