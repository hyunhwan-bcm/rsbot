//! Port of `bot/memory/household.py`: who lives with the person - "its just me and my daughter" -
//! and the ask that follows it.
//!
//! The household is filed on the chat's own memory as `memory["household"]`
//! ({"people": ["your daughter"], "alone": False}); somebody else living alone is filed beside it
//! as `memory["their_households"]` ({"mum": True}). Only the shape that says who is at home is
//! read, because "i live with my mum" is also a place, and the store reads that one.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, lists, res};
use crate::py::text as pt;
use crate::state::Memory;

res!("household"; WHO_LIVES, JUST_ME_AND, JUST_ME, ON_MY_OWN, SHOULD_I_MORE, YES_AND_NO, THEIR_HOUSE_ASK, THEIR_ALONE_SAID, THEIR_HOUSE_WHO, THE_ANSWER_ASKED);
lists!("household"; WITH_SOMEONE, ALONE, NOT_SAID, THEIR_HOUSE, THEIRS_NOT_SAID, THEIRS_NOT_SAID_WHO, SHOULD_MORE, THEN_ALONE, THEN_NOT_ALONE, THEIRS_WITH, THEIRS_ALONE);

/// `" ".join(text.lower().split()).strip(" .!,")`.
fn the_line(text: &str) -> String {
    pt::strip_chars(&pt::squash(&pt::lower(text)), " .!,").to_string()
}

/// Python: bot/memory/household.py:192 `_who_said(who)`
///
/// The relation as it goes back to them: "daughter" -> "your daughter".
pub fn _who_said(who: &str) -> String {
    let said = pt::lower(pt::strip(&pt::squash(who)));
    if said.is_empty() {
        return String::new();
    }
    format!("your {said}")
}

/// Python: bot/memory/household.py:200 `_who_asked(who, memory)`
///
/// The person an ask about somebody else is about: ("your mum", "mum") or ("she", "").
pub fn _who_asked(who: &str, memory: &Memory) -> (String, String) {
    let said = pt::lower(pt::strip(&pt::squash(who)));
    if let Some(rest) = said.strip_prefix("your ") {
        return (said.clone(), rest.to_string());
    }
    if let Some(Value::List(held)) = memory.get("others") {
        for person in held.iter().rev() {
            if let Some(name) = person.get("who").and_then(Value::as_str).filter(|name| !pt::strip(name).is_empty()) {
                let name = pt::lower(pt::strip(name));
                return (format!("your {name}"), name);
            }
        }
    }
    let alone = _their_households(memory);
    if alone.pairs().len() == 1 {
        let only = mem7::py_str(&alone.pairs()[0].0);
        return (format!("your {only}"), only);
    }
    (said, String::new())
}

/// Python: bot/memory/household.py:230 `_who_held(who, key)`
///
/// The person as the object of a sentence: "she" -> "her", "your mum" -> "your mum".
pub fn _who_held(who: &str, key: &str) -> String {
    if !key.is_empty() {
        return who.to_string();
    }
    match who {
        "she" => "her".into(),
        "he" => "him".into(),
        "they" => "them".into(),
        other => other.to_string(),
    }
}

/// `memory["turns"][-1]` when it is a turn, with its module and bot line.
fn last_turn(memory: &Memory) -> (Option<&str>, &str) {
    match memory.turns.last() {
        Some(turn) => (turn.module.as_deref(), turn.bot.as_str()),
        None => (None, ""),
    }
}

/// Python: bot/memory/household.py:244 `_the_ask_it_answers(said, memory)`
///
/// The question about this on the last turn (a dict with who, key and ask), if this line
/// answers it.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the dict or None.
pub fn _the_ask_it_answers(said: &str, memory: &Memory) -> Option<Value> {
    YES_AND_NO.match_(said)?;
    let (module, bot) = last_turn(memory);
    if let Some(asked) = memory.get("household_asked") {
        if asked.is_dict() && asked.get("ask") == Some(&Value::from("alone")) && module == Some("household") {
            return Some(asked.clone());
        }
    }
    // Nobody in this module asked it, but the answer that went to the person did.
    let found = THE_ANSWER_ASKED.search(bot)?;
    let (who, key) = _who_asked(found.name_str("who"), memory);
    Some(mem7::dict(vec![("who", Value::from(who)), ("key", Value::from(key)), ("ask", Value::from("alone"))]))
}

/// Python: bot/memory/household.py:263 `_their_households(memory)`
///
/// Who else the chat says lives alone, by relation: {"mum": True}.
pub fn _their_households(memory: &Memory) -> Value {
    match memory.get("their_households") {
        Some(held) if held.is_dict() => held.clone(),
        _ => Value::dict(),
    }
}

/// Python: bot/memory/household.py:269 `_held(memory)`
///
/// The household the chat has been told about, as {"people", "alone"}, or None.
pub fn _held(memory: &Memory) -> Option<Value> {
    let held = memory.get("household").filter(|held| held.is_dict())?;
    let people: Vec<Value> = mem7::iterate_or_empty(held.get("people")).into_iter().filter(|person| person.as_str().is_some_and(|p| !p.is_empty())).collect();
    let alone = mem7::get(held, "alone").truthy();
    if people.is_empty() && !alone {
        return None;
    }
    Some(mem7::dict(vec![("people", Value::from(people)), ("alone", Value::from(alone))]))
}

/// `memory.setdefault("their_households", {})[key] = value`, when what is kept is a dict.
fn keep_theirs(memory: &mut Memory, key: &str, value: bool) {
    let kept = memory.setdefault("their_households", Value::dict());
    if kept.is_dict() {
        kept.set_key(key, value);
    }
}

/// `{"people": people, "alone": alone, "said": said}`.
fn a_household(people: Vec<String>, alone: bool, said: &str) -> Value {
    mem7::dict(vec![("people", Value::from(people)), ("alone", Value::from(alone)), ("said", Value::from(said))])
}

/// Python: bot/memory/household.py:280 `observe(text, memory)`
///
/// File who lives with them, and say nothing: the reply is this module's `respond`.
pub fn observe(text: &str, memory: &mut Memory) {
    let said = the_line(text);
    if said.is_empty() {
        return;
    }
    for theirs in THEIR_ALONE_SAID.finditer(&said) {
        keep_theirs(memory, &pt::lower(theirs.name_str("who")), true);
    }
    if ON_MY_OWN.is_match(&said) {
        memory.set("household", a_household(Vec::new(), true, &said));
        return;
    }
    if let Some(found) = JUST_ME_AND.match_(&said) {
        let mut people = vec![_who_said(found.name_str("who"))];
        if let Some(more) = found.name("more").filter(|more| !more.is_empty()) {
            // "me and my daughter and my son": two people, and a name is not said twice.
            let second = _who_said(more);
            if !second.is_empty() && !people.contains(&second) {
                people.push(second);
            }
        }
        memory.set("household", a_household(people, false, &said));
        return;
    }
    if JUST_ME.match_(&said).is_some() {
        memory.set("household", a_household(Vec::new(), true, &said));
    }
}

/// `pick(pool, memory, seed_text=text).format(who=who)`.
fn said_from(pool: &[String], memory: &Memory, text: &str, who: &str) -> String {
    mem7::fmt(&mem7::pick(pool, memory, text), &[("who", who)])
}

/// Python: bot/memory/household.py:312 `answer(text, memory)`
///
/// The ask, or the household said back and nothing else, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer(text: &str, memory: &mut Memory) -> Option<String> {
    let said = the_line(text);
    if said.is_empty() {
        return None;
    }
    if let Some(asked) = _the_ask_it_answers(&said, memory) {
        memory.remove("household_asked");
        let no = ["no", "nope", "not"].iter().any(|start| said.starts_with(start))
            || [" doesnt", " doesn't", " dont", " don't", " isnt", " isn't", " arent", " aren't"].iter().any(|word| said.contains(word));
        let yes = !no;
        let key = mem7::text_of(asked.get("key"));
        if !key.is_empty() {
            keep_theirs(memory, &key, yes);
        }
        let who_said = Some(mem7::text_of(asked.get("who"))).filter(|w| !w.is_empty()).unwrap_or_else(|| "them".into());
        let who = _who_held(&who_said, &key);
        return Some(said_from(if yes { &THEN_ALONE } else { &THEN_NOT_ALONE }, memory, text, &who));
    }
    if SHOULD_I_MORE.is_match(&said) && last_turn(memory).0 == Some("household") {
        return Some(mem7::pick(&SHOULD_MORE, memory, text));
    }
    if let Some(found) = THEIR_HOUSE_WHO.search(&said) {
        let named = found.name("who").filter(|w| !w.is_empty()).or(found.name("who2").filter(|w| !w.is_empty()));
        let (asked_who, asked_key, them) = match named {
            Some(named) => {
                let (who, key) = _who_asked(named, memory);
                let them = _who_held(&who, &key);
                (who, key, them)
            }
            None => {
                let them = found.name_str("who3").to_string();
                (them.clone(), String::new(), them)
            }
        };
        // A question asked is remembered, so the next line can answer it.
        memory.set("household_asked", mem7::dict(vec![("who", Value::from(asked_who)), ("key", Value::from(asked_key)), ("ask", Value::from("who"))]));
        return Some(said_from(&THEIRS_NOT_SAID_WHO, memory, text, &them));
    }
    if let Some(found) = THEIR_HOUSE_ASK.search(&said) {
        let named = found.name("who").filter(|w| !w.is_empty()).or(found.name("who2").filter(|w| !w.is_empty())).unwrap_or("");
        let (who, key) = _who_asked(named, memory);
        let held = if key.is_empty() { None } else { _their_households(memory).get(&key).cloned() };
        match held {
            Some(Value::Bool(true)) => return Some(said_from(&THEIRS_ALONE, memory, text, &who)),
            Some(Value::Bool(false)) => return Some(said_from(&THEIRS_WITH, memory, text, &_who_held(&who, &key))),
            _ => {}
        }
        memory.set("household_asked", mem7::dict(vec![("who", Value::from(who.as_str())), ("key", Value::from(key)), ("ask", Value::from("alone"))]));
        return Some(said_from(&THEIRS_NOT_SAID, memory, text, &who));
    }
    if WHO_LIVES.is_match(&said) {
        let Some(held) = _held(memory) else {
            // A chat that gave a headcount holds one, and the count reader says it back.
            if mem7::truthy(crate::memory::counts::answer(&said, memory)) {
                return None;
            }
            return Some(mem7::pick(&NOT_SAID, memory, text));
        };
        if mem7::get(&held, "alone").truthy() {
            return Some(mem7::pick(&ALONE, memory, text));
        }
        let people: Vec<String> = mem7::iterate_or_empty(held.get("people")).iter().map(mem7::py_str).collect();
        let who = match people.len() {
            1 => people[0].clone(),
            2 => format!("{} and {}", people[0], people[1]),
            _ => {
                let (last, rest) = people.split_last().expect("IndexError: nobody");
                format!("{} and {}", rest.join(", "), last)
            }
        };
        return Some(said_from(&WITH_SOMEONE, memory, text, &who));
    }
    let found = JUST_ME_AND.match_(&said).or_else(|| JUST_ME.match_(&said))?;
    // `found.groupdict().get("who")`: JUST_ME has no such group.
    let who_group = found.name("who").unwrap_or("");
    if who_group.is_empty() {
        return None;
    }
    let who = _who_said(who_group);
    if who.is_empty() {
        return None;
    }
    let picked = mem7::pick(&THEIR_HOUSE, memory, text);
    Some(mem7::fmt(&picked, &[("who", &who), ("Who", &pt::upper_first(&who))]))
}

/// Python: bot/memory/household.py:396 `takes_as_it_comes(text, memory)`
///
/// This line, because the household is the whole of it.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    let said = the_line(text);
    if WHO_LIVES.is_match(&said) {
        // A line the count reader can answer is not answered here.
        return !mem7::truthy(crate::memory::counts::answer(&said, memory));
    }
    if THEIR_HOUSE_ASK.is_match(&said) || THEIR_HOUSE_WHO.is_match(&said) {
        return true;
    }
    if _the_ask_it_answers(&said, memory).is_some() {
        return true;
    }
    if SHOULD_I_MORE.is_match(&said) {
        return last_turn(memory).0 == Some("household");
    }
    JUST_ME_AND.match_(&said).or_else(|| JUST_ME.match_(&said)).is_some_and(|found| found.name("who").is_some_and(|who| !who.is_empty()))
}
